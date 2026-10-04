// `node bridge/setup.mjs application|servers`: what `svrngs start` asks the
// chat platform while it sets the bot up (DESIGN C11). The bot's token comes
// in SVRNGS_DISCORD_TOKEN, never as an argument, so no process list shows it.
// Prints one line of JSON: the answer, or `{refused: 'token'}`.

import { log } from './bridge.mjs';
import { discordSetup } from './discord.mjs';

const ASKS = ['application', 'servers'];
const what = process.argv[2];
const token = process.env.SVRNGS_DISCORD_TOKEN;
if (!ASKS.includes(what) || !token) {
  process.stderr.write(`usage: SVRNGS_DISCORD_TOKEN=... node setup.mjs ${ASKS.join('|')}\n`);
  process.exit(2);
}
const answer = await discordSetup({ token })[what]();
log('bridge:setup', { what, refused: answer.refused ?? null });
process.stdout.write(`${JSON.stringify(answer)}\n`);
