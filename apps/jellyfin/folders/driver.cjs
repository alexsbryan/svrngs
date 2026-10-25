// Drives stock jellyfin-web (served by the container) in Playwright browsers:
// log in, open each .strm movie, press Play, time first playback and three
// seeks, capture <video>.currentSrc, PlaybackInfo and PlayMethod. Writes one
// window mark per (browser, file) to marks.jsonl so the range-server log can
// be attributed by request start time.
// usage: node driver.cjs STATE_JSON OUT_DIR [browser:file comma list]
//
// Forked from svrn-docs rings/runs/strm-2026-10-05/driver.cjs@457f120 (gf-1):
// each play named browser:file and once (chromium:mp4,webkit:mkv), its
// marks carrying the run's PHASE; Jellyfin's ffmpeg looked for on this host,
// not in a container; quiet read from the meeting point's log (QUIET_LOG),
// not the range server's.
const fs = require('fs');
const path = require('path');
const { execSync } = require('child_process');
const PW = process.env.PW_MODULE || path.join(__dirname, '../../../web/node_modules/playwright');
const PHASE = process.env.PHASE || '';
const { chromium, webkit, devices } = require(PW);

const [stateFile, outDir, browserList] = process.argv.slice(2);
const state = JSON.parse(fs.readFileSync(stateFile, 'utf8'));
const BASE = state.base;
const marks = path.join(outDir, 'marks.jsonl');
const results = path.join(outDir, 'results.jsonl');
const now = () => Date.now() / 1000;
const log = (...a) => console.log(new Date().toISOString().slice(11, 19), ...a);
const sleep = (ms) => new Promise((r) => setTimeout(r, ms));

const CONFIGS = {
  chromium: { launcher: chromium, ctx: {} },
  webkit: { launcher: webkit, ctx: {} },
  'webkit-iphone': { launcher: webkit, ctx: { ...devices['iPhone 15'] } },
};
const FILES = [
  { key: 'mp4', name: 'ExpMp4 (2026)' },
  { key: 'mkv', name: 'ExpMkv (2026)' },
];

// Capture-phase listeners on document see the non-bubbling media events of
// whatever <video> jellyfin-web creates, from the moment it exists.
const INIT = () => {
  window.__ev = [];
  for (const e of ['loadstart', 'loadedmetadata', 'loadeddata', 'canplay', 'playing', 'waiting',
    'seeking', 'seeked', 'pause', 'error', 'stalled', 'emptied']) {
    document.addEventListener(e, (ev) => {
      const v = ev.target;
      if (!(v instanceof HTMLMediaElement)) return;
      window.__ev.push({ e, t: performance.now(), ct: v.currentTime, src: v.currentSrc,
        err: v.error ? v.error.code + ' ' + v.error.message : null });
    }, true);
  }
};

function ffmpegInContainer() {
  // Jellyfin's own ffmpeg and ffprobe, the app's, on this host.
  try {
    // Anchored: Jellyfin's own command line names its ffmpeg too.
    return execSync(`pgrep -fl '^[^ ]*Jellyfin.app/Contents/MacOS/ff(mpeg|probe) ' || true`).toString().trim();
  } catch (e) { return 'pgrep-failed: ' + e.message; }
}

function rangeLogLines() {
  try { return fs.readFileSync(process.env.QUIET_LOG, 'utf8').split('\n').filter(Boolean).length; }
  catch { return 0; }
}

async function quiet() {
  // Wait until Jellyfin has no ffmpeg running and the range log stops growing.
  let last = -1, stable = 0;
  for (let i = 0; i < 60; i++) {
    const n = rangeLogLines();
    const ff = ffmpegInContainer();
    stable = (n === last && !ff) ? stable + 1 : 0;
    last = n;
    if (stable >= 4) return;
    await sleep(1000);
  }
  log('WARN: did not reach quiet within 60 s', ffmpegInContainer());
}

