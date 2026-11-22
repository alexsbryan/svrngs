//! svrngs's record, as give reaches it: the loopback API of the svrngs
//! running on this computer, asked as its person (CHARTER fence 4: the
//! record API, and nothing else of svrngs's) with a code its token makes
//! for one connection ([`ask`]), never the token, and only once svrngs has
//! proved on it that it holds it ([`proof`]). Where it listens is the `api` file in
//! svrngs's data dir and the token is the `token` file beside it, the data
//! dir being `$SVRNGS_DIR`, else `~/.svrngs`, as svrngs's own command reads
//! it (`svrngs --help`; apps/events/store.mjs reads the same two files).
//!
//! Give's acts are `give/<kind>` in the record. [`gives`] takes them out of
//! the log with the prefix off, the shape the carried page reads at
//! `svrngs/acts`, so the fold reads the same acts from either.

use std::io::{BufRead, BufReader, Read, Write};
use std::net::{SocketAddr, TcpStream};
use std::path::{Path, PathBuf};
use std::sync::Arc;
use std::time::{Duration, Instant};

use serde_json::{Value, json};

/// Give's part of the record, and of what a carried app called `give` reads.
pub const PREFIX: &str = "give/";

#[derive(Debug, thiserror::Error)]
pub enum RecordError {
    #[error("svrngs is not set up on this computer: {path} cannot be read ({why}). Install svrngs and run it first")]
    NotSetUp { path: PathBuf, why: String },
    #[error("svrngs on this computer did not answer at {api}: {why}. Is it running?")]
    Unanswered { api: String, why: String },
    #[error(
        "Something other than svrngs answers at {api}, where svrngs listens, so give sent it none of your token. Stop that program (`lsof -nP -iTCP -sTCP:LISTEN` on a Mac, or `ss -ltnp` on Linux, names each program listening), then start svrngs again with `svrngs restart`"
    )]
    Unproven { api: String },
    #[error("the svrngs running here is older than this give: run `svrngs restart`, then try again")]
    Behind { api: String },
    #[error("svrngs on this computer refused {route}: {status} {said}")]
    Refused { route: String, status: u16, said: String },
    #[error("svrngs on this computer answered {route} with what give cannot read: {why}")]
    Unreadable { route: String, why: String },
}

/// A group this computer holds, as `/v1/groups` says it.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Group {
    /// What every route names it by.
    pub id: String,
    pub name: String,
    /// The group's synced folder here, if it keeps one.
    pub folder: Option<PathBuf>,
}

/// This computer's key, and the groups it holds.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Groups {
    pub key: String,
    pub groups: Vec<Group>,
}

/// What carries a call to svrngs's API: HTTP on this computer's loopback,
/// or a test's stand-in that answers as svrngs does (apps/give/tests/fake).
pub trait Answers: std::fmt::Debug + Send + Sync {
    /// `route` asked, `body` posted if there is one: the status and the
    /// text answered, or why nothing was.
    fn call(&self, route: &str, body: Option<&Value>) -> Result<(u16, String), RecordError>;
}

/// svrngs's API at `api`, asked with a code its token makes.
struct Loopback {
    api: String,
    token: String,
    /// How long svrngs has to answer: [`WITHIN`], but in a test.
    within: Duration,
}

/// Where svrngs listens, and never the token, wherever Give's record is
/// printed: a trace's, a panic's.
impl std::fmt::Debug for Loopback {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Loopback")
            .field("api", &self.api)
            .finish_non_exhaustive()
    }
}

