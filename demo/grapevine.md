# Grapevine — ask your neighbours' neighbours for a tile saw

**Sam** needs a tile saw on Saturday. Sam's group, **1400 Page**, is Sam
and **Jo**. Jo is also in **Waller block**, with **Kim**, who has one. Sam
asks in 1400 Page; Jo carries the ask to Waller block with one tap; Kim
offers hers. Sam and Kim agree, and only then see each other's names. Jo
passes their messages on and can't read them. When Sam takes the ask
back, it leaves Waller block too.

This walks you through that on one computer, with three people each on a
"computer" of their own: a home folder and a svrngs each. It's for
developers trying the grapevine app (`apps/grapevine/README.md`); a
person in a real group would have svrngs already, from `start.md`.

Every line and every button below is one a recorded run typed or pressed,
on one computer, with a program playing each person
(`runs/grapevine/2026-10-16/README.md`, "the run" below). The run's
whole scene, from Sam's ask to the ask gone from Kim's page, took 20.4 s.

**What everyone in a group sees.** Replies are sealed, so only Sam and
Kim can read them. But they travel through each group's shared history,
so everyone in a group a reply passes through sees that a sealed message
went by, how big it was, which ask it answers and when. Never what it
says. Each page says so on every conversation.

## Before you start

You need a copy of svrngs with its program built, and Node.js 20 or
later. In a terminal, in your copy:

```
cargo build -p svrngs
```

Then open three terminal windows, one each for Sam, Jo and Kim, each in
your copy of svrngs. Every line below is typed in the window it names.
Nothing here needs your password or changes your computer's settings;
everything is kept under `/tmp/grapevine`.

## Make the two groups

**1. Sam makes 1400 Page.** In Sam's window:

```
export HOME=/tmp/grapevine/sam; mkdir -p $HOME
target/debug/svrngs new "1400 Page" --as Sam &
```

The first line gives Sam a home folder of their own, so this svrngs is
Sam's and not yours. The second makes the group and prints its link,
`svrngs://join/…`. svrngs keeps running behind that window's next lines.
It says *"Leave this running; Ctrl-C stops it."*: here `kill %1` stops it
instead (**When you're done**).

**2. Jo joins it, and makes Waller block.** In Jo's window, with the link
Sam's window printed:

```
export HOME=/tmp/grapevine/jo; mkdir -p $HOME
target/debug/svrngs join 'svrngs://join/…' --as Jo --no-run
target/debug/svrngs new "Waller block" --as Jo &
```

The second line says *"You're in 1400 Page, as Jo."* (3.3–4.4 s in the
run's five takes). The third makes Jo's second group, prints its link, and
keeps running for both.

**3. Kim joins Waller block.** In Kim's window, with the link Jo's window
printed:

```
export HOME=/tmp/grapevine/kim; mkdir -p $HOME
target/debug/svrngs join 'svrngs://join/…' --as Kim &
```

It says *"You're in Waller block, as Kim."* and keeps running.

## Start each person's grapevine

**4. In each window, the same two lines:**

```
echo "{\"record\": \"$(cat $HOME/.svrngs/api)\", \"token\": \"$HOME/.svrngs/token\", \"data\": \"$HOME\"}" > $HOME/grapevine.json
node apps/grapevine/server.mjs $HOME/grapevine.json
```

The first tells grapevine where that person's svrngs answers, and where
to keep their own file of sealing keys. The second starts it and prints
its address, `{"port":63219,"at":"http://127.0.0.1:63219/",…}`. Open
that address in a browser: that person's page. It answers only on this
computer, and only that person's own page. There's no terminal after
this.

Each page shows that person's groups and nothing else: Sam's 1400 Page,
Jo's both, Kim's Waller block.

## The scene

**5. Kim lists her tile saw.** On Kim's page, under **What you have**, in
**tool**, type `tile saw` in **What**, keep **In** at Waller block, and
press **List it**. It stays in Waller block and is never passed on.

**6. Sam asks.** On Sam's page, under **Ask your groups**, in **tool**,
type `tile saw` in **What** and `Saturday` in **When**. Keep **In** at
1400 Page, choose **One ring away** in **How far**, and press **Ask**.
Sam's page says *"You asked: tile saw, Saturday"*.

**7. Jo carries it.** Reload Jo's page: under 1400 Page it says *"Sam
asks: tile saw, Saturday"*, with **Carry to Waller block** and, under
it, *"Everyone in Waller block will see what was asked, and that you
carried it; never who asked."* Press **Carry to Waller block**. (Jo's
page showed it 0.5 s after Sam pressed Ask, in the run.)

**8. Kim offers.** Reload Kim's page: *"A neighbour of Jo’s needs: tile
saw, Saturday — carried by Jo"*, and *"You might have this."*, because
Kim listed one. Type `Saturday morning` in **When** and `Waller and
Pierce` in **Where**, and press **Offer**.

**9. Sam reads the offer.** Reload Sam's page. Under **Your
conversations**: *"Someone who saw it offered: when Saturday morning;
where Waller and Pierce"*, and *"Sealed: only you two can read this."*
It takes one turn of Jo's grapevine, which passes messages on every 5
seconds (3.2 s after Kim's press, in the run). Jo's page says *"A sealed
message went by toward the person who asked: 96 bytes"*, and the time,
and nothing it says.

**10. Both say yes.** On Sam's page, type `Sam Reyes` in **Your name,
which they’ll see once you’ve both said yes**, and press **Yes**. Reload
Kim's page until it says *"The person who asked said yes"*: it still
names no one. Type `Kim Okafor` in the same box on Kim's page and press
**Yes**. Kim's page now says *"You both said yes: they’re Sam Reyes"*;
Sam's says *"they’re Kim Okafor"* after one more turn of Jo's (5.3 s in
the run).

**11. Sam takes it back.** On Sam's page, press **Take it back**. Reload
Kim's page: within one turn of Jo's grapevine the ask is gone from
Waller block (5.4 s in the run), because Jo's grapevine took Jo's carry
back too.

## When you're done

In each window, press Control-C to stop grapevine, then type
`kill %1` to stop that person's svrngs. Then:

```
rm -rf /tmp/grapevine
```

## When something does not work

- **`svrngs: …` instead of "You're in …"** after a join. The link was
  copied short, or the window that printed it was closed. Copy the whole
  link again from the window that is still running.
- **`cat: /tmp/grapevine/…/.svrngs/api: No such file or directory`** in
  step 4. That person's svrngs isn't running yet, or stopped. Press Enter
  in that window: if it stopped, the window says so, with what svrngs said
  before it. Then run that person's last line in step 1, 2 or 3 again.
- **"svrngs on this computer isn't answering, so this page can't say who
  you are."** on a page. That person's svrngs stopped. Start it again in
  that window: `target/debug/svrngs &`.
- **"This page answers only at its own address on this computer"**. You
  opened the page at `localhost` or another name. Open the exact address
  its line printed, with `127.0.0.1`.
- **A line with `ERROR noq_proto::connection: PTO expired while unset`**
  in a window, when you stop the svrngs of another. It's svrngs saying a
  connection to a stopped computer timed out; nothing to do.
- **Jo's page shows no ask after Sam's.** Reload it. If it still shows
  none, Jo's svrngs isn't running: see the item above.
