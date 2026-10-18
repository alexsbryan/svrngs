# House rules — agree on quiet hours as a house

**Ana**, **Bo** and **Cy** share a house, **Elm**. Ana brings House rules
to Elm and says how the house decides: most of you agree. She proposes
quiet hours. Cy objects, with a reason; Bo agrees, and with Ana's own
that's two of three, so it holds. Then Bo moves out. The rule still holds,
and the page says who has a say now.

This walks you through that on one computer, with three people each on a
"computer" of their own: a home folder and a svrngs each. It's for
developers trying House rules (`apps/canon/README.md`); a person in a real
house would have svrngs already, from `start.md`, and bring House rules
from their home page as step 5 does.

canon, the library under House rules, decides every answer the page
gives: what is a rule, what is still waiting, who has a say. svrngs carries
what each person wrote and vouches for who wrote it. It never reads a rule
and never enforces one.

Every line below is typed, and every button pressed, by `doc.py`
(`runs/canon/2026-10-17-doc/README.md`), with a program playing each
person. The times are from canon-run, a house of four on one computer
(`runs/canon/2026-10-17-house/README.md`, "the run" below).

## Before you start

You need a copy of svrngs with its program built, and Node.js 20 or
later. In a terminal, in your copy:

```
cargo build -p svrngs
```

Then open three terminal windows, one each for Ana, Bo and Cy, each in
your copy of svrngs. Every line below is typed in the window it names.
Nothing here needs your password or changes your computer's settings;
everything is kept under `/tmp/house`.

## Make the house

**1. Ana makes Elm.** In Ana's window:

```
export HOME=/tmp/house/ana; mkdir -p $HOME
target/debug/svrngs new "Elm" --as Ana &
```

The first line gives Ana a home folder of her own, so this svrngs is
Ana's and not yours. The second makes the group, prints its link,
`svrngs://join/…`, and keeps running behind that window's next lines.

**2. Bo joins.** In Bo's window, with the link Ana's window printed:

```
export HOME=/tmp/house/bo; mkdir -p $HOME
target/debug/svrngs join 'svrngs://join/…' --as Bo &
```

It says *"You're in Elm, as Bo."* and keeps running.

**3. Cy joins.** In Cy's window, with the same link:

```
export HOME=/tmp/house/cy; mkdir -p $HOME
target/debug/svrngs join 'svrngs://join/…' --as Cy &
```

It says *"You're in Elm, as Cy."* and keeps running.

**4. Open each home page.** In each window, the same line:

```
sed 's/127.0.0.1/svrngs.localhost/' $HOME/.svrngs/api
```

It prints that person's home page, `http://svrngs.localhost:…`. svrngs
keeps the address apps reach it at in `.svrngs/api`, and the home page is
at the same number. Open each in a browser, a tab each. There's no
terminal after this.

## The scene

**5. Ana brings House rules.** On Ana's home page, under Elm, **Bring
things to the table** lists Money, Events and House rules, each ticked.
Untick Money and Events, and press **Bring the ticked ones to Elm**. The
page says *"You brought House rules to Elm: everyone in it can open it in
a few seconds."* (221 ms in the run).

**6. Ana says how the house decides.** Reload Ana's home page and press
**Open House rules** in Elm's table. It asks *"How does your house decide
its rules?"*, with three choices and none ticked. Choose **Most of you
agree** and press **Make this how your house decides**. Under **How your
house decides** the page says *"A rule holds once more than half of the
people in your house when it was proposed agree, whoever proposed it among
them."*

**7. Ana proposes quiet hours.** Type `Quiet hours are 10pm to 7am` in
**Your rule** and press **Propose this rule to everyone in your house**.
Under **What’s proposed** it says *"You proposed it on"* and today's date.

**8. Cy objects.** Reload Cy's home page until Elm's table lists **Open
House rules** (under a second after Ana brought it, in the run), and press
it. Under **What’s proposed**, beside the quiet hours, press **Object**.
The page asks *"Why do you object?"*: type `I work late on Fridays` and
press **Send your objection**. It says *"You object: “I work late on
Fridays”"*, and the quiet hours are still proposed.

**9. Bo agrees.** On Bo's home page, press **Open House rules**. Under
**What’s proposed** it says *"Cy objects: “I work late on Fridays”"*.
Press **Agree**. With Ana's, that's two of the three, and the quiet hours
move under **Your house’s rules**: *"Quiet hours are 10pm to 7am (Ana
proposed it on …)"*. Reload Ana's House rules: it lists them too, as
*"you proposed it"*.

**10. Bo moves out.** On Bo's home page, under Elm, press **Leave Elm**.
It says *"You left Elm."* Reload Ana's House rules. The quiet hours still
hold, because Bo had a say when they were agreed. Under **Who has a say**
it says *"Everyone in your house has a say:"*, naming you and Cy, and
*"Bo had a say from … until …, when they left."*

## When you're done

In each window, type `kill %1` to stop that person's svrngs. Then:

```
rm -rf /tmp/house
```

## When something does not work

- **`svrngs: …` instead of "You're in …"** after a join. The link was
  copied short, or Ana's window was closed. Copy the whole link again
  from Ana's window, which is still running.
- **`sed: /tmp/house/…/.svrngs/api: No such file or directory`** in
  step 4. That person's svrngs isn't running yet, or stopped. Press Enter
  in that window: if it stopped, the window says so, with what svrngs said
  before it. Then run that person's last line in step 1, 2 or 3 again.
- **No "Open House rules" on Bo's or Cy's home page.** Reload it. Each
  computer lists House rules once it holds its files, under a second after
  Ana brought it, in the run.
- **"This computer can’t yet say who is in your house, so it can’t say
  what your house agreed."** Open the page again in a minute.
- **Ana's page still shows the quiet hours under "What’s proposed"**
  after Bo agreed. The page shows what it held when you opened it. Reload
  it.