impl Answers for Loopback {
    /// `route` asked on one connection, and only once svrngs has proved on
    /// it that it holds the token: a fresh challenge posted to `/v1/proof`,
    /// its answer checked against this connection's own two ends
    /// ([`proof`]), then the ask on that connection with the code made for
    /// the nonce svrngs gave it ([`ask`]). The token is sent nowhere, and
    /// what can't prove it holds it is sent no code either: a program
    /// holding svrngs's address while svrngs is down, of any account, or
    /// one carrying the challenge to svrngs on a connection of its own.
    #[expect(
        clippy::disallowed_methods,
        reason = "dial: svrngs's own API, proved before it is asked, by each answer's deadline"
    )]
    fn call(&self, route: &str, body: Option<&Value>) -> Result<(u16, String), RecordError> {
        let api = self.api.clone();
        let unanswered = |why: String| RecordError::Unanswered { api: api.clone(), why };
        let at: SocketAddr =
            (self.api.trim_start_matches("http://").parse()).map_err(|_| unanswered("that names no address".into()))?;
        let stream = TcpStream::connect_timeout(&at, self.within).map_err(|e| unanswered(e.to_string()))?;
        let ends = (stream.local_addr(), stream.peer_addr());
        let (Ok(client), Ok(server)) = ends else {
            return Err(unanswered("this connection's ends are unknown".into()));
        };
        (stream.set_write_timeout(Some(self.within))).map_err(|e| unanswered(e.to_string()))?;
        let mut on = BufReader::new(Line {
            stream,
            by: Instant::now() + self.within,
        });
        let challenge = challenge().ok_or_else(|| unanswered("no randomness here to challenge it with".into()))?;
        let asked = json!({ "challenge": challenge }).to_string();
        let (status, said) =
            exchange(&mut on, at, ("POST", "/v1/proof", None), Some(&asked)).map_err(|e| match e.kind() {
                std::io::ErrorKind::InvalidData => RecordError::Unproven { api: api.clone() },
                _ => unanswered(e.to_string()),
            })?;
        let said: Value = serde_json::from_str(&said).unwrap_or_default();
        let wanted = proof(&self.token, &challenge, client, server);
        let proved = status == 200 && same(said["proof"].as_str().unwrap_or_default().as_bytes(), wanted.as_bytes());
        // A nonce as svrngs draws it; a proof with none is a svrngs from
        // before it stopped taking the token.
        let nonce = said["nonce"]
            .as_str()
            .filter(|n| n.len() == 64 && n.bytes().all(|b| matches!(b, b'0'..=b'9' | b'a'..=b'f')));
        tracing::debug!(api, %client, status, proved, nonce = nonce.is_some(), "record: whether svrngs proved it holds the token on this connection and gave it a nonce, asked on it only if so");
        let code = match (proved, nonce, status) {
            (true, Some(nonce), _) => ask(&self.token, nonce, client, server),
            (true, None, _) | (false, _, 404) => return Err(RecordError::Behind { api }),
            (false, ..) => return Err(RecordError::Unproven { api }),
        };
        let method = if body.is_some() { "POST" } else { "GET" };
        let body = body.map(Value::to_string);
        on.get_mut().by = Instant::now() + self.within;
        exchange(&mut on, at, (method, route, Some(&code)), body.as_deref()).map_err(|e| unanswered(e.to_string()))
    }
}

/// `method` `route` asked on `on`, with `code` if there is one, `body`
/// posted if there is one, as HTTP/1.1 to svrngs at `at`: the status and
/// the text answered, by its length, which svrngs gives every answer. Its
/// head is read no further than [`HEAD_WITHIN`], and the proof's answer,
/// asked before anything has proved itself, no further than
/// [`PROVED_WITHIN`], so what holds svrngs's address can't fill Give's
/// memory (the 1.13.3 proof review, finding 7): past either, it is
/// unreadable.
fn exchange(
    on: &mut BufReader<Line>,
    at: SocketAddr,
    (method, route, code): (&str, &str, Option<&str>),
    body: Option<&str>,
) -> std::io::Result<(u16, String)> {
    let shown = code.map_or(String::new(), |c| format!("Authorization: Svrngs {c}\r\n"));
    let body = body.unwrap_or_default();
    let typed = if body.is_empty() {
        ""
    } else {
        "Content-Type: application/json\r\n"
    };
    let asked = format!(
        "{method} {route} HTTP/1.1\r\nHost: {at}\r\n{shown}{typed}Content-Length: {}\r\n\r\n{body}",
        body.len()
    );
    on.get_mut().stream.write_all(asked.as_bytes())?;
    let unreadable = |why: &str| std::io::Error::new(std::io::ErrorKind::InvalidData, why.to_string());
    let mut left = HEAD_WITHIN;
    let mut line = || {
        let mut line = String::new();
        let read = on.by_ref().take(left).read_line(&mut line)?;
        left -= read as u64;
        match (read, line.ends_with('\n')) {
            (0, _) => Err(std::io::Error::from(std::io::ErrorKind::UnexpectedEof)),
            (_, false) => Err(unreadable("its head is longer than svrngs's")),
            _ => Ok(line.trim_end().to_string()),
        }
    };
    let status = line()?
        .split(' ')
        .nth(1)
        .and_then(|s| s.parse().ok())
        .ok_or_else(|| unreadable("its answer is not HTTP"))?;
    let mut length = 0;
    loop {
        let header = line()?;
        if header.is_empty() {
            break;
        }
        if let Some((name, value)) = header
            .split_once(':')
            .filter(|(n, _)| n.eq_ignore_ascii_case("content-length"))
        {
            length = value
                .trim()
                .parse()
                .map_err(|_| unreadable(&format!("its {name} is not a number")))?;
        }
    }
    if code.is_none() && length > PROVED_WITHIN {
        return Err(unreadable("its answer is longer than a proof"));
    }
    let mut said = Vec::new();
    on.by_ref().take(length).read_to_end(&mut said)?;
    if said.len() as u64 != length {
        return Err(std::io::ErrorKind::UnexpectedEof.into());
    }
    Ok((status, String::from_utf8_lossy(&said).into_owned()))
}

