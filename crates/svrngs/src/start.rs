//! `svrngs start`: put the bot in a person's Discord, in DESIGN C11's five
//! steps — install (done, since this is running); make the bot and paste its
//! token; find it, check its switches, print the invite link; open the link;
//! pick the channel. Then it starts the group if this computer keeps none,
//! enables the service, and says whether the computer sleeps.
//!
//! Everything it prints is in `start.json`, in plain words (DESIGN D0;
//! `crates/svrngs/tests/plain_words.rs` reads it). What it asks Discord, it
//! asks through the bridge's adapter (`bridge/setup.mjs`), since Discord's
//! code lives there (D2b).

use std::io::{BufRead, Write};
use std::process::Command;
use std::time::Duration;

use anyhow::{Context, bail};
use serde::Deserialize;
use serde::de::DeserializeOwned;
use tracing::{debug, info};

use crate::home::{Chat, Home};
use crate::service::{self, Sleep};
use crate::store::GroupDir;

/// Everything `svrngs start` says, by the key it has in `start.json`.
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct Words {
    hello: String,
    need_node: String,
    getting_parts: String,
    parts_failed: String,
    step_bot: String,
    token_prompt: String,
    token_refused: String,
    token_again: String,
    found: String,
    switches_off: String,
    step_link: String,
    waiting: String,
    step_channel: String,
    channel_line: String,
    channel_prompt: String,
    channel_again: String,
    no_channels: String,
    group_prompt: String,
    group_kept: String,
    owner_prompt: String,
    owner_prompt_bare: String,
    name_again: String,
    service_on: String,
    service_off: String,
    sleeps: String,
    awake: String,
    sleep_unknown: String,
    done: String,
    stopped: String,
    discord_failed: String,
}

fn words() -> Words {
    serde_json::from_str(include_str!("start.json")).expect("start.json holds every word Words names")
}

/// `template` with each `{name}` replaced by its value.
fn fill(template: &str, vars: &[(&str, &str)]) -> String {
    vars.iter()
        .fold(template.to_string(), |t, (k, v)| t.replace(&format!("{{{k}}}"), v))
}

/// How often to ask Discord whether the bot has joined a server yet.
const POLL: Duration = Duration::from_secs(3);
/// The oldest Node.js the bridge runs on (bridge/package.json `engines`).
const NODE_MAJOR: u32 = 20;

