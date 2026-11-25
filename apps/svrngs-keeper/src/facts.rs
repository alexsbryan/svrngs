//! Read-only prefill for a command the person still has to give or confirm:
//! how the one program listening on the port was started, its arguments and
//! its folder, read only where it is the person's own.

use serde::Serialize;
use svrngs_keeper::host;

#[derive(Serialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum Facts {
    Prefill {
        port: u16,
        name: String,
        folder: String,
        command: String,
    },
    Unavailable {
        port: u16,
        why: String,
    },
}

/// `name` is the one given, else the program's own file name.
pub fn read(port: u16, name: Option<&str>) -> Facts {
    match prefill(port) {
        Ok((argv, folder, command)) => {
            let program = std::path::Path::new(&argv[0]).file_name().and_then(|p| p.to_str());
            let program = program.unwrap_or(&argv[0]);
            // Its arguments can hold a secret, so only their count is traced.
            tracing::debug!(port, program, args = argv.len(), "facts:prefill");
            let name = name.unwrap_or(program).to_owned();
            Facts::Prefill {
                port,
                name,
                folder,
                command,
            }
        }
        Err(why) => {
            tracing::debug!(port, %why, "facts:no_prefill");
            let why = format!("svrngs couldn't read how it was started: {why}. Fill in its command and folder.");
            Facts::Unavailable { port, why }
        }
    }
}

fn prefill(port: u16) -> Result<(Vec<String>, String, String), String> {
    let sockets = host::listeners(port)?;
    let first = sockets.first().ok_or("nothing listens on this port")?;
    let pid = *first.pids.first().ok_or("the program listening could not be named")?;
    if !sockets.iter().all(|s| s.pids.as_slice() == [pid]) {
        return Err("more than one program holds the port".into());
    }
    let (argv, folder) = host::started(pid)?;
    if argv.is_empty() {
        return Err("it gave no arguments".into());
    }
    let folder = folder
        .into_os_string()
        .into_string()
        .map_err(|_| "its folder's path is not UTF-8")?;
    let command = shlex::try_join(argv.iter().map(String::as_str)).map_err(|e| e.to_string())?;
    Ok((argv, folder, command))
}
