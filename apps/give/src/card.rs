//! A card: an app's own `compose.yaml` with an `x-svrngs` block (GIVE.md §4),
//! read once into what give runs. A value of [`Card`] is a card give can run:
//! every field it holds was read, and every field it could not run was
//! refused, by name, before anything started.
//!
//! The card is a subset of compose. A field give does not know is refused
//! rather than dropped, so an app never runs differently from its file
//! without its author hearing why. Five are refused for what they would let
//! an app do to the computer that keeps it: `build`, `privileged`, a custom
//! network, host networking, and a bind mount from the host.

use std::collections::BTreeMap;
use std::time::Duration;

use serde_yaml::{Mapping, Value};

/// What give runs: the app's services, its volumes, and the five fields of
/// its `x-svrngs` block.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Card {
    /// The app's name on the shelf.
    pub title: String,
    /// One line for the shelf.
    pub about: String,
    pub door: Door,
    /// The named volumes that are the app's state, and so snapshotted.
    pub state: Vec<String>,
    /// Every named volume the card declares, state or not.
    pub volumes: Vec<String>,
    pub asks: Vec<Ask>,
    /// In the order the file gives them, which is the order they start.
    pub services: Vec<Service>,
    /// The file's bytes, hashed (blake3, hex): each snapshot names the
    /// version that wrote it, and a copy is started only by that version
    /// (GIVE.md §4, "Versions and drift").
    pub version: String,
}

/// Which published port is the app's door, and how it is spoken.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Door {
    /// The port inside the app.
    pub port: u16,
    pub protocol: Protocol,
}

/// The three a door can be (GIVE.md §4).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Protocol {
    Http,
    Tcp,
    Udp,
}

