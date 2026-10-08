# Jellyfin with no sign-in

For whoever lends Jellyfin. Lent as it comes, `svrngs share 8096`, Jellyfin
asks each person to sign in once, with an account you make them. Run this
program in front of it instead and nobody signs in: each person who opens
Jellyfin from the group's list is let straight in as themselves, on a
computer or a phone, and two people watching at once stay apart. Nobody
who comes gets your Jellyfin password or anything that stands for it.
Measured on one computer (`demo/runs/mw/2026-10-08/`).

It runs on your computer only, beside Jellyfin, and you start and stop it
yourself; svrngs does neither.

**Once: build it.** With Rust's `cargo`, from this repository:

    cargo build --release --manifest-path apps/jellyfin/mw/Cargo.toml

It is then `apps/jellyfin/mw/target/release/jellyfin-mw`.

**Once: a code for it from Jellyfin.** In Jellyfin: your picture at the top
right, then Dashboard, "API Keys" and +; name it `svrngs`, and save. Copy
the code it lists. It lets the program make an account for each person who
comes; it never leaves your computer.

**Each time: run it, and lend it.** Start it, paste the code, and press
Enter:

    apps/jellyfin/mw/target/release/jellyfin-mw --listen 127.0.0.1:8097

It says it is listening on port 8097, and keeps running. Then lend its port,
not Jellyfin's — if Jellyfin is lent already, `svrngs unshare jellyfin`
first:

    svrngs share 8097 --kind jellyfin

`share` then says to make each person an account in Jellyfin's pages: with
this program running, skip that, it makes them. Each is named for the
person, hidden from Jellyfin's sign-in page, can watch every library, and
is no administrator and can delete nothing. They are in Dashboard, Users.

If your Jellyfin is not at port 8096, add `--jellyfin 127.0.0.1:<its
port>`. While the program is stopped, the lend does not open; start it
again with the same code. (A restart keeps each person's account; that
it signs them back in is not yet measured, `demo/runs/mw/2026-10-08/README.md`,
follow-up 4.)
