// kit.mjs — svrngs's diagram kit: the scene-as-data method of
// commonwealth-ai docs/diagrams/src, drawn in svrngs's own look
// (crates/svrngs-node/src/home/look.css; DESIGN §1 rule 3): the system's
// fonts, hard black rules, square corners, and colour by the names a browser
// knows. Nothing imitates a hand, paper or a stamp.

// One legend for every figure.
export const C = {
  paper: 'white',
  ink: 'black',
  soft: '#444',
  people: 'salmon',      // people, and the computers and phones they hold
  record: 'gold',        // the group's record: signed acts
  app: 'plum',           // apps: anything on a port
  wire: 'skyblue',       // the wire, and what we host
  ok: 'lightgreen',      // let in, passed
  no: 'tomato',          // refused, failed
  lit: 'yellow',         // the one thing to keep from a figure
  maybe: 'khaki',        // could not judge
};

export const FACE = `system-ui, -apple-system, 'Segoe UI', Helvetica, Arial, sans-serif`;
export const MONO = `ui-monospace, 'SF Mono', SFMono-Regular, Menlo, Consolas, monospace`;

const esc = (t) => String(t).replace(/&/g, '&amp;').replace(/</g, '&lt;').replace(/>/g, '&gt;');

export function rect(x, y, w, h, { fill = 'none', stroke = C.ink, sw = 3, dash = null } = {}) {
  const d = dash ? ` stroke-dasharray="${dash}"` : '';
  return `<rect x="${x}" y="${y}" width="${w}" height="${h}" fill="${fill}" stroke="${stroke}" stroke-width="${sw}"${d}/>`;
}

export function circle(cx, cy, r, { fill = 'none', stroke = C.ink, sw = 3, dash = null } = {}) {
  const d = dash ? ` stroke-dasharray="${dash}"` : '';
  return `<circle cx="${cx}" cy="${cy}" r="${r}" fill="${fill}" stroke="${stroke}" stroke-width="${sw}"${d}/>`;
}

export function line(x1, y1, x2, y2, { stroke = C.ink, sw = 3, dash = null } = {}) {
  const d = dash ? ` stroke-dasharray="${dash}"` : '';
  return `<line x1="${x1}" y1="${y1}" x2="${x2}" y2="${y2}" stroke="${stroke}" stroke-width="${sw}"${d}/>`;
}

// A straight shaft and a solid head: a plain arrow, as a browser would draw one.
export function arrow(x1, y1, x2, y2, { stroke = C.ink, sw = 3, dash = null, head = 13 } = {}) {
  const a = Math.atan2(y2 - y1, x2 - x1);
  const bx = x2 - head * Math.cos(a), by = y2 - head * Math.sin(a);
  const p = (t) => [bx + (head * 0.5) * Math.cos(a + t), by + (head * 0.5) * Math.sin(a + t)];
  const [l, r] = [p(Math.PI / 2), p(-Math.PI / 2)];
  return line(x1, y1, bx, by, { stroke, sw, dash }) +
    `<path d="M${x2} ${y2} L${l[0].toFixed(1)} ${l[1].toFixed(1)} L${r[0].toFixed(1)} ${r[1].toFixed(1)} Z" fill="${stroke}"/>`;
}

export function text(x, y, str, {
  size = 18, weight = 400, anchor = 'start', mono = false, italic = false, fill = C.ink, strike = false,
} = {}) {
  const st = italic ? ' font-style="italic"' : '';
  const dec = strike ? ' text-decoration="line-through"' : '';
  const fam = mono ? MONO : FACE;
  return `<text x="${x}" y="${y}" font-family="${fam}" font-size="${size}" font-weight="${weight}" text-anchor="${anchor}" fill="${fill}"${st}${dec}>${esc(str)}</text>`;
}

// Width of a line, estimated: there is no DOM to measure against, so each
// face carries an average advance per character.
export const width = (str, size, mono = false) => str.length * size * (mono ? 0.6 : 0.5);

export function wrap(str, w, size, mono = false) {
  const budget = Math.max(8, Math.floor(w / (size * (mono ? 0.6 : 0.5))));
  const lines = []; let cur = '';
  for (const word of str.split(' ')) {
    if ((cur ? cur + ' ' + word : word).length <= budget) cur = cur ? cur + ' ' + word : word;
    else { if (cur) lines.push(cur); cur = word; }
  }
  if (cur) lines.push(cur);
  return lines;
}

