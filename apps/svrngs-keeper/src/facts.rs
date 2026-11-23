//! Review a native takeover without staging or stopping anything.

use serde::Serialize;
use svrngs_keeper::{
    card::Card,
    host::{self, Identity, Original},
    promote::{self as p, Declined},
    runtime::Name,
    systemd::SystemdUser,
    unit,
};

#[derive(Serialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum Facts {
    Known(Offer),
    Copyable(Offer),
    Kept { port: u16, unit: String },
    Declined { port: u16, why: String },
}

#[derive(Serialize)]
pub struct Offer {
    pub port: u16,
    pub listener: Identity,
    pub name: String,
    pub what: String,
    pub omitted_settings: Vec<String>,
    pub omitted_flags: Vec<String>,
    pub linger: Result<bool, String>,
    pub token: String,
}

pub struct Draft {
    pub name: Name,
    known: bool,
    pub offer: Offer,
    pub recipe: Recipe,
}

#[derive(serde::Serialize, serde::Deserialize)]
pub struct Recipe {
    pub port: u16,
    pub dial: String,
    pub listener_exe: String,
    pub http: bool,
    pub chain: Vec<Identity>,
    pub original: Original,
    pub card: String,
    pub unit_text: String,
}

impl Draft {
    pub fn read(systemd: &SystemdUser, boot: &str, port: u16, name: Option<&str>) -> Result<Self, Declined> {
        let (addr, procs) = host::survey(port)?;
        if let Some(declined) = p::declines(&procs, port, host::ephemeral()) {
            return Err(declined);
        }
        let (listener, top) = (&procs[0], &procs[procs.len() - 1]);
        let home = std::env::var("HOME").map_err(|e| Declined::Cannot(e.to_string()))?;
        let dial = p::dial(&addr);
        let name = name.map_or_else(|| p::name_for(&top.argv), str::to_owned);
        let props = if p::known(&procs) {
            host::get(&dial, port, "/props").and_then(|(status, body)| match status {
                200 => p::props(&body),
                s => Err(format!("/props answered {s}")),
            })
        } else {
            Err("svrngs-keeper ships no card for it".into())
        };
        let known = props.is_ok();
        let (card, http, what, omitted_flags) = match props {
            Ok(props) => {
                let model = p::model(listener, &props, &home);
                let what = format!(
                    "llama-server with your model {model} on port {port}, context {}",
                    props.n_ctx
                );
                (
                    p::llama_card(&name, listener, &addr, port, &props, &home),
                    true,
                    what,
                    p::not_carried(&listener.argv),
                )
            }
            Err(why) => {
                tracing::debug!(why, "facts:copy_not_compose");
                let http = host::answers(&dial, port, true);
                let program = top.argv.first().unwrap_or(&top.exe).rsplit('/').next().unwrap_or("");
                let what = format!(
                    "{program} as it was started, on port {port}:\n  {}\n  {}",
                    p::shown(&top.argv),
                    p::homed(&top.cwd, &home)
                );
                (p::copy_card(&name, top, port, http, &home), http, what, Vec::new())
            }
        };
        let named = Name::new(&name).map_err(|e| Declined::Cannot(e.to_string()))?;
        let parsed = Card::parse(&card).map_err(|e| Declined::Cannot(e.to_string()))?;
        let native = parsed
            .native
            .as_ref()
            .ok_or_else(|| Declined::Cannot("draft isn't native".into()))?;
        let env = top.env.clone().map_err(Declined::Cannot)?;
        let manager = systemd.environment().unwrap_or_else(|why| {
            tracing::debug!(why, "facts:manager_environment_unread_name_all_settings");
            String::new()
        });
        let omitted_settings = p::added(&env, &manager);
        let recipe = Recipe {
            port,
            dial,
            listener_exe: listener.exe.clone(),
            http,
            chain: procs.iter().map(Identity::from).collect(),
            original: Original {
                exe: top.exe.clone(),
                argv: top.argv.clone(),
                cwd: top.cwd.clone(),
                env,
            },
            card,
            unit_text: unit::native(&named, &parsed, native),
        };
        let linger = systemd.lingers();
        let basis = serde_json::to_vec(&("keeper-native-v1", boot, &recipe, &linger, &manager))
            .map_err(|e| Declined::Cannot(e.to_string()))?;
        let token = blake3::hash(&basis).to_hex().to_string();
        tracing::debug!(port, name, known, "facts:drafted");
        Ok(Draft {
            name: named,
            known,
            recipe,
            offer: Offer {
                port,
                listener: Identity::from(listener),
                name,
                what,
                omitted_settings,
                omitted_flags,
                linger,
                token,
            },
        })
    }

    pub fn facts(self) -> Facts {
        if self.known {
            Facts::Known(self.offer)
        } else {
            Facts::Copyable(self.offer)
        }
    }
}

impl Facts {
    pub fn refused(port: u16, declined: Declined) -> Self {
        tracing::debug!(port, %declined, "facts:declined");
        match declined {
            Declined::Kept { port, unit } => Facts::Kept { port, unit },
            why => Facts::Declined {
                port,
                why: why.to_string(),
            },
        }
    }
}
