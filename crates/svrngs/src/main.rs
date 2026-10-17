// SPDX-License-Identifier: AGPL-3.0-or-later
//! `svrngs` — the binary. Everything it does is in the library beside it,
//! so the tests exercise the same code this runs (the shape of upstream's
//! `commonwealth/crates/commonwealth-rails/src/main.rs` at `e1a43bcd2`).

use std::process::ExitCode;

use svrngs::cli;

fn main() -> ExitCode {
    // `error` by default: what a person reads is the command's own words,
    // and a trace may use any (DESIGN §1). RUST_LOG=debug shows every
    // decision (ARCH 1).
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| tracing_subscriber::EnvFilter::new("error")),
        )
        .with_writer(std::io::stderr)
        .init();

    let command = match cli::parse(std::env::args()) {
        Ok(c) => c,
        Err(why) => {
            eprintln!("svrngs: {why}");
            eprintln!();
            eprintln!("{}", cli::USAGE);
            // Usage is a refusal, not an abstention: the command ran and
            // said no to what was typed.
            return ExitCode::from(1);
        }
    };
    let root = match cli::data_dir() {
        Ok(r) => r,
        Err(e) => return refuse(e),
    };
    // A host with no runtime is a precondition absent, not a refusal.
    let runtime = match tokio::runtime::Runtime::new() {
        Ok(rt) => rt,
        Err(e) => {
            eprintln!("svrngs: cannot start on this computer: {e}");
            return ExitCode::from(3);
        }
    };
    let relay = match cli::reach(&root, &command, cli::SHIPPED) {
        Ok(r) => r,
        Err(e) => return refuse(e),
    };
    match runtime.block_on(cli::run(command, root, &relay)) {
        Ok(()) => ExitCode::SUCCESS,
        Err(e) => refuse(e),
    }
}

fn refuse(e: cli::Refusal) -> ExitCode {
    eprintln!("svrngs: {e}");
    ExitCode::from(e.exit_code())
}
