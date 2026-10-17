// A question in your own words: also Buy Nothing's ISO post (GRAPEVINE_DEMO
// §7). It has no `notice`.
export default {
  kind: 'ask',
  ask: { fields: { question: 'text' } },
  show: (f) => f.question,
  reply: { fields: { answer: 'text' } },
  hops: 1,
  name: 'withheld',
};
