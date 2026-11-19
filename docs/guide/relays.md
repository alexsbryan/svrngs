# Run a relay for your group

A phone reaches your group's computers through a relay. svrngs runs one for
everyone — the public way — and a group can meet at one of its own instead:
a small computer that stays on, run by someone in the group. This page is
for whoever runs that computer.

A relay only connects people. Everything two computers say to each other is
encrypted end to end, so the relay passes it and cannot read it, and it
keeps nothing.

## What the computer needs

- A name that leads to it, like `relay.example.org`: its `A` record pointed
  at the computer's address, and its `AAAA` too when the computer has one.
- Inbound TCP ports 80 and 443, and UDP port 7842. That is all.
- Nothing else to set up: the relay asks Let's Encrypt for its certificate
  itself the first time it starts, and renews it.

## Start it

```
svrngs relay relay.example.org
```

It says *"Relaying at https://relay.example.org/ for the computers and
phones of the groups this computer is in. Leave this running."* Leave it
running the way you keep any small service — a systemd unit, or a terminal
you keep open. It reads the groups this computer is in from svrngs's data,
so those are the groups it relays for; run svrngs here as usual.

By default it lets through the computers and phones of those groups, and a
phone opening a lend of theirs: a lend's page dials from a fresh key that is
in no group, and the pass the page signs is what the relay admits it by.
Nobody else — a key of no group is refused, and each refusal is said in the
log.

`--open` relays for everyone that reaches it, each computer held to about a
megabit a second with an 8 MB burst, so a relay of yours can be the public
way for a wider group. `--staging` asks Let's Encrypt's trial service
instead, for trying a name first; no browser trusts those certificates.

To check it answers: `curl -fsS https://relay.example.org/ping` prints
nothing when the relay is up. A page at the name is not a relay, and the
check says so: `scripts/relay-answers.sh <url>` does the same from a
checkout.

## Point a group at it

When you make the group:

```
svrngs new house --relay relay.example.org
```

Or later, as the group's keeper:

```
svrngs link house --relay relay.example.org
```

The group's page then meets there first, and a phone's page dials it. Every
link carries the public way beside the group's own, so a newcomer whose key
is not in the group yet, and every phone while your computer is off, still
gets in.

## On a network with no name

```
svrngs relay --at 10.0.0.5:3340
```

serves the same relay over plain HTTP at an address, with no certificate:
for a network with no internet, where no certificate can be had. Computers
can meet there; a phone's page cannot use it, since a browser refuses plain
HTTP from an HTTPS page.

## What it does not do

- It cannot read what passes through it: the connections are encrypted end
  to end.
- It lends nothing itself; it only carries the connection.
- It is not a backup for the public way: if your relay is down, its groups'
  pages fall back to the public way, and back to yours when it answers.
