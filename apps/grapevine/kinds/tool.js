// A tool, by name and day (GRAPEVINE_DEMO §7). Someone who has listed one by
// that name is told they might help. Listings are plain records in their
// own group, never carried.
export default {
  kind: 'tool',
  ask: { fields: { what: 'text', when: 'text' } },
  show: (f) => `${f.what}, ${f.when}`,
  reply: { fields: { when: 'text', where: 'text' } },
  listing: { fields: { what: 'text' } },
  notice(ask, mine) {
    const name = (s) => String(s).trim().toLowerCase();
    return mine.records.some((r) => r.kind === 'tool' && typeof r.listing?.what === 'string' && name(r.listing.what) === name(ask.fields.what));
  },
  hops: 1,
  name: 'withheld',
};
