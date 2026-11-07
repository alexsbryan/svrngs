# svrngs releases

## V1.7, 7 November 2026

V1.7 lets you choose which of your groups can open what you lend, welcomes
new people into a group through a short address, gives the home page one
page per group, and keeps a phone's group working through a lost
connection. Update the computer that keeps your group first, then
everyone else's.

**Update your group's keeper first.** Computers on V1.6 and V1.7 don't
list each other's lends, so until a computer is updated, what it lends
isn't shown to the others, and it doesn't see theirs.

**Choose who can open what you lend.** Each lend on your home page asks
"Who can open it?", with one box for each of your groups. From a terminal,
`svrngs share <port> --group <group>` says which groups can open it; say
`--group` again for more. `svrngs unshare <name> --group <group>` takes
one group off and keeps the rest. When you update, each lend you already
have stays open to the groups this computer is in at that moment. A group
you join later doesn't get it unless you choose. Taking a group off a lend
cuts what its people already have open within two seconds, and an app's
sign-in and its saved entries follow the same choice.

**Welcome people with a short address.** **Invite people here** on a
group's page makes a temporary address that's easy to type, such as
`join.svrnme.sh/cedar-otter-moon`, which you can close at any time. Each
person who opens it gives their name once and installs svrngs. They then
arrive at a welcome to that group and its shared folder, with a way back to
all their groups. If the address service can't answer, the page says that
joining is temporarily unavailable; it never says the invitation has
ended, and people already in the group keep working.

**One page for each group.** The home page now shows your groups first.
Each group has its own page, and **This computer** holds the connection and
the address that opens svrngs again. The old long page stays at `/?ui=a`
for about two weeks.

**A phone keeps going.** A phone on the group's page joins through the
public meeting point when the group's own one can't be reached from it. It
learns the group's meeting points from the group's computers, with no new
invitation. When no computer answers, it says "This page keeps asking."
and tries again by itself until one does, so at home with the internet out
the same group keeps working, with nobody joining again.

**An app carried by several computers is one app.** Its copies on the
group's computers share one address in the group, and whichever computer
carries it serves it. A computer still on V1.6 lists its copy at its own
address, as before.

**Smaller fixes.**
- `svrngs restart` stops svrngs and starts it again, so you can use a new
  one after `svrngs update`.
- `svrngs open --print` shows an address you can open in another browser
  within a minute.
- The install line stops an svrngs that was started by hand, so a join
  after it isn't refused as already running.
- When an app's answer is lost after you send something, the page says to
  check whether it arrived before you send it again.

V1.7 is the build at `group.svrnme.sh`, from the commit tagged `v1.7`. Its
publication and readback are recorded in
`demo/runs/v1.7/2026-11-07/README.md`.

## V1.6, 3 November 2026

V1.6 lets a computer join another group while svrngs is running, and lets
svrngs update itself from the page it came from.

**Join another group without stopping svrngs.** On a computer where svrngs
runs from login, as `install.sh` keeps it, `svrngs join` used to refuse:
"svrngs is already running on this computer; stop it, then run this again."
Now `svrngs join <link>` asks the svrngs already running to join — the same
computer joins the second group, places its folder at `~/svrngs/<group>`,
and the running svrngs keeps running, for both groups. Your home page draws
every group you are in as its own section, and a group joined from a
terminal appears there the next time you come back to the page, with no
reload. A computer whose svrngs is from before this release still says to
stop it first.

**svrngs updates itself from its page.** The home page says when a new
svrngs is ready, with one Update button; `svrngs update` does the same from
a terminal. The new program comes from the page this computer was installed
from, is checked against that page's `SHA256SUMS` before it replaces the
running one, and svrngs restarts into it (launchd on macOS, systemd on
Linux). `install.sh` keeps the checksum it already checks; Give still
follows the install line.

**A stale page worker never answers a newer build**, and the page reloads
itself after an update.

V1.6 is the build at `group.svrnme.sh`, from commit `SOURCE`, tagged
`v1.6`. Its publication and readback are recorded in
`demo/runs/v1.6/2026-11-03/README.md`.

## V1.5, 3 November 2026

V1.5 adds more than one way for computers to reach their group, fixes a
few things the pages did not say or carry correctly, and makes the pages'
JavaScript code easier to read and extend.

