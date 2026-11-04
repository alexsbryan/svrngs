# Preflight, 2026-10-26: before a phone

What was found on the operator's Mac before any phone, so that S0 on the
phone tests only what a Mac can't. None of it meets a bar in DESIGN.md §6;
the bars are the phone's.

## It builds and links for iOS

- `./build-xcframework.sh`: `cargo build --target aarch64-apple-ios
  --release --locked`, Finished, with libc at 0.2.189 in `tunnel/Cargo.lock`.
- `cd ios && xcodegen generate && xcodebuild -scheme PhoneTunnel
  -destination 'generic/platform=iOS' CODE_SIGNING_ALLOWED=NO build`:
  BUILD SUCCEEDED, `nm` showing `T _tunnel_start` and `T _tunnel_stop` in
  the extension.
- The link needed `SystemConfiguration.framework` (DESIGN.md §9), and
  `IPHONEOS_DEPLOYMENT_TARGET=16.0` with no compiler cache, or ring's and
  blake3's C came out built for the SDK's iOS 27.1 (`otool -l`).
- `cargo test` in `tunnel/`, on macOS: 4 passed. Each was watched failing
  first, on a plant name uppercased and on the footprint read one field
  early.

## The path carries, on a Mac

The lender was svrngs at `24505c9e`, built from a copy of the tree, with
its own `HOME` and `SVRNGS_DIR`, in a group of its own, `phone-tunnel-run`.
It lent a stand-in, `python3 -m http.server` on 18096 serving one line, as
`svrngs share 18096 --tcp --kind standin`. The tunnel's key came from
`svrngs link phone-tunnel-run --device`, redeemed with `join --no-run` into
a directory of its own (DESIGN.md §3). The core ran in a driver on the same
Mac that called `tunnel_start` with port 28096.

**Carried.** `curl 127.0.0.1:28096/` answered `hello from the lend`.
`tunnel_start` returned 0, and the trace said `door: raw: the lend opened,
or refused lend=standin refused=None`. The samples:

```
{"available":null,"down":0,"footprint":5390936,"open":false,"path":"none","plant":"none","t":1793059826,"up":0}
{"available":null,"down":70730,"footprint":9060952,"open":true,"path":"direct","plant":"none","t":1793059836,"up":123081}
{"available":null,"down":71974,"footprint":9060952,"open":true,"path":"direct","plant":"none","t":1793059846,"up":124035}
{"available":null,"down":91315,"footprint":9224792,"open":true,"path":"direct","plant":"none","t":1793059856,"up":174312}
{"available":null,"down":92276,"footprint":9273944,"open":true,"path":"direct","plant":"none","t":1793059866,"up":179937}
```

So the core holds about 9 MiB on macOS, carrying one small page over a
direct path between two processes on one computer. That says nothing yet
of a film, a relay, or iOS, which are S1 to S3. `available` is null off
iOS, as the core says it is.

**Refused.** With the plant `stranger`, the endpoint's key was a fresh one,
`2ed59bef…`. The trace said `refused=Some(NotAdmitted)` at each try, and
`curl` to 28097 found nothing listening (exit 7), since a refused lend's
listener is dropped with it. The lender's own trace of the
refusal never showed. It is under target `transport`, at warn
(`crates/svrngs-node/src/iroh.rs:554`), and the lender ran with
`RUST_LOG=door=debug,play=debug`. For P4 the lender runs with `transport`
too (DESIGN.md §5).

Not tried on the Mac: the `leak` plant, since a Mac has no memory killer
to see it.
