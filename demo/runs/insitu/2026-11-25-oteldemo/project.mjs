// The projection: what of a service's OpenTelemetry leaves the site. The
// site decides, never the app: an app's instrumentation, or any library it
// calls, may put a client's name in a file name, an exception's message or a
// log line. So no string value passes as written unless the site can tell it
// carries no data:
//   - a key the site knows (OTel semantic conventions) with a value of the
//     form that key takes: exception.type a class name, a status an integer;
//   - a key the vendor's signed contract types: "int", "hash" (hex), or a
//     closed set of words the admin can read in the contract.
// Every other string is replaced by its shape: its length, the classes of
// characters in it, and a keyed hash, so equal values can be matched across
// events and nothing can be read back from them. An exception's message is
// dropped and its stack trace kept as code locations alone (file, line,
// function): the vendor's code, never the input. Pure: no I/O.
import { createHmac } from 'node:crypto';

const CLASS = [['digit', /\d/], ['upper', /[A-Z]/], ['lower', /[a-z]/], ['space', /\s/], ['punct', /[!-/:-@[-`{-~]/], ['non-ascii', /[^\x00-\x7f]/]];

/** The key that projects nothing: every value as written. Only the
 * benchmark's control arm uses it, to measure what the projection costs. */
export const RAW = Symbol('raw');

// A shape's length is a band, not a count: an exact length is a number's
// magnitude (the benchmark's F3b read a cache's size from it), so a shape
// says only which power of two its length falls in. Growth still shows
// (2 digits is "2-3", 8 is "8-15"); an amount or a date of birth doesn't.
export const band = (n) => (n < 2 ? String(n) : `${2 ** Math.floor(Math.log2(n))}-${2 ** (Math.floor(Math.log2(n)) + 1) - 1}`);

export function shape(s, key) {
  if (key === RAW) return s;
  const str = String(s);
  return {
    shape: true,
    len: band(str.length),
    classes: CLASS.filter(([, re]) => re.test(str)).map(([n]) => n),
    h: createHmac('sha256', key).update(str).digest('hex').slice(0, 10),
  };
}

const IDENT = /^[A-Za-z_][\w.]{0,79}$/;
const VERSION = /^\d+\.\d+\.\d+([-.][\w.]{1,20})?$/;

// A name made only of code: each token (split at / . : _ - ( ) [ ] and spaces)
// is an HTTP method, a word of letters, a short number, a short lowercase
// token with digits (v2, p50, v8js, http2), or a {template}. A token
// mixing letters and digits, a long number, an @ or a ? is data, and shapes
// the whole name. (Residual: a person's name written as one word of letters
// would pass in a name; a vendor's contract can list its names instead.)
const TOKEN = /^(?:[A-Za-z]{1,60}|\d{1,3}|[a-z]{1,4}\d{1,3}[a-z]{0,2}|\{[\w.]{1,40}\}|)$/;
export const codeName = (v) => typeof v === 'string' && v.length <= 120 && !/[@?=&]/.test(v) && v.split(/[\s/.:_()[\]-]+/).every((t) => TOKEN.test(t));
const INT = (v) => Number.isInteger(v);
const HOST = (v) => typeof v === 'string' && /^[a-z0-9-]{1,63}(\.[a-z0-9-]{1,63})*$/i.test(v) && !/\d{4,}/.test(v);

// Keys the site passes by the form their value must take: OpenTelemetry's
// semantic conventions, so a service instrumented the standard way says
// what it did without any contract. Keys that carry data by their nature
// (url.full, url.path, url.query, db.query.text, client.address, user.*,
// enduser.*) are never here, and always shape.
// Attributes whose values are words from a closed set, as the conventions
// and the runtimes' own metrics write them (process.cpu.time's `type`, a
// memory `state`). A value outside its set shapes.
const OF = (...words) => { const set = new Set(words); return (v) => set.has(v); };
const STATE = OF('used', 'free', 'cached', 'buffered', 'available', 'slab_reclaimable', 'slab_unreclaimable', 'idle', 'user', 'system', 'nice', 'iowait', 'interrupt', 'softirq', 'steal', 'wait', 'active', 'inactive', 'reserved', 'total', 'limit', 'open', 'closed', 'running', 'sleeping', 'blocked');
const KNOWN = {
  type: OF('user', 'system', 'voluntary', 'involuntary', 'heap', 'non_heap', 'major', 'minor', 'incremental', 'weakcb', 'read', 'write'),
  state: STATE, 'system.memory.state': STATE, 'system.cpu.state': STATE, 'process.cpu.state': STATE, 'cpu.mode': STATE, 'process.memory.state': STATE,
  direction: OF('receive', 'transmit', 'read', 'write'), 'network.io.direction': OF('receive', 'transmit'), 'disk.io.direction': OF('read', 'write'),
  'jvm.memory.type': OF('heap', 'non_heap'), 'v8js.gc.type': OF('major', 'minor', 'incremental', 'weakcb'),
  'v8js.heap.space.name': OF('new_space', 'old_space', 'code_space', 'large_object_space', 'new_large_object_space', 'code_large_object_space', 'map_space', 'read_only_space', 'shared_space', 'trusted_space', 'trusted_large_object_space', 'shared_large_object_space', 'shared_trusted_space', 'shared_trusted_large_object_space'),
  'cpython.gc.generation': (v) => [0, 1, 2].includes(v), generation: (v) => [0, 1, 2, '0', '1', '2'].includes(v), cpu: INT,
  'service.name': (v) => IDENT.test(v) || codeName(v),
  'service.version': (v) => VERSION.test(v),
  'service.namespace': codeName,
  'telemetry.sdk.language': codeName, 'telemetry.sdk.name': codeName, 'telemetry.sdk.version': (v) => VERSION.test(v),
  'process.runtime.name': codeName,
  'exception.type': (v) => IDENT.test(v) || codeName(v),
  'error.type': (v) => IDENT.test(v) || codeName(v) || /^\d{3}$/.test(v),
  'otel.status_code': (v) => ['OK', 'ERROR', 'UNSET'].includes(v),
  'http.request.method': (v) => /^[A-Z]{3,7}$/.test(v),
  'http.response.status_code': INT, 'http.status_code': INT,
  'http.route': (v) => typeof v === 'string' && v.startsWith('/') && codeName(v),
  'url.scheme': (v) => ['http', 'https'].includes(v),
  'network.protocol.name': codeName, 'network.protocol.version': (v) => /^[\d.]{1,5}$/.test(v), 'network.transport': codeName, 'network.type': codeName,
  'server.address': HOST, 'server.port': INT, 'net.peer.name': HOST, 'net.peer.port': INT,
  'rpc.system': codeName, 'rpc.service': codeName, 'rpc.method': codeName, 'rpc.grpc.status_code': INT, 'rpc.response.status_code': codeName,
  'db.system': codeName, 'db.system.name': codeName, 'db.operation.name': codeName, 'db.operation': codeName, 'db.collection.name': codeName, 'db.namespace': codeName, 'db.name': codeName,
  'messaging.system': codeName, 'messaging.operation.type': codeName, 'messaging.operation': codeName, 'messaging.destination.name': codeName,
  'thread.name': codeName, 'code.function': codeName, 'code.function.name': codeName, 'code.namespace': codeName, 'code.lineno': INT, 'code.line.number': INT,
  'feature_flag.key': codeName, 'feature_flag.provider_name': codeName, 'feature_flag.provider.name': codeName,
};

// The same keys written another way: Rust's tracing writes
// feature_flag_key for feature_flag.key. A key matches a known one when
// the two agree with dots, underscores and dashes dropped, and is then
// held to that key's form. Its own name is kept.
const SPELLED = Object.fromEntries(Object.keys(KNOWN).map((k) => [k.toLowerCase().replace(/[._-]/g, ''), KNOWN[k]]));
const known = (k) => KNOWN[k] ?? SPELLED[k.toLowerCase().replace(/[._-]/g, '')];

const TYPES = {
  int: (v) => Number.isInteger(v),
  hash: (v) => typeof v === 'string' && /^[0-9a-f]{8,64}$/.test(v),
};

/** Reads a contract from a signed recipe; parse, don't validate. */
export function contract(raw = {}) {
  const attrs = {};
  for (const [k, t] of Object.entries(raw.attributes ?? {})) {
    if (Array.isArray(t)) {
      if (t.length > 64 || t.some((w) => !/^[\w.-]{1,40}$/.test(w))) throw new Error(`the contract's words for ${k} must be at most 64, each a word of at most 40 letters`);
      attrs[k] = (v) => t.includes(v);
    } else if (TYPES[t]) attrs[k] = TYPES[t];
    else throw new Error(`the contract types ${k} as ${JSON.stringify(t)}; a type is int, hash or a list of words`);
  }
  const spans = new Set(raw.spans ?? []);
  return { attrs, spans, raw };
}

const value = (v) => v?.stringValue ?? (v?.intValue != null ? Number(v.intValue) : v?.boolValue ?? v?.doubleValue ?? (v?.arrayValue ? JSON.stringify((v.arrayValue.values ?? []).map(value)) : null));

function attrs(list, c, key) {
  const out = {};
  for (const { key: k, value: v } of list ?? []) {
    const x = value(v);
    if (typeof x === 'boolean' || (typeof x === 'number' && (c.attrs[k] ?? known(k))?.(x))) out[k] = x;
    else if ((known(k) ?? c.attrs[k])?.(x)) out[k] = x;
    else out[k] = shape(x, key);
  }
  return out;
}

// Code locations from a stack trace, in the forms Python, Java, .NET, Node
// and Go print them: file, line and function, never a message.
// An event's name may be a message: a library can name one "Error: card
// declined for <name>" (the benchmark's F2 passed "Error: Product Catalog
// Fail Feature Flag Enabled"). So an event name passes only if the
// conventions or the contract name it, or it is one identifier, no spaces.
const EVENTS = new Set(['exception', 'feature_flag.evaluation', 'feature_flag', 'message', 'rpc.message', 'event', 'log']);
const eventName = (n, c) => typeof n === 'string' && (EVENTS.has(n) || c.spans.has(n) || /^[A-Za-z][\w.]{0,79}$/.test(n));

const FRAMES = [
  [/File "([^"]+)", line (\d+), in ([\w<>.]+)/g, (m) => `${m[1].split('/').at(-1)}:${m[2]} ${m[3]}`],
  [/\bat ([\w$.<>]+)\(([\w.]+):(\d+)\)/g, (m) => `${m[2]}:${m[3]} ${m[1]}`],
  [/\bat ([\w.`<>]+)\([^)]*\) in [^\n]*?([\w.]+):line (\d+)/g, (m) => `${m[2]}:${m[3]} ${m[1]}`],
  [/\bat (?:async )?([\w$.<>]+) \((?:[^)\n]*\/)?([\w.-]+):(\d+):\d+\)/g, (m) => `${m[2]}:${m[3]} ${m[1]}`],
  [/\n([\w./-]+\.[\w]+)\(\S*\)\n\t(?:[^\n]*\/)?([\w.-]+\.go):(\d+)/g, (m) => `${m[2]}:${m[3]} ${m[1]}`],
];
const frames = (st) => FRAMES.flatMap(([re, f]) => [...String(st ?? '').matchAll(re)].map(f)).filter((f) => !/[@?=&]/.test(f)).slice(0, 30);

/** One line of the collector's file export: OTLP JSON, traces or logs. */
export function project(line, c, key) {
  const out = [];
  const rec = JSON.parse(line);
  for (const rs of rec.resourceSpans ?? []) {
    const res = attrs(rs.resource?.attributes, c, key);
    for (const ss of rs.scopeSpans ?? []) for (const s of ss.spans ?? []) {
      const events = (s.events ?? []).map((e) => {
        if (e.name !== 'exception') return { name: eventName(e.name, c) ? e.name : shape(e.name, key), attrs: attrs(e.attributes, c, key) };
        const a = Object.fromEntries((e.attributes ?? []).map(({ key: k, value: v }) => [k, value(v)]));
        return { name: 'exception', type: KNOWN['exception.type'](a['exception.type'] ?? '') ? a['exception.type'] : shape(a['exception.type'], key), at: frames(a['exception.stacktrace']), message: key === RAW ? a['exception.message'] ?? null : 'dropped', ...(key === RAW ? { stacktrace: a['exception.stacktrace'] ?? null } : {}) };
      });
      out.push({
        kind: 'span', service: res['service.name'], version: res['service.version'], t: s.startTimeUnixNano ? Math.round(Number(s.startTimeUnixNano) / 1e6) : null,
        name: c.spans.has(s.name) || codeName(s.name) ? s.name : shape(s.name, key),
        span_kind: typeof s.kind === 'number' ? ['unspecified', 'internal', 'server', 'client', 'producer', 'consumer'][s.kind] ?? null : String(s.kind ?? '').replace('SPAN_KIND_', '').toLowerCase() || null,
        trace: String(s.traceId).slice(0, 8), span: String(s.spanId).slice(0, 8), parent: s.parentSpanId ? String(s.parentSpanId).slice(0, 8) : null,
        status: s.status?.code === 2 || s.status?.code === 'STATUS_CODE_ERROR' ? 'error' : 'ok',
        ms: s.endTimeUnixNano && s.startTimeUnixNano ? Math.round((Number(s.endTimeUnixNano) - Number(s.startTimeUnixNano)) / 1e6) : null,
        attrs: attrs(s.attributes, c, key), events,
      });
    }
  }
  for (const rm of rec.resourceMetrics ?? []) {
    const res = attrs(rm.resource?.attributes, c, key);
    for (const sm of rm.scopeMetrics ?? []) for (const m of sm.metrics ?? []) {
      const type = ['gauge', 'sum', 'histogram', 'exponentialHistogram', 'summary'].find((t) => m[t]);
      const points = (m[type]?.dataPoints ?? []).slice(0, 50).map((d) => ({
        attrs: attrs(d.attributes, c, key),
        // A measurement is a number the code made; it passes.
        ...(d.asDouble != null || d.asInt != null ? { value: Number(d.asDouble ?? d.asInt) } : { count: Number(d.count ?? 0), sum: d.sum ?? null, ...(d.explicitBounds ? { bounds: d.explicitBounds, buckets: (d.bucketCounts ?? []).map(Number) } : {}) }),
        t: d.timeUnixNano ? Math.round(Number(d.timeUnixNano) / 1e6) : null,
      }));
      out.push({ kind: 'metric', service: res['service.name'], name: codeName(m.name) ? m.name : shape(m.name, key), unit: /^[\w{}\/%.*-]{0,20}$/.test(m.unit ?? '') ? m.unit ?? '' : null, type, points });
    }
  }
  for (const rl of rec.resourceLogs ?? []) {
    const res = attrs(rl.resource?.attributes, c, key);
    for (const sl of rl.scopeLogs ?? []) for (const l of sl.logRecords ?? []) {
      out.push({ kind: 'log', service: res['service.name'], t: Number(l.timeUnixNano) || Number(l.observedTimeUnixNano) ? Math.round((Number(l.timeUnixNano) || Number(l.observedTimeUnixNano)) / 1e6) : null, severity: /^[A-Z]{3,8}$/.test(l.severityText ?? '') ? l.severityText : null, trace: l.traceId ? String(l.traceId).slice(0, 8) : null, body: shape(value(l.body) ?? '', key), attrs: attrs(l.attributes, c, key) });
    }
  }
  return out;
}
