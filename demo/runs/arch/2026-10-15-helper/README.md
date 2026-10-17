# What lending the sign-in helper lets in — 2026-10-15

Row arch-helper (a) (`ralph/next/arch/STATE.md`), after v1ui's helper run
found the home page offering Jellyfin's helper itself as something to lend
(`demo/runs/v1ui/2026-10-15-helper/README.md`, finding 2). One host, one
run of each: not a rate (AGENTS.md 7).

## How it ran

```
cargo test -q -p svrngs-node --test door a_sign_in_helper_is_never   # a-red.out, at 2ab2522
W=/tmp/arch-helper python3 demo/runs/arch/2026-10-15-helper/mw.py     # mw.out
```

- **The door's test** (crates/svrngs-node/tests/door/sign_in.rs,
  `a_sign_in_helper_is_never_offered_nor_lent_from_the_page`): a stand-in
  helper on 8097, the port well_known.txt names jellyfin's helper at. Ada's
  page looks, a jellyfin is lent beside the helper (as `svrngs share <port>
  --sign-in 8097` writes it, and from the page), the page looks again,
  then is asked to lend the helper as `helper`. Bo opens it, typing ada's
  key and a stand-in into his request.
- **Jellyfin's own helper** (`mw.py`): scratch Jellyfin 12.0.0 on 8096,
  `/Applications/Jellyfin.app`, no films; an API key made; apps/jellyfin/mw
  built and started on 8097 with it, as apps/jellyfin/README.md starts it.
  It is asked what the door asked the stand-in for bo (the head a-red.out
  records), then `GET /`. Jellyfin's accounts are listed before and after.

## What it shows

At HEAD (a-red.out):

1. **The first look offers the helper** (`true`): 8097 answers and no
   lend names it yet (home.rs:279-282 leaves out a lend's sign-in, not the
   table's helper). Once a jellyfin is lent beside it, the second look
   leaves it out (`false`). Priya's page kept the first look's list on
   screen after she lent (the v1ui run's `lent.seen.words`), so it still
   offered it.
2. **The page lends it** (`200`, `kind: helper`): `POST /running` refuses
   nothing for being a sign-in (home.rs:308-369 `lend_one`, share.rs `lend_of`).
3. **Bo reaches it as himself, never as another.** The door told the
   helper bo's key, name, account and device, his typed key and stand-in
   displaced (door.rs:689, every `x-svrngs-` header the client typed): no
   sign-in as another person.
4. **The stand-in hands bo his own token in plain**: `{"secret":
   "secret-8139…", "headers": {"Authorization": …}, "script": …}`. The
   door swaps a sign-in's secret out of every answer only for a lend that
   signs in by it (door.rs:1354-1356, :1459 `outward`); a lend of the helper has none.

Jellyfin's own helper (mw.out):

5. **It answers bo nothing.** The door's head: `403`, "This port answers
   only svrngs's door: no x-svrngs-stand-in." (apps/jellyfin/mw/src/answer.rs:106-112); `GET /`:
   `404`. Neither answer holds the API key or a token; Jellyfin's accounts
   were `priya` before and after: none made.

So a member reaches the helper only as themselves. Jellyfin's helper lets
in nothing, since only the door's own ask carries the stand-in and the door
drops a member's. A helper without that check would give each member their
own token, which svrngs keeps from every browser (DESIGN §3, "The room",
call 6). Low, as REVIEW_FINDINGS #174 says; (b) leaves it unoffered and
unlent.
