//! Read-only prefill for a command the person still has to give or confirm.

use serde::Serialize;
use svrngs_keeper::{host, promote};

#[derive(Serialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum Facts<'a> {
    Prefill {
        port: u16,
        name: &'a str,
        folder: String,
        command: String,
    },
    Unavailable {
        port: u16,
        why: String,
    },
}

pub fn read(port: u16, name: &str) -> Facts<'_> {
    match prefill(port) {
        Ok((folder, command)) => {
            tracing::debug!(port, "facts:prefill");
            Facts::Prefill {
                port,
                name,
                folder,
                command,
            }
        }
        Err(why) => {
            tracing::debug!(port, %why, "facts:no_prefill");
            Facts::Unavailable { port, why }
        }
    }
}

fn prefill(port: u16) -> Result<(String, String), String> {
    let sockets = host::listeners(port)?;
    let first = sockets.first().ok_or("nothing listens on this port")?;
    let pid = *first.pids.first().ok_or("the listener could not be named")?;
    if !sockets.iter().all(|s| s.pids.as_slice() == [pid]) {
        return Err("the port does not have one identifiable listener".into());
    }
    let folder = host::llama_folder(pid)?;
    let model = host::props(promote::dial(&first.addr), port)?;
    let command = promote::llama_command(&folder, port, &model)?;
    Ok((
        folder
            .into_os_string()
            .into_string()
            .map_err(|_| "folder is not UTF-8")?,
        command,
    ))
}
