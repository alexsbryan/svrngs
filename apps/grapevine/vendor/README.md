# Vendored for the seal

Grapevine seals each message with NaCl's `crypto_box`, as tweetnacl's
`box` (GRAPEVINE_DEMO §13.5). The library is copied whole and unchanged
from npm and pinned here. Nothing is fetched at run time.

| directory | package | version | npm `dist.integrity` | sha256 of the tarball |
|---|---|---|---|---|
| `tweetnacl/` | `tweetnacl` | 1.0.3 | `sha512-6rt+RN7aOi1nGMyC4Xa5DdYiukl2UWCbcJft7YhxReBGQD7OAM8Pbxw6YMo4r2diNEA8FEmu32YOn9rhaiE5yw==` | `5f8dc49cb4483e206ba3ebb22abbccfc0218c59b60fc78cce24197ced5b9e102` |

It is the tarball's `nacl-fast.js`, `package.json` and `LICENSE` (the
Unlicense, public domain). Its `nacl.js`, the minified copies, `nacl.d.ts`
and its README are left out. 1.0.3 is tweetnacl's last release.

`grapevine.test.mjs` checks each file against its sha256 below, and fails
on one byte changed:

| file | sha256 |
|---|---|
| `tweetnacl/nacl-fast.js` | `6bcd37a3b20dce913f82d4b23e4e2b661058b4b953df8a3f8c45d56ac4f72447` |
| `tweetnacl/package.json` | `929dc634f6c254c5881612db9a4c52e2ef16cc1b3e3da3343b9e31e6cf037448` |
| `tweetnacl/LICENSE` | `88d9b4eb60579c191ec391ca04c16130572d7eedc4a86daa58bf28c6e14c9bcd` |

To check a copy against npm:

    npm pack tweetnacl@1.0.3
    mkdir t && tar xzf tweetnacl-1.0.3.tgz -C t
    for f in nacl-fast.js package.json LICENSE; do cmp tweetnacl/$f t/package/$f; done