async function login(page) {
  await page.goto(BASE + '/web/', { waitUntil: 'domcontentloaded' });
  const manual = page.locator('#txtManualName');
  const btnManual = page.locator('.btnManual');
  // The login view renders a user-card list or the manual form depending on
  // whether users are public; both exist in the DOM, so poll visibility.
  for (let i = 0; i < 150 && !(await manual.isVisible()); i++) {
    if (await btnManual.isVisible()) await btnManual.click({ timeout: 2000 }).catch(() => {});
    await sleep(200);
  }
  await manual.fill(state.user);
  await page.locator('#txtManualPassword').fill(state.pw);
  await page.locator('.manualLoginForm button[type=submit]').click();
  await page.waitForFunction(() => /#\/home/.test(location.hash), null, { timeout: 30000 });
}

async function seekAndTime(page, target) {
  return page.evaluate(async (target) => {
    const v = (document.querySelector('video.htmlvideoplayer') || document.querySelector('video'));
    return await new Promise((res) => {
      const t0 = performance.now();
      const evs = ['timeupdate', 'playing', 'seeked', 'canplay'];
      let iv, to;
      const done = (r) => { evs.forEach((e) => v.removeEventListener(e, check)); clearInterval(iv); clearTimeout(to); res(r); };
      // resumed = advancing past the target, not paused, not seeking
      function check() {
        if (!v.paused && !v.seeking && v.currentTime > target + 0.01) {
          done({ ms: Math.round(performance.now() - t0), ct: v.currentTime });
        }
      }
      evs.forEach((e) => v.addEventListener(e, check));
      iv = setInterval(check, 10);
      to = setTimeout(() => done({ ms: null, timeout: true, ct: v.currentTime, paused: v.paused,
        seeking: v.seeking, rs: v.readyState }), 10000);
      v.currentTime = target;
    });
  }, target);
}

async function runOne(cfgName, file, rep) {
  const cfg = CONFIGS[cfgName];
  const item = state.items[file.name];
  const browser = await cfg.launcher.launch({ headless: true });
  const context = await browser.newContext({ ...cfg.ctx });
  await context.addInitScript(INIT);
  const page = await context.newPage();
  const net = [], consoleLines = [];
  let playbackInfo = null, playMethod = null;
  page.on('console', (m) => consoleLines.push(`${m.type()}: ${m.text()}`.slice(0, 400)));
  page.on('request', (r) => {
    const u = r.url();
    if (u.includes(':18733') || /\/Videos\/|\/PlaybackInfo|\/Sessions\/Playing|master\.m3u8|\.ts|\.mp4/.test(u)) {
      net.push({ t: now(), method: r.method(), url: u.slice(0, 300), type: r.resourceType() });
      if (u.split('?')[0].endsWith('/Sessions/Playing')) {
        try { playMethod = JSON.parse(r.postData() || '{}').PlayMethod; } catch {}
      }
    }
  });
  page.on('response', async (r) => {
    if (/\/Items\/[^/]+\/PlaybackInfo/.test(r.url())) {
      try {
        const j = await r.json();
        const ms = (j.MediaSources || [])[0] || {};
        playbackInfo = {
          SupportsDirectPlay: ms.SupportsDirectPlay, SupportsDirectStream: ms.SupportsDirectStream,
          TranscodingUrl: ms.TranscodingUrl || null, TranscodeReasons: ms.TranscodeReasons || null,
          Container: ms.Container, Path: ms.Path, Protocol: ms.Protocol, IsRemote: ms.IsRemote,
          RunTimeTicks: ms.RunTimeTicks,
          Streams: (ms.MediaStreams || []).map((s) => `${s.Type}:${s.Codec}${s.Width ? ':' + s.Width + 'x' + s.Height : ''}`),
          ErrorCode: j.ErrorCode || null,
        };
      } catch (e) { playbackInfo = { parseError: String(e) }; }
    }
  });

  const rec = { phase: PHASE, browser: cfgName, file: file.key, rep, item_id: item.id };
  try {
    await login(page);
    rec.ua = await page.evaluate(() => navigator.userAgent);
    rec.canPlay = await page.evaluate(() => {
      const v = document.createElement('video');
      return { h264: v.canPlayType('video/mp4; codecs="avc1.640028, mp4a.40.2"'),
        mkv: v.canPlayType('video/x-matroska'), mkvH264: v.canPlayType('video/x-matroska; codecs="avc1.640028"'),
        hls: v.canPlayType('application/vnd.apple.mpegurl'),
        mseH264: !!(window.MediaSource && MediaSource.isTypeSupported('video/mp4; codecs="avc1.640028"')) };
    });
    // Window opens BEFORE the item page loads: everything from here to the
    // quiet point after stop is attributed to this (browser, file).
    rec.window_start = now();
    fs.appendFileSync(marks, JSON.stringify({ run: PHASE, browser: cfgName, file: file.key, rep, phase: 'start', t: rec.window_start, ua: rec.ua }) + '\n');
    await page.goto(`${BASE}/web/#/details?id=${item.id}&serverId=${state.server_id}`);
    const play = page.locator('.btnPlay >> visible=true').first();
    await play.waitFor({ state: 'visible', timeout: 30000 });
    await sleep(1000);
    await page.evaluate(() => { window.__tClick = performance.now(); });
    rec.t_click = now();
    await play.click();
    // first playback: a 'playing' event with currentTime advancing
    const first = await page.waitForFunction(() => {
      const v = (document.querySelector('video.htmlvideoplayer') || document.querySelector('video'));
      const p = (window.__ev || []).find((x) => x.e === 'playing');
      if (!v || !p || v.paused || v.currentTime <= 0) return null;
      const firstAdvance = performance.now();
      return { playing_ms: Math.round(p.t - window.__tClick), advancing_seen_ms: Math.round(firstAdvance - window.__tClick) };
    }, null, { timeout: 45000, polling: 20 }).then((h) => h.jsonValue()).catch((e) => ({ error: String(e).slice(0, 200) }));
    rec.first = first;
    if (!first.error) {
      await sleep(4000);
      const dur = await page.evaluate(() => (document.querySelector('video.htmlvideoplayer') || document.querySelector('video')).duration);
      rec.duration = dur;
      const D = Number.isFinite(dur) && dur > 0 ? dur : 120;
      rec.seeks = [];
      for (const frac of [0.25, 0.60, 0.10]) {
        const target = Math.round(D * frac * 1000) / 1000;
        const r = await seekAndTime(page, target);
        rec.seeks.push({ frac, target, ...r });
        await sleep(3000);
      }
    }
    rec.video = await page.evaluate(() => {
      const v = (document.querySelector('video.htmlvideoplayer') || document.querySelector('video'));
      if (!v) return null;
      const b = [];
      for (let i = 0; i < v.buffered.length; i++) b.push([v.buffered.start(i), v.buffered.end(i)]);
      return { currentSrc: v.currentSrc, src: v.getAttribute('src'), crossOrigin: v.crossOrigin,
        currentTime: v.currentTime, paused: v.paused, readyState: v.readyState, buffered: b,
        error: v.error ? v.error.code : null };
    });
    rec.events = await page.evaluate(() => (window.__ev || []).map((x) => ({ ...x, t: Math.round(x.t - (window.__tClick || 0)) })));
    // Stop the way a user does: leave the player (jellyfin-web reports
    // Sessions/Playing/Stopped, which ends any server transcode).
    await page.evaluate(() => history.back());
    await sleep(3000);
  } catch (e) {
    rec.error = String(e).slice(0, 500);
    log('ERROR', cfgName, file.key, rec.error);
  }
  rec.playbackInfo = playbackInfo;
  rec.playMethod = playMethod;
  rec.net = net;
  rec.console = consoleLines.filter((l) => /error|warn|play|media|video/i.test(l)).slice(-40);
  await context.close().catch(() => {});
  await browser.close().catch(() => {});
  await quiet();
  rec.window_end = now();
  fs.appendFileSync(marks, JSON.stringify({ run: PHASE, browser: cfgName, file: file.key, rep, phase: 'end', t: rec.window_end }) + '\n');
  fs.appendFileSync(results, JSON.stringify(rec) + '\n');
  log(cfgName, file.key, 'rep', rep, 'first', JSON.stringify(rec.first), 'seeks', JSON.stringify((rec.seeks || []).map((s) => s.ms)),
    'src', rec.video && rec.video.currentSrc, 'method', playMethod);
}

(async () => {
  // spec "browser:file", e.g. chromium:mp4,webkit:mp4,webkit:mkv; each once
  for (const spec of (browserList || 'chromium:mp4').split(',')) {
    const [b, key] = spec.split(':');
    await runOne(b, FILES.find((f) => f.key === key), 1);
  }
})().catch((e) => { console.error(e); process.exit(1); });
