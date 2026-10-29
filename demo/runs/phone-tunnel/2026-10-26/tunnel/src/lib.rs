// SPDX-License-Identifier: AGPL-3.0-or-later
//! The phone's door, a spike (../DESIGN.md §4): the node's own member half
//! of a TCP lend, served on the phone's 127.0.0.1 from inside a packet
//! tunnel, and every 10 s a sample of its memory.

use std::ffi::{CStr, c_char};
use std::io::Write;
use std::net::Ipv4Addr;
use std::path::{Path, PathBuf};
use std::sync::Mutex;
use std::sync::atomic::{AtomicU64, Ordering};
use std::time::{Duration, SystemTime, UNIX_EPOCH};

use iroh::endpoint::TransportAddrUsage;
use serde::{Deserialize, Serialize};
use svrngs_node::door::tcp::{Carried, carried};
use svrngs_node::iroh::{Endpoint, EndpointAddr, PublicKey, RelayConfig, SecretKey, build_relayed_endpoint};
use svrngs_node::lend::LendName;
use tokio::io::{AsyncBufReadExt, BufReader};
use tokio::sync::oneshot;
use tokio::time::{MissedTickBehavior, interval};
use tracing::debug;
use tracing_subscriber::{filter::Targets, prelude::*};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Plant {
    None,
    Stranger,
    Leak,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Config {
    #[serde(rename = "seed_hex", deserialize_with = "hex::serde::deserialize")]
    seed: [u8; 32],
    lender: PublicKey,
    lend: LendName,
    port: u16,
    pub plant: Plant,
    log_dir: PathBuf,
    /// The one key the probe answers (PHONE_REPLICA §5, phase 1); none, no probe.
    #[serde(default)]
    probe: Option<PublicKey>,
}

/// What a probe dial asks on: the phone's vitals, one JSON line.
pub const PROBE: &[u8] = b"svrngs/probe/0";

#[derive(Serialize)]
#[serde(rename_all = "lowercase")]
enum Way {
    Direct,
    Relay,
    None,
}

/// What `tunnel_start` returns. `Unlogged`: no `log_dir`, or no trace file in it, so nothing else could be said.
#[repr(i32)]
enum Started {
    Running = 0,
    Unlogged = 1,
    Refused = 2,
    Already = 3,
}

// The extension's sleeps and wakes as iOS tells its provider, counted for the probe.
static SLEPT: AtomicU64 = AtomicU64::new(0);
static WOKE: AtomicU64 = AtomicU64::new(0);

// The C caller holds no handle, so the process holds the one tunnel.
static RUNNING: Mutex<Option<(oneshot::Sender<()>, std::thread::JoinHandle<()>)>> = Mutex::new(None);

unsafe extern "C" {
    static mach_task_self_: u32;
    fn task_info(task: u32, flavor: u32, out: *mut u64, count: *mut u32) -> i32;
    #[cfg(target_os = "ios")]
    fn os_proc_available_memory() -> usize;
}

/// # Safety
/// `config_json` is null or a NUL-terminated string.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn tunnel_start(config_json: *const c_char) -> i32 {
    let raw = (!config_json.is_null()).then(|| unsafe { CStr::from_ptr(config_json) }.to_string_lossy());
    started(raw.as_deref().unwrap_or("")) as i32
}

#[unsafe(no_mangle)]
pub extern "C" fn tunnel_stop() {
    let running = RUNNING.lock().unwrap_or_else(|e| e.into_inner()).take();
    debug!(target: "tunnel", running = running.is_some(), "stop: the runtime told to stop and joined, if one ran");
    if let Some((stop, thread)) = running {
        let _ = stop.send(());
        let _ = thread.join();
    }
}

#[unsafe(no_mangle)]
pub extern "C" fn tunnel_slept() {
    let n = SLEPT.fetch_add(1, Ordering::Relaxed) + 1;
    debug!(target: "tunnel", n, "slept: iOS put the provider to sleep");
}

#[unsafe(no_mangle)]
pub extern "C" fn tunnel_woke() {
    let n = WOKE.fetch_add(1, Ordering::Relaxed) + 1;
    debug!(target: "tunnel", n, "woke: iOS woke the provider");
}

