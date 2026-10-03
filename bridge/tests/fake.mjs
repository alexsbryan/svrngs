// The fake adapter: the five calls, with the chat driven by a test. Each
// event resolves when the bridge has finished handling it.

export function fakeAdapter() {
  let on = null;
  const chat = {
    posts: [],
    replies: [],
    commands: null,
    closed: false,
    async connect(handlers) {
      on = handlers;
    },
    async post(message) {
      chat.posts.push(message);
    },
    async reply(input, message) {
      chat.replies.push({ to: input.interaction, ...message });
    },
    async commands(schemas) {
      chat.commands = schemas;
    },
    async close() {
      chat.closed = true;
    },
  };
  chat.join = (p) => on.joined(p);
  chat.leave = (p) => on.left({ ...p, why: 'left' });
  chat.ban = (p) => on.left({ ...p, why: 'banned' });
  chat.input = (p, i) => on.input({ kind: 'message', mentions: [], ...p, ...i });
  return chat;
}
