// The engine's side of every call (DESIGN C4): it runs before any app code,
// removes what would let two copies compute different answers, and is the
// fold's own loop and the JSON boundary, so neither is in shell code.
// Evaluated as a script; its value is the function `quickjs.rs` calls.
(function () {
  'use strict';
  // Date, WeakRef, FinalizationRegistry and Intl are never added to the
  // context (quickjs.rs picks its intrinsics); these come with the base objects.
  delete Math.random;
  for (const proto of [String.prototype, Number.prototype, Array.prototype, Object.prototype, BigInt.prototype]) {
    for (const name of ['localeCompare', 'toLocaleLowerCase', 'toLocaleUpperCase', 'toLocaleString']) {
      delete proto[name];
    }
  }
  const parse = JSON.parse;
  const stringify = JSON.stringify;
  const plain = Object.prototype;
  const own = Object.keys;
  const hasOwn = (o, k) => Object.prototype.hasOwnProperty.call(o, k);

  /** Where `v` stops being exactly JSON, or null if it never does. */
  function inexact(v, at) {
    if (v === null || typeof v === 'string' || typeof v === 'boolean') return null;
    if (typeof v === 'number') {
      if (!Number.isFinite(v)) return `${at} is ${v}`;
      return Object.is(v, -0) ? `${at} is -0` : null;
    }
    if (Array.isArray(v)) {
      for (let i = 0; i < v.length; i++) {
        if (!hasOwn(v, i)) return `${at}[${i}] is a hole`;
        const why = inexact(v[i], `${at}[${i}]`);
        if (why) return why;
      }
      return null;
    }
    if (typeof v === 'object' && Object.getPrototypeOf(v) === plain) {
      for (const k of own(v)) {
        const why = inexact(v[k], `${at}.${k}`);
        if (why) return why;
      }
      return null;
    }
    const what = typeof v === 'object' ? `a ${Object.getPrototypeOf(v)?.constructor?.name ?? 'bare object'}` : typeof v;
    return `${at} is ${what}`;
  }

  /** The envelope `quickjs.rs` reads: `{ok}` or `{inexact}`. */
  function out(v, at) {
    const why = inexact(v, at);
    return why ? stringify({ inexact: why }) : stringify({ ok: v });
  }

  return function call(app, name, a, b) {
    const needs = name === 'fold' ? (a == null ? ['init', 'reduce'] : ['reduce']) : [name];
    const missing = needs.find((n) => typeof app[n] !== 'function');
    if (missing) return stringify({ missing });
    switch (name) {
      case 'fold': {
        let state = a == null ? app.init() : parse(a);
        for (const act of parse(b)) state = app.reduce(state, act);
        return out(state, 'the state');
      }
      case 'propose':
      case 'say':
      case 'due':
        return out(app[name](parse(a), parse(b)), `what ${name} returned`);
      default:
        return stringify({ missing: name });
    }
  };
})();