pub fn run(home: &Home) -> anyhow::Result<()> {
    let w = words();
    let mut ask = Ask {
        lines: std::io::stdin().lock(),
        stopped: &w.stopped,
    };
    println!("{}", w.hello);
    node_ready(&w)?;

    println!("{}", w.step_bot);
    let mut token = home.discord_token()?;
    if token.is_some() {
        println!("{}", w.token_again);
    }
    let app = loop {
        let t = match token.take() {
            Some(t) => t,
            None => ask.line(&w.token_prompt)?,
        };
        match setup::<Application>(&w, &t, "application")? {
            Asked::Answered(app) => {
                home.set_discord_token(&t)?;
                break app;
            }
            Asked::Refused {
                refused: Refusal::Token,
            } => {
                info!("start:token_refused");
                println!("{}", w.token_refused);
            }
        }
    };
    let token = home.discord_token()?.context("the token was just kept")?;
    println!("{}", fill(&w.found, &[("bot", &app.name)]));
    let mut off = app.switches_off;
    while !off.is_empty() {
        debug!(?off, "start:switches_off");
        let switches = off.join(" and ");
        ask.line(&fill(
            &w.switches_off,
            &[("switches", &switches), ("settings", &app.settings)],
        ))?;
        off = match setup::<Application>(&w, &token, "application")? {
            Asked::Answered(a) => a.switches_off,
            Asked::Refused { .. } => bail!("{}", w.token_refused),
        };
    }

    println!("{}", fill(&w.step_link, &[("invite", &app.invite)]));
    let mut said_waiting = false;
    let channels = loop {
        let servers = match setup::<Servers>(&w, &token, "servers")? {
            Asked::Answered(s) => s.servers,
            Asked::Refused { .. } => bail!("{}", w.token_refused),
        };
        let channels: Vec<(Server, Channel)> = servers
            .iter()
            .flat_map(|s| s.channels.iter().map(move |c| (s.clone(), c.clone())))
            .collect();
        match (servers.is_empty(), channels.is_empty()) {
            (true, _) => {
                if !said_waiting {
                    println!("{}", w.waiting);
                    said_waiting = true;
                }
                std::thread::sleep(POLL);
            }
            (false, true) => {
                info!(servers = servers.len(), "start:no_channels");
                ask.line(&w.no_channels)?;
            }
            (false, false) => break channels,
        }
    };

    println!("{}", w.step_channel);
    for (n, (server, channel)) in channels.iter().enumerate() {
        let n = (n + 1).to_string();
        println!(
            "{}",
            fill(
                &w.channel_line,
                &[("n", &n), ("channel", &channel.name), ("server", &server.name)]
            )
        );
    }
    let (server, channel) = loop {
        let typed = ask.line(&w.channel_prompt)?;
        match typed
            .trim()
            .parse::<usize>()
            .ok()
            .and_then(|n| channels.get(n.wrapping_sub(1)))
        {
            Some(picked) => break picked.clone(),
            None => println!("{}", w.channel_again),
        }
    };
    info!(server = %server.id, channel = %channel.id, "start:channel");

    match home.the_group()? {
        Some(id) => {
            let (_, journal) = GroupDir::open(&home.group_dir(&id), crate::now())?;
            println!("{}", fill(&w.group_kept, &[("group", journal.genesis().name.as_str())]));
        }
        None => {
            let prompt = fill(&w.group_prompt, &[("name", &server.name)]);
            let name = ask.name(&prompt, Some(&server.name), &w.name_again)?;
            home.create_group(&name, crate::now())?;
        }
    }
    let owner = match &app.owner {
        Some(o) => ask.name(&fill(&w.owner_prompt, &[("name", o)]), Some(o), &w.name_again)?,
        None => ask.name(&w.owner_prompt_bare, None, &w.name_again)?,
    };
    home.set_chat(&Chat {
        channel: channel.id.clone(),
        owner,
    })?;

    match service::enable(home) {
        Ok(_) => println!("{}", w.service_on),
        Err(e) => {
            info!(error = %e, "start:service_off");
            println!("{}", fill(&w.service_off, &[("why", &format!("{e:#}"))]));
        }
    }
    match service::sleep() {
        Sleep::Never => println!("{}", w.awake),
        Sleep::AfterMinutes(m) => println!("{}", fill(&w.sleeps, &[("minutes", &m.to_string())])),
        Sleep::CannotTell(why) => {
            debug!(%why, "start:sleep_unknown");
            println!("{}", w.sleep_unknown);
        }
    }
    println!("{}", fill(&w.done, &[("channel", &channel.name)]));
    Ok(())
}

/// What a person types, a line at a time.
struct Ask<'a, R> {
    lines: R,
    stopped: &'a str,
}

impl<R: BufRead> Ask<'_, R> {
    fn line(&mut self, prompt: &str) -> anyhow::Result<String> {
        print!("{prompt}");
        std::io::stdout().flush()?;
        let mut line = String::new();
        if self.lines.read_line(&mut line)? == 0 {
            println!();
            bail!("{}", self.stopped);
        }
        Ok(line.trim().to_string())
    }

    /// A name for the group or a person: what they type, else `default`.
    fn name(&mut self, prompt: &str, default: Option<&str>, again: &str) -> anyhow::Result<String> {
        loop {
            let typed = self.line(prompt)?;
            let name = if typed.is_empty() {
                default.unwrap_or("")
            } else {
                &typed
            };
            if svrngs_core::Name::try_from(name).is_ok() {
                return Ok(name.to_string());
            }
            println!("{again}");
        }
    }
}

