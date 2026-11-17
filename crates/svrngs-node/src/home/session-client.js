// WebSockets use a one-use open minted by a proved fetch. An app's own
// service worker runs in a realm of its own this page starts, never in the
// session worker (session.js `realmKept`), as on a phone beside the lend's.
(() => {
  const Native = window.WebSocket;
  // A socket's `on<type>` handlers, which a page sets on it by name, as on
  // a browser's WebSocket; tsc reads no such field off the class. Each is
  // handed what `dispatchEvent` takes, the DOM's Event: named so, since
  // scripts/vocab_check.py reads a JSDoc comment as code, and `event` is an
  // app's noun there (apps/events/nouns.txt).
  /** @typedef {Parameters<EventTarget['dispatchEvent']>[0]} Dispatched */
  /** @typedef {Record<string, ((e: Dispatched) => void) | null | undefined>} Handlers */
  class Socket extends EventTarget {
    /**
     * @param {string | URL} input
     * @param {string | string[]} [protocols]
     */
    constructor(input, protocols) {
      super();
      const url = new URL(input, location.href);
      url.protocol = url.protocol.replace('http', 'ws');
      const own = new URL(url);
      own.protocol = own.protocol.replace('ws', 'http');
      // Another origin's socket is the browser's own, which `new` answers
      // in this one's place.
      if (own.origin !== location.origin)
        return /** @type {Socket} */ (/** @type {unknown} */ (new Native(input, protocols)));
      this.url = url.href;
      this.readyState = 0;
      this.binaryType = 'blob';
      this.bufferedAmount = 0;
      this.extensions = '';
      this.protocol = '';
      (async () => {
        const path = url.pathname + url.search;
        const res = await fetch(`/svrngs-socket?at=${encodeURIComponent(path)}`, {
          method: 'POST',
        });
        if (!res.ok) throw new Error('this browser could not open the socket');
        const { code } = await res.json();
        if (this.readyState !== 0) return;
        url.search += `${url.search ? '&' : '?'}__svrngs_socket=${code}`;
        const socket = protocols === undefined ? new Native(url) : new Native(url, protocols);
        // Absent until the node has answered, which `send`, `close` and the
        // rest read it by.
        /** @type {WebSocket | undefined} */
        this.socket = socket;
        socket.binaryType = this.binaryType;
        for (const type of ['open', 'message', 'error', 'close'])
          socket.addEventListener(type, (fired) => {
            this.readyState = socket.readyState;
            this.protocol = socket.protocol;
            this.extensions = socket.extensions;
            // The native socket's own kind of notice, with its fields: a
            // MessageEvent's data, a CloseEvent's code and reason.
            const copy = new /** @type {new (type: string, init: Dispatched) => Dispatched} */ (
              fired.constructor
            )(type, fired);
            this.dispatchEvent(copy);
            /** @type {Handlers} */ (/** @type {unknown} */ (this))[`on${type}`]?.(copy);
          });
      })().catch((why) => {
        this.readyState = 3;
        const failed = new ErrorEvent('error', { message: why.message });
        this.dispatchEvent(failed);
        /** @type {Handlers} */ (/** @type {unknown} */ (this)).onerror?.(failed);
      });
    }
    /** @param {string | Blob | ArrayBuffer | ArrayBufferView<ArrayBuffer>} data */
    send(data) {
      if (this.readyState === 0)
        throw new DOMException('The socket is not open', 'InvalidStateError');
      this.socket?.send(data);
    }
    /**
     * @param {number} [code]
     * @param {string} [reason]
     */
    close(code, reason) {
      this.readyState = 2;
      this.socket?.close(code, reason);
    }
    /** @param {BinaryType} value */
    set binaryType(value) {
      this.binary = value;
      if (this.socket) this.socket.binaryType = value;
    }
    // The constructor sets it, through the setter, to 'blob'.
    /** @returns {BinaryType} */
    get binaryType() {
      return /** @type {BinaryType} */ (this.binary);
    }
    get bufferedAmount() {
      return this.socket?.bufferedAmount ?? 0;
    }
    set bufferedAmount(_) {}
  }
  for (const [name, value] of Object.entries({ CONNECTING: 0, OPEN: 1, CLOSING: 2, CLOSED: 3 })) {
    Object.defineProperty(Socket, name, { value, enumerable: true });
    Object.defineProperty(Socket.prototype, name, { value, enumerable: true });
  }
  window.WebSocket = /** @type {typeof WebSocket} */ (/** @type {unknown} */ (Socket));
  // Absent where the origin is not secure (Service Workers,
  // `navigator.serviceWorker` is `[SecureContext]`).
  const workers = /** @type {ServiceWorkerContainer | undefined} */ (navigator.serviceWorker);
  // svrngs's worker, an older one, not answering a realm's hand-over.
  const UNHEARD = "svrngs's worker did not answer: open this page again";
  // The app's own worker: a dedicated worker at its script, loaded through
  // svrngs's worker under the policy this app's pages get, and handed the
  // fetches this page asks once its prelude runs; its registration a stand-in.
  if (workers)
    workers.register = async (script, options) => {
      const svrngs = workers.controller;
      if (!svrngs) throw new Error('svrngs has not opened this origin');
      const url = new URL(script, location.href);
      const realm = new MessageChannel();
      const told = new MessageChannel();
      const heard = () => new Promise((done) => (told.port1.onmessage = done));
      let next = heard();
      svrngs.postMessage({ realm: url.href, isolated: crossOriginIsolated }, [
        realm.port1,
        told.port2,
      ]);
      await Promise.race([next, new Promise((_, no) => setTimeout(no, 10000, new Error(UNHEARD)))]);
      next = heard();
      const app = new Worker(url, { type: options?.type ?? 'classic' });
      app.postMessage('svrngs-realm', [realm.port2]);
      await new Promise((ready, failed) => {
        void next.then(ready);
        app.onerror = (error) =>
          failed(new Error(`this app's worker did not start: ${error.message}`));
      });
      // What an app's page reads of its registration; it has no more.
      return /** @type {ServiceWorkerRegistration} */ (
        /** @type {unknown} */ ({
          scope: new URL(options?.scope ?? './', url).href,
          active: svrngs,
          update: async () => {},
          unregister: async () => false,
        })
      );
    };
})();
