//! The `give` command.
//!
//!   give check [--agree] [--name <name>] <card dir>
//!
//! RUST_LOG=debug shows every decision, each refusal with the field or the
//! fact that refused it.

use std::path::PathBuf;
use std::process::ExitCode;

use give::check::{Step, check};
use give::podman::Podman;
use give::runtime::Name;

const USAGE: &str = "usage: give check [--agree] [--name <name>] <card dir>

  Start the app a card describes from its pinned image, wait until it is
  healthy, snapshot it, stop it, restore the snapshot and start it again;
  then remove all of it. --agree says yes to the card's asks, once you have
  read them. --name runs it under another name than svrngs-check-<dir>.";

fn main() -> ExitCode {
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| tracing_subscriber::EnvFilter::new("error")),
        )
        .with_writer(std::io::stderr)
        .init();
    match run(std::env::args().skip(1).collect()) {
        Ok(code) => code,
        Err(why) => {
            eprintln!("give: {why}\n\n{USAGE}");
            ExitCode::from(2)
        }
    }
}

fn run(args: Vec<String>) -> anyhow::Result<ExitCode> {
    let mut args = args.into_iter();
    anyhow::ensure!(args.next().as_deref() == Some("check"), "the one command is check");
    let (mut agree, mut name, mut dir) = (false, None, None);
    while let Some(a) = args.next() {
        match a.as_str() {
            "--agree" => agree = true,
            "--name" => name = Some(args.next().ok_or_else(|| anyhow::anyhow!("--name needs a name"))?),
            _ if dir.is_none() && !a.starts_with('-') => dir = Some(PathBuf::from(a)),
            _ => anyhow::bail!("{a} is not an argument check takes"),
        }
    }
    let dir = dir.ok_or_else(|| anyhow::anyhow!("check needs a card's directory"))?;
    let kind = dir
        .canonicalize()?
        .file_name()
        .map(|n| n.to_string_lossy().to_lowercase())
        .unwrap_or_default();
    let name = Name::new(&name.unwrap_or_else(|| format!("svrngs-check-{kind}")))?;
    let scratch = std::env::temp_dir().join(format!("give-check-{}", std::process::id()));
    let mut passed = |step: Step, what: String| {
        let what = if what.is_empty() {
            String::new()
        } else {
            format!(": {what}")
        };
        println!("passed  {step}{what}");
    };
    match check(&Podman::default(), &dir, name, &scratch, agree, &mut passed) {
        Ok(card) => {
            println!("{} can go on the shelf.", card.title);
            Ok(ExitCode::SUCCESS)
        }
        Err(failed) => {
            println!("failed  {failed}");
            Ok(ExitCode::FAILURE)
        }
    }
}
