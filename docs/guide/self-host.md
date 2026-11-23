# Host your own group page and relay

Your group keeps its files and records on its own computers, even if you use
the default page and relay. [Who sees what](how-it-works.md#whats-ours-and-who-sees-what)
explains what those services can see. Here’s how to replace them.

## Use your own relay

Build the command with `cargo build --release -p svrngs`. It’s now at
`target/release/svrngs`; add that directory to your path or use the full path
in the commands below.

Follow [Run a relay](relays.md) to set up its name, ports and certificate.
If it’s your public entry point, start it with `svrngs relay <name> --open`.
Create a group with `svrngs new <group> --relay <name>`. Friends on computers
with svrngs can run `svrngs join <link>` without a hosted page.

To move an existing group, stop svrngs and run
`svrngs link <group> --relay <name>` as the group’s keeper. The new link
carries the new relay.

## Serve your own page and downloads

Run [`scripts/build-web.sh`](../../scripts/build-web.sh) followed by
[`scripts/build-release.sh`](../../scripts/build-release.sh) (each script
lists its prerequisites). Serve `target/web/site/` over HTTPS at your host
and its wildcard subdomains. Match the framing headers in
[`web/vercel.json`](../../web/vercel.json): `frame-ancestors 'none'` everywhere
except `/sign.html`, which needs `frame-ancestors https:`.

Add `--origin https://<your host>/` to `svrngs new`, or change the origin of
an existing group with `svrngs link <group> --origin https://<your host>/`.
To use the one-line installer, use your host in both places in the
[install command](start.md#start-a-group-on-your-computer).

## Replace the default relay in your build

Set `DEFAULT_RELAY` in [`crates/svrngs/src/cli.rs`](../../crates/svrngs/src/cli.rs)
to your relay, then rebuild. That replaces `relay.svrnme.sh` as the build’s
default. n0’s public relays can still serve as an emergency fallback.

See [Packaging and reach](../DESIGN.md#7-packaging-and-reach) for the page,
installer and signed updates.
