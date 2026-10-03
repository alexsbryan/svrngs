// The bridge's core (DESIGN C5): a chat platform behind five calls, and the
// host's loopback append behind three. It keeps no state of its own and never
// loads app code (crates/svrngs/tests/bridge_holds_no_app.rs holds it to that).
//
// The adapter's five calls:
//   connect({joined, left, input})  open the platform; deliver its events
//   post(message)                   post in the bound channel
//   reply(input, message)           answer one input; `private` shows it to its speaker only
//   commands(schemas)               register the chat commands apps declare
//   close()
// Every event names its speaker by the platform's stable id
// (`discord:<user id>`) and display name; `left` says `left` or `banned`; an
// input carries the platform's interaction id, which a retry repeats.

import { readFileSync } from 'node:fs';

const WORDS = JSON.parse(readFileSync(new URL('./words.json', import.meta.url), 'utf8'));

/** One line of JSON on stderr per decision, named `bridge:<action>` (ARCH_PRINCIPLES 1). */
export function log(event, fields = {}) {
  process.stderr.write(`${JSON.stringify({ event, ...fields })}\n`);
}

function fill(template, vars) {
  return template.replace(/\{(\w+)\}/g, (_, k) => {
    if (vars[k] === undefined) throw new Error(`the words need {${k}}`);
    return vars[k];
  });
}

/**
 * Connect `adapter` to the host. `owner` names whose computer keeps the group,
 * for the greeting; `commands` are the schemas apps declare (none before apps).
 */
export async function startBridge({ adapter, host, owner, commands = [] }) {
  if (!owner) throw new Error('the bridge needs the name of whose computer this is');
  const group = await host.group();
  const words = (key, vars = {}) => fill(WORDS[key], { group: group.name, owner, ...vars });

  /** Write what the chat said about a person into the guest book. */
  async function note(person, event) {
    const r = await host.guest({ speaker: person.speaker, name: person.name, event });
    log(r.removed ? 'bridge:guest_removed' : 'bridge:guest', { speaker: person.speaker, event, new: r.new ?? false });
    if (r.arrived) await adapter.post({ text: words('welcome', { name: person.name }) });
    return r;
  }

  async function input(i) {
    for (const m of i.mentions ?? []) await note(m, 'seen');
    const r = await note(i, 'seen');
    if (r.removed) {
      if (i.kind !== 'message') await adapter.reply(i, { text: words('removed'), private: true });
      return;
    }
    // A command or a tap goes to the host's handler runner, which arrives
    // with the first app (DESIGN D7 E0.3); until then an input only enters
    // its speaker in the guest book.
    log('bridge:input', { speaker: i.speaker, kind: i.kind, interaction: i.interaction });
  }

  /** An event that failed is said, never dropped: in the chat when it was a person's input. */
  const guarded = (name, fn) => async (e) => {
    try {
      await fn(e);
    } catch (err) {
      log('bridge:handler_failed', { handler: name, speaker: e.speaker, error: String(err?.message ?? err) });
      if (name === 'input' && e.kind !== 'message') {
        await adapter
          .reply(e, { text: words('not_written'), private: true })
          .catch((why) => log('bridge:reply_failed', { speaker: e.speaker, error: String(why?.message ?? why) }));
      }
    }
  };

  await adapter.connect({
    joined: guarded('joined', (p) => note(p, 'joined')),
    left: guarded('left', (p) => note(p, p.why)),
    input: guarded('input', input),
  });
  await adapter.commands(commands);
  // A group no one has entered from the chat has not heard the greeting.
  if (!group.entered) await adapter.post({ text: words('greeting') });
  log('bridge:start', { group: group.name, people: group.people });
  return { close: () => adapter.close() };
}