pub fn parsed(raw: &str) -> Result<Config, serde_json::Error> {
    serde_json::from_str(raw)
}

fn started(raw: &str) -> Started {
    #[derive(Deserialize)]
    struct Logged {
        log_dir: PathBuf,
    }
    // The log directory first: it is where every refusal after it is said.
    let Some(Logged { log_dir }) = serde_json::from_str(raw).ok().filter(|l: &Logged| traced(&l.log_dir)) else {
        return Started::Unlogged;
    };
    let config = match parsed(raw) {
        Ok(config) => config,
        Err(e) => {
            debug!(target: "tunnel", error = %e, ?log_dir, "refused: the config does not parse");
            return Started::Refused;
        }
    };
    let mut running = RUNNING.lock().unwrap_or_else(|e| e.into_inner());
    if running.is_some() {
        debug!(target: "tunnel", "refused: a tunnel already runs in this process");
        return Started::Already;
    }
    debug!(target: "tunnel", plant = ?config.plant, lender = %config.lender, lend = %config.lend, port = config.port, "start: a runtime on a thread of its own");
    let (stop, stopped) = oneshot::channel();
    let thread = std::thread::spawn(
        move || match tokio::runtime::Builder::new_multi_thread().enable_all().build() {
            Ok(runtime) => runtime.block_on(run(config, stopped)),
            Err(e) => debug!(target: "tunnel", error = %e, "stopped: no runtime"),
        },
    );
    *running = Some((stop, thread));
    Started::Running
}

/// The `tunnel` and `door` traces at debug, to `trace.log`; false if the file cannot be opened there.
fn traced(log_dir: &Path) -> bool {
    let file = std::fs::create_dir_all(log_dir).and_then(|()| {
        std::fs::OpenOptions::new()
            .create(true)
            .append(true)
            .open(log_dir.join("trace.log"))
    });
    let Ok(file) = file else { return false };
    let targets = Targets::new()
        .with_target("tunnel", tracing::Level::DEBUG)
        .with_target("door", tracing::Level::DEBUG)
        // What iroh does in the 2.9 s iOS grants each wake (../../phone-replica/2026-10-27/README.md, "Diagnosis").
        .with_target("iroh", tracing::Level::DEBUG);
    let layer = tracing_subscriber::fmt::layer()
        .with_ansi(false)
        .with_writer(Mutex::new(file));
    let set = tracing::subscriber::set_global_default(tracing_subscriber::registry().with(layer.with_filter(targets)));
    debug!(target: "tunnel", first = set.is_ok(), "traced: a second start in one process keeps the first start's file");
    true
}

async fn run(config: Config, mut stopped: oneshot::Receiver<()>) {
    let key = match config.plant {
        Plant::Stranger => SecretKey::generate(),
        Plant::None | Plant::Leak => SecretKey::from_bytes(&config.seed),
    };
    debug!(target: "tunnel", key = %key.public(), plant = ?config.plant, "the endpoint's key: the seed's, or under stranger one no group holds");
    let alpns = config.probe.map(|_| vec![PROBE.to_vec()]).unwrap_or_default();
    let endpoint = match build_relayed_endpoint(key, alpns, &RelayConfig::default()).await {
        Ok(endpoint) => endpoint,
        Err(e) => return debug!(target: "tunnel", error = e, "stopped: no endpoint"),
    };
    let born = std::time::Instant::now();
    let _probe = config.probe.map(|dialer| tokio::spawn(answering(endpoint.clone(), dialer, born)));
    let (mut lend, mut leaked) = (None::<Carried>, Vec::new());
    let (mut sample, mut leak) = (interval(Duration::from_secs(10)), interval(Duration::from_secs(1)));
    sample.set_missed_tick_behavior(MissedTickBehavior::Delay);
    loop {
        tokio::select! {
            _ = &mut stopped => break,
            _ = sample.tick() => {
                let open = lend.as_ref().is_some_and(|(_, task)| !task.is_finished());
                sampled(&endpoint, &config, open).await;
                debug!(target: "tunnel", open, probe = config.probe.is_some(), "the lend open, or opened again now: a lender closing it or not yet up is tried each sample; none under the probe");
                // Under the probe the phone only holds its endpoint, so nothing of the door's wakes it (PHONE_REPLICA §5).
                if !open && config.probe.is_none() { lend = opened(&endpoint, &config).await }
            }
            Some(words) = async { match &mut lend { Some((told, _)) => told.recv().await, None => std::future::pending().await } } => {
                debug!(target: "tunnel", words, "the lender refused a program's connection");
            }
            _ = leak.tick(), if config.plant == Plant::Leak => {
                // Written, not zeroed: a zeroed page is never touched, so it would not count.
                leaked.push(vec![0xA5u8; 1 << 20]);
                debug!(target: "tunnel", mib = leaked.len(), "leak: 1 MiB more, touched and kept");
            }
        }
    }
    drop(lend);
    endpoint.close().await;
    debug!(target: "tunnel", "stopped: the lend dropped and the endpoint closed");
}