impl Protocol {
    /// The transport podman publishes it on.
    pub fn transport(self) -> Transport {
        match self {
            Protocol::Http | Protocol::Tcp => Transport::Tcp,
            Protocol::Udp => Transport::Udp,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Transport {
    Tcp,
    Udp,
}

impl Transport {
    pub fn as_str(self) -> &'static str {
        match self {
            Transport::Tcp => "tcp",
            Transport::Udp => "udp",
        }
    }
}

/// A question the app's terms put to whoever runs it, and what its yes sets.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Ask {
    pub say: String,
    pub link: String,
    /// The environment a yes sets, in every service.
    pub sets: Vec<(String, String)>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Service {
    pub name: String,
    pub image: Image,
    pub environment: Vec<(String, String)>,
    pub mounts: Vec<Mount>,
    /// The ports inside the app its `ports` lists. Only the door is
    /// published, on this computer's loopback, whatever host port the file
    /// gives; the rest are for `podman compose up` as the author tries it.
    pub ports: Vec<(u16, Transport)>,
    pub health: Option<Healthcheck>,
    pub entrypoint: Option<Vec<String>>,
    pub command: Option<Vec<String>>,
    pub user: Option<String>,
    pub working_dir: Option<String>,
    /// `platform`: which of the image's builds to run, `os/arch`, where the
    /// card's author found the one for this computer's own does not run.
    pub platform: Option<String>,
    /// `mem_limit`, in bytes.
    pub memory: Option<u64>,
    /// `stop_grace_period`; compose's default is 10 s.
    pub stop_grace: Duration,
}

/// An image reference pinned by digest, so every computer that keeps the
/// app runs the same bytes.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Image(String);

impl Image {
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

/// A named volume of the card's, or an anonymous one, mounted at `target`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Mount {
    pub volume: Option<String>,
    pub target: String,
    pub read_only: bool,
}

/// Compose's `healthcheck`, with compose's defaults where the card is silent.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Healthcheck {
    pub test: HealthTest,
    pub interval: Duration,
    pub timeout: Duration,
    pub retries: u32,
    pub start_period: Duration,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum HealthTest {
    /// `["CMD", ...]`: run as given.
    Exec(Vec<String>),
    /// `["CMD-SHELL", "..."]` or a string: run by the image's shell.
    Shell(String),
}

/// Why a card was not read: the field, as a path, and what give will not do.
#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
pub enum Refused {
    #[error("the card is not YAML give can read: {0}")]
    Unreadable(String),
    #[error("{field}: {why}")]
    Field { field: String, why: String },
}

impl Card {
    /// Read a card from its `compose.yaml` text.
    pub fn parse(text: &str) -> Result<Card, Refused> {
        let card = read(text).map(|card| Card {
            version: blake3::hash(text.as_bytes()).to_hex().to_string(),
            ..card
        });
        if let Err(Refused::Field { field, why }) = &card {
            tracing::debug!(field, why, "card refused");
        }
        card
    }

    /// The memory every service declares, in bytes; what keeping it needs
    /// beyond podman's own.
    pub fn memory(&self) -> u64 {
        self.services.iter().filter_map(|s| s.memory).sum()
    }
}

fn refuse<T>(field: &str, why: impl Into<String>) -> Result<T, Refused> {
    Err(Refused::Field {
        field: field.to_string(),
        why: why.into(),
    })
}

const NETWORK: &str = "give runs each app on a network of its own, so a custom network is refused";

fn read(text: &str) -> Result<Card, Refused> {
    let root: Value = serde_yaml::from_str(text).map_err(|e| Refused::Unreadable(e.to_string()))?;
    let root = mapping(&root, "the card")?;
    for key in keys(root, "the card")? {
        match key {
            "services" | "volumes" | "x-svrngs" => {}
            // The project's name and compose's old version line say nothing
            // give runs; another `x-` field is an extension compose ignores.
            "name" | "version" => {}
            k if k.starts_with("x-") => {}
            "networks" => return refuse("networks", NETWORK),
            k => return refuse(k, "give does not run this field of compose"),
        }
    }
    let volumes = read_volumes(root.get("volumes"))?;
    let services = mapping(get(root, "services", "the card")?, "services")?;
    if services.is_empty() {
        return refuse("services", "the card names no service");
    }
    let mut out = Vec::new();
    for (name, service) in services {
        let name = string(name, "services")?;
        out.push(read_service(&name, service, &volumes)?);
    }
    let x = mapping(get(root, "x-svrngs", "the card")?, "x-svrngs")?;
    for key in keys(x, "x-svrngs")? {
        if !matches!(key, "title" | "about" | "door" | "state" | "asks") {
            return refuse(
                &format!("x-svrngs.{key}"),
                "x-svrngs has five fields: title, about, door, state and asks",
            );
        }
    }
    let title = string(get(x, "title", "x-svrngs")?, "x-svrngs.title")?;
    let about = string(get(x, "about", "x-svrngs")?, "x-svrngs.about")?;
    let door = read_door(get(x, "door", "x-svrngs")?, &out)?;
    // `state` is never assumed: an app that keeps nothing says `state: []`.
    let state = list(get(x, "state", "x-svrngs")?, "x-svrngs.state")?
        .iter()
        .map(|v| {
            let v = string(v, "x-svrngs.state")?;
            if volumes.contains(&v) {
                Ok(v)
            } else {
                refuse("x-svrngs.state", format!("names {v}, which `volumes` does not declare"))
            }
        })
        .collect::<Result<Vec<_>, _>>()?;
    let asks = match x.get("asks") {
        None => Vec::new(),
        Some(v) => list(v, "x-svrngs.asks")?
            .iter()
            .map(read_ask)
            .collect::<Result<_, _>>()?,
    };
    Ok(Card {
        title,
        about,
        door,
        state,
        volumes,
        asks,
        services: out,
        version: String::new(),
    })
}

fn read_volumes(v: Option<&Value>) -> Result<Vec<String>, Refused> {
    let Some(v) = v else { return Ok(Vec::new()) };
    let mut out = Vec::new();
    for (name, options) in mapping(v, "volumes")? {
        let name = string(name, "volumes")?;
        let empty = match options {
            Value::Null => true,
            Value::Mapping(m) => m.is_empty(),
            _ => false,
        };
        if !empty {
            // A driver or an external volume is the computer's, not the app's.
            return refuse(
                &format!("volumes.{name}"),
                "a volume is declared with no options; give makes and keeps it",
            );
        }
        out.push(name);
    }
    Ok(out)
}

fn read_service(name: &str, v: &Value, volumes: &[String]) -> Result<Service, Refused> {
    let at = format!("services.{name}");
    let m = mapping(v, &at)?;
    for key in keys(m, &at)? {
        let field = format!("{at}.{key}");
        match key {
            "image" | "environment" | "volumes" | "ports" | "healthcheck" | "entrypoint" | "command" | "user"
            | "working_dir" | "platform" | "mem_limit" | "stop_grace_period" => {}
            // give keeps the app running itself, and starts services in the
            // file's order.
            "restart" | "depends_on" => tracing::debug!(field, "not read: give starts and restarts the app itself"),
            "build" => {
                return refuse(
                    &field,
                    "give runs an app's published image, pinned by digest, and never builds one",
                );
            }
            "privileged" if m.get(key) == Some(&Value::Bool(false)) => {}
            "privileged" => return refuse(&field, "a privileged app could act as the computer that keeps it"),
            "network_mode" if m.get(key).and_then(Value::as_str) == Some("host") => {
                return refuse(
                    &field,
                    "host networking would put the app on this computer's own network",
                );
            }
            "network_mode" | "networks" => return refuse(&field, NETWORK),
            _ => return refuse(&field, "give does not run this field of compose"),
        }
    }
    let image = string(get(m, "image", &at)?, &format!("{at}.image"))?;
    let image = pinned(&image)
        .map(|()| Image(image))
        .or_else(|why| refuse(&format!("{at}.image"), why))?;
    Ok(Service {
        name: name.to_string(),
        image,
        environment: match m.get("environment") {
            None => Vec::new(),
            Some(v) => pairs(v, &format!("{at}.environment"))?,
        },
        mounts: match m.get("volumes") {
            None => Vec::new(),
            Some(v) => list(v, &format!("{at}.volumes"))?
                .iter()
                .map(|v| read_mount(v, &format!("{at}.volumes"), volumes))
                .collect::<Result<_, _>>()?,
        },
        ports: match m.get("ports") {
            None => Vec::new(),
            Some(v) => list(v, &format!("{at}.ports"))?
                .iter()
                .map(|v| read_port(v, &format!("{at}.ports")))
                .collect::<Result<_, _>>()?,
        },
        health: m
            .get("healthcheck")
            .map(|v| read_health(v, &format!("{at}.healthcheck")))
            .transpose()?
            .flatten(),
        entrypoint: m
            .get("entrypoint")
            .map(|v| words(v, &format!("{at}.entrypoint")))
            .transpose()?,
        command: m
            .get("command")
            .map(|v| words(v, &format!("{at}.command")))
            .transpose()?,
        user: m.get("user").map(|v| scalar(v, &format!("{at}.user"))).transpose()?,
        working_dir: m
            .get("working_dir")
            .map(|v| string(v, &format!("{at}.working_dir")))
            .transpose()?,
        platform: m
            .get("platform")
            .map(|v| string(v, &format!("{at}.platform")))
            .transpose()?,
        memory: m
            .get("mem_limit")
            .map(|v| bytes(v, &format!("{at}.mem_limit")))
            .transpose()?,
        stop_grace: match m.get("stop_grace_period") {
            None => Duration::from_secs(10),
            Some(v) => duration(v, &format!("{at}.stop_grace_period"))?,
        },
    })
}

/// `repo[:tag]@sha256:<64 hex>`.
fn pinned(image: &str) -> Result<(), String> {
    let digest = image.rsplit_once("@sha256:").map(|(_, d)| d);
    match digest {
        Some(d) if d.len() == 64 && d.bytes().all(|b| b.is_ascii_hexdigit() && !b.is_ascii_uppercase()) => Ok(()),
        _ => Err(format!(
            "{image} is not pinned by digest (@sha256:…), so two computers could run different apps"
        )),
    }
}

fn read_mount(v: &Value, at: &str, volumes: &[String]) -> Result<Mount, Refused> {
    const BIND: &str = "a bind mount reaches this computer's own files; the app's state is a named volume";
    let (source, target, read_only) = match v {
        Value::String(s) => {
            let parts: Vec<&str> = s.split(':').collect();
            match parts.as_slice() {
                [target] => (None, target.to_string(), false),
                [source, target] => (Some(source.to_string()), target.to_string(), false),
                [source, target, opts] => (
                    Some(source.to_string()),
                    target.to_string(),
                    opts.split(',').any(|o| o == "ro"),
                ),
                _ => return refuse(at, format!("{s} is not `volume:/path[:ro]`")),
            }
        }
        Value::Mapping(m) => {
            match m.get("type").and_then(Value::as_str) {
                Some("volume") | None => {}
                Some("bind") => return refuse(at, BIND),
                Some(t) => return refuse(at, format!("a mount of type {t} is not one give runs")),
            }
            for key in keys(m, at)? {
                if !matches!(key, "type" | "source" | "target" | "read_only") {
                    return refuse(&format!("{at}.{key}"), "give does not run this field of a mount");
                }
            }
            let source = m.get("source").map(|v| string(v, at)).transpose()?;
            let target = string(get(m, "target", at)?, at)?;
            (source, target, m.get("read_only") == Some(&Value::Bool(true)))
        }
        _ => return refuse(at, "a mount is `volume:/path` or a mapping"),
    };
    match source {
        Some(s) if s.starts_with(['/', '.', '~']) || s.contains('/') => refuse(at, BIND),
        Some(s) if !volumes.contains(&s) => refuse(at, format!("names {s}, which `volumes` does not declare")),
        volume => Ok(Mount {
            volume,
            target,
            read_only,
        }),
    }
}

fn read_health(v: &Value, at: &str) -> Result<Option<Healthcheck>, Refused> {
    let m = mapping(v, at)?;
    for key in keys(m, at)? {
        if !matches!(
            key,
            "test" | "interval" | "timeout" | "retries" | "start_period" | "disable"
        ) {
            return refuse(&format!("{at}.{key}"), "give does not run this field of a healthcheck");
        }
    }
    if m.get("disable") == Some(&Value::Bool(true)) {
        return Ok(None);
    }
    let test = match get(m, "test", at)? {
        Value::String(s) => HealthTest::Shell(s.clone()),
        v => {
            let words = list(v, &format!("{at}.test"))?
                .iter()
                .map(|w| scalar(w, &format!("{at}.test")))
                .collect::<Result<Vec<_>, _>>()?;
            match words.split_first() {
                Some((first, rest)) if first == "CMD" && !rest.is_empty() => HealthTest::Exec(rest.to_vec()),
                Some((first, [shell])) if first == "CMD-SHELL" => HealthTest::Shell(shell.clone()),
                Some((first, [])) if first == "NONE" => return Ok(None),
                _ => {
                    return refuse(
                        &format!("{at}.test"),
                        "a test is [CMD, …], [CMD-SHELL, \"…\"], [NONE] or a string",
                    );
                }
            }
        }
    };
    let time = |key: &str, default: u64| match m.get(key) {
        None => Ok(Duration::from_secs(default)),
        Some(v) => duration(v, &format!("{at}.{key}")),
    };
    Ok(Some(Healthcheck {
        test,
        // Compose's own defaults (compose-spec, "healthcheck").
        interval: time("interval", 30)?,
        timeout: time("timeout", 30)?,
        retries: match m.get("retries") {
            None => 3,
            Some(v) => v
                .as_u64()
                .and_then(|n| u32::try_from(n).ok())
                .map_or_else(|| refuse(&format!("{at}.retries"), "retries is a whole number"), Ok)?,
        },
        start_period: time("start_period", 0)?,
    }))
}

fn read_door(v: &Value, services: &[Service]) -> Result<Door, Refused> {
    let at = "x-svrngs.door";
    let m = mapping(v, at)?;
    for key in keys(m, at)? {
        if !matches!(key, "port" | "protocol") {
            return refuse(&format!("{at}.{key}"), "a door has a port and a protocol");
        }
    }
    let port = get(m, "port", at)?
        .as_u64()
        .and_then(|p| u16::try_from(p).ok())
        .filter(|p| *p != 0);
    let Some(port) = port else {
        return refuse(&format!("{at}.port"), "a port is a number from 1 to 65535");
    };
    let protocol = match get(m, "protocol", at)?.as_str() {
        Some("http") => Protocol::Http,
        Some("tcp") => Protocol::Tcp,
        Some("udp") => Protocol::Udp,
        _ => return refuse(&format!("{at}.protocol"), "a door's protocol is http, tcp or udp"),
    };
    if !services.iter().any(|s| s.ports.contains(&(port, protocol.transport()))) {
        return refuse(
            at,
            format!("{port}/{} is not in any service's ports", protocol.transport().as_str()),
        );
    }
    Ok(Door { port, protocol })
}

/// The port inside the app: `"25565"`, `"25565:25565/tcp"`,
/// `"127.0.0.1:8080:80"`, or `{target: 80, protocol: tcp}`.
fn read_port(v: &Value, at: &str) -> Result<(u16, Transport), Refused> {
    let (target, protocol) = match v {
        Value::Mapping(m) => (
            scalar(get(m, "target", at)?, at)?,
            m.get("protocol").map(|p| string(p, at)).transpose()?,
        ),
        v => {
            let s = scalar(v, at)?;
            let (ports, protocol) = s
                .split_once('/')
                .map_or((s.as_str(), None), |(p, t)| (p, Some(t.to_string())));
            (ports.rsplit(':').next().unwrap_or(ports).to_string(), protocol)
        }
    };
    let transport = match protocol.as_deref() {
        None | Some("tcp") => Transport::Tcp,
        Some("udp") => Transport::Udp,
        Some(t) => return refuse(at, format!("{t} is not tcp or udp")),
    };
    match target.parse::<u16>() {
        Ok(p) if p != 0 => Ok((p, transport)),
        _ => refuse(at, format!("{target} is not one port from 1 to 65535")),
    }
}

fn read_ask(v: &Value) -> Result<Ask, Refused> {
    let at = "x-svrngs.asks";
    let m = mapping(v, at)?;
    for key in keys(m, at)? {
        if !matches!(key, "say" | "link" | "sets") {
            return refuse(&format!("{at}.{key}"), "an ask has say, link and sets");
        }
    }
    let link = string(get(m, "link", at)?, &format!("{at}.link"))?;
    if !link.starts_with("https://") {
        return refuse(
            &format!("{at}.link"),
            "an ask links to its terms at an https:// address",
        );
    }
    Ok(Ask {
        say: string(get(m, "say", at)?, &format!("{at}.say"))?,
        link,
        sets: pairs(get(m, "sets", at)?, &format!("{at}.sets"))?,
    })
}

fn mapping<'a>(v: &'a Value, at: &str) -> Result<&'a Mapping, Refused> {
    v.as_mapping().map_or_else(|| refuse(at, "is a mapping of fields"), Ok)
}

