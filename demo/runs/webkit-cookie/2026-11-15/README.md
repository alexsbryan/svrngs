# Linux's WebKit refuses the signer's cookie under .test, 2026-11-15

Why `carried::an_app_s_entry_is_approved_by_the_group_s_signer_and_carried::webkit`
failed on Linux, 3 runs of 3 in the gate image at 250aa65b and in the
Nov 12 rounds (C5, `demo/runs/flakes/2026-11-12/`), and passed on the Mac
(the V1.8.1 gate, `demo/runs/v1.8.1/2026-11-14/gate-0002248d.txt`).

C5 read it as a deletion that reached the cookie store late, and
approve.mjs waited up to 10 s for it (76e29f22). The deletion never came:
in the failing runs the signer logged "adopted the approver … and deleted
its cookie" six times, once for each ask, because each ask found the cookie
again.

## What was measured

`names.mjs`, in the gate image (`localhost/svrngs-gate`, Playwright
1.59.1), with the checkout at 250aa65b mounted at `/work` for its
Playwright and the tests' certificate. Each case: a top-level page at the
parent name sets `__Host-svrngs-money` as `web/page/hand.js` sets the
handoff; a page at the child name frames the parent name, whose script
deletes the cookie as `web/sign.js` `approver` does, then sets a `probe`
cookie with `SameSite=None`. `after` is the frame's `document.cookie`
after both writes; `jar` is what the context's store holds then.
`names.txt` is its output.

| parent (the page) | top (the app) | WebKit | Chromium |
|---|---|---|---|
| group.svrnme.test | money.group.svrnme.test | **both writes refused**: the cookie kept, no probe | deleted, probe set |
| group.svrnme.sh | money.group.svrnme.sh | deleted, probe set | deleted, probe set |
| group.example.com | money.group.example.com | deleted, probe set | deleted, probe set |
| group.svrnme.example.com | money.group.svrnme.example.com | deleted, probe set | deleted, probe set |
| group.svrnme.sh | group.svrnme.sh | deleted, probe set | deleted, probe set |

The frame reads the cookie in every case, and `document.hasStorageAccess()`
answers true in every case. A first run of the same deletion (not kept)
found the four ways of writing it alike: `Max-Age=0`, an `Expires` in
1970, `Max-Age=-1`, and no `SameSite`, each refused from the frame under
.test and each taken at the top level, in a persistent profile and an
ephemeral one.

## What it means

Production's names delete the cookie in Linux's WebKit: group.svrnme.sh
and its apps' names under it are one site there. Only the tests' name was
refused, and it is under .test, which the public suffix list does not
name, while .sh and example.com are named. So the tests now serve the page
at `group.svrnme.example.com` (web/tests/page.mjs `PAGE_HOST`), with the
same shape as production's: the page's name, and each app at a name under
it.
