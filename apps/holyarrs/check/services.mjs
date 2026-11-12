// The scene's owned pod: stock HolyArrs and two real qBittorrents, a local
// seed and downloader. Only this pod is removed; its evidence stays on disk.
import assert from 'node:assert/strict';
import { execFileSync, spawnSync } from 'node:child_process';
import { randomBytes } from 'node:crypto';
import { mkdirSync, readFileSync, writeFileSync } from 'node:fs';
import { resolve, dirname } from 'node:path';
import { fileURLToPath } from 'node:url';
import { setTimeout as pause } from 'node:timers/promises';
import { film, release } from './fixture.mjs';

const here = dirname(fileURLToPath(import.meta.url));
const dir = resolve(process.argv[3]);
const state = `${dir}/services.json`;
const podman = (...args) =>
  execFileSync('podman', args, { encoding: 'utf8', timeout: 120_000 }).trim();
const save = (to, value) => writeFileSync(to, JSON.stringify(value, null, 2), { mode: 0o600 });
const images = {
  holyarrs: 'docker.io/holyarrs/holyarrs:1.0.10',
  qbit: 'lscr.io/linuxserver/qbittorrent:5.1.4',
  node: 'docker.io/library/node:24.22.1-slim',
};
async function until(what, f, seconds = 90) {
  let last;
  const end = Date.now() + seconds * 1000;
  do {
    try {
      last = await f();
      if (last) return last;
    } catch (e) {
      last = String(e);
    }
    await pause(500);
  } while (Date.now() < end);
  throw new Error(`${what} did not arrive in ${seconds}s; last: ${JSON.stringify(last)}`);
}