fn keys<'a>(m: &'a Mapping, at: &str) -> Result<Vec<&'a str>, Refused> {
    m.keys()
        .map(|k| k.as_str().map_or_else(|| refuse(at, "a field's name is text"), Ok))
        .collect()
}

fn get<'a>(m: &'a Mapping, key: &str, at: &str) -> Result<&'a Value, Refused> {
    m.get(key).map_or_else(|| refuse(at, format!("has no {key}")), Ok)
}

fn list<'a>(v: &'a Value, at: &str) -> Result<&'a Vec<Value>, Refused> {
    v.as_sequence().map_or_else(|| refuse(at, "is a list"), Ok)
}

fn string(v: &Value, at: &str) -> Result<String, Refused> {
    v.as_str().map(str::to_string).map_or_else(|| refuse(at, "is text"), Ok)
}

/// Text, a number or a yes/no, as compose reads an environment value.
fn scalar(v: &Value, at: &str) -> Result<String, Refused> {
    match v {
        Value::String(s) => Ok(s.clone()),
        Value::Number(n) => Ok(n.to_string()),
        Value::Bool(b) => Ok(b.to_string()),
        _ => refuse(at, "is text or a number"),
    }
}

/// A list of words. A command as one string would need compose's own
/// splitting, which give does not guess at.
fn words(v: &Value, at: &str) -> Result<Vec<String>, Refused> {
    match v {
        Value::String(_) => refuse(at, "give takes a command as a list of words, [\"sh\", \"-c\", \"…\"]"),
        v => list(v, at)?.iter().map(|w| scalar(w, at)).collect(),
    }
}

