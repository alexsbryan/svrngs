//! Read-only prefill for a command the person still has to give or confirm.

use serde::Serialize;
use svrngs_keeper::{host, promote};

#[derive(Serialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum Facts {
    Prefill { port: u16, card: String },
    Unavailable { port: u16, why: String },
}

pub fn read(port: u16, name: &str) -> Facts {
    match prefill(port, name) {
        Ok(card) => {
            tracing::debug!(port, "facts:prefill");
            Facts::Prefill { port, card }
        }
        Err(why) => {
            tracing::debug!(port, %why, "facts:no_prefill");
            Facts::Unavailable { port, why }
        }
    }
}

fn prefill(port: u16, name: &str) -> Result<String, String> {
    let sockets = host::ss(port)?;
    let first = sockets.first().ok_or("nothing listens on this port")?;
    let pid = *first.pids.first().ok_or("the listener could not be named")?;
    if !sockets.iter().all(|s| s.pids.as_slice() == [pid]) {
        return Err("the port does not have one identifiable listener".into());
    }
    let folder = host::llama_folder(pid)?;
    let model = host::props(promote::dial(&first.addr), port)?;
    let home = std::env::var("HOME").map_err(|e| e.to_string())?;
    Ok(promote::llama_card(name, &folder, port, &model, &home))
}
