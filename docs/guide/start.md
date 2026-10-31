# Start: make a group, invite friends, join

This page takes you from nothing to a group with your friends in it, on
computers and phones, and to opening the first thing someone lends. It
takes a few minutes. [How it works](how-it-works.md) explains the ideas.

## Start a group on your computer

On a Mac or a Linux computer, open Terminal (on a Mac: Applications, then
Utilities), paste this line, and press Enter:

```
curl -fsSL https://group.svrnme.sh/install.sh | sh -s -- 'https://group.svrnme.sh/'
```

The same line is on `https://group.svrnme.sh/`, with a **Copy** button.
That page also says, in red, *"Open this page from your group’s link."*:
that's for phones. On a computer, the line is all you need.

Within a few seconds it says something like this (your name, chip and
number differ):

```
Getting svrngs for this computer from https://group.svrnme.sh/get/svrngs-darwin-aarch64.tar.gz
Checked: svrngs-darwin-aarch64.tar.gz is the one the release lists, unchanged on its way here.
Installed svrngs in /Users/sam/.local/bin.
Added /Users/sam/.local/bin to where your terminal looks for programs, in /Users/sam/.zshrc. Open a new terminal window, and svrngs runs there by name.
svrngs runs now, and whenever you log in (/Users/sam/Library/LaunchAgents/sh.svrnme.svrngs.plist).
Your home page, the same address each time svrngs starts: http://svrngs.localhost:63742/ Your home page is open in your browser.
To open it again, open svrngs from your Applications.
```

Your browser opens your **home page**. Bookmark it: the address stays the
same each time svrngs starts. With no group yet, it says *"You’re in no
group yet. Join one below from the link someone sent you, or make your
own."*

The line asks no password and changes none of your computer's settings.
Everything stays in your own folder: the program in `~/.local/bin`, its
data in `~/.svrngs`, each group's folder in `~/svrngs`. Run the same line
again any time to update svrngs in place.

In this guide, `svrngs` in a command means `~/.local/bin/svrngs`. In a new
terminal window the short name works; if it says *command not found*,
type the long one.

There is no Windows build yet. On Windows, join a group in the browser,
the way a phone does (below).

## Make a group

On your home page, under **Make a group**, type a name in *A name for the
group* (`Lab`) and your own in *Your name in it* (`Sam`). Press **Make a
group**. The page says *"You made Lab. Invite someone to it below."*, and
shows where Lab's folder is on your computer, with **Open this folder**.

Under Lab you see who is in it, a list of what the group has (empty for
now), and **Bring things to the table**: Money, Events and House rules,
which come with svrngs. [Apps](apps.md) says what each does.

## Invite someone

Press **Invite someone to Lab**. The page says *"Send this link to the
person you are inviting:"*, then the link. Press **Copy the link** and
paste it in your chat.

- The link works for a day, while your computer is on. Your computer has
  to be on when they open it.
- Several people can join by the same link. Making a new one stops the old
  one letting anyone new in; everyone already in stays.
- To add your own phone or second computer, press **Add your phone or
  another computer** instead. That link joins as you, so send it to nobody
  else. See [People](people.md).

## Join on a computer

Bo opens Sam's link in a browser on a Mac. The page says *"On this
computer, paste this line in a terminal: it gets svrngs, joins you, and
opens your home page, with the group’s folder on your disk."*, with the
line and **Copy**. Bo copies it, pastes it into Terminal, and presses
Enter. Among the lines it prints:

```
You're in Lab, as Bo.
Lab's folder is /Users/bo/svrngs/Lab: what anyone in Lab puts there reaches this computer, and what you put there reaches theirs.
```

Then Bo's home page opens, with Lab on it. svrngs names you by the name
your computer knows you by. To choose another, add `--as` and your name to
the end of the line, after the link: `... '<the link>' --as 'Bo'`.

Below the line, the page also has a name box and **Join**. That joins the
browser, the way a phone does, not the computer. On a Mac or Linux
computer, use the line.

### Already have svrngs? Join from your home page

No terminal needed. On your home page, under **Join a group from a link**,
paste the link in *The link someone sent you*. The button then says
**Join Lab**. Type your name in *Your name in it*, and it says **Join Lab
as Bo**. Press it. The page says *"You’re in Lab, as Bo."*

## Join on a phone

Ana opens Sam's link on a phone. The page asks *"What’s your name?
Everyone in Lab sees it."* Ana types `Ana` and taps **Join**. In a few
seconds it says *"You’re in Lab, as Ana"*, and lists what the group lends.
Nothing is installed on the phone.

Someone's computer in Lab has to be on for the phone to reach the group.

### On an iPhone: add it to the Home Screen first

On an iPhone, the page says *"To keep Lab on this phone, add this page to
your Home Screen first, and join from there."* Do that before you type
your name:

1. In Safari, with the link open, tap Share, then Add to Home Screen.
2. Open the new icon from your Home Screen.
3. Type your name there and tap **Join**.

Safari can forget a site, and you with it. A Home Screen app keeps its own
storage, apart from Safari's, so if you join in Safari and add it to the
Home Screen afterwards, the Home Screen one joins again, as someone new.

## Open what someone lends

**On a computer:** on your home page, under Lab, find the row **Sam’s
jellyfin** and press it. It opens in a tab of its own. The *Now* column
says *on*, or *full* when as many people are using it as Sam allows.

**On a phone:** on the group's page, tap **Sam’s jellyfin**. It opens in
the same tab.

Some apps ask each person to sign in once, with an account their lender
makes them (Jellyfin does). Others let you straight in. If nothing is
listed, nobody lends anything yet; [Lend](lend.md) shows how.

## Keep it: bookmark what you use

Everything your group lends has an address that doesn't change.

- **On a computer:** at the foot of the group's section on your home page,
  each lend shows its address, a long name ending in
  `.localhost:63742`, with *Bookmark this.* beside it. Bookmark that
  address, and it opens there next time, through your home page.
- **On a phone:** keep the group's page on your Home Screen. To open one
  thing by name, add `#open=` and its name to the group's page:
  `https://group.svrnme.sh/#open=jellyfin@Sam` opens Sam's jellyfin.

## Next

- [Lend something of yours](lend.md): Jellyfin, Navidrome, Immich, anything on a port.
- [Lend or use an AI model](models.md): llama-server, and the address for a program.
- [Play a game together](games.md): Minecraft, Factorio.
- [Use the apps that come with svrngs](apps.md): Money, Events, House rules.
- [Add your phone, have a guest, remove someone, leave](people.md).
- [When something doesn't work](troubleshooting.md).