**More than one meeting point.** A group can name several of its own
meeting points with `svrngs new --relay <name> --relay <another>`. Computers
try the group's own first, fall back to the built-in ones while none
answers, and return when one answers again. A computer joining starts with
the meeting points in its invitation rather than waiting on an unrelated
internet service first. A local-network-only run joined, shared files and
opened lends with the internet unavailable. Recovery can still take time;
the recorded one-host fallback run took 22–78 seconds to reopen a lend.

**Update computers before using the new invitations.** Invitations carrying
two meeting points need V1.5. Older computers keep using the group's
previously saved meeting point; they do not gain the new fallback behavior.
If updating from V1.3.1 or earlier, follow V1.4's keeper-first and browser
opening instructions below too.

**Loading stays visible.** After making or joining a group, your home page
says it is reading that group until its people are loaded. A group already
on the page keeps its section while that happens.

**A refused Stop says why.** If Stop under **What apps use** cannot be
completed, its reason appears beside that Stop.

**Binary messages travel correctly.** The browser's connection to a lent
app now handles split WebSocket headers, the two-byte length form, and
DataView payloads correctly.

**For developers.** The pages' JavaScript is formatted, checked through
JSDoc types and structural lint, and split into named views, pure machines
and I/O edges. The code in the repository is still the code that runs:
there is no JavaScript build transform. The focused closing review and
verification are recorded in `demo/runs/economize/2026-11-03-js-read/CLOSE.md`
and `ralph/REVIEW_FINDINGS.md`.

V1.5 is the build at `group.svrnme.sh`, from commit `5a4cb567`, tagged
`v1.5`. Its publication and readback are recorded in
`demo/runs/v1.5/2026-11-03/README.md`.

## V1.4, 2 November 2026

V1.4 rebuilds both pages, your computer's and your group's, and closes the
ways another program or app could act as you. Update the computer that
keeps your group first, then everyone else's.

**Update your group's keeper first.** An app you open from your group's
page now saves an entry only once your group's page approves it, while the
app is open to you, so an app can't save as you after its time runs out.
Until the computer that keeps your group runs this svrngs, saving in an app
says "The group's keeper needs to open the new svrngs before apps can save
entries." A computer still on V1.3.1 shows on the group's page as running
an older svrngs, and what it lends isn't listed there until it's updated.

**Open svrngs from your computer once.** Your browser now proves each
request to svrngs with a key of its own, never a cookie, so another
program on your computer can't use your browser's place. The first time
after you update, a page says "Open svrngs from your computer". Do that,
and every app opens as before.

**Apps write only when you ask.** An app on your computer saves an entry
as you only for a request your browser made in that app. A program sends
`x-svrngs-append` with the kind its lend declares. Saying you're coming
through the room's door on the local network isn't saved any more,
because that door can't tell who you are. Open the event from your group's
page instead.

**Safer on your computer.** svrngs's own folder and secret files are
private to you from the moment they're made. A synced folder can't be led
outside itself by a link. A Give card can't reach into your computer's
files. Pantry's token goes only to Pantry.

**The same pages, rebuilt.** Both pages draw what they did, and update in
place when something changes, with no reload.

## V1.3.1, 31 October 2026

V1.3.1 makes everything your group lends something you can bookmark, on a
phone and on a computer, and opens lends in Safari on an iPhone again. The
group's page has the phone half already. Update your computers for the
rest; a computer still on V1.3 works with everyone as before.

**Bookmark what your group lends.** Every lend has an address that doesn't
change. Open it any time in a browser that has joined, and it opens, even
after the browser has stopped it or its pass has run out: svrngs takes you
through your group's page and back to the same place. On a computer, each
lend's row has a link to keep, with **Bookmark this**, and a lend's address
on your own computer, `http://<label>.localhost:<port>/`, now opens there
too, through your home page. A browser that hasn't joined says so, in
place of a page that doesn't exist.

**Open a lend by its name.** `films@Sam` opens Sam's films: add
`#open=films@Sam` to your group's page, or `&open=films@Sam` to the end of
your group's link. Two people with one name are told apart by when each
joined, `films@Sam (joined 24 Oct)`, so a name never opens the wrong thing,
and a name that matches nothing says so. The day someone joined is now
counted in UTC on every page, so a name works the same in every time zone.

