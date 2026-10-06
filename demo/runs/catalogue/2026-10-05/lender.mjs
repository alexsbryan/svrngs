// What the lender does once in each app before lending it (run.sh `apps`):
// the app's own first-run setup, then an account for Bo, the account each
// person signs in once with (DESIGN §3, "Lent apps", call 4). Every step is
// the app's own API on its port, on the lender's computer; nothing here
// passes through svrngs.
//
//   node lender.mjs <app> <bo's password>    app: home-assistant|immich|nextcloud|actual|paperless
//   env: SAM_PW, the lender's own password in each
//
// Prints one JSON line: what the act needs to check the app afterwards
// (the lender's token where the app has one), kept in the scratch dir.
const [app, boPw] = process.argv.slice(2);
const SAM_PW = process.env.SAM_PW;
const ok = async (res, what) => {
  if (!res.ok) throw new Error(`${what}: ${res.status} ${await res.text()}`);
  return res;
};
const json = (body) => ({ 'content-type': 'application/json', body: JSON.stringify(body) });

async function homeAssistant() {
  const O = 'http://127.0.0.1:8123';
  const client_id = `${O}/`;
  const post = (path, body, token) =>
    fetch(`${O}${path}`, { method: 'POST', headers: { 'content-type': 'application/json', ...(token ? { authorization: `Bearer ${token}` } : {}) }, body: JSON.stringify(body) });
  const done = new Set((await (await ok(await fetch(`${O}/api/onboarding`), 'onboarding')).json()).filter((s) => s.done).map((s) => s.step));
  let code;
  if (!done.has('user')) {
    code = (await (await ok(await post('/api/onboarding/users', { client_id, name: 'Sam', username: 'sam', password: SAM_PW, language: 'en' }), 'onboarding user')).json()).auth_code;
  } else {
    // Signed in as a person does, by its login flow.
    const flow = await (await ok(await post('/auth/login_flow', { client_id, handler: ['homeassistant', null], redirect_uri: `${O}/?auth_callback=1` }), 'login flow')).json();
    code = (await (await ok(await post(`/auth/login_flow/${flow.flow_id}`, { client_id, username: 'sam', password: SAM_PW }), 'sign-in')).json()).result;
  }
  const token = (await (await ok(await fetch(`${O}/auth/token`, { method: 'POST', body: new URLSearchParams({ grant_type: 'authorization_code', code, client_id }) }), 'token')).json()).access_token;
  for (const step of ['core_config', 'analytics']) if (!done.has(step)) await ok(await post(`/api/onboarding/${step}`, {}, token), step);
  if (!done.has('integration')) await ok(await post('/api/onboarding/integration', { client_id, redirect_uri: `${O}/?auth_callback=1` }, token), 'integration');
  // A long-lived token for the act's lamp, and Bo's account: both only over
  // its socket, which Node 20 has no client for, so a headless page asks.
  const { launch } = await import('../../../../web/tests/page.mjs');
  const { context, close } = await launch('chromium');
  const page = await context.newPage();
  await page.goto(`${O}/manifest.json`);
  const made = await page.evaluate(async ({ token, boPw }) => {
    const ws = new WebSocket(`ws://${location.host}/api/websocket`);
    let id = 0;
    const waiting = new Map();
    const ask = (msg) => new Promise((done, fail) => {
      const n = ++id;
      waiting.set(n, (m) => (m.success ? done(m.result) : fail(new Error(JSON.stringify(m.error)))));
      ws.send(JSON.stringify({ id: n, ...msg }));
    });
    await new Promise((done, fail) => {
      ws.onmessage = (e) => {
        const m = JSON.parse(e.data);
        if (m.type === 'auth_required') ws.send(JSON.stringify({ type: 'auth', access_token: token }));
        else if (m.type === 'auth_ok') done();
        else if (m.type === 'auth_invalid') fail(new Error('auth_invalid'));
        else if (m.type === 'result') waiting.get(m.id)?.(m);
      };
    });
    const lasting = await ask({ type: 'auth/long_lived_access_token', client_name: `catalogue run ${Date.now()}`, lifespan: 30 });
    const users = await ask({ type: 'config/auth/list' });
    if (!users.some((u) => u.name === 'Bo')) {
      const bo = await ask({ type: 'config/auth/create', name: 'Bo', group_ids: ['system-users'], local_only: false });
      await ask({ type: 'config/auth_provider/homeassistant/create', user_id: bo.user.id, username: 'bo', password: boPw });
    }
    ws.close();
    return lasting;
  }, { token, boPw });
  await close();
  return { token: made, bo: 'bo' };
}

async function immich() {
  const O = 'http://127.0.0.1:2283/api';
  await ok(await fetch(`${O}/auth/admin-sign-up`, { method: 'POST', headers: { 'content-type': 'application/json' }, body: JSON.stringify({ email: 'sam@house.test', password: SAM_PW, name: 'Sam' }) }), 'admin sign-up');
  const { accessToken } = await (await ok(await fetch(`${O}/auth/login`, { method: 'POST', headers: { 'content-type': 'application/json' }, body: JSON.stringify({ email: 'sam@house.test', password: SAM_PW }) }), 'login')).json();
  await ok(await fetch(`${O}/admin/users`, { method: 'POST', headers: { 'content-type': 'application/json', authorization: `Bearer ${accessToken}` }, body: JSON.stringify({ email: 'bo@house.test', password: boPw, name: 'Bo', shouldChangePassword: false }) }), 'bo');
  return { token: accessToken, bo: 'bo@house.test' };
}

async function actual() {
  const O = 'http://127.0.0.1:5006';
  // Actual keeps one password for its server, not accounts (CENSUS.md):
  // the lender sets it, and each person types it once.
  await ok(await fetch(`${O}/account/bootstrap`, { method: 'POST', headers: { 'content-type': 'application/json' }, body: JSON.stringify({ password: boPw }) }), 'bootstrap');
  const { data } = await (await ok(await fetch(`${O}/account/login`, { method: 'POST', headers: { 'content-type': 'application/json' }, body: JSON.stringify({ loginMethod: 'password', password: boPw }) }), 'login')).json();
  return { token: data.token, bo: '' };
}

async function paperless() {
  const O = 'http://127.0.0.1:8000';
  const basic = `Basic ${Buffer.from(`sam:${SAM_PW}`).toString('base64')}`;
  // Bo may add, see and change documents: the lender's choice of what a
  // person may do there, made a superuser for this dry run alone.
  await ok(await fetch(`${O}/api/users/`, { method: 'POST', headers: { 'content-type': 'application/json', authorization: basic }, body: JSON.stringify({ username: 'bo', password: boPw, is_superuser: true, is_staff: false, is_active: true }) }), 'bo');
  return { token: basic, bo: 'bo' };
}

const steps = { 'home-assistant': homeAssistant, immich, actual, paperless };
// Nextcloud's container installs itself with the lender's account (run.sh);
// Bo's is made with its own `occ user:add` there.
if (!steps[app]) {
  console.error(`lender.mjs: no app ${app}`);
  process.exit(2);
}
console.log(JSON.stringify(await steps[app]()));