/// Node.js is new enough, and the bridge's packages are installed.
fn node_ready(w: &Words) -> anyhow::Result<()> {
    let version = Command::new("node").arg("--version").output();
    let major = version.as_ref().ok().filter(|o| o.status.success()).and_then(|o| {
        String::from_utf8_lossy(&o.stdout)
            .trim()
            .trim_start_matches('v')
            .split('.')
            .next()?
            .parse::<u32>()
            .ok()
    });
    debug!(?major, "start:node");
    if major.is_none_or(|m| m < NODE_MAJOR) {
        bail!("{}", w.need_node);
    }
    let bridge = crate::bridge_dir();
    if bridge.join("node_modules/discord.js").exists() {
        return Ok(());
    }
    println!("{}", w.getting_parts);
    let out = Command::new("npm")
        .args(["ci", "--omit=dev", "--no-audit", "--no-fund"])
        .current_dir(&bridge)
        .output();
    info!(ok = ?out.as_ref().map(|o| o.status.success()), "start:npm_ci");
    match out {
        Ok(o) if o.status.success() => Ok(()),
        Ok(o) => bail!(
            "{}",
            fill(&w.parts_failed, &[("why", String::from_utf8_lossy(&o.stderr).trim())])
        ),
        Err(e) => bail!("{}", fill(&w.parts_failed, &[("why", &format!("npm: {e}"))])),
    }
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "snake_case")]
enum Refusal {
    Token,
}

/// What `bridge/setup.mjs` answered: the thing asked for, or a refusal.
#[derive(Debug, Deserialize)]
#[serde(untagged)]
enum Asked<T> {
    Refused { refused: Refusal },
    Answered(T),
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct Application {
    #[allow(dead_code)]
    id: String,
    name: String,
    owner: Option<String>,
    switches_off: Vec<String>,
    settings: String,
    invite: String,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct Servers {
    servers: Vec<Server>,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(deny_unknown_fields)]
struct Server {
    id: String,
    name: String,
    channels: Vec<Channel>,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(deny_unknown_fields)]
struct Channel {
    id: String,
    name: String,
}

/// Ask Discord `what` through the bridge's adapter, with the bot's token.
fn setup<T: DeserializeOwned>(w: &Words, token: &str, what: &str) -> anyhow::Result<Asked<T>> {
    let script = crate::bridge_dir().join("setup.mjs");
    let out = Command::new("node")
        .arg(&script)
        .arg(what)
        .env("SVRNGS_DISCORD_TOKEN", token)
        .output()
        .with_context(|| format!("node {}", script.display()))?;
    let stdout = String::from_utf8_lossy(&out.stdout);
    if !out.status.success() {
        let why = String::from_utf8_lossy(&out.stderr);
        bail!("{}", fill(&w.discord_failed, &[("why", why.trim())]));
    }
    let last = stdout.lines().last().unwrap_or_default();
    serde_json::from_str(last).with_context(|| format!("node setup.mjs {what} answered {last:?}"))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn every_word_is_in_start_json_and_fills() {
        let w = words();
        assert_eq!(
            fill(&w.found, &[("bot", "The House Bot")]),
            "\nStep 3 of 5: found your bot on Discord, The House Bot."
        );
    }

    #[test]
    fn the_bridge_answers_as_setup_reads_it() {
        let refused: Asked<Application> = serde_json::from_str(r#"{"refused":"token"}"#).unwrap();
        assert!(matches!(
            refused,
            Asked::Refused {
                refused: Refusal::Token
            }
        ));
        let app = r#"{"id":"1","name":"b","owner":null,"switches_off":[],"settings":"s","invite":"i"}"#;
        assert!(matches!(
            serde_json::from_str::<Asked<Application>>(app).unwrap(),
            Asked::Answered(_)
        ));
    }
}