**On an iPhone, keep your group on the Home Screen.** Safari can forget a
site, and you with it. A page on the Home Screen keeps its own, but it
doesn't share Safari's, so add your group's page to the Home Screen first
and join from there. The page says so before it asks your name, and the
Home Screen shows svrngs's mark.

**Lends open in Safari on an iPhone again.** Since V1.2, Safari on an
iPhone said "This could not be opened" for every lend, while other phone
browsers opened them.

**In two groups?** A lend's address opens in the group that lends it. If
both lend it, the page asks which.

**Smaller things.** Every page shows svrngs's mark in its tab. On a
computer, the group's page says how a lend reaches you, under **How this
reaches you**. An address the page can't read says why. A browser that
keeps no cookies for sites on localhost is told so, in place of going round
and round.

V1.3.1 is the build at `group.svrnme.sh`, from commit `086e216b`, tagged
`v1.3.1`.

## V1.3, 27 October 2026

V1.3 lends anything on a port, a game included, and Give keeps one for
your group on whichever of your computers is on. svrngs now opens only
from your own computer, and a program shows its token. Update every
computer in your group: a computer still on V1.2 shows a port a friend
lends as an app it can't open, and has no Play.

**Lend a game, or anything on a port.** svrngs carried only pages before.
Now a program that speaks UDP or TCP, such as a Factorio or Minecraft
server, is lent with `svrngs share <port> --udp --kind <name>`, or
`--tcp`. On each friend's home page its row has **Play**. Pressing it
opens the game on their own computer and copies the address to paste into
the game: "Connect to 127.0.0.1:41821." The address is the same each
time, so the game's list of servers keeps working. While Play is on, any
program on that computer can reach the game as that person, so it is
listed under "What acts as you" with its Stop, and nothing listens once
it's off. Someone removed from the group is cut off within seconds,
whatever their own computer does. Not yet played between two homes: one
rehearsal ran 9 minutes of Factorio with both computers on one desk, with
no desync and no drop. How a game plays through a relay is still to be
seen.

**Keep a game for the group.** [Give](apps/give/README.md) keeps an app
such as a Minecraft or Factorio server on one of your computers at a time,
in Podman, and copies its world to everyone every five minutes and
whenever it stops. Someone brings Give to the group once, with `give page
<group>`, and then everyone has its page. **Start it for the group** asks
only what the game's makers ask of whoever runs it, such as Minecraft's
terms. Moving out? **Move it to…** another computer: its person says yes,
and it starts there from the last copy, so nothing is lost. If the
computer keeping it goes quiet for fifteen minutes, anyone whose computer
can keep it is asked **Keep it here?**, with the newest copy. When the old
computer comes back, what it had since is kept as a copy anyone can
**Start instead**. Nothing is merged and nothing is thrown away. A
computer that keeps needs Podman and Give's program, and Give's page gives
the line for the program: svrngs's own line with `--give` at the end, from
your group's page. Computers without them still play and still hold the
copies. That's the trade: a game kept by whichever of you is on, a few
minutes of play at risk when its computer dies, for a server nobody rents.

**Open svrngs from your computer.** Your home page, and each app your
computer opens, now open only from svrngs on this computer. svrngs puts an
opener in your Applications, or on Linux in your applications menu: open
it, and your home page opens. After that your bookmarks keep working,
across restarts. Reached any other way, the page says "Open svrngs from
your computer". Before, a program in a container on a Mac could reach your
home page and act as you. Now it is refused.