/// Each probe dial from `dialer` answered with one line of vitals; any other key closed unanswered.
async fn answering(endpoint: Endpoint, dialer: PublicKey, born: std::time::Instant) {
    while let Some(incoming) = endpoint.accept().await {
        tokio::spawn(async move {
            let conn = match incoming.await {
                Ok(conn) => conn,
                Err(e) => return debug!(target: "tunnel", error = %e, "probe: a dial's handshake failed"),
            };
            if conn.remote_id() != dialer {
                debug!(target: "tunnel", from = %conn.remote_id(), "probe: a dial from a key not the dialer's, closed unanswered");
                return conn.close(0u32.into(), b"not admitted");
            }
            let vitals = serde_json::json!({
                "t": SystemTime::now().duration_since(UNIX_EPOCH).ok().map(|d| d.as_secs()),
                "up_s": born.elapsed().as_secs(), "footprint": footprint(), "available": available(),
                "slept": SLEPT.load(Ordering::Relaxed), "woke": WOKE.load(Ordering::Relaxed),
            });
            let answered = async {
                let (mut send, recv) = conn.accept_bi().await?;
                BufReader::new(recv).read_line(&mut String::new()).await?;
                send.write_all(format!("{vitals}\n").as_bytes()).await?;
                send.finish()?;
                Ok::<_, std::io::Error>(())
            };
            let answered = tokio::time::timeout(Duration::from_secs(10), answered).await;
            debug!(target: "tunnel", ?answered, "probe: the dialer's dial answered with vitals, or not");
            // The dialer closes once it has read the line; waiting for that lets the line arrive.
            let _ = tokio::time::timeout(Duration::from_secs(10), conn.closed()).await;
        });
    }
}

async fn opened(endpoint: &Endpoint, config: &Config) -> Option<Carried> {
    let local = match tokio::net::TcpListener::bind((Ipv4Addr::LOCALHOST, config.port)).await {
        Ok(local) => local,
        Err(e) => {
            debug!(target: "tunnel", port = config.port, error = %e, "not open: another program holds the port");
            return None;
        }
    };
    let carried = carried(endpoint, EndpointAddr::new(config.lender), &config.lend, local).await;
    debug!(target: "tunnel", lend = %config.lend, port = config.port, refused = ?carried.as_ref().err(), "the lend opened on 127.0.0.1 and carried to its lender, or not");
    carried.ok()
}

async fn sampled(endpoint: &Endpoint, config: &Config, open: bool) {
    let s = &endpoint.metrics().socket;
    let info = endpoint.remote_info(config.lender).await;
    let paths: Vec<_> = info
        .into_iter()
        .flat_map(|i| i.into_addrs())
        .filter(|a| matches!(a.usage(), TransportAddrUsage::Active))
        .collect();
    // A relay path is kept open beside a direct one (iroh-1.0.2 src/socket/remote_map/remote_state.rs:705),
    // so any open IP path is direct.
    let path = match (
        paths.iter().any(|a| a.addr().is_ip()),
        paths.iter().any(|a| a.addr().is_relay()),
    ) {
        (true, _) => Way::Direct,
        (_, true) => Way::Relay,
        _ => Way::None,
    };
    let t = SystemTime::now().duration_since(UNIX_EPOCH).ok().map(|d| d.as_secs());
    let sample = serde_json::json!({
        "t": t, "footprint": footprint(), "available": available(), "open": open, "path": path, "plant": config.plant,
        // Every QUIC byte the endpoint sent and took in, over IP and relay: it holds the one connection.
        "slept": SLEPT.load(Ordering::Relaxed), "woke": WOKE.load(Ordering::Relaxed),
        "up": s.send_ipv4.get() + s.send_ipv6.get() + s.send_relay.get(),
        "down": s.recv_data_ipv4.get() + s.recv_data_ipv6.get() + s.recv_data_relay.get(),
    });
    // Opened, written and closed each time, unbuffered, so the last sample before a kill is on disk (P5).
    let file = std::fs::OpenOptions::new()
        .create(true)
        .append(true)
        .open(config.log_dir.join("samples.jsonl"));
    let written = file.and_then(|mut f| f.write_all(format!("{sample}\n").as_bytes()));
    debug!(target: "tunnel", error = ?written.err(), "sampled, or the sample not written");
}

