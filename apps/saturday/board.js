// Saturday's board apart from the page (index.js): the reads it subscribes
// to, the writes each button sends, the board it draws from the rows, and
// every sentence it says. Nothing here touches the page or the network, so
// its laws run in Node (tests/board.test.mjs).

/** The plans on the board, each with its total: the votes whose `yes` is 1. */
export const PLANS = `SELECT p.id, p.title, p.at, p.place,
  (SELECT count(*) FROM votes AS v WHERE v.plan = p.id AND v.yes = 1) AS votes
FROM plans AS p WHERE p.off IS NOT 1
ORDER BY p.at, p.title, p.id`;

/**
 * The things to bring and the claims, one list, each row saying which it
 * is: the things by their key, the trimmed lower case of their words, so
 * "chips" comes before "Ice"; the claims by their ID.
 */
export const THINGS = `SELECT 'thing' AS kind, plan, thing AS id, words, NULL AS person_key
FROM things WHERE off IS NOT 1
UNION ALL
SELECT 'claim', plan, id, what, person_key FROM claims WHERE withdrawn IS NOT 1
ORDER BY plan, kind DESC, id`;

/** The plans this person voted for: read with their key, after each change to the totals. */
export const MINE = 'SELECT plan FROM votes WHERE person_key = ? AND yes = 1';

// Each write is one list of ops in svrngs/sql's write form, rows named by
// key and their values given, never SQL: a write that computes or checks
// nothing holds the same on every copy it reaches (docs/guide/database.md,
// "Write"). `id` is how the page makes a key; tests pass their own.

/** A new plan, keyed by a random ID so two plans added apart stay two. */
export const addPlan = ({ title, at, place }, you, id = crypto.randomUUID()) => [
  { insert: 'plans', row: { id, title, at: at || null, place: place || null, added_by: you } },
];
/** A plan taken off the board: its row kept, so a claim on it still has a plan. */
export const takeOff = (plan) => [{ update: 'plans', key: { id: plan }, set: { off: 1 } }];

/** This person's vote for a plan, or their vote taken back: always their own one row. */
export const vote = (plan, you, yes) => [
  { insert: 'votes', row: { person_key: you, plan, yes: yes ? 1 : 0 } },
];

/** "I'll bring `what`", as this person's own row. */
export const claim = (plan, what, you, id = crypto.randomUUID()) => [
  { insert: 'claims', row: { person_key: you, id, plan, what, withdrawn: 0 } },
];
/** A claim's words changed, by the claimer: their key is in the row's. */
export const change = (c, what) => [
  { update: 'claims', key: { person_key: c.person_key, id: c.id }, set: { what } },
];
/** A claim taken back: kept, and no longer shown. */
export const withdraw = (c) => [
  { update: 'claims', key: { person_key: c.person_key, id: c.id }, set: { withdrawn: 1 } },
];

/** A thing to bring as its key: what two people mean by it the same, trimmed, in lower case, one space apart. */
export const keyOf = (words) => words.trim().replace(/\s+/g, ' ').toLowerCase();

/**
 * A pasted list as the things it names: one a line, its bullet or number
 * dropped, blank lines and repeats left out, in the order pasted.
 */
export function lines(text) {
  const seen = new Map();
  for (const line of text.split(/\r\n?|\n/)) {
    const words = line
      .replace(/^\s*(?:[-*•]|\d+[.)])\s+/, '')
      .trim()
      .replace(/\s+/g, ' ');
    if (words && !seen.has(keyOf(words))) seen.set(keyOf(words), words);
  }
  return [...seen].map(([thing, words]) => ({ thing, words }));
}

/** A pasted list landed all at once: one write, a row a line, each back on the list if it was taken off. */
export const paste = (plan, text) =>
  lines(text).map(({ thing, words }) => ({
    insert: 'things',
    row: { plan, thing, words, off: 0 },
  }));
/** A thing taken off a plan's list. */
export const unlist = (plan, thing) => [
  { update: 'things', key: { plan, thing }, set: { off: 1 } },
];

/**
 * The board from what was read: each plan, in PLANS' order, with its total,
 * whether `you` voted for it, its things to bring, each with the claims that
 * bring it, and the claims that bring something not on its list. A claim
 * brings a thing when its words, as a key, are the thing's. `names` maps a
 * person's key to the name to show; a key it lacks shows as "Someone".
 */
export function board({ plans, things, mine, you, names }) {
  const voted = new Set(mine.map((r) => r.plan));
  const name = (key) => names.get(key) ?? 'Someone';
  const shown = plans.map((p) => ({
    ...p,
    votes: Number(p.votes) || 0,
    yours: voted.has(p.id),
    things: [],
    others: [],
  }));
  const byId = new Map(shown.map((p) => [p.id, p]));
  const claims = [];
  for (const row of things) {
    const plan = byId.get(row.plan);
    if (!plan) continue;
    if (row.kind === 'thing') plan.things.push({ thing: row.id, words: row.words, claims: [] });
    else
      claims.push({
        plan,
        claim: {
          id: row.id,
          person_key: row.person_key,
          what: row.words ?? '',
          who: name(row.person_key),
          yours: row.person_key === you,
        },
      });
  }
  for (const { plan, claim: c } of claims) {
    const thing = plan.things.find((t) => t.thing === keyOf(c.what));
    (thing ? thing.claims : plan.others).push(c);
  }
  return shown;
}

/** How long after the board stopped answering it is read again. */
export const RETRY_MS = 3_000;

/** What the page says while it can't show the board, by why. */
export const SAID = {
  opening: 'Opening your group’s board…',
  syncing: 'Still syncing with your group. The board shows once it has caught up.',
  unreachable:
    'None of your group’s computers is answering, so the board can’t open right now. It tries again by itself.',
};
/** The board the route wouldn't read, in its words. */
export const shut = (words) => `The board didn’t open: ${words}`;
/** Something the page tried to save and couldn't, in the route's words. */
export const unsaved = (what, words) => `${what} didn’t go through: ${words}`;
/** Why, where the route gave no words of its own. */
export const NONE = 'None of your group’s computers answered.';
export const answeredWith = (status) => `Your group’s computer answered ${status}.`;
export const UNREAD = 'Its answer couldn’t be read.';
/**
 * A write too large for the route to take at once, which it answers 413
 * in words about an "act". How large is the route's to say: a paste is
 * sent whole, and only its refusal said here.
 */
export const TOO_LONG = 'it’s too long to save at once. Paste part of it, then the rest.';
/** A plan's total, said as a person reads it. */
export const total = (n) => (n === 1 ? '1 vote' : `${n} votes`);
/** Who brings a thing, said beside it. */
export const bringing = (claims) =>
  claims.length ? `${claims.map((c) => (c.yours ? 'you' : c.who)).join(', ')} bringing it` : '';
