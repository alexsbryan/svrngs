// mark.mjs — the svrngs mark, its one home: docs/brand/src/build.mjs draws
// its files from it, and docs/diagrams/src/kit.mjs signs each figure with it.
//
// Three people, each a dot with a ring through the one place they meet.
// Three equal rings through one point make a fourth of the same size, through
// all three people (Johnson's theorem, 1916): the group's ring, in gold, the
// record they keep. Inside it nothing is anyone's, so each person's ring keeps
// only its outer arc and stops short of the gold where it meets another's.
// The large cut adds each person's far reach, a ring through the other two.

const R = Math.PI / 180;
const K = 46;                          // a person's ring, and the group's
const FAR = K * Math.sqrt(3);          // a far ring passes through the other two
const ANG = [-90, 30, 150];            // one person at the top
const at = (a, d, [cx, cy] = [0, 0]) => [cx + d * Math.cos(a * R), cy + d * Math.sin(a * R)];
const PEOPLE = ANG.map((a) => at(a, K));
const f = (n) => +n.toFixed(2);

// Each cut is drawn for a range of sizes, and is heavier the smaller it is
// shown: below 64 pixels the far rings fill in, below 32 thin lines vanish.
// `door` is the chord left open in the gold at each person (0: whole), `gap`
// where two people's rings stop short of it.
export const CUTS = {
  large: { far: 5, near: 7.5, gold: 11, dot: 10.5, dotLine: 5, door: 48, gap: 22, farGap: 46 },
  medium: { far: 0, near: 4.5, gold: 7, dot: 10, dotLine: 4, door: 0, gap: 14 },
  small: { far: 0, near: 12, gold: 14, dot: 14, dotLine: 6, door: 0, gap: 26 },
};

// An arc from a0 to a1 degrees, clockwise, a1 > a0.
function arc(c, r, a0, a1, stroke, sw) {
  const [sx, sy] = at(a0, r, c), [ex, ey] = at(a1, r, c);
  return `<path d="M${f(sx)} ${f(sy)} A${f(r)} ${f(r)} 0 ${a1 - a0 > 180 ? 1 : 0} 1 ${f(ex)} ${f(ey)}" fill="none" stroke="${stroke}" stroke-width="${sw}"/>`;
}
const half = (chord, r) => Math.asin(chord / 2 / r) / R;

// The mark's shapes around its center, in its own units. `people` is the
// dots' fill: salmon, or white on a salmon band, where salmon would vanish.
export function shapes(cut = 'large', { ink = 'currentColor', people = 'salmon' } = {}) {
  const c = CUTS[cut];
  let s = '';
  if (c.far) for (const [i, p] of PEOPLE.entries()) s += arc(p, FAR, ANG[i] - 150 + half(c.farGap, FAR), ANG[i] + 150 - half(c.farGap, FAR), ink, c.far);
  if (c.door) for (const a of ANG) s += arc([0, 0], K, a + half(c.door, K), a + 120 - half(c.door, K), 'gold', c.gold);
  else s += `<circle cx="0" cy="0" r="${K}" fill="none" stroke="gold" stroke-width="${c.gold}"/>`;
  for (const [i, p] of PEOPLE.entries()) s += arc(p, K, ANG[i] - 120 + half(c.gap, K), ANG[i] + 120 - half(c.gap, K), ink, c.near);
  for (const [x, y] of PEOPLE) s += `<circle cx="${f(x)}" cy="${f(y)}" r="${c.dot}" fill="${people}" stroke="${ink}" stroke-width="${c.dotLine}"/>`;
  return s;
}

// The square the mark sits in, in its own units. Its rings reach 2K up from
// the center and 1.5K down, so its middle is K/4 above; it is widest across,
// a bottom person's ring reaching K * cos 30° + its radius out.
export function box(cut = 'large') {
  const c = CUTS[cut], line = Math.max(c.far, c.near, c.gold, c.dotLine);
  const half = K * Math.cos(30 * R) + (c.far ? FAR : K) + line / 2 + 2;
  return { x: f(-half), y: f(-K / 4 - half), w: f(2 * half), h: f(2 * half) };
}

// The mark placed at (x, y), `size` wide, for a figure or a page.
export function place(x, y, size, cut, o = {}) {
  const b = box(cut), s = size / b.w;
  return `<g transform="translate(${f(x - b.x * s)} ${f(y - b.y * s)}) scale(${f(s)})">${shapes(cut, o)}</g>`;
}
