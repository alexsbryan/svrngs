// SPDX-License-Identifier: AGPL-3.0-or-later
//! The replica spike's dialer (../DESIGN.md): every `every` seconds, the
//! phone's key dialled on the probe and, at the same moment, a control dial
//! through a relay to an endpoint of the dialer's own, one JSON line a slot
//! with its verdict: answered, failed (the control answered and the phone did
//! not), could-not-judge (neither did), never-ran (a slot the schedule owed).
//!
//! `replica-dialer <dir> <phone-key> [every-s] [within-s]`: `<dir>` keeps the
//! dialer's key, made on the first run, and `dials.jsonl`.

use std::path::Path;
use std::time::{Duration, Instant, SystemTime, UNIX_EPOCH};

use iroh::endpoint::{TransportAddrUsage, presets};
use iroh::{Endpoint, EndpointAddr, PublicKey, SecretKey};
use serde_json::{Value, json};
use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};

/// The tunnel's probe ALPN (../../../phone-tunnel/2026-10-26/tunnel/src/lib.rs, `PROBE`).
const PROBE: &[u8] = b"svrngs/probe/0";

#[tokio::main]
async fn main() -> Result<(), String> {
    let args: Vec<String> = std::env::args().collect();
    let [_, dir, phone, rest @ ..] = &args[..] else {
        return Err("usage: replica-dialer <dir> <phone-key> [every-s] [within-s]".into());
    };
    let phone: PublicKey = phone.parse().map_err(|e| format!("not a key: {e}"))?;
    let secs = |i: usize, d: u64| {
        rest.get(i)
            .map_or(Ok(d), |s| s.parse::<u64>().map_err(|e| e.to_string()))
    };
    let (every, within) = (Duration::from_secs(secs(0, 60)?), Duration::from_secs(secs(1, 30)?));
    let dir = Path::new(dir);
    std::fs::create_dir_all(dir).map_err(|e| e.to_string())?;

    let dialer = Endpoint::builder(presets::N0)
        .secret_key(kept(&dir.join("dialer.key"))?)
        .bind()
        .await
        .map_err(|e| e.to_string())?;
    let control = Endpoint::builder(presets::N0)
        .alpns(vec![PROBE.to_vec()])
        .bind()
        .await
        .map_err(|e| e.to_string())?;
    control.online().await;
    // The control is reached by its relay alone, as a phone on cellular mostly is.
    let relay = control
        .addr()
        .relay_urls()
        .next()
        .cloned()
        .ok_or("the control has no relay")?;
    let control_at = EndpointAddr::new(control.id()).with_relay_url(relay.clone());
    tokio::spawn(answering(control.clone()));
    println!(
        "dialer {} dials phone {phone} every {every:?}, within {within:?}; control {} at {relay}",
        dialer.id(),
        control.id()
    );

    let out = dir.join("dials.jsonl");
    let start = Instant::now();
    let mut slot = 0u64;
    loop {
        let due = start + every * slot as u32;
        tokio::time::sleep_until(due.into()).await;
        // A slot the process was too late for is written, never skipped silently.
        let now = (start.elapsed().as_secs_f64() / every.as_secs_f64()) as u64;
        for owed in slot..now {
            write(&out, json!({"slot": owed, "verdict": "never-ran"})).await;
        }
        slot = slot.max(now);
        let (p, c) = tokio::join!(
            dial(&dialer, EndpointAddr::new(phone), within),
            dial(&dialer, control_at.clone(), within)
        );
        let verdict = match (p["ok"].as_bool(), c["ok"].as_bool()) {
            (Some(true), _) => "answered",
            (_, Some(true)) => "failed",
            _ => "could-not-judge",
        };
        let line = json!({"slot": slot, "t": unix(), "verdict": verdict, "phone": p, "control": c});
        println!("{line}");
        write(&out, line).await;
        slot += 1;
    }
}

/// One dial within `within`: its time to the vitals' line, the path, the vitals; or why not.
async fn dial(from: &Endpoint, to: EndpointAddr, within: Duration) -> Value {
    let id = to.id;
    let began = Instant::now();
    let asked = async {
        let conn = from.connect(to, PROBE).await.map_err(|e| format!("connect: {e}"))?;
        let (mut send, recv) = conn.open_bi().await.map_err(|e| format!("open: {e}"))?;
        send.write_all(b"vitals\n").await.map_err(|e| format!("write: {e}"))?;
        let mut line = String::new();
        BufReader::new(recv)
            .read_line(&mut line)
            .await
            .map_err(|e| format!("read: {e}"))?;
        conn.close(0u32.into(), b"read");
        serde_json::from_str::<Value>(&line).map_err(|e| format!("vitals: {e}: {line:?}"))
    };
    let got = tokio::time::timeout(within, asked)
        .await
        .unwrap_or_else(|_| Err(format!("no answer within {within:?}")));
    let ms = began.elapsed().as_millis() as u64;
    let paths: Vec<_> = from
        .remote_info(id)
        .await
        .into_iter()
        .flat_map(|i| i.into_addrs())
        .filter(|a| matches!(a.usage(), TransportAddrUsage::Active))
        .collect();
    let path = if paths.iter().any(|a| a.addr().is_ip()) {
        "direct"
    } else if paths.iter().any(|a| a.addr().is_relay()) {
        "relay"
    } else {
        "none"
    };
    match got {
        Ok(vitals) => json!({"ok": true, "ms": ms, "path": path, "vitals": vitals}),
        // A missed dial's vitals are absent, never zero (PHONE_REPLICA §5).
        Err(error) => json!({"ok": false, "ms": ms, "path": path, "error": error}),
    }
}

/// The control's answer: the same one line a phone gives, so both dials do the same work.
async fn answering(endpoint: Endpoint) {
    while let Some(incoming) = endpoint.accept().await {
        tokio::spawn(async move {
            let Ok(conn) = incoming.await else { return };
            let Ok((mut send, recv)) = conn.accept_bi().await else {
                return;
            };
            let _ = BufReader::new(recv).read_line(&mut String::new()).await;
            let _ = send
                .write_all(format!("{}\n", json!({"control": true, "t": unix()})).as_bytes())
                .await;
            let _ = send.finish();
            let _ = tokio::time::timeout(Duration::from_secs(10), conn.closed()).await;
        });
    }
}

/// The dialer's key, kept so the phone's build names the same one each run.
fn kept(path: &Path) -> Result<SecretKey, String> {
    if let Ok(bytes) = std::fs::read(path) {
        let bytes: [u8; 32] = bytes
            .try_into()
            .map_err(|_| format!("{} is not 32 bytes", path.display()))?;
        return Ok(SecretKey::from_bytes(&bytes));
    }
    let key = SecretKey::generate();
    std::fs::write(path, key.to_bytes()).map_err(|e| e.to_string())?;
    Ok(key)
}

fn unix() -> u64 {
    SystemTime::now().duration_since(UNIX_EPOCH).map_or(0, |d| d.as_secs())
}

async fn write(out: &Path, line: Value) {
    let file = tokio::fs::OpenOptions::new().create(true).append(true).open(out).await;
    if let Ok(mut f) = file {
        let _ = f.write_all(format!("{line}\n").as_bytes()).await;
    }
}
