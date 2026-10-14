# Vendored for the page

The stranger's page checks each line's Ed25519 signature and each piece's
BLAKE3 in the browser, before it plays. These two libraries do it, copied
whole and unchanged from npm, pinned here; nothing is fetched at run time.

| directory | package | version | npm `dist.integrity` | sha256 of the tarball |
|---|---|---|---|---|
| `noble-curves/` | `@noble/curves` | 2.4.0 | `sha512-P4/62zrgfH33CneE3Dn4WhJVA22YUU0eR51wKIan4NVRvwsA0YnPTwWGpNbpuacSujmSFLvyzpyuR30+fbq2Ew==` | `55279c71b0201d6c0e58d0206d15825359e02a3932f326ba7b7909ac2d334aa9` |
| `noble-hashes/` | `@noble/hashes` | 2.4.0 | `sha512-X5XaVWZIBCT7HHZGm5I7ZQXDwLG+bGXuSrMQAW+7Zvl87h1kmc1ZB1VSRJcpUfoUrGQp4Fkoxm5kZ+Ms+aW+eA==` | `e1946149b780017b2564fcc092cb01c04e1d7f20627d0296c17da8868f4436dc` |

Each is the tarball's `.js` files, `package.json` and `LICENSE` (MIT, Paul
Miller); its TypeScript sources, `.d.ts` and README are left out. To check
a copy against npm:

    npm pack @noble/curves@2.4.0 @noble/hashes@2.4.0
    mkdir c h && tar xzf noble-curves-2.4.0.tgz -C c && tar xzf noble-hashes-2.4.0.tgz -C h
    for f in $(cd noble-curves && find . -type f ! -name README.md); do cmp noble-curves/$f c/package/$f; done

`@noble/curves` imports `@noble/hashes/...` by bare name; the page's import
map points that at `noble-hashes/`.