/// `phys_footprint`, the number the memory killer counts: the 19th u64 of `task_vm_info`, its rev1 (mach/task_info.h: TASK_VM_INFO 22, TASK_VM_INFO_REV1_COUNT 38).
fn footprint() -> Option<u64> {
    let (mut info, mut count) = ([0u64; 19], 38u32);
    let kr = unsafe { task_info(mach_task_self_, 22, info.as_mut_ptr(), &mut count) };
    (kr == 0 && count == 38).then_some(info[18])
}

/// The bytes left under the limit. The OS's 0 means "not an app" as well as "over it" (os/proc.h), so it is null; on a Mac there is none.
fn available() -> Option<u64> {
    #[cfg(target_os = "ios")]
    return Some(unsafe { os_proc_available_memory() } as u64).filter(|&n| n > 0);
    #[cfg(not(target_os = "ios"))]
    None
}

#[cfg(test)]
mod tests {
    use super::*;

    fn config(plant: &str, seed: &str) -> String {
        let key = SecretKey::generate().public();
        format!(
            r#"{{"seed_hex":"{seed}","lender":"{key}","lend":"jellyfin","port":8096,"plant":"{plant}","log_dir":"/tmp/x"}}"#
        )
    }

    #[test]
    fn each_plant_parses_and_a_stray_one_is_refused() {
        let seed = "00".repeat(32);
        assert_eq!(parsed(&config("none", &seed)).unwrap().plant, Plant::None);
        assert_eq!(parsed(&config("stranger", &seed)).unwrap().plant, Plant::Stranger);
        assert_eq!(parsed(&config("leak", &seed)).unwrap().plant, Plant::Leak);
        assert!(parsed(&config("Leak", &seed)).is_err());
    }

    #[test]
    fn a_short_seed_or_a_stray_field_is_refused() {
        assert!(parsed(&config("none", &"00".repeat(31))).is_err());
        assert!(parsed(&config("none", &"00".repeat(32)).replace("\"port\"", "\"prot\"")).is_err());
    }

    #[test]
    fn the_footprint_is_the_one_proc_pid_rusage_reads() {
        let mut ru: libc::rusage_info_v0 = unsafe { std::mem::zeroed() };
        let read =
            unsafe { libc::proc_pid_rusage(std::process::id() as i32, libc::RUSAGE_INFO_V0, (&raw mut ru).cast()) };
        let (ours, theirs) = (footprint().unwrap(), ru.ri_phys_footprint);
        assert_eq!(read, 0);
        assert!(
            ours.abs_diff(theirs) < 1 << 20,
            "task_info {ours}, proc_pid_rusage {theirs}"
        );
    }

    #[test]
    fn the_probe_is_off_unless_a_dialer_is_named() {
        let seed = "00".repeat(32);
        assert!(parsed(&config("none", &seed)).unwrap().probe.is_none());
        let dialer = SecretKey::generate().public();
        let named = config("none", &seed).replace("}", &format!(r#","probe":"{dialer}"}}"#));
        assert_eq!(parsed(&named).unwrap().probe, Some(dialer));
    }

    #[test]
    fn a_config_without_a_log_dir_is_unlogged() {
        assert!(matches!(started("{}"), Started::Unlogged));
    }
}
