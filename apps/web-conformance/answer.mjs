// The conformance canary's sign-in (mwp-4; DESIGN §3, "The room", call 6):
// what a lend of the canary with accounts (serve.mjs, given the lender's
// key) declares as its sign-in, so the door signs each person in as
// themselves with nothing typed. Asked `POST /` by the door with the
// person's `x-svrngs-user`, the account name the door makes unique by
// their key (never `x-svrngs-person`, a name two people may share), it has
// the canary find or make that account by the lender's key, and answers
// its token as the
// person's secret, a bearer `authorization` of it, and a script that
// stores the door's stand-in, as the ask names it in `x-svrngs-stand-in`,
// where the canary's page reads its sign-in.
//
//   node apps/web-conformance/answer.mjs <the canary's port> <lender's key>
//
// Listens on 127.0.0.1, any port, and prints `{"port": N}`. No dependency.
// CANARY_PLANT=one-account (its plant) answers the first person's account
// for everyone.
import http from 'node:http';

const [canary, lender] = process.argv.slice(2);
let first = null;

async function token(name) {
  const res = await fetch(`http://127.0.0.1:${canary}/accounts`, {
    method: 'POST',
    headers: { authorization: `Bearer ${lender}`, 'content-type': 'application/json' },
    body: JSON.stringify({ name }),
  });
  if (!res.ok) throw new Error(`the canary made no account for ${name}: ${res.status}`);
  return (await res.json()).token;
}

const server = http.createServer(async (req, res) => {
  const user = req.headers['x-svrngs-user'];
  const standIn = req.headers['x-svrngs-stand-in'];
  if (req.method !== 'POST' || !user || !standIn) {
    res.writeHead(400).end('asked with no account name or no stand-in');
    return;
  }
  try {
    if (process.env.CANARY_PLANT === 'one-account') first ??= user;
    const secret = await token(first ?? user);
    const script = `localStorage.setItem('canary-token', ${JSON.stringify(standIn)});`;
    res.writeHead(200, { 'content-type': 'application/json' });
    res.end(JSON.stringify({ secret, headers: { authorization: `Bearer ${secret}` }, script }));
  } catch (e) {
    res.writeHead(502).end(String(e));
  }
});

server.listen(0, '127.0.0.1', () => console.log(JSON.stringify({ port: server.address().port })));