async function up() {
  let digests;
  try {
    podman('info');
    execFileSync('ffmpeg', ['-version'], { timeout: 10_000, stdio: 'pipe' });
    digests = Object.fromEntries(
      Object.entries(images).map(([k, image]) => [
        k,
        JSON.parse(podman('image', 'inspect', image))[0].RepoDigests,
      ]),
    );
  } catch (e) {
    console.log(
      JSON.stringify({
        never_ran: `HolyArrs scene prerequisites: ${String(e).split('\n')[0]}. See apps/holyarrs/README.md.`,
      }),
    );
    return;
  }
  mkdirSync(dir, { recursive: true });
  const pod = `svrngs-test-holyarrs-${randomBytes(6).toString('hex')}`;
  const s = {
    pod,
    dir,
    film,
    library: resolve(process.argv[4] ?? `${dir}/data/movies`),
    ana: { name: 'Ana', password: randomBytes(24).toString('hex') },
    bo: { name: 'Bo', password: randomBytes(24).toString('hex') },
  };
  save(state, s);
  for (const sub of [
    'seed',
    'data/movies',
    'data/downloads',
    'seed-config/qBittorrent',
    'download-config/qBittorrent',
  ])
    mkdirSync(`${dir}/${sub}`, { recursive: true });
  execFileSync(
    'ffmpeg',
    [
      '-v',
      'error',
      '-y',
      '-f',
      'lavfi',
      '-i',
      'testsrc2=size=1280x720:rate=24',
      '-f',
      'lavfi',
      '-i',
      'sine=frequency=440:sample_rate=48000',
      '-t',
      '75',
      '-c:v',
      'libx264',
      '-preset',
      'ultrafast',
      '-b:v',
      '2M',
      '-g',
      '48',
      '-pix_fmt',
      'yuv420p',
      '-c:a',
      'aac',
      '-movflags',
      '+faststart',
      `${dir}/seed/${release}`,
    ],
    { timeout: 120_000 },
  );
  for (const [name, port, peer] of [
    ['seed', 8081, 6881],
    ['download', 8080, 6882],
  ]) {
    writeFileSync(
      `${dir}/${name}-config/qBittorrent/qBittorrent.conf`,
      `[BitTorrent]\nSession\\Port=${peer}\nSession\\DHTEnabled=false\nSession\\PeXEnabled=false\nSession\\LSDEnabled=false\n[Preferences]\nWebUI\\Port=${port}\nWebUI\\LocalHostAuth=false\nWebUI\\AuthSubnetWhitelist=0.0.0.0/0\nWebUI\\AuthSubnetWhitelistEnabled=true\nWebUI\\HostHeaderValidation=false\nWebUI\\CSRFProtection=false\nWebUI\\ClickjackingProtection=false\n`,
    );
  }
  podman(
    'pod',
    'create',
    '--name',
    pod,
    ...[6262, 8080, 8081, 18080].flatMap((p) => ['-p', `127.0.0.1::${p}`]),
  );
  podman('pod', 'start', pod);
  const infra = podman('pod', 'inspect', pod, '--format', '{{.InfraContainerID}}');
  const ports = Object.fromEntries(
    podman('port', infra)
      .split('\n')
      .map((l) => {
        const [p, at] = l.split(' -> ');
        return [p.split('/')[0], Number(at.split(':').at(-1))];
      }),
  );
  Object.assign(s, {
    at: `http://127.0.0.1:${ports[6262]}`,
    fixture: `http://127.0.0.1:${ports[18080]}`,
    downloader: `http://127.0.0.1:${ports[8080]}`,
    seed: `http://127.0.0.1:${ports[8081]}`,
    port: ports[6262],
  });
  save(state, s);
  s.images = digests;
  s.configVolume = `${pod}-config`;
  save(state, s);
  const start = (name, image, args = [], flags = []) =>
    podman('run', '-d', '--pod', pod, '--name', `${pod}-${name}`, ...flags, image, ...args);
  start(
    'fixture',
    images.node,
    ['node', '/scene/fixture.mjs', 'serve'],
    ['-v', `${here}:/scene:ro`, '-v', `${dir}/seed:/seed:ro`, '-v', `${dir}/data:/data`],
  );
  for (const [name, port] of [
    ['seed', 8081],
    ['download', 8080],
  ])
    start(
      name,
      images.qbit,
      [],
      [
        '-e',
        'PUID=0',
        '-e',
        'PGID=0',
        '-e',
        `WEBUI_PORT=${port}`,
        '-v',
        `${dir}/${name}-config:/config`,
        ...(name === 'seed' ? ['-v', `${dir}/seed:/seed:ro`] : []),
        '-v',
        `${dir}/data:/data`,
      ],
    );
  start(
    'holyarrs',
    images.holyarrs,
    [],
    [
      '-e',
      'PUID=0',
      '-e',
      'PGID=0',
      '-e',
      'HOLYARRS_METADATA_URL=http://127.0.0.1:18080',
      '-e',
      'HOLYARRS_METADATA_SERVICE_ALLOW_LOCAL=true',
      '-e',
      'HOLYARRS_RELEASE_FEED=http://127.0.0.1:18080/releases/latest',
      '-e',
      'HOLYARRS_SAVE_PATH=/data/downloads',
      '-v',
      `${s.configVolume}:/config`,
      '-v',
      `${dir}/data:/data`,
      '-v',
      `${s.library}:/data/movies`,
    ],
  );
  await until('HolyArrs ready', async () => (await fetch(`${s.at}/api/v1/auth/state`)).ok);
  await until('seed ready', async () => (await fetch(`${s.seed}/api/v2/app/version`)).ok);
  await until(
    'downloader ready',
    async () => (await fetch(`${s.downloader}/api/v2/app/version`)).ok,
  );
  const torrent = await (await fetch(`${s.fixture}/release.torrent`)).blob();
  const form = new FormData();
  form.set('torrents', torrent, 'scene.torrent');
  form.set('savepath', '/seed');
  const added = await fetch(`${s.seed}/api/v2/torrents/add`, { method: 'POST', body: form });
  assert.equal(added.status, 200, await added.text());
  await until('local seed checked complete', async () => {
    const torrents = await (await fetch(`${s.seed}/api/v2/torrents/info`)).json();
    return torrents.length === 1 && torrents[0].progress === 1;
  });
  save(state, s);
  console.log(JSON.stringify({ state, port: s.port, at: s.at, pod }));
}

