// The glue: `bind(exports)` to `fold(ops, now)`, `now` the page's clock in
// seconds, or undefined to ask who keeps with no time, over either build's exports
// (the wasm instance's, or the module wasm2js wrote), as
// apps/canon/build/call.mjs does it for canon. It moves bytes and nothing
// else; every answer is the fold's (apps/give/fold/src/lib.rs).

export function bind(exports) {
  const enc = new TextEncoder();
  const dec = new TextDecoder();
  return function fold(ops, now) {
    const bytes = enc.encode(JSON.stringify({ ops, now }));
    const at = exports.give_input(bytes.length);
    // Read `buffer` fresh each time: a grown memory is a new buffer.
    new Uint8Array(exports.memory.buffer, at, bytes.length).set(bytes);
    const len = exports.give_call();
    const answer = JSON.parse(dec.decode(new Uint8Array(exports.memory.buffer, exports.give_output(), len)));
    if ('error' in answer) throw new Error(`give's fold: ${answer.error}`);
    return answer.ok;
  };
}