/// Give's one connection to svrngs, each read waiting only what is left
/// before `by`, the one deadline of an answer, its head and its body (the
/// never-send-the-key review, finding 5): so a peer giving the proof a
/// byte at a time, each in time, is still refused at the deadline.
struct Line {
    stream: TcpStream,
    by: Instant,
}

impl Read for Line {
    #[expect(clippy::disallowed_methods, reason = "what is left of an answer's deadline")]
    fn read(&mut self, buf: &mut [u8]) -> std::io::Result<usize> {
        let left = self.by.saturating_duration_since(Instant::now());
        // A read timeout of zero is refused: nothing is left, so none is waited.
        let left = Some(left)
            .filter(|l| !l.is_zero())
            .ok_or(std::io::ErrorKind::TimedOut)?;
        self.stream.set_read_timeout(Some(left))?;
        self.stream.read(buf)
    }
}

/// The most of an answer's head [`exchange`] reads: far past svrngs's.
const HEAD_WITHIN: u64 = 16 << 10;

/// The most of the proof's answer [`exchange`] reads, as the Omarchy
/// helper reads it: far past svrngs's, some 90 bytes.
const PROVED_WITHIN: u64 = 4 << 10;

/// svrngs's proof that it holds `token`, for `challenge` on the connection
/// from `client` to `server`: HMAC-SHA256 under the token, trimmed, over
/// `svrngs proof v1\n<challenge>\n<client>\n<server>`, the challenge as 64
/// lowercase hex characters and each end as Rust prints a `SocketAddr`
/// (`127.0.0.1:50000`, `[::1]:50001`), as 64 lowercase hex characters.
/// Give's copy of svrngs's one decider of it (crates/svrngs-node/src/api.rs
/// `proof`), as give links no svrngs crate (tests/surface.rs); each held
/// to the same vectors (tests/record.rs, api_tests.rs).
pub fn proof(token: &str, challenge: &str, client: SocketAddr, server: SocketAddr) -> String {
    mac(token, &format!("svrngs proof v1\n{challenge}\n{client}\n{server}"))
}

/// The code Give asks svrngs with on the connection from `client` to
/// `server`, for the `nonce` svrngs's proof gave it: HMAC-SHA256 under the
/// token, trimmed, over `svrngs ask v1\n<nonce>\n<client>\n<server>`, as
/// [`proof`] prints each part, sent as `Authorization: Svrngs <code>`.
/// Give's copy of svrngs's one decider of it (crates/svrngs-node/src/api.rs
/// `ask`), held to the same vectors (tests/record.rs, api_tests.rs).
pub fn ask(token: &str, nonce: &str, client: SocketAddr, server: SocketAddr) -> String {
    mac(token, &format!("svrngs ask v1\n{nonce}\n{client}\n{server}"))
}

/// `message`'s HMAC-SHA256 under `token`, trimmed, in lowercase hex.
fn mac(token: &str, message: &str) -> String {
    let key = ring::hmac::Key::new(ring::hmac::HMAC_SHA256, token.trim().as_bytes());
    hex(ring::hmac::sign(&key, message.as_bytes()).as_ref())
}

/// A fresh challenge for one connection: 32 random bytes as lowercase hex.
#[expect(clippy::disallowed_methods, reason = "draws a proof's challenge")]
fn challenge() -> Option<String> {
    let mut drawn = [0u8; 32];
    getrandom::fill(&mut drawn).ok()?;
    Some(hex(&drawn))
}

fn hex(bytes: &[u8]) -> String {
    bytes.iter().map(|b| format!("{b:02x}")).collect()
}

/// Whether `a` is `b`, in a time that doesn't tell where they differ.
fn same(a: &[u8], b: &[u8]) -> bool {
    a.len() == b.len() && a.iter().zip(b).fold(0, |acc, (x, y)| acc | (x ^ y)) == 0
}

/// svrngs's loopback API on this computer.
#[derive(Debug, Clone)]
pub struct Record {
    answers: Arc<dyn Answers>,
}

/// svrngs's data dir, as its command reads it.
pub fn data_dir() -> Option<PathBuf> {
    let set = std::env::var_os("SVRNGS_DIR").filter(|d| !d.is_empty());
    set.map(PathBuf::from)
        .or_else(|| std::env::var_os("HOME").map(|h| PathBuf::from(h).join(".svrngs")))
}

fn read(path: &Path) -> Result<String, RecordError> {
    let text = std::fs::read_to_string(path).map_err(|e| RecordError::NotSetUp {
        path: path.to_path_buf(),
        why: e.to_string(),
    })?;
    Ok(text.trim().to_string())
}

impl Record {
    /// The record of the svrngs set up in `data_dir`.
    pub fn at(data_dir: &Path) -> Result<Record, RecordError> {
        Record::within(data_dir, WITHIN)
    }

