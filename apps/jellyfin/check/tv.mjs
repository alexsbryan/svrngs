// The screen in the room, and a phone's "Play on", as u6-2 drives them
// (moved from screen.mjs, behaviour kept, so Demo B's dry run drives the
// screen by the same instrument, u6-9). The
// screen is jellyfin-web in Playwright's Chromium at the room address, a
// profile kept on disk as a TV's app keeps what it stored; every request
// it makes off the room address is kept and refused, as Sam's Jellyfin
// across town would not answer a TV at Dee's.
//
// A step that did not happen within its time is null with what the page
// showed: never a number guessed.
import fs from 'node:fs';
import { shown, timed } from './person.mjs';
import { launch } from '../../../web/tests/page.mjs';

export const masked = (u) => u.replace(/(ApiKey|api_key|Token|Secret)=[^&]+/gi, '$1=<masked>');

// The screen: its profile's Chromium at a TV's size, at `room`. Every
// request kept by where it went, in `log` (a file under `dir`); one off
// the room address refused, socket or not.
export async function screen(profile, { room: at, dir, log: file, T0 }) {
  const { context, close } = await launch('chromium', { profile });
  const room = new URL(at).host;
  const asked = { at_room: 0, elsewhere: [], sockets: [] };
  fs.mkdirSync(dir, { recursive: true });
  const log = fs.openSync(`${dir}/${file}`, 'a');
  const web = (u) => /^(https?|wss?):$/.test(u.protocol);
  context.on('request', (r) => {
    const u = new URL(r.url());
    if (!web(u)) return;
    if (u.host === room) asked.at_room++;
    fs.writeSync(log, JSON.stringify({ ms: Date.now() - T0, method: r.method(), host: u.host, path: masked(u.pathname + u.search) }) + '\n');
  });
  await context.route((u) => web(u) && u.host !== room, (route) => {
    asked.elsewhere.push({ ms: Date.now() - T0, url: masked(route.request().url()) });
    return route.abort('addressunreachable');
  });
  await context.routeWebSocket((u) => u.host !== room, (ws) => {
    asked.elsewhere.push({ ms: Date.now() - T0, url: masked(ws.url()), socket: true });
    ws.close();
  });
  const page = context.pages()[0] ?? (await context.newPage());
  await page.setViewportSize({ width: 1920, height: 1080 });
  asked.browser = await page.evaluate(() => navigator.userAgent);
  page.on('websocket', (ws) => asked.sockets.push(masked(ws.url())));
  const logs = [];
  page.on('pageerror', (e) => logs.push(`pageerror: ${e}`));
  page.on('response', (r) => { if (r.status() >= 400) logs.push(`http ${r.status()} ${r.request().method()} ${masked(r.url()).slice(0, 140)}`); });
  return { context, page, asked, logs, close };
}

// In the screen: what jellyfin-web keeps of the server, and who it is.
export const kept = (page) => page.evaluate(async () => {
  const c = JSON.parse(localStorage.getItem('jellyfin_credentials') ?? '{}');
  const s = c.Servers?.[0] ?? {};
  let me = null;
  try { me = (await window.ApiClient.getJSON(window.ApiClient.getUrl('Users/Me'))).Name; } catch (e) { me = `refused: ${e?.status ?? e}`; }
  return {
    servers: c.Servers?.length ?? 0, ManualAddress: s.ManualAddress ?? null, LocalAddress: s.LocalAddress ?? null,
    RemoteAddress: s.RemoteAddress ?? null, LastConnectionMode: s.LastConnectionMode ?? null, token: s.AccessToken ? 'held' : null,
    serverAddress: window.ApiClient?.serverAddress() ?? null, device: localStorage.getItem('_deviceId2'), signed_in_as: me,
  };
}).catch((e) => ({ error: String(e).split('\n')[0] }));

// The screen's session as the lender's Jellyfin (`jf`, person.mjs's
// `lender`) sees it.
export async function session(jf, device) {
  const s = (await jf('GET', '/Sessions')).find((x) => x.DeviceId === device);
  return s ? { user: s.UserName, client: s.Client, version: s.ApplicationVersion, device_name: s.DeviceName, remote_end: s.RemoteEndPoint,
    remote_control: s.SupportsRemoteControl, media_control: s.SupportsMediaControl, playing: s.NowPlayingItem?.Name ?? null,
    paused: s.PlayState?.IsPaused ?? null, play_method: s.PlayState?.PlayMethod ?? null } : null;
}

// In a page: its <video>, if any.
export const video = (page) => page.evaluate(() => { const v = document.querySelector('video'); return v && { ct: +v.currentTime.toFixed(2), paused: v.paused, rs: v.readyState, dur: v.duration }; }).catch(() => null);
export async function until(page, ok, within) {
  const t0 = Date.now();
  while (Date.now() - t0 < within) {
    const v = await video(page);
    if (ok(v)) return { ms: Date.now() - t0, video: v };
    await page.waitForTimeout(100);
  }
  return { ms: null, video: await video(page), shown: await shown(page) };
}

// The screen opened at `room`; signed in typing `who`'s (its password
// `pw(who)`), or left at what it shows when `who` is none.
export async function open(s, who, room, pw) {
  const step = {};
  step.open_ms = await timed(s, 'the room address', async () => {
    await s.page.goto(room);
    await s.page.waitForSelector('#txtManualName, .manualLoginForm, .btnManual, .btnQuick, .card', { timeout: 60_000 });
  });
  if (step.open_ms === null) return step;
  await s.page.waitForTimeout(1500);
  step.asked = (await s.page.locator('#txtManualName:visible, .manualLoginForm:visible, .btnManual:visible').count()) > 0;
  step.quick_connect_offered = (await s.page.locator('.btnQuick:visible').count()) > 0;
  if (!who || !step.asked) return step;
  step.signed_in_ms = await timed(s, 'signed in', async () => {
    const manual = s.page.locator('.btnManual:visible');
    if (await manual.count()) await manual.first().click();
    await s.page.fill('#txtManualName', who);
    await s.page.fill('#txtManualPassword', pw(who));
    await s.page.locator('.manualLoginForm button[type=submit]').click();
    await s.page.waitForSelector('.card', { timeout: 30_000 });
  });
  return step;
}

// jellyfin-web's own "Play on", on a phone's page `p`: "Cast to Device" at
// the top — the icon, or the player's name once one is picked — and its
// menu, as the menu writes each item.
export function playOn(p) {
  const cast = p.getByRole('button', { name: 'Cast to Device' });
  const header = () => cast.first().innerText().then((t) => t.trim() || '(icon)', () => null);
  const menu = async () => {
    await cast.first().click();
    // As a person waits for the list: tapped 1.5 s after its first item
    // showed, the phone's play went to the phone itself (02:09-02:13).
    await p.waitForTimeout(2000);
    return (await p.locator('[role=menuitem]:visible').allInnerTexts()).map((x) => x.replace(/\s+/g, ' ').trim()).filter(Boolean);
  };
  return { cast, header, menu };
}
