# When something doesn't work

Find the words svrngs showed you, under where you were when it happened.
Each entry says what it means and what to do. Names in the examples (Sam,
Lab, jellyfin) stand for yours.

In a terminal, svrngs puts `svrngs:` in front of what it refuses. `svrngs`
in a command means `~/.local/bin/svrngs` if your terminal doesn't find the
short name.

## Installing, and your home page

- **`svrngs: svrngs runs on Mac and Linux computers, Intel or ARM; this one is … so nothing was installed.`**
  There's no build for this computer, Windows included. Join your group in
  the browser instead, the way a phone does: open the link, type your
  name, press **Join**.
- **`svrngs: paste the whole line from your group's page, its link included: it says where svrngs comes from.`**
  The line was cut short. Press **Copy** on the page again and paste the
  whole line.
- **`svrngs: svrngs did not start within 30 seconds; /Users/<you>/.svrngs/svrngs.log says why.`**
  Open that file; its last lines say what went wrong. Paste the install
  line again once you've dealt with it.
- **`command not found: svrngs`**
  Open a new terminal window, or type the whole path,
  `~/.local/bin/svrngs`.
- **The home page says *"Open svrngs from your computer"*.**
  Your home page opens only from svrngs on your computer. Open svrngs from
  your Applications (on Linux, your applications menu), or type `svrngs
  open`. After that, your bookmark works again.
- ***"This browser didn’t keep what svrngs needs to open this page: it keeps no cookies for sites on localhost."***
  Your browser blocks cookies for `localhost` sites. Allow them in its
  settings, then open svrngs again.
- **`svrngs: svrngs is not running on this computer: start it, then run this again`**, or the home page doesn't open at all.
  Paste the install line again: it updates svrngs and starts it. Logging
  out and back in starts it too.
- **`svrngs: svrngs is already running on this computer (… is held); stop it, then run this again`**, after typing `svrngs` alone.
  Nothing is wrong: svrngs already runs whenever you log in. To open your
  home page, type `svrngs open`.

## Joining a group

- ***"Open this page from your group’s link."***, in red, on
  `group.svrnme.sh`.
  You opened the page without a link. On a computer, the line below it
  makes a new group; to join one, open the link you were sent. On a
  phone, open the link from the chat.
- ***"This link lets nobody in. Ask for a new one."*** (phone), or
  *"This link lets nobody new in: ask whoever sent it for a new one."*
  (home page or terminal).
  The link is more than a day old, or a newer link replaced it. Ask for a
  new one.
