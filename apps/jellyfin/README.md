# Jellyfin with no sign-in

For whoever lends Jellyfin. Lent as it comes, `svrngs share 8096`, Jellyfin
asks each person to sign in once, with an account you make them. Run this
program beside it and name it when you lend Jellyfin, and nobody signs in:
each person who opens Jellyfin from the group's list is let straight in as
themselves, on a computer or a phone, and two people watching at once stay
apart. Nobody who comes gets your Jellyfin password or anything that
stands for it, nor even their own account's. Measured on one computer
(`demo/runs/mwp/2026-10-08/`).

It runs on your computer only, beside Jellyfin, and you start and stop it
yourself; svrngs does neither. svrngs asks it who each person is the first
time they open Jellyfin; it makes them an account and answers. svrngs does
the rest.

**Once: build it.** With Rust's `cargo`, from this repository:

    cargo build --release --manifest-path apps/jellyfin/mw/Cargo.toml

It is then `apps/jellyfin/mw/target/release/jellyfin-mw`.

**Once: a code for it from Jellyfin.** In Jellyfin: your picture at the top
right, then Dashboard, "API Keys" and +; name it `svrngs`, and save. Copy
the code it lists. It lets the program make an account for each person who
comes; it never leaves your computer.

**Each time: run it.** Start it, paste the code, and press Enter:

    apps/jellyfin/mw/target/release/jellyfin-mw --listen 127.0.0.1:8097

It says it is listening on port 8097, and keeps running.

**Once: lend Jellyfin, naming it.** Lend Jellyfin's own port, and give
`--sign-in` the program's — if Jellyfin is lent already, `svrngs unshare
jellyfin` first:

    svrngs share 8096 --sign-in 8097

`share` asks you no password and says each person is signed in by what
answers on port 8097, with nothing typed. Each person's account is named
for them, can watch every library you do not keep back (below), and is no administrator and can delete
nothing; they are in Dashboard, Users.

**To keep a library back.** Name it as Jellyfin's pages show it, once for
each, when you start the program:

    apps/jellyfin/mw/target/release/jellyfin-mw --listen 127.0.0.1:8097 --keep-back "Home Videos"

It says what it keeps back, and nobody it signs in sees that library; a
library you add later is seen by everyone it signs in after. A name your
Jellyfin has no library by stops it as it starts, naming the ones it has.

If your Jellyfin is not at port 8096, give `share` its port, and the
program `--jellyfin 127.0.0.1:<its port>`. Someone opening Jellyfin for the
first time while the program or Jellyfin is stopped is told your Jellyfin
is not answering now; start it again, with the same code. A restart keeps
each person's account.
