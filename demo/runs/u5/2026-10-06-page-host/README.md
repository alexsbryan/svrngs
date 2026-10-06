# u5-10: what `svrngs new --origin` asks of a page's host, live (2026-10-06)

`svrngs new house --as Sam --origin <page>` on this host, `RUST_LOG=cli=debug`,
each with its own `$SVRNGS_DIR`, at the fix (`23d5f1c`). Both runs are kept.

- `read-1.txt` (13:02Z, the worktree before the commit):
  `https://www.svrnme.sh/g/` is **refused**, exit 1. The page's own address
  gives the page (2951 bytes). `<nobody>.www.svrnme.sh` resolves but its
  TLS handshake ends in EOF, so it serves nothing.
  `https://group.svrnme.sh/` is **not judged**, exit 3. This Mac's system
  resolver failed the name (`dscacheutil -q host` printed nothing, and
  `getaddrinfo` with AF_UNSPEC raised errno 8). `dig` and AF_INET answered.
  The name was deployed at 05:54 local (`65794de`), so the resolver was
  still holding a stale negative answer. The check reported that it could
  not judge, rather than refusing.
- `read-2.txt` (13:08Z, `23d5f1c`, after the resolver cleared):
  `https://group.svrnme.sh/` **opens each app at an address of its own**.
  The page's address and `<nobody>.group.svrnme.sh` both gave the same
  2951 bytes, and `new` printed "each app opens at an address of its own
  under group.svrnme.sh". Its exit 124 is the run's `timeout 120`, since
  `new` stays running by design. `https://www.svrnme.sh/g/` is **refused**
  again, exit 1, with the same reason as in read-1.
