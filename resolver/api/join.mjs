// SPDX-License-Identifier: AGPL-3.0-or-later
// The one Vercel Node hosting adapter for the short-invitation resolver.
//
// Vercel's request-headers contract (https://vercel.com/docs/headers/request-headers,
// read 2026-11-04) states the platform "overwrite[s] the X-Forwarded-For header
// and do[es] not forward external IPs ... to prevent IP spoofing"; `x-real-ip`
// and `x-vercel-forwarded-for` are "identical to the x-forwarded-for header".
// That overwritten value is therefore the real trusted ingress IP, and a caller
// cannot set it. There is no client-supplied fallback: an absent or malformed
// value refuses service, exactly as handler.mjs requires.
//
// The Upstash bearer token is read from the server-side environment only; it is
// never written into the site, the build or a process argument. handler.mjs
// defaults the database URL to the operator's Free database.
import { createResolver } from '../handler.mjs';

// One handler, built once per instance from the environment as the module
// loads. A missing token yields handler.mjs's own `configuration` refusal on
// every request rather than a crash at import time.
const handle = createResolver({ token: process.env.SVRNGS_RESOLVER_TOKEN ?? '' });

export default {
  fetch(request) {
    // The platform overwrites this header; its left-most value is the original
    // client. A value handler.mjs cannot parse as an IP is a refusal there.
    const ip = (request.headers.get('x-forwarded-for') ?? '').split(',')[0].trim();
    return handle(request, ip);
  },
};
