// SPDX-License-Identifier: AGPL-3.0-or-later
// The shim in each page of a lend on a phone (DESIGN §3, "Lent
// apps", call 1): what a page asks of its browser that the lend's worker
// (lend.js), not the browser, holds. It runs first in the page, put there
// by the worker, which names its lend (`l`) and the cookies its pages may
// read (`c`) on this script's address.
// - `document.cookie` reads the lend's cookies the worker keeps, which no
//   worker can set in the browser, and writing one hands it to the worker.
//   An answer that set cookies names those pages may read, taken in at once.
// - `WebSocket`, which a worker never sees, is carried to the worker over
//   a message port, and the worker speaks it to the lend's computer.
// - An app's own worker, which would put itself over the lend's, is
//   handed to the lend's worker, which asks its listeners first.
// - An entry the worker signs for the app is first approved by the group's
//   signer (sign.html), which only a page can frame: framed here, once, at
//   the address the worker derives from its own host (lend.js `SIGNER`).
//
// A classic script, which shares the page's global scope: its names are
// kept inside the one function it calls, apart from the app's.
(() => {
  // A classic script, so the browser sets `currentScript` while it runs.
  const me = new URL(/** @type {HTMLScriptElement} */ (document.currentScript).src);
  // Absent where the origin is not secure (Service Workers,
  // `navigator.serviceWorker` is `[SecureContext]`).
  const workers = /** @type {ServiceWorkerContainer | undefined} */ (navigator.serviceWorker);
  const worker = () => workers?.controller;
  /** @type {Promise<HTMLIFrameElement> | undefined} */
  let signer;
  // What the lend's worker posts its pages: an entry to approve and the
  // signer that approves it (lend.js `approved`), else store.js `LOST`,
  // which has no `approve`.
  /** @typedef {{ approve?: object, signer: string }} Approving */
  workers?.addEventListener(
    'message',
    async (/** @type {MessageEvent<Approving | null>} */ event) => {
      const port = event.ports[0];
      if (!event.data?.approve || !port || event.source !== worker()) return;
      const url = new URL(event.data.signer);
      signer ??= framed(url);
      // Loaded, the frame has its window.
      const inFrame = /** @type {Window} */ ((await signer).contentWindow);
      inFrame.postMessage(event.data.approve, url.origin, [port]);
    },
  );

  // The signer at `url`, framed hidden, once it has loaded.
  /**
   * @param {URL} url
   * @returns {Promise<HTMLIFrameElement>}
   */
  function framed(url) {
    return new Promise((loaded) => {
      const frame = document.createElement('iframe');
      frame.src = url.href;
      frame.hidden = true;
      frame.onload = () => loaded(frame);
      document.documentElement.append(frame);
    });
  }

  // What lend.js's `worker` answers a script handed it.
  /** @typedef {'running' | { error: string }} Started */
  /**
   * @param {{ worker: string }} message
   * @returns {Promise<Started>}
   */
  const ask = (message) =>
    new Promise((done) => {
      const { port1, port2 } = new MessageChannel();
      port1.onmessage = (event) => done(event.data);
      const serving = worker();
      if (serving) {
        serving.postMessage(message, [port2]);
      } else {
        done({ error: 'no worker serves this page' });
      }
    });

  // A `name=value; …` line of cookies, as `[name, value]` pairs.
  /**
   * @param {string} line
   * @returns {[string, string][]}
   */
  const pairs = (line) =>
    line
      .split('; ')
      .filter((cookie) => cookie.includes('='))
      .map((cookie) => [
        cookie.slice(0, cookie.indexOf('=')),
        cookie.slice(cookie.indexOf('=') + 1),
      ]);
  const jar = new Map(pairs(me.searchParams.get('c') ?? ''));
  Object.defineProperty(Document.prototype, 'cookie', {
    configurable: true,
    get: () => [...jar].map(([name, value]) => `${name}=${value}`).join('; '),
    set: (line) => {
      // `split` gives at least one part, the empty string's included.
      const [pair, ...attrs] = /** @type {[string, ...string[]]} */ (String(line).split(';'));
      const equals = pair.indexOf('=');
      if (equals < 1) return;
      const gone = attrs.some((attr) => {
        const [name, value = ''] = /** @type {[string, ...string[]]} */ (
          attr.split('=').map((part) => part.trim())
        );
        return (
          (/^max-age$/i.test(name) && Number(value) <= 0) ||
          (/^expires$/i.test(name) && Date.parse(value) < Date.now())
        );
      });
      const name = pair.slice(0, equals).trim();
      if (gone) {
        jar.delete(name);
      } else {
        jar.set(name, pair.slice(equals + 1).trim());
      }
      worker()?.postMessage({ cookie: String(line) });
    },
  });

  // An answer that set cookies names, in `x-svrngs-page-cookies`, those
  // the lend's pages may read now (lend.js); taken in before the page
  // reads the answer, by fetch or by XMLHttpRequest, as a browser's
  // cookies are (Immich reads its sign-in's unreloaded).
  const PAGE_COOKIES = 'x-svrngs-page-cookies';
  /** @param {string | null} line */
  const takeIn = (line) => {
    if (line === null) return;
    jar.clear();
    for (const [name, value] of pairs(line)) jar.set(name, value);
  };
  const fetched = window.fetch;
  /** @param {Parameters<typeof fetch>} args */
  window.fetch = function (...args) {
    return fetched.apply(this, args).then((response) => {
      takeIn(response.headers.get(PAGE_COOKIES));
      return response;
    });
  };
  // Unbound on purpose: it is called by `apply` below, with the request as `this`.
  // oxlint-disable-next-line typescript/unbound-method
  const opened = XMLHttpRequest.prototype.open;
  // Either of `open`'s two forms, `async` and after it optional.
  /** @param {[string, string | URL, boolean?, (string | null)?, (string | null)?]} args */
  XMLHttpRequest.prototype.open = function (...args) {
    this.addEventListener('readystatechange', () => {
      // 2, HEADERS_RECEIVED: the answer's headers, before its body.
      if (this.readyState === 2) takeIn(this.getResponseHeader(PAGE_COOKIES));
    });
    // `apply` reads `open`'s last form, whose `async` is not optional.
    return opened.apply(this, /** @type {Parameters<typeof opened>} */ (args));
  };

  // What the constructor's `Object.assign` puts on each socket, which tsc
  // does not read off the call, as a browser's WebSocket names them.
  /** @typedef {Pick<WebSocket, 'binaryType'>} Assigned */
  // A socket's `on…` handler, as `fire` finds it by the event's type.
  /** @typedef {Record<string, ((this: Socket, event: Event) => void) | null | undefined>} Handlers */
  class Socket extends EventTarget {
    /**
     * @param {string | URL} url
     * @param {string | string[]} [protocols]
     */
    constructor(url, protocols) {
      super();
      this.url = new URL(url, location.href).href.replace(/^http/, 'ws');
      Object.assign(this, {
        readyState: 0,
        protocol: '',
        extensions: '',
        bufferedAmount: 0,
        binaryType: 'blob',
      });
      Object.assign(this, { onopen: null, onmessage: null, onclose: null, onerror: null });
      const { port1, port2 } = new MessageChannel();
      this.port = port1;
      port1.onmessage = ({ data }) => {
        if ('open' in data) {
          Object.assign(this, { readyState: 1, protocol: data.open });
          this.fire(new Event('open'));
        }
        if ('message' in data) {
          const said = data.message;
          const { binaryType } = /** @type {Assigned} */ (/** @type {unknown} */ (this));
          const blob = typeof said !== 'string' && binaryType === 'blob';
          this.fire(new MessageEvent('message', { data: blob ? new Blob([said]) : said }));
        }
        if ('closed' in data) {
          const ready = this.readyState > 0;
          this.readyState = 3;
          if (!ready || data.closed === 1006) this.fire(new Event('error'));
          this.fire(
            new CloseEvent('close', {
              code: data.closed,
              reason: data.reason,
              wasClean: data.closed !== 1006,
            }),
          );
        }
      };
      const listed = protocols === undefined ? [] : [protocols].flat();
      const asked = { socket: { url: this.url, protocols: listed } };
      const serving = worker();
      if (serving) {
        serving.postMessage(asked, [port2]);
        return;
      }
      // No worker to carry it: closed as a socket whose connection failed.
      const closed = /** @type {(event: { data: object }) => void} */ (port1.onmessage);
      setTimeout(() => closed({ data: { closed: 1006, reason: 'no worker serves this page' } }));
    }
    /** @param {Event} event */
    fire(event) {
      this.dispatchEvent(event);
      /** @type {Handlers} */ (/** @type {unknown} */ (this))[`on${event.type}`]?.call(this, event);
    }
    /** @param {string | Blob | ArrayBuffer | ArrayBufferView} data */
    send(data) {
      if (this.readyState !== 1) {
        throw new DOMException('the socket is not open', 'InvalidStateError');
      }
      if (data instanceof Blob) {
        void data.arrayBuffer().then((send) => this.port.postMessage({ send }));
        return;
      }
      if (ArrayBuffer.isView(data)) {
        // Of a view, a typed array or a DataView, the bytes it views copied
        // to a buffer of their own: a DataView has no `slice`, and the view's
        // own buffer may hold more than it views (web/tests/shim.test.mjs).
        const viewed = new Uint8Array(data.buffer, data.byteOffset, data.byteLength);
        this.port.postMessage({ send: viewed.slice().buffer });
        return;
      }
      this.port.postMessage({ send: data });
    }
    close(code = 1000) {
      if (this.readyState > 1) return;
      this.readyState = 2;
      this.port.postMessage({ close: code });
    }
  }
  // On the class and on each socket, as a browser's are: Home Assistant's
  // library reads `socket.OPEN`.
  const states = { CONNECTING: 0, OPEN: 1, CLOSING: 2, CLOSED: 3 };
  Object.assign(Socket, states);
  Object.assign(Socket.prototype, states);
  // `CONNECTING` and the rest are on it by `Object.assign`, just above.
  window.WebSocket = /** @type {typeof WebSocket} */ (/** @type {unknown} */ (Socket));

  if (workers) {
    workers.register = async (script) => {
      const said = await ask({ worker: new URL(script, location.href).href });
      if (said !== 'running') throw new TypeError(`the app's worker did not start: ${said.error}`);
      return workers.ready;
    };
  }
})();
