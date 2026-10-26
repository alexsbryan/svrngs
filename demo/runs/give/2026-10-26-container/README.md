# give-session (c): a container on a Mac is refused, and the home page still opens — 2026-10-26

The row's recorded run (`ralph/next/give/STATE.md`, give-session (c)). On a
Mac running podman, a container reaches the Mac's `127.0.0.1` through
gvproxy at `host.containers.internal` (svrn-docs `rings/current/GIVE.md`
§5, measured 2026-10-25): before give-token and give-session it read the
record and the home page, and could say the person's yes. This run asks
svrngs of this build the same things from a container, then opens the home
page as the person does, with `svrngs open`.

## How it ran

```sh
CARGO_INCREMENTAL=0 cargo build -p svrngs
sh demo/runs/give/2026-10-26-container/run.sh > demo/runs/give/2026-10-26-container/run.out 2>&1
```

- svrngs at 6189d90f (`target/debug/svrngs`), over a scratch data dir under
  `target/give-container/`, in no group; podman 6.0.1, its machine
  `applehv`; macOS 27.0.1.
- The container is `docker.io/library/python:3.12-slim`, already on this
  Mac, named `svrngs-test-container`, run `--rm` and removed by the
  script's trap too: none was left (`podman ps -a`).
- `svrngs open` ran with a stand-in for macOS's `open` first on `PATH`,
  which keeps the address it is handed, so no browser of this Mac's was
  opened. The script then redeems that address as the browser would, with
  `curl`.

`run.out` is the whole output, svrngs's traces of every ask included.

## What it showed

From the container, at `host.containers.internal:<api port>`:

| Ask | Answer |
|---|---|
| the record, `GET /v1/groups` | 401, "send this computer's token, which …/token holds" |
| an open, `POST /v1/open` | 401, the same: an open is the token's |
| the home page, typed (`Sec-Fetch-Site: none`, `navigate`) | 401, "Open svrngs from your computer" |
| the home page's yes, `POST /choices`, with its exact `Origin` | 401, "Open svrngs from your computer" |

On the Mac, the person's own `svrngs open`:

- said "Your home page is open in your browser.", having handed the browser
  `http://svrngs.localhost:<port>/svrngs-open?code=…`;
- that address, redeemed: `303 See Other`, `location: /`, and the session's
  cookie set;
- the home page with that cookie: `200`, "Your groups";
- the same code again: `401`, traced "that code was already redeemed".

The traces name the fact behind each answer: `holds=false` for the record
and the open, `shown=Neither` at the home page, `shown=Session` once
opened.

## Verdicts

- **A container's request is refused:** passed, all four asks.
- **The home page still opens:** passed, through `svrngs open`, its code
  single-use.
- Not asked here: a lend's origin from the container. This computer lends
  nothing in no group; the refusal there is the same check
  (`origin.rs` `lent`), held by `crates/svrngs-node/tests/session.rs` and by
  the browser tests' `bare` fetch (a53a6090).