- ***"The computer that sent this link did not answer. Try again later."***
  (phone), *"The computer that sent this link cannot be reached. It may be
  off: try again later."* (home page), or `svrngs: the computer that sent
  this link cannot be reached; it may be off, so try again later`.
  The computer that made the link is asleep or off. Ask its owner to wake
  it, then try again.
- ***"Someone in Lab already goes by Bo. …"***
  Someone in the group has that name. If it's you, on your computer press
  **Add your phone or another computer** and open that link here.
  Otherwise, join under another name.
- ***"You’re no longer in Lab on this computer, and you can’t join it again from here yet."***
  You left Lab from this computer, which can't rejoin yet. Join from your
  phone or another computer, with a new link.
- **`svrngs: svrngs is in /Users/<you>/.local/bin, and did not join the group: it said why above.`**
  svrngs installed, but the join was refused. The line above it says why;
  find it in this section.

## Opening something lent

- ***"☾ Sam’s window is dark"***
  Sam's computer is asleep or off. It opens again once that computer
  answers; press **Try again** then. On a phone you may see *"The computer
  lending this did not answer. Try again later."* instead.
- ***"The computer lending this did not answer, so it may be off. What you
  sent may have gone through, so check before you send it again."***
  You saved or sent something and the answer never came back. The app may
  have it already, so look in the app first, and send it again only if it
  is not there.
- ***"Sam's computer did not answer, so what they lend is not listed."***
  The same, seen from your home page or the group's page. Reload when
  Sam's computer is on.
- ***"Sam’s jellyfin is not answering now: try again later"***
  Sam's computer is on, but Jellyfin isn't running there. Ask Sam to start
  it.
- ***"jellyfin is lent to 2 people at once, and 2 are using it now: try again when one stops."***
  Sam chose how many people may use it at once. Try again in a while, or
  ask Sam to let more in; [Lend](lend.md) shows how.
- ***"llama-server is lent without /slots"*** (or another path).
  The lender keeps that part back on purpose, so one person can't see or
  change another's use of it. The rest works.
- ***"This browser hasn’t joined your group yet, so it can’t open this."***
  You opened a bookmark in a browser that never joined. Open your group's
  link from the chat in this browser first, then the bookmark.
- ***"Sam’s computer runs an older svrngs, so this page cannot show what Lab lends."***
  Ask Sam to paste the install line again; it updates in place.
- ***"You are no longer in this group, so nothing of it is open to you."***
  You were removed from the group, or left it.

## On a phone

- ***"You have Lab … open in another tab"***
  One tab at a time reaches a group. Press **Open it here**, or close the
  other tab.
- ***"Sam’s computer did not answer. Try again later."***, when the group's
  page opens.
  No computer in the group answered. A phone needs at least one of them on.
- ***"This could not be opened. Open it again from your group’s page."***
  Open the group's page, from your Home Screen or the chat link, and tap
  the thing again.
- ***"This browser could not open the page."***
  Reload the page. If it keeps saying so, check that a computer in the
  group is on.
- **The group's page has forgotten you**, on an iPhone, and asks your name
  again. Safari can forget a site. Join again from a page added to the
  Home Screen, as [Start](start.md) shows; you may need a new link.

## Lending

- **``svrngs: say what is on port 8080: `svrngs share 8080 --kind <kind>`, its own name if it is one svrngs knows: …``**
  svrngs can't tell what runs there by its number. Type the line again
  with `--kind` and its name, such as `--kind llama-server`.
- **`svrngs: nothing answers on port 8096 on this computer: start it, then share it`**,
  or *"Nothing answers at http://localhost:8096 now: start it, then lend
  it."* on the home page. Start the program, then lend it again.
- **The terminal waits after `svrngs share`**, its last line asking about a
  password. Press Enter if the program takes none.
- ***"You looked just now: nothing else on this computer is running at an address."***
  Start the program you want to lend, then press **Look for programs
  running on this computer** again.
- ***"You already lend something called jellyfin: give it another name."***
  Use `--name` with `svrngs share`, or stop lending the other one first.

- ***"Give it a name and say what it is first: svrngs does not know what runs there."***
  On the home page, fill in **Its name:** and **What it is:**, one word
  each, such as `coder` and `llama-server`.
- **`svrngs: you lend nothing called films from this computer; you lend …`**
  after `svrngs unshare`. Use one of the names it lists.
- ***"films already signs people in with svrngs: make a new secret with `svrngs share <port> --sign-in-with-svrngs`."***
  The secret is shown only once. Run that command for a new one, and put
  it in the app's settings.

## Programs on your computer

- **A program gets a page saying *"Open svrngs from your computer"***, or
  a 401. A program reaches what your group lends only with your svrngs
  token. Give it the text in `~/.svrngs/token` as its API key, or as the
  header `Authorization: Bearer <token>`. [Models](models.md) has examples.
- **`{"error":"this page answers only itself"}`**
  The program was given your home page's address. Give it the address
  **Copy its address for a program** copies instead.

## Removing someone, and leaving

- ***"You asked Sam to remove Dee from House. Dee stays in it until Sam does."***
  Not an error. Only the person who decides removes; they've been asked.
- **`svrngs: … you cannot leave it yet: hand that to someone else first`**,
  or on the home page *"You keep the order of removals in House: choose who
  keeps it after you, then leave."* You decide who is removed, so hand that
  on first: `svrngs handover <name> --group House`. [People](people.md).
- ***"svrngs cannot tell which person called sam in House you mean, so nothing changed."***
  Two people share the name. Add one of the `--joined …` choices it lists
  to the command, quotes included.
- **`svrngs: more than one group here is called Trip; say which by the start of its id: …`**
  Use the first 8 letters of the id it lists in place of the name.
- **`svrngs: svrngs is not running on this computer: start it, then run this again`**,
  after `remove`, `handover` or `leave`. These ask the svrngs running on
  your computer. Open svrngs from your Applications, then run it again.

## TV and the room

- ***"Closed for now: this computer is on another WiFi than the one where you turned it on."***
  It opens again when your computer is back on that WiFi.
- ***"Closed for now: … another program on this computer holds the port it needs."***
  Stop that program and it opens again.
- **`svrngs: nobody in your groups lends jellyfin to this computer now`**
  after `svrngs room`. Check the name your home page shows; its lender may
  have stopped lending it.

## Games

- ***"Closed for now: another program on this computer holds 127.0.0.1:34197. Stop that program, and it opens again."***
  After you press **Play**, something on your own computer already uses
  that port, often your own copy of the game's server. Stop it.
- ***"This computer can’t keep apps yet."***, on Give's page.
  Keeping a game needs Podman and Give's program. The page links **Get
  Podman** and shows the install line with `--give`. [Games](games.md).
- ***"To keep Factorio on this computer, turn on Rosetta in Podman Desktop’s settings, then start Podman again."***
  On a Mac with Apple silicon, do what it says.
- ***"The shelf can’t be read just now: bring Give to your group again."***
  Run `give page <group>` again.
- ***"This copy didn’t start. Try the copy from …?"***
  The newest copy didn't come up. Try the one before it.
- **`svrngs: a port is lent as UDP or as TCP: give one of --udp and --tcp`**
  Use one of them, not both.
