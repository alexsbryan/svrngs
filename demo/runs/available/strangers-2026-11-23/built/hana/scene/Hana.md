# Hana in scene

Simulated, by deepseek-flash, through deepinfra (https://api.deepinfra.com/v1/openai). on their laptop, with svrngs’s This computer page open. Cy uses coder and notes in Maple St house. A housemate set the two shares up, and they haven’t used this page before.

**failed**: ran out of patience. Clock 84.0 s, patience left 0 words.

## What the stage counted

| Screen | Words shown | Words read | Things to tap | Seconds |
|---|---|---|---|---|
| This computer | 403 | 149 | 13 | 75.0 |

## Hard faults

None.

## Messages

- heard Cy, in the house chat: After you restart your laptop tonight and sign back in, which of coder and notes can I open tomorrow without you doing anything else? Just tell me here; no need to change anything.
- said in the house chat: notes — it's run by svrngs, so it'll be up after I sign back in. coder isn't running and I'd have to start it myself, so it won't be open tomorrow unless I do that.
- helped: false

## What they said

**Tell a friend how it went.** I was looking at the svrngs "This computer" page to answer Cy's question about which of coder and notes would be available after a restart. I found that notes is already running on port 8099 and is "Run by svrngs", while coder is not running and says "Shared while you run it" with a "Run it with svrngs" action. So my answer was going to be: notes will be open after signing back in, coder will not be unless I start it. But while inspecting the page I tapped "notes" and the interface switched to "You've given up." I didn't get to send the answer to Cy in the house chat — say now returns the same message.

**What do you think this is for?** I think the "This computer" page is where I manage what my laptop shares with my groups: which programs are lent out, who can open them, and what's currently running here. In this case I was using it to check whether coder and notes would still be up after a restart, so I could tell Cy which one he could open without me doing anything.

**What didn't fit together?** What didn't fit together was that I couldn't find a clear setting saying svrngs itself starts when I sign in. The page showed notes already running "by svrngs" and coder as "Shared while you run it" with a "Run it with svrngs" button, which made notes look like the automatic one and coder the manual one. But then tapping "notes" in the running list dropped me to "You've given up" before I could answer Cy, so I never got to send the message or verify the startup behavior directly.

**What did you try that didn't do what you expected?** I tapped "What acts as you" expecting settings about startup, but it showed a permissions breakdown by group instead. Then I tapped "notes" under "Already running here" expecting to open notes or see its details, and the screen changed to "You've given up" — after that, "back", "look", and "say" all just repeated that message, so I couldn't get back to the page or send Cy an answer.

**Where did you stop, and why?** I stopped on the "You've given up" screen, not by choice — the interface switched to it after I tapped "notes" in the running-programs list, and I couldn't get back to the page. My intended last step was to answer Cy in the house chat: notes is run by svrngs and already running, so he should be able to open it tomorrow after I sign back in; coder is shared only while I run it, so he couldn't open that one unless I start it.

**Gave up because:** *(absent)*