**For developers.** A program on this computer that uses the record API
shows the token in svrngs's data folder, `Authorization: Bearer <the
token file>`; without it every route answers 401 `no_token`, naming the
file. svrngs's command and its own apps already do. `svrngs open` opens
the home page from a terminal. A carried app's `svrngs/acts` says the page
its group's links are on, as `page`. Give links no part of svrngs: it
uses the record API, `svrngs share`, the group's folder and its own page,
and an app joins its shelf as one `compose.yaml` with five lines of
`x-svrngs` (`apps/give/README.md`).

**A friend's computer that restarts is found again in seconds.** Before,
when a friend's computer restarted, yours could go on trying the way it
had reached the old one, and what you wrote waited a minute or more to
reach them. Now yours lets that go and finds them again, in about 4
seconds (`demo/runs/redial/2026-10-27/`).

**In Money, the payer names who was in on it.** Money's form asks "Who
was in on it?", with each other person by name. Each one named sees
"Kim says you were in on Friday’s pizza: $18.00." on their own Money
page, with "Yes, I was in on pizza" and "Not me". The payer names, but
never signs for anyone: a share is yours only once you say so. Money's
page shows it as soon as you're named, with no reload.

**Your home page leads with what's in your group.** Each app is a link
of its name, such as **Money**, with what it does as you beside it: "It
reads and writes who paid for what, and who owes whom." Pressing it is
still your yes, and nothing else is asked; the phone's group page says it
the same way. Opening an app to this WiFi has moved below your group's
list. The buttons say **Invite someone to House** and **Add your phone or
another computer**, and how to open the page again is at its foot. In
no group yet, it shows only how to join a group from a link, or make
one. After you join, the group's page says "You’re in House, as Ana",
and each app says what it's for before what opening it lets it do.

**Two people with one name stay two in Money and Events too.** Money keeps
a balance for each person, and Events lists each person who's coming, even
when two of them share a name. The pages tell them apart by when they
joined: "sam (joined 3 Oct)". So do the lists of who's visiting, who was
removed, and whose computer didn't answer. An entry a computer wrote for
someone before V1.2 still counts as theirs. If an older Money said someone
paid sam back while two people are called sam, Money says it can't tell
which sam, and the payer can say it again.

V1.3 is the build at `group.svrnme.sh`, from commit `d21578a6`, tagged
`v1.3`.

## V1.2, 25 October 2026

V1.2 brings House rules, where your house writes down what it
agrees and how it agrees it, and it makes plain what can act as you: an app
says what it does as you when you open it, and only your own phone or
computer writes as you. Update every computer in your group: until all of
them are on it, an older one leaves out what your phone writes in an app.

**Opening an app is your yes.** The button that opens an app says what it
does as you: "Open Money. It reads and writes who paid for what, and who
owes whom.", or "Open Sam’s notes. It writes what you say there into
House’s record, as you. It also opens your friends’ films as you."
Pressing it is the yes, and nothing else is asked. An app gets a question
only when it reaches for something you didn't open it for, such as a page
you opened from a bookmark trying to open a friend's films as you. It is
told "Say yes on your home page", and the question waits there, one tap.

**What acts as you, in one place.** Your home page, and the group's page
on your phone, list everything that can act as you, by what it can do:
what reads and writes for you, what opens your friends' things as you,
what knows your name, what is open to this WiFi, and what can decide for
you, which is only your own computer and your own phone. Each line says
since when, and has its Stop. A Stop holds from the app's next request on
a computer, and within 12 hours on a phone. When an app's files change,
one line says "1 app changed. See what’s new", and the version you have
keeps running until you press **Open the new version**.

**Only your own phone or computer writes as you.** Before, what you did on
your phone was written by whichever computer your phone reached, and a
member's computer could write in someone else's name or choose which
version of an app ran for them; the page could only say "bo, through cy’s
computer". Now your phone signs what you do on it, your computer signs
what you do on it, and every other computer only carries it. Nobody
else's computer can choose what runs as you. An entry an older svrngs
wrote that way still shows as "bo, through cy’s computer", and still
counts.

**Your phone may ask you once.** After the update, your phone's group page
may show "1 app changed. See what’s new" for an app you had already said
yes to. That yes went through a computer, and now only your phone's own
counts. Until you press **Open the new version**, the version you last
said yes to on your phone, or the app as your group brought it, runs for
you. Nobody on a computer is asked.

**Two people with one name stay two.** Their yeses, their versions and
what they wrote are their own, and the pages tell them apart by when they
joined: "Ana (joined 3 Oct)". Not yet everywhere: Money's balances and
Events' list of who's coming still count two people of one name as one.
That's next.

**In Money, only you change your entry.** If kim tries to change your £40
dinner, it stays counted as you wrote it, and Money says why kim's change
isn't: "kim tried to change alex’s entry, which only alex can change".
Events already worked this way.

**What you won't notice.** The key your phone opens an app with stays where
the app's own pages can't copy it. A frame or a form a page sends into a
friend's app is held to the same rule as the page's own requests to it.

**What an app you open can do.** An app can't load code from anywhere
but its own files, or send what it reads to a CDN or a tracker by
accident. It can still share what it reads, if its code does. What
protects you is that your group chose it: its files are pinned, each new
version runs as you only after your yes, and the record says who brought
or changed it. That's the trade: apps from people you know, trusted as
you trust the programs on your computer, rather than a sandbox. Bring
what you trust.

**When a friend's computer runs an older svrngs,** what you write from your
phone in an app they lend waits, and the page says "Sam’s computer runs an
older svrngs: what you write here reaches your group when a computer on
the new svrngs is on."

**House rules.** On your home page, under **Bring things to the table**,
tick House rules and press **Bring the ticked ones to** your group. The
first person to open it answers one question, "How does your house decide
its rules?", with one of three answers: anyone proposes a rule and it holds
after two days unless someone objects; most of you agree; or everyone
agrees. Then anyone writes a rule and presses **Propose this rule to
everyone in your house**. The rest of you press **Agree**, or **Object** and
say why, and the page says when it has become your house's rule. You change
how your house decides the same way you agree to anything else.
- Everyone in the group has a say, from the day they joined until the day
  they left. "Most" and "everyone" count the people in your house when the
  rule was proposed, so a rule agreed last month stays agreed when someone
  joins or leaves today.
- When people leave, the rest of you still decide, including how you
  decide.
- An agreement made from a phone is signed on that phone, so it reads as
  that person's alone: "Agreed: Dee."
- It runs on every computer and phone in your group. Nobody keeps a server
  on, and nobody keeps the book.

[`demo/house-rules.md`](demo/house-rules.md) walks through it on one
computer, with three people each on a computer of their own.

**canon decides, svrngs shows.** House rules is built on
[canon](https://github.com/alexsbryan/canon), a library for a group's
rules, at its release 0.5.0. canon decides every answer the page gives:
what is a rule, what is still waiting, who has a say. svrngs carries what
each of you wrote and vouches for who wrote it. It never reads a rule and
never enforces one. If your house agrees on quiet hours from 10pm, nothing
in svrngs stops a party at 11; you do.

**Its files run as you only after your yes.** House rules is on the table
like Money and Events, so V1.1's rule holds for it too. If someone in the
group changes its files, your page says "1 app changed. See what’s new",
and the version you have keeps working until you press **Open the new
version**.

**In the run.** A house of four, three on computers and one on a phone,
chose "most of you agree" and agreed on quiet hours over one objection.
Then one person left, and the house changed to "everyone agrees". Another
left, and the two still there changed it back. Each person was a program, and everything ran on one
computer. The scene took 91.2 s. House rules' 5.7 MB reached the other
two computers within 1.2 s of being brought, in each of three takes. Once
the third person agreed, every computer and the phone listed the rule
within 6.1 s. The phone took 4.5 to 4.8 s to open House rules, through
n0's relays. Nobody has used House rules in a real house yet
(`demo/runs/canon/2026-10-17-house/README.md`).

**What House rules doesn't do yet.**
- canon can also draw people by lot, take turns with a shared thing, and
  let a smaller group decide its own corner. The page offers none of these
  yet. In canon 0.5.0 they would count the house itself among its people,
  so they wait for a canon that counts only people.
- Right after the agreement that changes how your house decides, your page
  can show the old way, with nothing proposed, for about a second. Open it
  again and it shows the new one (#206).
- After someone leaves, a phone can reach that person's computer first and
  say "This browser could not open the page.", while other computers in the
  group are on. Open the page again (#205).

**If you build apps.** A carried app's `svrngs/acts` now says each person's
time in the group, `spans`, those who left included, so an app can count a
past entry with whoever was in the group when it was written. Each act
carries `person_key`, the same from each of a person's devices: key on it,
and show the name. Your computer's record API writes only as you. A lent
app that writes for whoever opens it names the act in its answer's
`x-svrngs-append` header, and the asker's own computer or phone signs it,
within the kind its lender lent it as. [`docs/DESIGN.md`](docs/DESIGN.md)
§3 has each.

**In the runs.** Every demo scene plays on each check of this release, each
person a program on one computer, in Chromium and WebKit: Money, Events,
House rules, the grapevine, the doc, the group's life and the pantry, and
films and movie night where Jellyfin is installed. The last full check
passed in 801 s. Checking who signed each entry costs a group's 2,000
entries about 4 ms more when phones wrote them, on 72 ms
(`demo/runs/authorship/2026-10-24/COST.md`). Nobody has used this release
in a real group yet.

V1.2 is the build at `group.svrnme.sh`, from commit `0c9dc094`, tagged
`v1.2`.

## V1.1.3, 20 October 2026

V1.1.3 says goodnight. When the computer lending something doesn't answer,
as when its lid is closed, opening what it lends shows that person's
window, dark:

> ☾ Alex’s window is dark
>
> Alex’s computer didn’t answer, so Alex’s doc can’t open right now. It
> opens again once that computer answers.
>
> Try again
>
> Goodnight from Alex.

It's the same on a computer and on a phone, where the page names what
you opened. It never says the computer is asleep, or since when. A
program that asks still gets svrngs's JSON, `away`.

On a phone:
- **llama-server's chat page opens.** The phone's page asks for gzip and
  unpacks it, where llama-server answered 415 before.
- **A live app comes back after the lid opens.** Once the lender's
  computer answers again, the sockets the phone's page opens to it open
  again, without opening the page again.
- **If the first computer to answer then fails,** the phone's page asks
  the next one instead of stopping (#205).

That's the trade: a phone fetches the goodnight page from
`group.svrnme.sh` when it needs it, and without it says the plain words.
In a run on one Linux machine, with a phone in each engine on the Mac, the
goodnight showed 4.0 seconds after the phone asked. A socket opened on the
first try once the computer was back, and the chat page opened in under
3.3 seconds (`demo/runs/v1.1.3/2026-10-20/`).

Everything else is V1.1.2's. House rules is still for the next release.

V1.1.3 is the build at `group.svrnme.sh`, from commit `d1b4309e`, tagged
`v1.1.3`.

## V1.1.2, 18 October 2026

V1.1.2 puts your group on a screen. On your home page, each group has
**Show** your group **on a screen**. It opens a page for a TV, a spare
laptop or a projector:
- everyone in the group, lit while their computer answers;
- what each person lends, inside their window, each opening in a tab as you;
- who joined and what they lent since the screen opened, newest first, the
  newest marked.

The page reads your group again once a second. The list lives on that
screen only, and starts over when you open the screen again. It never
says when someone went off: their window just stops being lit.

Your home page now lists people in the order they joined, whoever started
the group first, and so does the group's page on a phone. Before, the
order came from their keys.

That's the trade: a screen left open asks every computer in your group
what it lends, once a second. With 25 computers on one machine, the one
showing the screen spent at most a tenth of a core on it, and each
person's window lit about 4 seconds after they began to join, most of it
their wait for n0's relay. When the computer that started the group
stopped answering, another person's screen drew it "did not answer" 3.5
seconds later, and everything else still opened
(`demo/runs/screen/2026-10-18/`, `demo/runs/v1.1.2/2026-10-18/`).

Everything else is V1.1.1's. House rules is still for the next release.

V1.1.2 is the build at `group.svrnme.sh`, from commit `aacb3fe5`, tagged
`v1.1.2`.

## V1.1.1, 18 October 2026

V1.1.1 stops a Linux computer from holding on to memory. Each time a page
asked it what everyone in your group lends, it kept the memory that answer
used and never gave it back: about 20 KiB each time in a group of 25. A
home page left open, which asks once a second, grew by about 300 MB an
hour. If you run svrngs on Linux, run the install line again; it updates
in place. On a Mac nothing changes, since macOS gives the memory back.

The Linux build now uses mimalloc to hand out memory, in place of the one
that came with the toolchain that builds it. With 25 computers on one
Linux machine, a computer asked back to back kept 0.43 KiB each time
instead of 21, and asked once a second for half an hour it stayed flat.

That's the trade: a Linux computer starts out using about 15 to 35 MB
more memory, and the first time it asks everyone in a group of 25 at
once, it spends about half a second more CPU doing it. After that, asking
costs it no more than before. Those numbers are from one 32-core machine,
and nobody has measured a laptop yet (`demo/runs/v1.1.1/2026-10-18/`).

Everything else is V1.1's. House rules is still for the next release.

V1.1.1 was built from commit `6db0cccf`, tagged `v1.1.1`.

## V1.1, 17 October 2026

V1.1 closes a way for one person in a group to act as everyone else, and
fixes what an audit of V1 found. If you installed svrngs before 17
October, run the install line again; it updates in place.

**An app runs as your group brought it.** Before, anyone in a group could
change the files of an app the group had brought, such as Money or Events,
and everyone else's browser ran the change as them, writing in their name.
Now each computer serves an app as it was brought. When someone changes
it, your home page says who and asks, "Open the new version?" The new
version runs for you only after you press **Yes, use the new version**. The
old one keeps working until then.

**The group's page can't reach programs on your computer.** The published
page allowed connections to your own computer, left over from testing. It
never needed them, and now it can't make them.

**What the audit fixed:**
- A link pasted on your home page, or an app's request to send you
  somewhere, stays inside the app it came from. A crafted link could open
  another website.
- A member can't make every computer in the group use gigabytes of
  memory, or hang, with one entry numbered far ahead of their last.
- A forged copy of an entry that arrives first can't hide the real one.
- If your computer loses power while it writes, the next entry it writes is
  no longer lost.
- When you leave a group, your computer keeps telling the others until
  each one answers, so a computer that was off at the time hears it once
  it's back.
- A computer whose clock runs fast can't make every invite link look
  expired.
- A computer whose clock reads before 1970 doesn't let anyone into a lent
  app after their access has run out.

Everything under "What V1 doesn't do yet" still holds.

V1.1 was built from commit `a555a225`.

## V1, 16 October 2026

svrngs keeps what your group shares on your own computers: the money you
split, the nights you plan, the films and music you own, a friend's model.
Everyone in the group opens it from a link in the chat you already use.
Nobody makes an account, and nothing asks for a password.

A group is the people you already talk to: a house, a team, the friends you
plan with, about twenty-five people. svrngs runs on each person's computer
and connects them directly. Your phone joins through the computers of the
people in your group.

V1 is early. Every scene below was run end to end on one computer, with a
program playing each person. Nobody has used it with real people yet.

### Start a group

One person makes the group. On a Mac or a Linux computer, open Terminal,
paste this line and press Enter:

```
curl -fsSL https://group.svrnme.sh/install.sh | sh -s -- 'https://group.svrnme.sh/'
```

It installs svrngs in your own folder and checks the download against the
release's list. It starts svrngs now and whenever you log in, then opens your
home page in the browser. It needs no password and changes none of your
computer's settings. On the home page:

1. Type a name for the group and your own name.
2. Press **Make a group**, then **Make a link**.
3. Send the link to your chat.

Someone who opens that link on a computer sees the same one line, with
your group already in it. On a phone, they type their name and press
**Join**. Either way, they're in within seconds, as themselves.

[`demo/start.md`](demo/start.md) walks through it, with every line and
button and what each one says.

### What you can do

**See who's around, and what's in the group.**
- Your home page shows each group's people by name, lit while their
  computer is on.
- Below them is one table of everything the group has: what it is, whose it
  is, and how it opens.
- A computer that doesn't answer is named as not answering. It is never
  shown as on, and never left out.

**Lend what's already running, from the page.**
- Whatever runs on your computer can be lent to your group: Jellyfin or
  Navidrome for the library, Immich for photos, Actual for money,
  llama-server for a model.
- Your home page lists what's running here. One press lends it, and
  everyone in the group can open it.
- For Jellyfin, if its sign-in helper runs beside it, everyone gets in as
  themselves with no account to make.

**Split a bill, plan a night.**
- Money and Events come with svrngs. Tick them on your home page to bring
  them to a group.
- They live in the group's folder on every computer that holds it, so they
  keep working when the computer that brought them is off.
- In the recorded run, five newcomers each settled a gas bill and played a
  film within 37 seconds of opening the link.

**Your phone reaches whoever's awake.**
- A phone dials the computer that sent its link and every other lender at
  once. It goes through the first that answers and tells you whose: "through
  Cy's computer".
- When the link's computer was asleep, the phone opened Money in under 4
  seconds. It used to wait 14.
- Open the group in a second tab and it says the group is open in another
  tab, with **Open it here**.

**Join another group, or leave one.**
- To join a second group, paste its link on your home page. You don't need
  a terminal.
- **Leave this group** leaves it. Your other computers find out, and the
  others see "Ana left on 16 Oct".
- If you keep the order of removals in a group, the page asks you to hand
  that on first.

**Ask across groups, through people you both know.** The grapevine is a
reference app for developers. Here is how a tile saw finds its way from one
group to another:
1. Sam asks for a tile saw in the building's group.
2. Jo is in that group and in the block's group too, and carries the ask
   there with one tap.
3. Kim offers a saw, and Sam says yes. Only then do the two see each other's
   names.

The replies are sealed: Jo's computer passes them on and can't read them.
The scene runs end to end in 20 seconds, with three computers run on one
machine. It's in [`apps/grapevine/`](apps/grapevine/), and
[`demo/grapevine.md`](demo/grapevine.md) walks through it.

### If you build apps

Anything you can run on localhost, you can share with your group. Your app
gets three things and needs nothing else:
- the person asking, verified, in request headers;
- a record the whole group keeps, which outlives the computer your app runs
  on;
- reach to every member's computer.

There's no login, no SDK and no bundle to make. [`docs/DESIGN.md`](docs/DESIGN.md)
§3 is the whole surface.

The record keeps which computer wrote each entry beside the person it
speaks for, so your app can show "Bo, through Cy's computer" wherever it
shows who did what. Money, Events and the grapevine already do.

A grapevine kind is one file and one line. The third kind, rooms, is 24
lines. A kind's file imports nothing, so it can't touch routing, sealing or
who carried what. [`apps/grapevine/kinds/README.md`](apps/grapevine/kinds/README.md)
shows how.

### Where your things go

Your group's money, plans, files and messages stay on the computers of the
people in the group. Nobody else holds a copy, and that includes us. We
host the page a link opens and the downloads the install line fetches.
Both are static files, and none of your group's things pass through them.

Computers reach each other directly when they can. When they can't, the
traffic goes through n0's public relays, encrypted end to end. Traffic from
a phone's browser always goes through a relay. A relay sees that two
computers are talking and how much, but never what they say.

Inside the group, people see what's lent to them and what the group's apps
show. On the grapevine, everyone in a group that a sealed reply crosses can
see that a reply passed, with its size and its time. They can't see its
words. The grapevine's page says so on every thread.

If you installed svrngs before 16 October, run the install line again; it
updates in place. Earlier builds let any program on your computer reach
your friends' lent apps as you. This release closes that.

### What V1 doesn't do yet

**Leaving is final from that computer.** You can't rejoin a group you left
from the same computer. You can rejoin from your phone, which keeps a key
for each group.

**An app can't prove who wrote an entry.** A program on a member's computer
can write in another member's name. V1 shows which computer wrote the entry
rather than refusing it. The fix after V1 is for a phone to sign its own
writes.

**It runs on n0's relays.** svrngs has a relay of its own, which anyone can
run. It's built, and switched off until it's hosted for production.

**Computers: Mac and Linux.** The install line has builds for macOS and
Linux, on Intel and on ARM. Phones join from the browser. There's no
Windows build.

### What changed underneath

An architectural audit this week found code kept for history and two paths
doing one job. Cutting them closed a security hole along the way, the one
the install note above describes. V1 is smaller than the code before the
audit, even with leaving and the grapevine added:

| | Before the audit | V1 |
|---|---:|---:|
| Lines of code in svrngs | 16,963 | 16,350 |
| Lines in its core | 3,149 | 2,992 |
| Writing an entry in a group of 25 holding 10,000 | 0.79 s | 0.064 s |

- **The record.** It no longer signs a digest of everything held into every
  entry. In a group of 25, that digest was 88% of each line.
- **Apps.** They can no longer erase their own earlier entries on every copy.
- **Connections.** A computer reaches another directly over its encrypted
  connection, with no local port in between.

V1 was built from commit `de8923f7`. svrngs is open software under the AGPL,
version 3 or later.
