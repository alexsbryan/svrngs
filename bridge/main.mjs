// `node bridge/main.mjs`: the bridge between one Discord channel and this
// computer's svrngs host. Everything it needs comes from the environment, so
// the one place each value is decided stays the host's (`svrngs serve` prints
// the token file; `svrngs start` will set all of these):
//   SVRNGS_HOST            the host's loopback address, e.g. http://127.0.0.1:7570
//   SVRNGS_TOKEN_FILE      the file holding the host's capability token
//   SVRNGS_DISCORD_TOKEN   the Discord application's bot token
//   SVRNGS_CHANNEL         the id of the channel the group lives in
//   SVRNGS_OWNER           whose computer this is, as the greeting says it

import { readFileSync } from 'node:fs';

import { log, startBridge } from './bridge.mjs';
import { discordAdapter } from './discord.mjs';
import { hostClient } from './host.mjs';

const NEEDS = ['SVRNGS_HOST', 'SVRNGS_TOKEN_FILE', 'SVRNGS_DISCORD_TOKEN', 'SVRNGS_CHANNEL', 'SVRNGS_OWNER'];
const missing = NEEDS.filter((k) => !process.env[k]);
if (missing.length) {
  log('bridge:missing_env', { missing });
  process.stderr.write(`the bridge needs ${missing.join(', ')} set\n`);
  process.exit(2);
}
const env = process.env;
const host = hostClient({ url: env.SVRNGS_HOST, token: readFileSync(env.SVRNGS_TOKEN_FILE, 'utf8').trim() });
const adapter = discordAdapter({ token: env.SVRNGS_DISCORD_TOKEN, channelId: env.SVRNGS_CHANNEL });
const bridge = await startBridge({ adapter, host, owner: env.SVRNGS_OWNER });
for (const signal of ['SIGINT', 'SIGTERM']) {
  process.once(signal, async () => {
    log('bridge:stop', { signal });
    await bridge.close();
    process.exit(0);
  });
}
