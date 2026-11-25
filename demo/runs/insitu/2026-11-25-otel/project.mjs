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

export function shape(s, key) {
  const str = String(s);
  return {
    shape: true,
    len: str.length,
    classes: CLASS.filter(([, re]) => re.test(str)).map(([n]) => n),
    h: createHmac('sha256', key).update(str).digest('hex').slice(0, 10),
  };
}

const IDENT = /^[A-Za-z_][\w.]{0,79}$/;
const VERSION = /^\d+\.\d+\.\d+([-.][\w.]{1,20})?$/;

// Keys the site passes by the form their value must take.
const KNOWN = {
  'service.name': (v) => IDENT.test(v),
  'service.version': (v) => VERSION.test(v),
  'exception.type': (v) => IDENT.test(v),
  'http.response.status_code': (v) => Number.isInteger(v),
  'http.request.method': (v) => /^[A-Z]{3,7}$/.test(v),
};

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

const value = (v) => v?.stringValue ?? (v?.intValue != null ? Number(v.intValue) : v?.boolValue ?? v?.doubleValue ?? null);

function attrs(list, c, key) {
  const out = {};
  for (const { key: k, value: v } of list ?? []) {
    const x = value(v);
    if (typeof x === 'boolean' || (typeof x === 'number' && (c.attrs[k] ?? KNOWN[k]))) out[k] = x;
    else if ((KNOWN[k] ?? c.attrs[k])?.(x)) out[k] = x;
    else out[k] = shape(x, key);
  }
  return out;
}

const FRAME = /File "([^"]+)", line (\d+), in ([\w<>.]+)/g;
const frames = (st) => [...String(st ?? '').matchAll(FRAME)].map(([, f, l, fn]) => `${f.split('/').at(-1)}:${l} ${fn}`);

/** One line of the collector's file export: OTLP JSON, traces or logs. */
export function project(line, c, key) {
  const out = [];
  const rec = JSON.parse(line);
  for (const rs of rec.resourceSpans ?? []) {
    const res = attrs(rs.resource?.attributes, c, key);
    for (const ss of rs.scopeSpans ?? []) for (const s of ss.spans ?? []) {
      const events = (s.events ?? []).map((e) => {
        if (e.name !== 'exception') return { name: c.spans.has(e.name) ? e.name : shape(e.name, key) };
        const a = Object.fromEntries((e.attributes ?? []).map(({ key: k, value: v }) => [k, value(v)]));
        return { name: 'exception', type: IDENT.test(a['exception.type'] ?? '') ? a['exception.type'] : shape(a['exception.type'], key), at: frames(a['exception.stacktrace']), message: 'dropped' };
      });
      out.push({
        kind: 'span', service: res['service.name'], version: res['service.version'],
        name: c.spans.has(s.name) ? s.name : shape(s.name, key),
        trace: String(s.traceId).slice(0, 8), span: String(s.spanId).slice(0, 8), parent: s.parentSpanId ? String(s.parentSpanId).slice(0, 8) : null,
        status: s.status?.code === 2 || s.status?.code === 'STATUS_CODE_ERROR' ? 'error' : 'ok',
        ms: s.endTimeUnixNano && s.startTimeUnixNano ? Math.round((Number(s.endTimeUnixNano) - Number(s.startTimeUnixNano)) / 1e6) : null,
        attrs: attrs(s.attributes, c, key), events,
      });
    }
  }
  for (const rl of rec.resourceLogs ?? []) {
    const res = attrs(rl.resource?.attributes, c, key);
    for (const sl of rl.scopeLogs ?? []) for (const l of sl.logRecords ?? []) {
      out.push({ kind: 'log', service: res['service.name'], severity: /^[A-Z]{3,8}$/.test(l.severityText ?? '') ? l.severityText : null, trace: l.traceId ? String(l.traceId).slice(0, 8) : null, body: shape(value(l.body) ?? '', key), attrs: attrs(l.attributes, c, key) });
    }
  }
  return out;
}
