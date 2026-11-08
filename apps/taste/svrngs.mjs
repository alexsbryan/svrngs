// What taste asks of the svrngs running on this computer, as a program on
// it: the record's address from the data dir's `api` file and the `token`
// beside it (give-token), as apps/pantry/pantry.mjs `recordAt` and `tokenAt`
// read them. The record serves a program, never a page (crates/svrngs-node/
// src/api.rs `not_from_a_page`), so only this program asks it.
import { execFile } from 'node:child_process';
import { existsSync, readFileSync } from 'node:fs';
import { homedir } from 'node:os';
import { join } from 'node:path';

/// svrngs's data dir: `SVRNGS_DIR`, else `~/.svrngs`, as svrngs and the
/// pantry name it.
export const dataDir = () => process.env.SVRNGS_DIR || join(homedir(), '.svrngs');

/// The record's address and token, read on each ask: svrngs writes `api`
/// again each time it starts (api.rs `write_address`).
export function record() {
  const at = readFileSync(join(dataDir(), 'api'), 'utf8').trim().replace(/\/+$/, '');
  let token = '';
  try {
    token = readFileSync(join(dataDir(), 'token'), 'utf8').trim();
  } catch {
    // None kept: the record refuses, naming the file, which is said then.
  }
  return { at, token };
}

/// `GET <path>` of the record, as this computer's program: its JSON.
export async function get(path) {
  const { at, token } = record();
  const res = await fetch(`${at}${path}`, {
    headers: token ? { authorization: `Bearer ${token}` } : {},
    redirect: 'error',
    signal: AbortSignal.timeout(10_000),
  });
  if (!res.ok) throw new Error(`svrngs answered ${path} with ${res.status}: ${await res.text()}`);
  return res.json();
}

/// The groups this computer stands in, and its own key (surface.rs `groups`
/// answers `key`, "this computer's key").
export async function groups() {
  const held = await get('/v1/groups');
  return { me: held.key, groups: (held.groups ?? []).filter((g) => !g.removed && !g.left) };
}

/// The person this computer is: in each group, the first key of the person
/// its key stands for (`/v1/rail/people`'s `person_key`), which the door sends
/// as `x-svrngs-key` from every device of theirs (door.rs `KEY_HEADER`). One
/// key a group, as a person's first key there is the device they joined it
/// from first. Asked on each request, so a group joined since counts at once.
export async function ownerKeys() {
  const { me, groups: held } = await groups();
  const keys = new Set();
  for (const g of held) {
    const { people } = await get(`/v1/rail/people?namespace=${encodeURIComponent(g.id)}`);
    const mine = (people ?? []).find((p) => p.key === me);
    if (mine?.person_key) keys.add(mine.person_key);
  }
  return keys;
}

/// Every lend of `kind` in this computer's groups, in the record's order,
/// each at its origin on this computer, which answers a program with the
/// token (pantry.mjs `probe`).
export async function lendsOf(kind) {
  const { groups: held } = await groups();
  const found = [];
  const seen = new Set();
  for (const g of held) {
    const body = await get(`/v1/lends?namespace=${encodeURIComponent(g.id)}`);
    for (const lend of body.lent ?? []) {
      if ((lend.offered?.kind ?? lend.kind) !== kind || seen.has(lend.origin)) continue;
      seen.add(lend.origin);
      found.push(lend);
    }
  }
  return found;
}

/// The `svrngs` command, as a person runs it: on the `PATH`, else where
/// web/install.sh puts it, `~/.local/bin`, which a new terminal may not look
/// in yet (install.sh says so).
export function command() {
  const installed = join(homedir(), '.local/bin/svrngs');
  const onPath = (process.env.PATH ?? '')
    .split(':')
    .some((d) => d && existsSync(join(d, 'svrngs')));
  return onPath || !existsSync(installed) ? 'svrngs' : installed;
}

/// `svrngs <args>`, nothing typed at it: what it printed, or why it failed.
export function run(args) {
  return new Promise((done) => {
    const child = execFile(command(), args, { timeout: 60_000 }, (error, stdout, stderr) =>
      done({ ok: !error, said: `${stdout}${stderr}`.trim(), error }),
    );
    child.stdin?.end();
  });
}
