// The glue: one function, `call(request) -> answer`, over either build's
// exports (the wasm instance's, or the module wasm2js wrote). It moves bytes
// and nothing else; every answer is canon's (src/lib.rs).

export function bind(exports) {
  const enc = new TextEncoder();
  const dec = new TextDecoder();
  return function call(request) {
    const bytes = enc.encode(JSON.stringify(request));
    const at = exports.canon_input(bytes.length);
    // Read `buffer` fresh each time: a grown memory is a new buffer.
    new Uint8Array(exports.memory.buffer, at, bytes.length).set(bytes);
    const len = exports.canon_call();
    const text = dec.decode(new Uint8Array(exports.memory.buffer, exports.canon_output(), len));
    const answer = JSON.parse(text);
    if ('error' in answer) throw new Error(`canon: ${answer.error}`);
    return answer.ok;
  };
}
