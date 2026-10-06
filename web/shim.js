// SPDX-License-Identifier: AGPL-3.0-or-later
// The shim in each page of a lend on a phone (u5-2; DESIGN §3, "Lent
// apps", call 1): what a page asks of its browser that the lend's worker
// (lend.js), not the browser, holds. It runs first in the page, put there
// by the worker, which names its lend (`l`) and the cookies its pages may
// read (`c`) on this script's address.
// - `document.cookie` reads the lend's cookies the worker keeps, which no
//   worker can set in the browser, and writing one hands it to the worker.
// - `WebSocket`, which a worker never sees, is carried to the worker over
//   a message port, and the worker speaks it to the lend's computer.
// - An app's own worker, which would put itself over the lend's, is
//   handed to the lend's worker, which asks its listeners first.
(() => {
  const me = new URL(document.currentScript.src);
  const worker = () => navigator.serviceWorker?.controller;
  const ask = (message) =>
    new Promise((done) => {
      const { port1, port2 } = new MessageChannel();
      port1.onmessage = (e) => done(e.data);
      if (worker()) worker().postMessage(message, [port2]);
      else done({ error: 'no worker serves this page' });
    });

  const jar = new Map(
    (me.searchParams.get('c') ?? '')
      .split('; ')
      .filter((c) => c.includes('='))
      .map((c) => [c.slice(0, c.indexOf('=')), c.slice(c.indexOf('=') + 1)]),
  );
  Object.defineProperty(Document.prototype, 'cookie', {
    configurable: true,
    get: () => [...jar].map(([n, v]) => `${n}=${v}`).join('; '),
    set: (line) => {
      const [pair, ...attrs] = String(line).split(';');
      const eq = pair.indexOf('=');
      if (eq < 1) return;
      const gone = attrs.some((a) => {
        const [k, v = ''] = a.split('=').map((s) => s.trim());
        return (/^max-age$/i.test(k) && Number(v) <= 0) || (/^expires$/i.test(k) && Date.parse(v) < Date.now());
      });
      if (gone) jar.delete(pair.slice(0, eq).trim());
      else jar.set(pair.slice(0, eq).trim(), pair.slice(eq + 1).trim());
      worker()?.postMessage({ cookie: String(line) });
    },
  });

  class Socket extends EventTarget {
    constructor(url, protocols) {
      super();
      this.url = new URL(url, location.href).href.replace(/^http/, 'ws');
      Object.assign(this, { readyState: 0, protocol: '', extensions: '', bufferedAmount: 0, binaryType: 'blob' });
      Object.assign(this, { onopen: null, onmessage: null, onclose: null, onerror: null });
      const { port1, port2 } = new MessageChannel();
      this.port = port1;
      port1.onmessage = ({ data }) => {
        if ('open' in data) {
          Object.assign(this, { readyState: 1, protocol: data.open });
          this.fire(new Event('open'));
        }
        if ('message' in data) {
          const d = data.message;
          this.fire(new MessageEvent('message', { data: typeof d !== 'string' && this.binaryType === 'blob' ? new Blob([d]) : d }));
        }
        if ('closed' in data) {
          const opened = this.readyState > 0;
          this.readyState = 3;
          if (!opened || data.closed === 1006) this.fire(new Event('error'));
          this.fire(new CloseEvent('close', { code: data.closed, reason: data.reason, wasClean: data.closed !== 1006 }));
        }
      };
      const asked = { socket: { url: this.url, protocols: protocols === undefined ? [] : [].concat(protocols) } };
      if (worker()) worker().postMessage(asked, [port2]);
      else setTimeout(() => port1.onmessage({ data: { closed: 1006, reason: 'no worker serves this page' } }));
    }
    fire(event) {
      this.dispatchEvent(event);
      this[`on${event.type}`]?.call(this, event);
    }
    send(data) {
      if (this.readyState !== 1) throw new DOMException('the socket is not open', 'InvalidStateError');
      if (data instanceof Blob) data.arrayBuffer().then((send) => this.port.postMessage({ send }));
      else this.port.postMessage({ send: ArrayBuffer.isView(data) ? data.slice().buffer : data });
    }
    close(code = 1000) {
      if (this.readyState > 1) return;
      this.readyState = 2;
      this.port.postMessage({ close: code });
    }
  }
  Object.assign(Socket, { CONNECTING: 0, OPEN: 1, CLOSING: 2, CLOSED: 3 });
  window.WebSocket = Socket;

  if (navigator.serviceWorker) {
    navigator.serviceWorker.register = async (script) => {
      const said = await ask({ worker: new URL(script, location.href).href });
      if (said !== 'running') throw new TypeError(`the app's worker did not start: ${said?.error}`);
      return navigator.serviceWorker.ready;
    };
  }
})();