// A paragraph from its first baseline; gives its markup and where it ends.
export function para(x, y, str, { w = 300, size = 17, lh = 1.3, ...o } = {}) {
  const lines = wrap(str, w, size, o.mono);
  const s = lines.map((ln, i) => text(x, y + i * size * lh, ln, { size, ...o })).join('');
  return { s, end: y + (lines.length - 1) * size * lh };
}

// A highlight drawn under a line of text, as `mark` does.
export function mark(x, y, str, size, { mono = false, fill = C.lit } = {}) {
  return `<rect x="${x - 4}" y="${y - size * 0.86}" width="${width(str, size, mono) + 8}" height="${size * 1.18}" fill="${fill}"/>`;
}

export function tick(cx, cy, s = 9, stroke = C.ink) {
  return `<path d="M${cx - s} ${cy} L${cx - s * 0.3} ${cy + s * 0.75} L${cx + s} ${cy - s * 0.8}" fill="none" stroke="${stroke}" stroke-width="3.2"/>`;
}
export function cross(cx, cy, s = 8, stroke = C.ink) {
  return `<path d="M${cx - s} ${cy - s} L${cx + s} ${cy + s} M${cx + s} ${cy - s} L${cx - s} ${cy + s}" fill="none" stroke="${stroke}" stroke-width="3.2"/>`;
}

// A computer: its screen and the line of its base.
export function computer(x, y, w, { fill = C.paper } = {}) {
  const h = Math.round(w * 0.6);
  return rect(x, y, w, h, { fill, sw: 3 }) + line(x - w * 0.1, y + h + 6, x + w * 1.1, y + h + 6, { sw: 4 });
}
// A phone: a tall box with a home bar.
export function phone(x, y, w, { fill = C.paper } = {}) {
  const h = Math.round(w * 1.8);
  return rect(x, y, w, h, { fill, sw: 3 }) + line(x + w * 0.35, y + h - 8, x + w * 0.65, y + h - 8, { sw: 3 });
}

// The figure's name in a band of colour, as the pages' h1 is.
export function band(W, title, sub, n, of) {
  return rect(24, 24, W - 48, 112, { fill: C.people, sw: 3 }) +
    text(48, 84, title, { size: 42, weight: 900 }) +
    text(50, 118, sub, { size: 21 }) +
    text(W - 48, 56, `${n} / ${of}`, { size: 16, mono: true, anchor: 'end' });
}

// A numbered heading with a rule over it, as a page's section.
export function heading(x, y, w, n, str) {
  return line(x, y - 30, x + w, y - 30, { sw: 3 }) +
    text(x, y, `${n}`, { size: 22, weight: 900 }) +
    text(x + 26, y, str, { size: 22, weight: 900 });
}

// The legend and the figure's test, along its foot.
export function foot(W, H, test) {
  let s = line(24, H - 64, W - 24, H - 64, { sw: 3 });
  s += text(40, H - 32, test, { size: 15, mono: true });
  const keys = [[C.people, 'people'], [C.record, 'the record'], [C.app, 'apps'], [C.wire, 'wire & hosts'], [C.ok, 'let in'], [C.no, 'refused']];
  let x = W - 40;
  for (const [fill, label] of keys.reverse()) {
    x -= width(label, 14) + 6;
    s += text(x, H - 32, label, { size: 14 });
    x -= 20;
    s += rect(x, H - 45, 15, 15, { fill, sw: 2 });
    x -= 16;
  }
  return s;
}

export function svg(W, H, body, title, desc) {
  return `<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 ${W} ${H}" width="${W}" height="${H}" role="img" aria-labelledby="t d">
<title id="t">${esc(title)}</title>
<desc id="d">${esc(desc)}</desc>
<rect x="0" y="0" width="${W}" height="${H}" fill="${C.paper}"/>
<rect x="1.5" y="1.5" width="${W - 3}" height="${H - 3}" fill="none" stroke="${C.ink}" stroke-width="3"/>
${body}
</svg>
`;
}
