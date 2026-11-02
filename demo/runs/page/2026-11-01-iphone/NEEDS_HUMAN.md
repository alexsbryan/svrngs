# The iPhone run: the operator's steps

The loop does not run this. These steps are for you, on this Mac and your
iPhone. Once you're done, tell the watching session what each bar in
`PREDICTIONS.md` showed. It writes `README.md` here.

## Before you start

- On your iPhone, open Tailscale and connect. The iPhone (`iphone-14`) was
  last seen on the tailnet 57 days ago.
- Use the main checkout once page-parity is merged. Write down the commit
  you run: `git -C ~/dev/svrngs log -1 --format=%h`.

## 1. Build both v2 pages

```
cd ~/dev/svrngs
cargo build -p svrngs --features test-support
scripts/build-web.sh --ui v2
```

This puts the phone's page in `target/web/site-v2`.

## 2. Serve the phone's page on your tailnet

In a terminal of its own, left running:

```
python3 -m http.server 8787 --bind 127.0.0.1 --directory ~/dev/svrngs/target/web/site-v2
```

In another:

```
tailscale serve --bg 8787
```

It prints an https address, `https://alexs-macbook-pro.tailf85b04.ts.net/`.
If it says the tailnet's https is off, it gives a link to the setting that
turns it on. Turn it on, then run the line again.

## 3. Start Alex's computer, in a data dir of its own

In a terminal of its own, left running:

```
mkdir -p ~/svrngs-scratch
export SVRNGS_DIR=~/svrngs-scratch/page-iphone RUST_LOG=folder=debug
~/dev/svrngs/target/debug/svrngs new Elm --as Alex --ui v2 2>&1 | tee ~/svrngs-scratch/page-iphone.log
```

Don't put the data dir under `/tmp`, because macOS clears it at midnight.
If svrngs says its address is taken, your own svrngs is running on it: quit
that one for the run.

It prints a link, `svrngs://join/<token>`. The page's address and that
token make the phone's link:

```
https://alexs-macbook-pro.tailf85b04.ts.net/#<token>
```

`--origin` is left out on purpose. It refuses any host that does not serve
the page at every name under it, and the tailnet's address doesn't.

## 4. Open Alex's home page on the Mac

In another terminal:

```
SVRNGS_DIR=~/svrngs-scratch/page-iphone ~/dev/svrngs/target/debug/svrngs open
```

Leave that tab open, then switch to another tab in the same window.

## 5. On the iPhone

1. Open the phone's link in Safari. Join Elm as **Sam**. (P0)
2. Under "Bring things to the table", tick **Events**.
3. Tap **Bring the ticked ones to Elm** twice, quickly.
   - Look at what the page says, and where it says it. (P1)
   - Note whether "Bringing…" showed and the button was held. (P2)
4. Without reloading, watch Elm's list of apps for Events. Note how many
   seconds it took, or that it never came. (P3)

## 6. On the Mac

1. Switch back to Alex's home page tab. Don't reload it. Is Events in
   Elm's index? (P4)
2. Count the bring lines in Alex's log (P2):

   ```
   grep -c 'shelf: the apps asked brought' ~/svrngs-scratch/page-iphone.log
   ```

## 7. Tell the watching session

Tell it the commit, each bar's result, the count from step 6, and anything
the page said that you didn't expect. A screenshot of the phone after step
5.4 helps.

To stop, press Ctrl-C in each of the terminals, then run
`tailscale serve reset`, which clears this Mac's serve config: anything else you serve with it goes too.
