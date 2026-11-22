// SPDX-License-Identifier: AGPL-3.0-or-later
//! cr-sqlite, built from its source for the target and linked into
//! svrngs-db as one object whose only global symbol is the shim SQLite calls
//! as it starts ([`SHIM`]; `shim.c`, `/.cargo/config.toml`), so no Rust in
//! svrngs calls cr-sqlite. Its source is the submodule `vendor/cr-sqlite`,
//! the fork's `svrngs` branch (svrn-docs `rings/current/APP_STATE.md` §8 q8),
//! built as `app-state-diligence/b1-targets.md`, "What B1's `build.rs` must
//! do", has it, for the release's four targets (scripts/build-release.sh)
//! and x86_64 Linux's gnu, the gate's (scripts/gate/Containerfile). Any
//! other target stops the build, saying so. The build asks no network: a
//! fresh checkout or worktree runs [`FETCH`] once.

use std::path::PathBuf;
use std::process::Command;

/// The shim's name, the one global the object may define (`shim.c`).
const SHIM: &str = "svrngs_crsqlite";

/// cr-sqlite's source, from this crate's directory.
const SOURCE: &str = "../../vendor/cr-sqlite/core";

/// What a fresh checkout or worktree runs once, so this build needs no network.
const FETCH: &str = "run `git submodule update --init` and then `cargo fetch --manifest-path \
    vendor/cr-sqlite/core/rs/bundle_static/Cargo.toml` once";

/// Where the programs this build runs come from, said when one doesn't start.
const TOOLS: &str = "git, ld and nm come with Xcode's command line tools on a Mac (xcode-select --install), \
    and with git and binutils on Linux; rust-lld and rust-objcopy with rustc, as rust-toolchain.toml pins it";

fn main() {
    let env = |k: &str| std::env::var(k).unwrap_or_else(|_| panic!("cargo sets {k}"));
    let core = PathBuf::from(env("CARGO_MANIFEST_DIR")).join(SOURCE);
    let (out, target) = (PathBuf::from(env("OUT_DIR")), env("TARGET"));
    println!("cargo::rerun-if-changed=shim.c");
    println!("cargo::rerun-if-changed={SOURCE}");
    // A Mac's arch, for Apple's linker, which localizes as it links; ELF's,
    // on any host, is localized after.
    let mac = match target.as_str() {
        "aarch64-apple-darwin" => Some("arm64"),
        "x86_64-apple-darwin" => Some("x86_64"),
        "x86_64-unknown-linux-gnu" | "x86_64-unknown-linux-musl" | "aarch64-unknown-linux-musl" => None,
        _ => panic!("svrngs-db builds cr-sqlite for the release's four targets and x86_64 Linux gnu, not {target}"),
    };
    // Mach-O's names carry a leading underscore.
    let shim = format!("{}{SHIM}", mac.map_or("", |_| "_"));
    let sha = run(Command::new("git").arg("-C").arg(&core).args(["rev-parse", "HEAD"]));
    // The bundle's manifest, copied with an empty `[workspace]` of its own,
    // its paths to its crates made whole: a checkout's worktree sits inside
    // the checkout's workspace, which cargo else takes it for a stray of.
    let (rs, copied) = (core.join("rs"), out.join("bundle_static"));
    std::fs::create_dir_all(copied.join("src")).expect("OUT_DIR is written");
    for file in ["Cargo.lock", "src/lib.rs"] {
        let to = copied.join(file);
        std::fs::copy(rs.join("bundle_static").join(file), to).expect("the bundle is copied");
    }
    let manifest = std::fs::read_to_string(rs.join("bundle_static/Cargo.toml"));
    let manifest = manifest.expect("the bundle's manifest is read");
    let manifest = manifest.replace("path=\"../", &format!("path=\"{}/", rs.display()));
    std::fs::write(copied.join("Cargo.toml"), manifest + "\n[workspace]\n").expect("written");
    // The bundle, by the cargo and rustc running this build, with none of
    // its wrappers or flags, which are this workspace's (a clippy run's).
    let mut bundle = Command::new(env("CARGO"));
    bundle.current_dir(&copied);
    bundle.env("CRSQLITE_COMMIT_SHA", String::from_utf8_lossy(&sha).trim());
    for flag in [
        "RUSTC_WRAPPER",
        "RUSTC_WORKSPACE_WRAPPER",
        "CARGO_ENCODED_RUSTFLAGS",
        "RUSTFLAGS",
    ] {
        bundle.env_remove(flag);
    }
    let features = ["--features", "static,omit_load_extension", "--target", &target];
    bundle.args(["rustc", "--release", "--frozen"]).args(features);
    run(bundle.arg("--target-dir").arg(out.join("rs")));
    let objects = cc::Build::new()
        .files(["crsqlite", "changes-vtab", "ext-data", "version-guard"].map(|f| core.join(format!("src/{f}.c"))))
        .file("shim.c")
        .includes([core.join("src"), core.join("src/sqlite")])
        .define("SQLITE_CORE", None)
        .define("HAVE_GETHOSTUUID", "0")
        .std("c99")
        .opt_level(2)
        .warnings(false)
        .compile_intermediates();
    let all = out.join("crsqlite_all.o");
    let rs = out.join(format!("rs/{target}/release/libcrsql_bundle_static.a"));
    if let Some(arch) = mac {
        let mut ld = Command::new("ld");
        ld.args(["-r", "-arch", arch, "-o"]).arg(&all).args(&objects).arg(rs);
        run(ld.args(["-exported_symbol", &shim]));
    } else {
        // The lld and objcopy rustc ships beside itself, the same on a Mac
        // cross-building and on Linux, so no host's own binutils decide.
        let sysroot = run(Command::new(env("RUSTC")).args(["--print", "sysroot"]));
        let tools = PathBuf::from(String::from_utf8_lossy(&sysroot).trim());
        let tools = tools.join("lib/rustlib").join(env("HOST")).join("bin");
        let mut ld = Command::new(tools.join("rust-lld"));
        run(ld.args(["-flavor", "gnu", "-r", "-o"]).arg(&all).args(&objects).arg(rs));
        let mut objcopy = Command::new(tools.join("rust-objcopy"));
        run(objcopy.arg(format!("--keep-global-symbol={shim}")).arg(&all));
    }
    // A missed localization links without a word and dies at run, the
    // bundle's allocator and panic handler shared with svrngs's
    // (b1-targets.md, failure 4): so any other global stops the build.
    let defined = run(Command::new("nm").args(["-g", "-U"]).arg(&all));
    let defined = String::from_utf8_lossy(&defined);
    let globals: Vec<&str> = defined.lines().filter_map(|l| l.split_whitespace().last()).collect();
    assert_eq!(globals, [shim.as_str()], "one global, the shim");
    // Archived by the target's archiver, which cargo-zigbuild names, as
    // Apple's writes an ELF archive no index (b1-targets.md, failure 3); and
    // linked whole, as nothing in Rust names the shim: only SQLite does.
    cc::Build::new()
        .object(&all)
        .link_lib_modifier("+whole-archive")
        .compile("crsqlite_svrngs");
}

/// What `cmd` printed, or the build stopped with what it said, and [`FETCH`].
fn run(cmd: &mut Command) -> Vec<u8> {
    let ran = cmd
        .output()
        .unwrap_or_else(|e| panic!("{cmd:?} did not start: {e}. {TOOLS}."));
    let said = String::from_utf8_lossy(&ran.stderr);
    assert!(
        ran.status.success(),
        "{cmd:?} failed: {said}\nIf it lacks its source or a crate, {FETCH}."
    );
    ran.stdout
}
