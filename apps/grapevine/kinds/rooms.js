// A place to stay: a bed or a sofa for someone's visit, for a few nights.
// Written as kinds/README.md says, the third kind and the first not
// shipped with the app (GRAPEVINE_DEMO §7, "For developers"). Someone who
// has listed a spare room in that town is told they might help.
export default {
  kind: 'room',
  ask: {
    fields: {
      where: 'text',
      from: 'date',
      nights: 'number',
      for: { choice: ['one person', 'two people', 'a family'] },
    },
  },
  show: (f) => `a room in ${f.where} for ${f.for}, ${f.nights} night${f.nights === 1 ? '' : 's'} from ${f.from}`,
  reply: { fields: { where: 'text', note: 'text' } },
  listing: { fields: { where: 'text', sleeps: { choice: ['one person', 'two people', 'a family'] } } },
  notice(ask, mine) {
    const town = (s) => String(s).trim().toLowerCase();
    return mine.records.some((r) => r.kind === 'room' && typeof r.listing?.where === 'string' && town(r.listing.where) === town(ask.fields.where));
  },
  hops: 2,
  name: 'withheld',
};
