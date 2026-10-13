# Demo C's dry run with n0 out of the path — predictions, before any data

Row relay-run (`ralph/next/relay/STATE.md`): Demo C's dry run again,
`demo/runs/c/dry-2026-10-12-rerun/`'s steps, on this one Mac, every person
a program, with every computer and the phone's page on one `svrngs relay`
on this host and n0 out of the path. Written and committed before the
driver ran. It is not Demo C's run: nobody is a person, so no people bar is
judged (AGENTS.md 7), and no record goes in `runs/c/<date>.json`. One host,
one run of each: not a rate.

## The setup, and what changed from the rerun's

As `dry-2026-10-12-rerun/PREDICTIONS.md`'s setup — the page and the four
tarballs built from this commit and served on this host, Sam's Jellyfin
through its middleware, Ana's Navidrome set as `svrngs share 4533` prints,
`media.sh`'s trip — but:

- **The relay.** `svrngs relay`'s own `relay::run`, started on
  `127.0.0.1:<port>`, plain HTTP, by `the_verbs_relay`
  (`crates/svrngs/tests/n0_off.rs`, built `--release`), a process of its own:
  the verb as typed serves 80, 443 and 7842 on every interface with a
  Let's Encrypt certificate, which this host is not to open
  (`ralph/next/relay/STATE.md`, the row's correction). One port for the
  whole run, chosen free at the start.
- **Sam's group is met there**: `svrngs new lake-trip --as Sam --origin
  <page> --relay http://127.0.0.1:<port>/`. Every other computer learns it
  from the link and keeps it (`cli::keep_group`), and every `svrngs` after
  meets its groups there (`cli::reach`). The rerun's computers met at n0's
  relays, with n0's lookup; nothing of n0's is built into this build
  (relay-default, `cli::reach`: `n0_services: false`).
- **Two takes of the one relay, on one group.** *Open* (`RELAY_OPEN`, as
  `--open`) from before Sam's `new` through every step of the rerun's
  `run.py`. Then *the group's*: the open relay stopped and the relay started
  again at the same address over Sam's data directory, admitting only keys
  standing in a group Sam's computer holds (`relay::Admits`), for the steps
  a relay for one group changes — its members carrying on, Bo removed, and
  two newcomers. Demo C from the start through a relay for the group alone
  would stop at Ana, the first key not yet in it (GROUP_RELAYS.md §4.3, "a
  phone dialling before its key has synced … is the case to measure"), so
  that case is measured at its end instead, once the group stands.
- **Cy's phone keeps its store** (`web/tests/page.mjs` `launch`'s
  `profile`), so it opens the group's page again in the second take as the
  same phone, and its requests off this host are refused and listed by
  Playwright (`offline`, relay-default (b)'s), since lsof cannot see
  WebKit's network process (relay-n0-off's README, prediction 8).
- **What is sampled**: every two seconds, `lsof -i -n -P` of each process
  the run starts — the relay, each person's `svrngs` found by its binary's
  path (so launchd's, from the line's LaunchAgent, too), the page's server,
  Jellyfin, its middleware, Navidrome, and every Playwright browser process
  by its path — each new (process, protocol, remote) kept once. n0's
  addresses are its hosts' (relay-n0-off's `N0_HOSTS`) by `dig` at the
  start, with `relay.svrnme.sh`'s beside them.
- **Traces**: the rerun's, with `cli=debug` and `relay=debug` added.
- **Left out**: `name.py`'s second `ana` (the rerun's prediction 10), a
  refusal no row of this queue touched.

## Predictions

### The open relay: the rerun's steps through it

1. **The relay and the link.** The relay says "Relaying at
   http://127.0.0.1:<port>/ for every computer and phone that reaches it,
   each sending at most 70 Mbit/s. Leave this running." Sam's `new` prints
   the link on the local page within 10 s; every computer's trace names
   that relay alone as where it meets its groups ("cli: the relays this
   computer meets its groups at").
2. **The rerun's steps 1–8 hold** as they held in the rerun, on the same
   bounds: Ana in by the line; each `share` as before; Alex, Bo and Dee
   each done within 5 min, the lends listed within 5 s of the line's exit,
   the film and the album within 20 s of their play, no sign-in; Navidrome
   naming each `<name>-<16 hex>`; Bo removed and refused within 30 s; ana
   asleep said of `ana` alone; one account per person per app.
3. **The newcomers' times near the rerun's.** On one host computers reach
   each other directly and a relay only introduces them, so each of Alex,
   Bo and Dee is done within 20,600 ms (the rerun's 12,736–13,762 ms, half
   again), the line done within 7,000 ms (4,272–4,553), and both lends
   listed within 1,000 ms of the line's exit (346–403).
4. **Cy's phone through the relay.** The name asked and Cy in within
   10 s; both lends listed; Sam's film playing **sooner after its play than
   the rerun's 12,255 ms through n0's relay**, this relay being on
   loopback; the album within 2 min of the link; Navidrome signing in
   `Cy-<16 hex>`.
5. **The folder.** Sam's `after.jpg` on Ana's, Alex's and Dee's disks
   within 5 s (the rerun's 0.7–0.9 s), never Bo's in 60 s; all thirty on
   Eve's disk within 30 s of Eve being in (the rerun's 9.6 s) with Ana's
   computer stopped.

### n0 out of the path

6. **No address of n0's in any sample**, from any process: none of the
   addresses `dig` gives for n0's relays and lookup hosts. Nor
   `relay.svrnme.sh`'s: every computer meets the group's relay. The only
   remote addresses off this host from a `svrngs` are the router's
   (192.168.1.1:5000, iroh's port mapper, as relay-n0-off saw); any other
   process's are named. **The phone asks nothing off this host**: Playwright
   lists no request it refused. No trace names `iroh.link`, `pkarr`,
   `relay.n0` or `relay.svrnme.sh`.

### The group's relay

7. **It admits the group and refuses a key outside it.** Restarted over
   Sam's data, it says "Relaying at http://127.0.0.1:<port>/ for the
   computers and phones of the groups this computer is in." Within 90 s its
   trace admits the computers of Sam, ana, alex, dee and eve, each "the key
   stands in a group held here" with its person, and **refuses Bo's
   computer**, removed, "refused — the key stands in no group held here".
8. **The group carries on through it.** Sam's `after2.jpg` reaches Ana's,
   Alex's, Dee's and Eve's disks within 60 s, never Bo's; Alex's home page
   lists Sam's jellyfin and ana's navidrome; Cy's phone, opened again, is
   admitted and plays Sam's film within 2 min.
9. **Nobody new joins through it.** Fay, at the line on a computer: her key
   refused at the relay; svrngs says "this computer has not reached
   http://127.0.0.1:<port>/ in 10 s, where its group reaches it; …", then
   that "the computer that sent this link cannot be reached; it may be off,
   so try again later", and the line exits non-zero, Fay not in. Those
   words blame Sam's computer, not the relay that refused her — a finding,
   if it holds. Gus, on a phone: refused at the relay; the page says "The
   computer that sent this link did not answer through 127.0.0.1:<port>.
   Try again later." within 70 s of the link opening, Gus never in.

### Not judged

10. **Apart, every people bar, the network**: never-ran, as in the rerun —
    every computer and the relay on one host. **The public half** —
    `relay.svrnme.sh` deployed and a real phone on cellular joining and
    playing through it — is the operator's, written with its command,
    never run here.
11. **A packet capture** of this host, which would judge what lsof cannot
    (each DNS name asked): `sudo`, the operator's.
