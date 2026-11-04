# Can Safari hand a key to its worker?

Run 2026-10-30, during HUMAN-ios-observe's check 3: the operator's iPhone
(Safari 26.6.1, Firefox 158.1) opened `index.html` at `http://127.0.0.1:8096/`
through the phone tunnel, whose lend `jellyfin` was pointed at `serve.py` on
the lender's 18097 for the run. The page posts six messages to its worker and
sends each answer back to `serve.py`, which wrote `phone.log`. The first
block of `phone.log` is Playwright's WebKit 26.4 on the Mac.

| | Mac WebKit | iPhone Safari | iPhone Firefox |
|---|---|---|---|
| A a plain message | arrives | arrives | arrives |
| B Ed25519, `extractable: false` (as `web/open.js`) | arrives, signs | **lost, `messageerror`** | arrives, signs |
| C Ed25519, extractable | arrives, signs | **lost, `messageerror`** | arrives, signs |
| D HMAC, `extractable: false` | arrives, signs | **lost, `messageerror`** | arrives, signs |
| E the key as pkcs8 bytes | arrives | arrives | arrives |
| F a key the page kept in IndexedDB, read by the worker | signs | **reads nothing** | signs |

So since 0c9032ed every lend opened from iPhone Safari failed with "the
lend’s worker did not say it holds the pass" (`web/open.js`), and no check
caught it: the checks' WebKit is the Mac's. The operator chose the key's
bytes in the worker's memory alone; 3946c340 is the fix.
