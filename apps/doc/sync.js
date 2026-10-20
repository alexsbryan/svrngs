// SPDX-License-Identifier: AGPL-3.0-or-later
// The page's half of the doc's socket: the Yjs sync and carets over it,
// and whether the server is answering. The server says HERE every second
// (protocol.js); a page that hears nothing from it for QUIET_MS says
// goodnight (`on.away`), keeps every edit in its own doc meanwhile, and
// opens a new socket until one answers. Then the Yjs sync sends what was
// written here and takes what was written there, and the two are one doc
// again (`on.joined`). Nothing here touches the DOM, so it runs under
// `node --test` against the real server (doc.test.mjs).
//
// On a phone the socket is svrngs's: the lend's worker carries it to the
// lender's computer (svrngs web/shim.js, web/lend.js `socket`). When the
// lender's computer stops answering, that socket neither closes nor says
// so for as long as the connection under it takes to time out, so
// silence, not a close, is what this half reads.
import { AWARENESS, HELLO, HERE_EVERY_MS, LINES, SYNC, json, read } from "./protocol.js";
import { awarenessProtocol, decoding, encoding, syncProtocol } from "./vendor/doc-bundle.js";

/** Silence for this long, five of the server's HEREs missed, is goodnight. */
export const QUIET_MS = 5 * HERE_EVERY_MS;
/** A socket not open after this long is given up and another opened. */
export const CONNECT_WITHIN_MS = 8000;
/** How often the page looks. */
const TICK_MS = 500;
/** The waits before each new socket after one closes, the last repeated. */
const BACKOFF_MS = [500, 1000, 2000, 4000];

/**
 * One page's sync. `on` takes `hello({you, key, lender})`, `lines(entries)`,
 * `away({kept})`, `kept(n)`, `here()` and `joined({kept})`; `kept` is how
 * many edits this page has made while away, said again as each is made.
 * `poke`, when given, is asked before each socket after the first: one
 * plain request to the server, awaited, whatever it answers. `clock` stands in for the page's timers in
 * the tests.
 */
export function sync({ url, doc, awareness, WebSocket, poke = null, on = {}, clock = globalThis }) {
  let ws = null;
  let state = "opening";
  let heard = clock.Date.now();
  let ticked = heard;
  let opening = 0;
  let next = 0;
  let tries = 0;
  let kept = 0;
  let joining = false;
  let poking = false;
  const trace = (what, fields) => on.trace?.(what, fields);

  const send = (bytes) => {
    if (ws?.readyState === 1) ws.send(bytes);
  };
  const message = (type, write) => {
    const e = encoding.createEncoder();
    encoding.writeVarUint(e, type);
    write(e);
    return encoding.toUint8Array(e);
  };
  const mine = () => message(AWARENESS, (e) => encoding.writeVarUint8Array(e, awarenessProtocol.encodeAwarenessUpdate(awareness, [awareness.clientID])));

  doc.on("update", (update, origin, _doc, tr) => {
    if (state === "away" && tr.local) {
      kept += 1;
      on.kept?.(kept);
    }
    if (origin !== self) send(message(SYNC, (e) => syncProtocol.writeUpdate(e, update)));
  });
  awareness.on("update", ({ added, updated, removed }, origin) => {
    if (origin === self) return;
    if ([...added, ...updated, ...removed].includes(awareness.clientID)) send(mine());
  });

  const drop = () => {
    if (!ws) return;
    const old = ws;
    ws = null;
    old.onopen = old.onmessage = old.onclose = old.onerror = null;
    try {
      old.close();
    } catch {
      // A socket already closed has nothing to close.
    }
  };
  const later = (why) => {
    drop();
    next = clock.Date.now() + BACKOFF_MS[Math.min(tries, BACKOFF_MS.length - 1)];
    trace("socket: another in a moment", { why, tries, in_ms: next - clock.Date.now() });
    tries += 1;
  };
  const connect = async () => {
    if (tries > 0 && poke) {
      poking = true;
      await poke().catch(() => {});
      poking = false;
      if (ws) return;
    }
    opening = clock.Date.now();
    trace("socket: opening", { url, state });
    try {
      ws = new WebSocket(url);
    } catch (e) {
      return later(`it could not be opened: ${e}`);
    }
    ws.binaryType = "arraybuffer";
    ws.onopen = () => {
      send(message(SYNC, (e) => syncProtocol.writeSyncStep1(e, doc)));
      if (awareness.getLocalState() !== null) send(mine());
    };
    ws.onmessage = (event) => {
      heard = clock.Date.now();
      tries = 0;
      if (state !== "here") {
        joining = state === "away";
        state = "here";
        trace("server: answering", { joining });
        on.here?.();
      }
      if (typeof event.data === "string") return trace("server: refused — a text message, which the server never sends");
      const m = read(new Uint8Array(event.data));
      if (m.type === SYNC) {
        const reply = encoding.createEncoder();
        encoding.writeVarUint(reply, SYNC);
        const step = syncProtocol.readSyncMessage(m.body, reply, doc, self);
        if (encoding.length(reply) > 1) send(encoding.toUint8Array(reply));
        if (step === syncProtocol.messageYjsSyncStep2 && joining) {
          joining = false;
          trace("doc: joined again", { kept });
          on.joined?.({ kept });
          kept = 0;
        }
      } else if (m.type === AWARENESS) {
        awarenessProtocol.applyAwarenessUpdate(awareness, decoding.readVarUint8Array(m.body), self);
      } else if (m.type === HELLO) {
        on.hello?.(json(m.body));
      } else if (m.type === LINES) {
        on.lines?.(json(m.body));
      }
    };
    ws.onclose = () => later("it closed");
    ws.onerror = () => later("it failed");
  };

  // Goodnight: the server has said nothing for QUIET_MS. Everyone else's
  // carets go, since none of them can move now; the socket is let go, as
  // it may never close by itself.
  const away = () => {
    state = "away";
    const others = [...awareness.getStates().keys()].filter((id) => id !== awareness.clientID);
    awarenessProtocol.removeAwarenessStates(awareness, others, self);
    trace("server: not answering, so goodnight", { quiet_ms: clock.Date.now() - heard });
    on.away?.({ kept });
    later("the server went quiet");
  };

  const tick = () => {
    const now = clock.Date.now();
    // A page its browser froze (a phone locked, a tab put away) heard
    // nothing because it listened to nothing: it is given QUIET_MS afresh,
    // and a socket at once.
    if (now - ticked > 3 * TICK_MS) {
      trace("page: back after its browser froze it", { frozen_ms: now - ticked });
      heard = Math.max(heard, now);
      next = 0;
    }
    ticked = now;
    if (state !== "away" && now - heard > QUIET_MS) return away();
    if (ws?.readyState === 0 && now - opening > CONNECT_WITHIN_MS) return later("it did not open in time");
    if (!ws && !poking && now >= next) connect();
  };

  const self = {
    start() {
      connect();
      self.timer = clock.setInterval(tick, TICK_MS);
      return self;
    },
    stop() {
      clock.clearInterval(self.timer);
      drop();
    },
    get state() {
      return state;
    },
  };
  return self;
}
