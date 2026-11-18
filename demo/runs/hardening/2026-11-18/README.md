# The Linux release as static-pie with full RELRO, 2026-11-18

namcap warned that the v1.11.1 packages' `/usr/bin/svrngs` and
`/usr/bin/give` lack PIE and full RELRO
(`demo/runs/pacman/2026-11-18/06-namcap.log`). Both come from
`scripts/build-release.sh`. `run.sh` drove this from the Mac (Apple
silicon) from 21:16 to 21:27 UTC, at c25fcd5f with `build-release.sh`
changed in the working tree, so the binaries say `c25fcd5f-dirty`. Its
script is the same at 3b9a0506 and c25fcd5f. Every build held the gate
lock. `why.sh` ran after it. Each `<n>-<step>.log` prints every command
before its output and exit status. The toolchain was rustc 1.95.0, zig
0.17.0 and cargo-zigbuild 0.23.4.

## Why it was EXEC (00-why)

rustc asks for static-pie on x86_64: its target says
`static-position-independent-executables`, and the link line it hands
cargo-zigbuild's `zig cc` carries `-static-pie` and `-Wl,-z,relro,-z,now`.
aarch64's target leaves that out, so rustc asks for `-static -no-pie`.
zig's cc drops `-static-pie` ("argument unused during compilation") and
links a fixed-address EXEC. Static and non-PIE, an EXEC has no dynamic
section, so `-z now` has nowhere to write BIND_NOW. A `-pie` later on
the line is what zig takes. For a musl target it still links nothing
dynamic: DYN, BIND_NOW, FLAGS_1 NOW PIE, GNU_RELRO, no INTERP and no
NEEDED. So the change is `-C link-arg=-pie` for the two Linux targets,
set as `CARGO_TARGET_<TRIPLE>_RUSTFLAGS` in `build-release.sh`. That
leaves macOS's flags alone.

## Before and after

`readelf` is the Mac's `llvm-readelf` (LLVM's). On the Fedora box it is
GNU's.

| | before (01-before) | after (02-after, from the tarballs) |
|---|---|---|
| `readelf -h` Type, svrngs and give, x86_64 and aarch64 | EXEC | DYN |
| `readelf -d` | no dynamic section | FLAGS BIND_NOW, FLAGS_1 NOW PIE, no NEEDED |
| `readelf -l` | GNU_RELRO, first LOAD at 0x1000000, no INTERP | GNU_RELRO and DYNAMIC, first LOAD at 0, no INTERP |

01-before builds with the script's own Linux line from before the change:
`cargo zigbuild --release --locked -p svrngs -p give --target <triple>`.
02-after runs the whole changed `build-release.sh`, which built macOS's two
as well, and reads the bytes its tarballs carry. On the Fedora box, `file`
says `ELF 64-bit LSB pie executable, x86-64 ... static-pie linked` for both
(04-x86_64).

## They run

- **x86_64** (04-x86_64): on the Fedora box (Linux 7.3.3, Fedora 45),
  `./svrngs version` says `svrngs 1.11.1 (c25fcd5f-dirty)` and exits 0.
  `./give` prints its usage and exits 2, as it does with no command.
- **aarch64** (05-aarch64): in the Mac's Podman machine (Linux 7.0.12
  aarch64, Fedora 44), from a busybox image, the same: `svrngs version`
  exits 0 and `give` prints its usage and exits 2.

## The plant, refused (03-plant)

An ambient `RUSTFLAGS`, even an empty one, replaces cargo's per-target
rustflags, which would silently ship an EXEC again. So `build-release.sh`
now reads each Linux binary's ELF type (bytes 16-17 of the header) and
refuses one that is not DYN, the way it refuses a platform that does not
build. `RUSTFLAGS= scripts/build-release.sh` built both Linux targets as
EXEC (`readelf -h`). It refused both (`not built: linux-x86_64
linux-aarch64`), left their tarballs out of `get/` and `SHA256SUMS`, and
exited 1. Only aarch64's refusal line is in the log's last 8 lines; x86_64's
line was cut by the log's tail. macOS's svrngs from this run is the same
binary as 02-after's (sha256 `d9b134f0…`, darwin-aarch64), so only the
tarball bytes differ.

## Not judged

- **namcap itself**: not rerun on packages built from these binaries.
  readelf reports the ELF type, the flags and the segments that namcap's
  PIE and RELRO warnings are about.
- **More than starting**: only `svrngs version` and `give` with no command
  ran. The static-pie startup relocates the binary before `main`, so a
  run that reaches `main` is the test of that step. Nothing past `main`
  ran: no group, no service, nothing under load.
- **aarch64 on hardware**: it ran in a VM on Apple silicon, not on an
  aarch64 Linux computer.
- **A release**: none was cut. v1.11.1's binaries stay as they were. The
  next release made by `deploy-page.sh` carries this.