async function setup() {
  const s = JSON.parse(readFileSync(state));
  let cookie = '',
    csrf = '';
  const call = async (method, path, body) => {
    const r = await fetch(`${s.at}/api/v1${path}`, {
      method,
      headers: { cookie, 'content-type': 'application/json', 'x-csrf-token': csrf },
      ...(body === undefined ? {} : { body: JSON.stringify(body) }),
    });
    const c = r.headers.getSetCookie();
    if (c.length) cookie = c.map((v) => v.split(';')[0]).join('; ');
    const text = await r.text();
    assert.ok(r.ok, `${method} ${path}: ${r.status} ${text}`);
    const got = text ? JSON.parse(text) : {};
    if (got.csrf) csrf = got.csrf;
    return got;
  };
  const auth = await call('GET', '/auth/state');
  await call('POST', auth.needs_setup ? '/auth/setup' : '/auth/login', s.ana);
  await call('GET', '/auth/me');
  const folders = await call('GET', '/library-folders');
  s.folder =
    folders.library_folders.find((f) => f.path === '/data/movies') ??
    (await call('POST', '/library-folders', {
      path: '/data/movies',
      kind: 'movie',
      label: 'Friends films',
    }));
  await call('POST', '/auth/setup/done', {});
  await call('POST', '/parts/wishlist/on', {});
  await call('POST', '/downloaders', {
    name: 'Local qBittorrent',
    kind: 'qbittorrent',
    url: 'http://127.0.0.1:8080',
    username: 'admin',
    secret: 'isolated-scene',
    allow_local_network: true,
    writable: true,
    category: 'svrngs-scene',
  });
  await call('POST', '/sources', {
    name: 'Local scene source',
    format: 'torznab',
    url: 'http://127.0.0.1:18080/source',
    allow_local_network: true,
    kinds: ['movie'],
    use_for: { feed: false, auto_search: true, hand_search: true },
  });
  await call('PUT', '/settings/asks', { library_folder_id: s.folder.id, search_on_approve: true });
  if (process.argv[4]) {
    s.jellyfin = JSON.parse(process.argv[4]);
    await call('POST', '/media-servers', {
      name: 'Ana films',
      kind: 'jellyfin',
      url: `http://host.containers.internal:${s.jellyfin.port}`,
      token: readFileSync(s.jellyfin.token_file, 'utf8'),
      enabled: true,
      on_import: true,
      path_from: '/data/movies',
      path_to: s.library,
      allow_local_network: true,
    });
  }
  s.adminCookie = cookie;
  s.csrf = csrf;
  if (!process.argv.includes('--newcomer')) {
    const invite = await call('POST', '/users/invites', {
      role: 'member',
      note: 'Bo, prepared before the scene',
    });
    await call('POST', `/auth/invites/${invite.path.split('/').at(-1)}`, s.bo);
  }
  save(state, s);
  console.log(JSON.stringify({ setup: true, folder: s.folder }));
}

try {
  if (process.argv[2] === 'up') await up();
  else if (process.argv[2] === 'setup') await setup();
  else if (process.argv[2] === 'down') {
    const s = JSON.parse(readFileSync(state));
    for (const name of ['holyarrs', 'download', 'seed', 'fixture']) {
      try {
        const logs = spawnSync('podman', ['logs', `${s.pod}-${name}`], {
          encoding: 'utf8',
          timeout: 10_000,
        });
        writeFileSync(`${dir}/${name}.log`, `${logs.stdout ?? ''}${logs.stderr ?? ''}`, {
          mode: 0o600,
        });
      } catch {
        /* pod may have failed to start */
      }
    }
    podman('pod', 'rm', '-f', s.pod);
    if (s.configVolume) podman('volume', 'rm', s.configVolume);
    console.log(JSON.stringify({ stopped: s.pod }));
  } else throw new Error('usage: node services.mjs up|setup|down <scratch>');
} catch (e) {
  console.error(e);
  process.exitCode = 1;
}