    /// [`Record::at`], svrngs given `within` to answer each call, not
    /// [`WITHIN`]: a test's shorter clock.
    pub fn within(data_dir: &Path, within: Duration) -> Result<Record, RecordError> {
        let (api, token) = (read(&data_dir.join("api"))?, read(&data_dir.join("token"))?);
        tracing::debug!(api, data_dir = %data_dir.display(), "record: where svrngs listens, and its token read");
        Ok(Record::by(Arc::new(Loopback { api, token, within })))
    }

    /// The record `answers` carries calls to, its refusals naming where.
    pub fn by(answers: Arc<dyn Answers>) -> Record {
        Record { answers }
    }

    fn call(&self, route: &str, body: Option<&Value>) -> Result<Value, RecordError> {
        let (status, text) = self.answers.call(route, body)?;
        let path = route.split('?').next().unwrap_or(route).to_string();
        if !(200..300).contains(&status) {
            tracing::debug!(route = path, status, said = text, "record: refused");
            return Err(RecordError::Refused {
                route: path,
                status,
                said: text,
            });
        }
        serde_json::from_str(&text).map_err(|e| RecordError::Unreadable {
            route: path,
            why: e.to_string(),
        })
    }

    /// This computer's key and the groups it holds.
    pub fn groups(&self) -> Result<Groups, RecordError> {
        let read = self.call("/v1/groups", None)?;
        let unreadable = |why: &str| RecordError::Unreadable {
            route: "/v1/groups".into(),
            why: why.into(),
        };
        let key = read["key"].as_str().ok_or_else(|| unreadable("no key"))?.to_string();
        let groups = (read["groups"].as_array().ok_or_else(|| unreadable("no groups"))?.iter())
            .filter_map(|g| {
                let folder = g["folders"]
                    .as_array()
                    .and_then(|f| f.first()?.as_str().map(PathBuf::from));
                let name = g["name"].as_str().unwrap_or_default().to_string();
                Some(Group {
                    id: g["id"].as_str()?.to_string(),
                    name,
                    folder,
                })
            })
            .collect();
        Ok(Groups { key, groups })
    }

    /// Give's acts in `group`, in the record's order, the prefix off.
    pub fn acts(&self, group: &str) -> Result<Vec<Value>, RecordError> {
        let read = self.call(&format!("/v1/rail/log?namespace={}", query(group)), None)?;
        let ops = read["ops"].as_array().ok_or_else(|| RecordError::Unreadable {
            route: "/v1/rail/log".into(),
            why: "no ops".into(),
        })?;
        Ok(gives(ops))
    }

    /// `payload`, whose `kind` is give's own without the prefix, written to
    /// `group`'s record as this computer's person.
    pub fn write(&self, group: &str, mut payload: Value) -> Result<(), RecordError> {
        let kind = payload["kind"].as_str().unwrap_or_default();
        payload["kind"] = format!("{PREFIX}{kind}").into();
        let route = format!("/v1/rail/append?namespace={}", query(group));
        self.call(&route, Some(&json!({ "op": "record", "payload": payload })))?;
        tracing::debug!(group, kind = %payload["kind"], "record: written");
        Ok(())
    }
}

/// How long svrngs has to answer one call on this computer.
const WITHIN: Duration = Duration::from_secs(10);

/// A group's id, as a query's value: ids are hex, so this only guards.
fn query(group: &str) -> String {
    group
        .bytes()
        .map(|b| match b {
            b'a'..=b'z' | b'A'..=b'Z' | b'0'..=b'9' | b'-' | b'_' | b'.' => (b as char).to_string(),
            _ => format!("%{b:02X}"),
        })
        .collect()
}

/// Give's acts among a group's `ops`, as `/v1/rail/log` gives them, the
/// prefix taken off each act's kind, and each correction voiding one of
/// them: what svrngs's door gives a carried app called `give` at its
/// `svrngs/acts` (crates/svrngs-node/src/carried.rs, `acts`).
pub fn gives(ops: &[Value]) -> Vec<Value> {
    let own = |o: &Value| o["payload"]["kind"].as_str().is_some_and(|k| k.starts_with(PREFIX));
    let mine: std::collections::BTreeSet<_> = ops.iter().filter(|o| own(o)).filter_map(|o| o["id"].as_str()).collect();
    let voids_mine = |o: &Value| o["corrects"].as_str().is_some_and(|id| mine.contains(id));
    (ops.iter().filter(|o| own(o) || voids_mine(o)))
        .map(|o| {
            let mut o = o.clone();
            if let Some(kind) = o["payload"]["kind"].as_str().and_then(|k| k.strip_prefix(PREFIX)) {
                o["payload"]["kind"] = kind.to_string().into();
            }
            o
        })
        .collect()
}
