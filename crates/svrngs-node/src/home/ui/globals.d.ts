// SPDX-License-Identifier: AGPL-3.0-or-later
// The pages' trace sink, a global: ui/trace.js sets it on the page and
// web/lend.js in the lend's worker, and every module calls it through
// globalThis, so one that node loads with no page, as web/tests load
// ui/machine.js, still runs. Read by scripts/types.mjs alone, never served
// (home.rs and scripts/build-web.sh serve named .js).
// `var`, since only a `var` is a property of globalThis; `let` would not be.
// oxlint-disable-next-line no-var
declare var __trace: ((line: string) => void) | undefined;
// The direct path's two plants, set by the browser tests on a lend's page
// and read by web/shim.js (webrtc-page): `__svrngsNoWebrtc` withholds the
// channel's datagrams, `__svrngsCloseWebrtc` closes it after that many.
// oxlint-disable-next-line no-var
declare var __svrngsNoWebrtc: boolean | undefined;
// oxlint-disable-next-line no-var
declare var __svrngsCloseWebrtc: number | undefined;