/// `KEY: value` or `["KEY=value"]`.
fn pairs(v: &Value, at: &str) -> Result<Vec<(String, String)>, Refused> {
    let mut out = BTreeMap::new();
    match v {
        Value::Mapping(m) => {
            for (k, v) in m {
                out.insert(
                    string(k, at)?,
                    scalar(v, &format!("{at}.{}", k.as_str().unwrap_or("?")))?,
                );
            }
        }
        v => {
            for item in list(v, at)? {
                let item = string(item, at)?;
                let Some((k, v)) = item.split_once('=') else {
                    return refuse(at, format!("{item} is not KEY=value"));
                };
                out.insert(k.to_string(), v.to_string());
            }
        }
    }
    Ok(out.into_iter().collect())
}

/// Compose's durations: `1m30s`, `500ms`, `10s`; a bare number is seconds.
pub fn duration(v: &Value, at: &str) -> Result<Duration, Refused> {
    if let Some(n) = v.as_u64() {
        return Ok(Duration::from_secs(n));
    }
    let s = string(v, at)?;
    let (mut total, mut number) = (Duration::ZERO, String::new());
    let mut rest = s.as_str();
    while let Some(c) = rest.chars().next() {
        if c.is_ascii_digit() {
            number.push(c);
            rest = &rest[1..];
            continue;
        }
        let unit = ["ms", "us", "h", "m", "s"].into_iter().find(|u| rest.starts_with(u));
        let (Some(unit), Ok(n)) = (unit, number.parse::<u64>()) else {
            return refuse(at, format!("{s} is not a duration such as 30s or 1m30s"));
        };
        total += match unit {
            "h" => Duration::from_secs(n * 3600),
            "m" => Duration::from_secs(n * 60),
            "s" => Duration::from_secs(n),
            "ms" => Duration::from_millis(n),
            _ => Duration::from_micros(n),
        };
        number.clear();
        rest = &rest[unit.len()..];
    }
    if !number.is_empty() || s.is_empty() {
        return refuse(at, format!("{s} is not a duration such as 30s or 1m30s"));
    }
    Ok(total)
}

/// Compose's byte sizes: `512m`, `2g`, `1024k`, `1073741824`.
fn bytes(v: &Value, at: &str) -> Result<u64, Refused> {
    if let Some(n) = v.as_u64() {
        return Ok(n);
    }
    let s = string(v, at)?.to_ascii_lowercase();
    let s = s.strip_suffix('b').unwrap_or(&s);
    let (n, unit) = s.split_at(s.find(|c: char| !c.is_ascii_digit()).unwrap_or(s.len()));
    let scale = match unit {
        "" => 1,
        "k" => 1 << 10,
        "m" => 1 << 20,
        "g" => 1 << 30,
        _ => return refuse(at, format!("{s} is not a size such as 512m or 2g")),
    };
    n.parse::<u64>()
        .map(|n| n * scale)
        .map_or_else(|_| refuse(at, format!("{s} is not a size such as 512m or 2g")), Ok)
}
