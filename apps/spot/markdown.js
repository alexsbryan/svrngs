// SPDX-License-Identifier: AGPL-3.0-or-later
// The Markdown a chat model writes, as a tree the page builds with the DOM:
// a node is a string (text) or { tag, children, ...attrs }, its tag one of
// TAGS. Nothing here is HTML, so nothing the model writes is ever parsed as
// HTML, and a link goes only to http, https or mailto. Pure, so it is tested
// in Node (markdown.test.mjs). An unclosed fence, as a streamed answer has
// mid-way, runs to the end as code.

export const TAGS = new Set([
  'p',
  'br',
  'h1',
  'h2',
  'h3',
  'h4',
  'h5',
  'h6',
  'hr',
  'pre',
  'code',
  'blockquote',
  'ul',
  'ol',
  'li',
  'table',
  'thead',
  'tbody',
  'tr',
  'th',
  'td',
  'strong',
  'em',
  'del',
  'a',
]);

const FENCE = /^ {0,3}(`{3,}|~{3,})\s*([\w+#.-]*)/;
const HEADING = /^ {0,3}(#{1,6})\s+(.*?)\s*#*\s*$/;
const RULE = /^ {0,3}([-*_])(\s*\1){2,}\s*$/;
const ITEM = /^(\s*)([-*+]|\d{1,9}[.)])\s+(.*)$/;
const QUOTE = /^ {0,3}>\s?(.*)$/;
const DIVIDER = /^\s*\|?\s*:?-+:?\s*(\|\s*:?-+:?\s*)*\|?\s*$/;

const el = (tag, children = [], attrs = {}) => ({ tag, children, ...attrs });
// A row's cells, split at each | but an escaped one. (No lookbehind: iOS
// Safari before 16.4 cannot parse one, and the whole page would not run.)
function cells(line) {
  const row = line.trim().replace(/^\|/, '').replace(/\|$/, '');
  const out = [''];
  for (let k = 0; k < row.length; k++) {
    if (row[k] === '\\' && row[k + 1] === '|') out[out.length - 1] += row[++k];
    else if (row[k] === '|') out.push('');
    else out[out.length - 1] += row[k];
  }
  return out.map((c) => c.trim());
}

export function parse(src) {
  return blocks(String(src).replace(/\r\n?/g, '\n').split('\n'));
}

function blocks(lines) {
  const out = [];
  let i = 0;
  const startsBlock = (l, next) =>
    FENCE.test(l) ||
    HEADING.test(l) ||
    RULE.test(l) ||
    ITEM.test(l) ||
    QUOTE.test(l) ||
    (l.includes('|') && next !== undefined && DIVIDER.test(next) && next.includes('-'));
  while (i < lines.length) {
    const line = lines[i];
    if (line.trim() === '') {
      i++;
      continue;
    }
    let m;
    if ((m = line.match(FENCE))) {
      const [, fence, lang] = m;
      const body = [];
      i++;
      while (
        i < lines.length &&
        !new RegExp(`^ {0,3}${fence[0]}{${fence.length},}\\s*$`).test(lines[i])
      )
        body.push(lines[i++]);
      i++;
      out.push(el('pre', [el('code', [body.join('\n')], lang ? { lang } : {})]));
    } else if ((m = line.match(HEADING))) {
      out.push(el(`h${m[1].length}`, inline(m[2])));
      i++;
    } else if (RULE.test(line)) {
      out.push(el('hr'));
      i++;
    } else if (
      line.includes('|') &&
      DIVIDER.test(lines[i + 1] ?? '') &&
      lines[i + 1].includes('-')
    ) {
      const align = cells(lines[i + 1]).map((c) =>
        c.startsWith(':') && c.endsWith(':')
          ? 'center'
          : c.endsWith(':')
            ? 'right'
            : c.startsWith(':')
              ? 'left'
              : null,
      );
      const row = (l, tag) =>
        el(
          'tr',
          cells(l).map((c, k) => el(tag, inline(c), align[k] ? { align: align[k] } : {})),
        );
      const head = el('thead', [row(line, 'th')]);
      const body = [];
      i += 2;
      while (i < lines.length && lines[i].includes('|') && lines[i].trim() !== '')
        body.push(row(lines[i++], 'td'));
      out.push(el('table', [head, el('tbody', body)]));
    } else if (QUOTE.test(line)) {
      const inner = [];
      // A quote runs to a blank line; a line without its > continues it.
      while (i < lines.length && lines[i].trim() !== '') {
        inner.push(lines[i].match(QUOTE)?.[1] ?? lines[i]);
        i++;
      }
      out.push(el('blockquote', blocks(inner)));
    } else if ((m = line.match(ITEM))) {
      const indent = m[1].length;
      const ordered = /\d/.test(m[2]);
      const start = ordered ? parseInt(m[2], 10) : null;
      const items = [];
      while (i < lines.length) {
        const im = lines[i].match(ITEM);
        if (!im || im[1].length !== indent || /\d/.test(im[2]) !== ordered) break;
        const body = [im[3]];
        const width = im[1].length + im[2].length + 1;
        i++;
        // Its continuation: lines indented past its marker, blank lines among them.
        while (i < lines.length) {
          const l = lines[i];
          if (l.trim() === '') {
            const next = lines[i + 1];
            if (next !== undefined && next.match(/^\s*/)[0].length > indent && next.trim() !== '') {
              body.push('');
              i++;
              continue;
            }
            break;
          }
          const lead = l.match(/^\s*/)[0].length;
          if (lead > indent) body.push(l.slice(Math.min(lead, width)));
          else if (!startsBlock(l, lines[i + 1]) && !ITEM.test(l)) body.push(l);
          else break;
          i++;
        }
        const inner = blocks(body);
        // A tight item is its text alone, not a paragraph.
        items.push(
          el('li', inner.length === 1 && inner[0].tag === 'p' ? inner[0].children : inner),
        );
        while (
          i < lines.length &&
          lines[i].trim() === '' &&
          (lines[i + 1] ?? '').match(ITEM)?.[1].length === indent
        )
          i++;
      }
      out.push(el(ordered ? 'ol' : 'ul', items, start !== null && start !== 1 ? { start } : {}));
    } else {
      const para = [];
      while (
        i < lines.length &&
        lines[i].trim() !== '' &&
        (para.length === 0 || !startsBlock(lines[i], lines[i + 1]))
      )
        para.push(lines[i++].trim());
      out.push(
        el(
          'p',
          para.flatMap((l, k) => (k ? [el('br'), ...inline(l)] : inline(l))),
        ),
      );
    }
  }
  return out;
}

// Inline spans, earliest first; at one place, the first in this list.
const SPANS = [
  [/\\([\\`*_{}[\]()#+\-.!~|>])/, (m) => [m[1]]],
  [/(`+)([^`]|[^`][\s\S]*?[^`])\1(?!`)/, (m) => [el('code', [m[2].trim() || m[2]])]],
  [/\[([^\]\n]+)\]\(\s*<?([^)\s>]+)>?(?:\s+"[^"]*")?\s*\)/, (m) => link(m[2], inline(m[1]), m[0])],
  [/\*\*(?=\S)([\s\S]*?\S)\*\*(?!\*)/, (m) => [el('strong', inline(m[1]))]],
  [/__(?=\S)([\s\S]*?\S)__(?!\w)/, (m) => [el('strong', inline(m[1]))]],
  [/~~(?=\S)([\s\S]*?\S)~~/, (m) => [el('del', inline(m[1]))]],
  [/\*([^\s*](?:[^*\n]*[^\s*])?)\*/, (m) => [el('em', inline(m[1]))]],
  // _x_ outside a word only, so snake_case stays text; the char before is kept.
  [/(^|[^\w])_([^\s_](?:[^_\n]*[^\s_])?)_(?!\w)/, (m) => [m[1], el('em', inline(m[2]))]],
  [/https?:\/\/[^\s<>()]*[^\s<>().,;:!?'"*_~]/, (m) => link(m[0], [m[0]], m[0])],
];

// A link to anywhere else is shown as it was written, as text: refused, not hidden.
function link(href, children, source) {
  return /^(https?:\/\/|mailto:)/i.test(href) ? [el('a', children, { href })] : [source];
}

export function inline(text) {
  const out = [];
  let rest = text;
  while (rest) {
    let best = null;
    for (const [re, make] of SPANS) {
      const m = rest.match(re);
      if (m && (best === null || m.index < best.m.index)) best = { m, make };
    }
    if (!best) break;
    if (best.m.index > 0) out.push(rest.slice(0, best.m.index));
    out.push(...best.make(best.m));
    rest = rest.slice(best.m.index + best.m[0].length);
  }
  if (rest) out.push(rest);
  // Adjacent text joined, so a node is one run of text.
  return out.reduce((acc, n) => {
    if (n === '') return acc;
    if (typeof n === 'string' && typeof acc.at(-1) === 'string') acc[acc.length - 1] += n;
    else acc.push(n);
    return acc;
  }, []);
}
