# svrngs releases

## Next, not released yet

The next release brings House rules: your house writes down what it
agrees, and how it agrees it. It's in this repository, and not yet in the
build at `group.svrnme.sh`.

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
- An agreement made from a phone says whose computer it went through:
  "Agreed: Dee, through Ana’s computer."
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
group changes its files, your page asks "Open the new version?", and the
version you have keeps working until you press **Yes, use the new version
of House rules**.

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
past entry with whoever was in the group when it was written.
[`docs/DESIGN.md`](docs/DESIGN.md) §3 has it.

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

V1.1 is the build at `group.svrnme.sh`, from commit `a555a225`.

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
