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
  // The app's own worker, in a realm of its own (session.js `registered`).
  if (workers)
    workers.register = async (script, options) => {
      if (!workers.controller) throw new Error('svrngs has not opened this origin');
      return (await import('/svrngs-session.js')).registered(workers.controller, script, options);
    };
})();
