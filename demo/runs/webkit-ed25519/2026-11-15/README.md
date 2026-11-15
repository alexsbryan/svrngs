# Linux's WebKit refuses an Ed25519 seed that begins with a zero byte, 2026-11-15

Why `carried::an_app_s_entry_is_approved_by_the_group_s_signer_and_carried::webkit`
failed once in the gate image at b7aece62 (its first open: "OPEN:
DataError: Data provided to an operation does not meet requirements") and
once in 8 runs alone at that commit (its fourth open: "SIGN: refused: Data
provided to an operation does not meet requirements"), and passed the rest.

Each open makes keys from fresh random seeds: the holder's (web/page/hand.js,
imported by web/open.js `withHolder` and, its key lost, by web/lend.js
`holdPass`) and the approver's (web/sign.js `approver`). Each was imported
as PKCS #8 (RFC 8410).

## What was measured

In the gate image (Playwright 1.59.1; Linux's WebKit is WPE, its WebCrypto
libgcrypt's), each probe on a page at 127.0.0.1. `runs.txt` has the output.

- `ed.mjs`: 3,000 random seeds, imported as PKCS #8, signed, verified.
  WebKit: 7 refused at the import, DataError, each seed's first byte 0
  (the six samples kept: `00…`); none failed later. Chromium: none.
- `jwk.mjs`: 300 seeds, 200 with the first byte 0, each imported both ways,
  the JWK `{kty: OKP, crv: Ed25519, d, x}` with `x` from Node, and its
  signature compared with Node's for the same message. WebKit: PKCS #8
  refused all 200; the JWK took all 300 and signed each as Node does.
  Chromium: both forms, all 300.
- `page.mjs`: web/authority.js's `ed25519`, as changed, in the page: all
  300 imported, each signing as Node does and matching its holder
  (`holderMatches`), in both engines.

## What it means

A person on Linux whose browser is WebKit's (GNOME Web, and the tests'
stand-in for a phone's Safari) had about one open in 256 to 430 fail (one
or two seeds an open). The Mac's WebKit, whose WebCrypto is not
libgcrypt's, passed its gates. The page now imports every Ed25519 seed as
a JWK, with the public half each site already holds: the holder the group's
page named (open.js), the approver's `actor` (sign.js); the worker is
handed the JWK where it was handed the PKCS #8 bytes (lend.js).
