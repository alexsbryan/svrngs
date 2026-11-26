// scripts/release.sh, its three steps typed with nothing after them (DESIGN
// §7): `build` builds the commit that set Cargo.toml's version, by that
// commit's own build scripts, in the worktree kept beside the checkout and
// under the gate lock, and saves what `sign` and `page` read; `sign` signs
// only a commit on origin/main whose tag does not exist; `page` publishes
// only a site whose sums the release key signed, and leaves none of the
// Vercel CLI's files in it.
//
// Each test makes a repository of its own, svrngs's files release.sh reads
// and two build scripts that say whose they are, with an origin of its own.
// lockf, npm, curl (a relay answering), vercel and gh are stand-ins on PATH
// that keep what they were asked, so nothing is built, locked, published or
// asked of the internet. gpg is real, its home an empty folder, so no key
// this computer keeps can sign; the signing test makes a throwaway key
// there. The signed site is v1.11.2's own sums and signature
// (crates/svrngs-node/tests/fixtures/).
//
// Planted, each watched failing: the default commit taken as HEAD, or as the
// last to change Cargo.toml; the build scripts run from the checkout, not
// the release tree; the build run without the gate lock; target/debug kept;
// the tree's own changes not refused; state that is missing not refused; a
// tag that exists or a commit not on origin/main not refused; an argument
// after sign taken; the signature's check removed (the unsigned site and
// the changed one deployed); and the Vercel CLI's files left in the site.

import assert from 'node:assert/strict';
import { execFileSync, spawnSync } from 'node:child_process';
import {
  chmodSync,
  copyFileSync,
  existsSync,
  mkdirSync,
  mkdtempSync,
  readFileSync,
  realpathSync,
  rmSync,
  writeFileSync,
} from 'node:fs';
import { tmpdir } from 'node:os';
import { dirname, join } from 'node:path';
import { test } from 'node:test';

const ROOT = new URL('../..', import.meta.url).pathname;
const SIGNED = join(ROOT, 'crates/svrngs-node/tests/fixtures/v1.11.2');
const READ = [
  'scripts/release.sh',
  'scripts/relay-answers.sh',
  'svrngs-release-key.asc',
  'crates/svrngs/src/cli.rs',
  'web/install.sh',
  'web/vercel.json',
];
const RELEASES =
  '# svrngs releases\n\n## V1.0.0, 1 January 2026\n\nThe first.\n\n## V0.9.0, 1 December 2025\n\nOlder.\n';
// What a stand-in does after it keeps its name and arguments, one line each.
const STANDS_IN = {
  lockf: 'shift 2\nexec "$@"',
  npm: '',
  curl: '',
  // Each argument apart, since the notes are lines of their own.
  gh: 'printf "%s\\0" "$@" >"$0.args"',
  vercel:
    'for a in "$@"; do [ "$prev" = --cwd ] && cwd=$a; prev=$a; done\n' +
    '[ "$1" != link ] || { mkdir -p "$cwd/.vercel"; touch "$cwd/.env.local" "$cwd/.gitignore"; }',
};

// A repository whose commit `prepare` sets version 1.0.0 and whose next,
// `after`, changes its build scripts and Cargo.toml but not the version,
// both pushed.
function fixture(t) {
  const at = realpathSync(mkdtempSync(join(tmpdir(), 'rel-')));
  t.after(() => {
    spawnSync('gpgconf', ['--kill', 'gpg-agent'], {
      env: { ...process.env, GNUPGHOME: join(at, 'gpg') },
    });
    rmSync(at, { recursive: true, force: true });
  });
  const repo = join(at, 'svrngs');
  const work = join(at, 'svrngs-release');
  const asked = join(at, 'asked');
  // Without git's own variables: a hook exports GIT_INDEX_FILE, the index
  // of the commit being made, which every git here would otherwise write.
  // Watched: under the pre-commit hook the fixture's `git add -A` rewrote
  // it, and the commit's own check found only the fixture's Cargo.toml.
  const env = {
    ...Object.fromEntries(Object.entries(process.env).filter(([k]) => !k.startsWith('GIT_'))),
    PATH: `${join(at, 'bin')}:${process.env.PATH}`,
    GNUPGHOME: join(at, 'gpg'),
  };
  const put = (path, text, mode = 0o644) => {
    mkdirSync(dirname(join(repo, path)), { recursive: true });
    writeFileSync(join(repo, path), text);
    chmodSync(join(repo, path), mode);
  };
  const git = (...args) => execFileSync('git', args, { cwd: repo, env, encoding: 'utf8' }).trim();
  const builds = (whose) => {
    put(
      'scripts/build-web.sh',
      `#!/bin/sh\nmkdir -p "$2"\necho ${whose} >"$2/index.html"\n`,
      0o755,
    );
    put(
      'scripts/build-release.sh',
      `#!/bin/sh\nmkdir -p "$2/get" && cd "$2/get"\necho ${whose} >svrngs-linux-x86_64.tar.gz\nshasum -a 256 svrngs-linux-x86_64.tar.gz >SHA256SUMS\n`,
      0o755,
    );
  };
  for (const f of READ) {
    mkdirSync(dirname(join(repo, f)), { recursive: true });
    copyFileSync(join(ROOT, f), join(repo, f));
  }
  put('.gitignore', 'target/\nnode_modules/\n');
  put('Cargo.toml', '[workspace.package]\nversion = "1.0.0"\n');
  put('RELEASES.md', RELEASES);
  builds('prepare');
  git('init', '-q', '-b', 'main');
  git('config', 'user.email', 'release@example.invalid');
  git('config', 'user.name', 'release test');
  git('add', '-A');
  git('commit', '-q', '-m', 'prepare V1.0.0');
  const prepare = git('rev-parse', 'HEAD');
  builds('after');
  put(
    'Cargo.toml',
    '[workspace.package]\nversion = "1.0.0"\n\n[workspace.dependencies]\nserde = "1"\n',
  );
  git('commit', '-q', '-am', 'a change after it');
  const after = git('rev-parse', 'HEAD');
  execFileSync('git', ['init', '-q', '--bare', join(at, 'origin.git')], { env });
  git('remote', 'add', 'origin', join(at, 'origin.git'));
  git('push', '-q', 'origin', 'main');
  mkdirSync(join(at, 'bin'));
  mkdirSync(join(at, 'gpg'), { mode: 0o700 });
  for (const [name, body] of Object.entries(STANDS_IN)) {
    writeFileSync(join(at, 'bin', name), `#!/bin/sh\necho "${name} $*" >>"${asked}"\n${body}\n`);
    chmodSync(join(at, 'bin', name), 0o755);
  }
  return {
    at,
    repo,
    work,
    git,
    put,
    prepare,
    after,
    env,
    site: join(work, 'target/release-site'),
    release(...args) {
      rmSync(asked, { force: true });
      const said = spawnSync(join(repo, 'scripts/release.sh'), args, { env, encoding: 'utf8' });
      const all = existsSync(asked) ? readFileSync(asked, 'utf8').trim().split('\n') : [];
      return { ...said, asked: (tool) => all.filter((l) => l.startsWith(`${tool} `)) };
    },
    state() {
      return readFileSync(join(work, 'target/release.state'), 'utf8');
    },
  };
}

test('build builds the commit that set the version, by its own scripts, in the kept tree under the gate lock', (t) => {
  const f = fixture(t);
  const built = f.release('build');
  assert.equal(built.status, 0, built.stderr);
  assert.equal(built.stdout, '  scripts/release.sh sign\n');
  assert.match(f.state(), new RegExp(`^TAG=v1\\.0\\.0\nCOMMIT=${f.prepare}\nSITE=${f.site}\n`));
  assert.equal(readFileSync(join(f.site, 'index.html'), 'utf8'), 'prepare\n');
  assert.ok(existsSync(join(f.site, 'vercel.json')));
  assert.equal(f.git('-C', f.work, 'rev-parse', 'HEAD'), f.prepare);
  assert.deepEqual(built.asked('npm'), ['npm ci --prefix web']);
  assert.equal(built.asked('lockf').length, 1);
  assert.match(built.asked('lockf')[0], /^lockf -k \/tmp\/svrngs-gate\.lock bash -c /);

  // The same tree again, at the commit named, with no target/debug left.
  mkdirSync(join(f.work, 'target/debug'), { recursive: true });
  const again = f.release('build', f.after);
  assert.equal(again.status, 0, again.stderr);
  assert.match(f.state(), new RegExp(`COMMIT=${f.after}\n`));
  assert.equal(readFileSync(join(f.site, 'index.html'), 'utf8'), 'after\n');
  assert.ok(!existsSync(join(f.work, 'target/debug')));
});

test('build refuses a release tree with changes of its own, and builds nothing', (t) => {
  const f = fixture(t);
  assert.equal(f.release('build').status, 0);
  writeFileSync(join(f.work, 'stray.txt'), 'mine\n');
  const said = f.release('build', f.after);
  assert.equal(said.status, 1, said.stderr);
  assert.match(said.stderr, /has changes of its own/);
  assert.match(said.stderr, new RegExp(`git -C ${f.work} status`));
  assert.deepEqual(said.asked('lockf'), []);
  assert.equal(f.git('-C', f.work, 'rev-parse', 'HEAD'), f.prepare);
});

test('sign and page refuse when nothing is built, and take nothing after them', (t) => {
  const f = fixture(t);
  for (const step of ['sign', 'page']) {
    const said = f.release(step);
    assert.equal(said.status, 1, said.stderr);
    assert.match(said.stderr, /there is no release built/);
    assert.match(said.stderr, /scripts\/release\.sh build/);
    const more = f.release(step, 'v1.0.0', f.prepare, f.site);
    assert.equal(more.status, 1, more.stderr);
    assert.match(more.stderr, /takes nothing after it/);
    assert.deepEqual([...said.asked('gh'), ...said.asked('vercel')], []);
  }
});

test('sign refuses a version already tagged', (t) => {
  const f = fixture(t);
  assert.equal(f.release('build').status, 0);
  f.git('tag', 'v1.0.0', f.prepare);
  const said = f.release('sign');
  assert.equal(said.status, 1, said.stderr);
  assert.match(said.stderr, /v1\.0\.0 is tagged already/);
  assert.deepEqual(said.asked('gh'), []);
  assert.ok(!existsSync(join(f.site, 'get/SHA256SUMS.sig')));
});

test('sign refuses a commit not on origin/main, and tags nothing', (t) => {
  const f = fixture(t);
  f.put('Cargo.toml', '[workspace.package]\nversion = "1.0.1"\n');
  f.git('commit', '-q', '-am', 'prepare V1.0.1, not pushed');
  assert.equal(f.release('build').status, 0);
  const said = f.release('sign');
  assert.equal(said.status, 1, said.stderr);
  assert.match(said.stderr, /is not on origin\/main/);
  assert.match(said.stderr, /git push origin main/);
  assert.equal(f.git('tag', '-l'), '');
  assert.deepEqual(said.asked('gh'), []);
});

test('sign signs each download, the sums and the installer, tags the commit and makes the release', (t) => {
  const f = fixture(t);
  assert.equal(f.release('build').status, 0);
  execFileSync(
    'gpg',
    [
      '--batch',
      '--passphrase',
      '',
      '--quick-generate-key',
      'svrngs releases',
      'ed25519',
      'sign',
      '1d',
    ],
    {
      env: f.env,
      stdio: 'ignore',
    },
  );
  const said = f.release('sign');
  assert.equal(said.status, 0, said.stderr);
  assert.equal(said.stdout, '  scripts/release.sh page\n');
  const get = join(f.site, 'get');
  for (const file of ['svrngs-linux-x86_64.tar.gz', 'SHA256SUMS', 'install.sh']) {
    execFileSync('gpg', ['--verify', join(get, `${file}.sig`), join(get, file)], {
      env: f.env,
      stdio: 'ignore',
    });
  }
  assert.equal(
    readFileSync(join(get, 'install.sh'), 'utf8'),
    readFileSync(join(ROOT, 'web/install.sh'), 'utf8'),
  );
  assert.match(readFileSync(join(get, 'SHA256SUMS'), 'utf8'), /^[a-f0-9]{64}  install\.sh$/m);
  execFileSync('shasum', ['-a', '256', '-c', 'SHA256SUMS'], { cwd: get, stdio: 'ignore' });
  assert.match(f.git('ls-remote', '--tags', 'origin'), /refs\/tags\/v1\.0\.0/);
  const [, ...gh] = readFileSync(join(f.at, 'bin/gh.args'), 'utf8').split('\0').slice(0, -1);
  const notes = gh.splice(gh.indexOf('--notes'), 2)[1];
  assert.deepEqual(gh.slice(0, 6), [
    'create',
    'v1.0.0',
    '--verify-tag',
    '--latest',
    '--title',
    'V1.0.0',
  ]);
  assert.match(notes, /The first\./);
  assert.match(notes, /Signed by the svrngs release key, [0-9A-F]{40}:/);
  assert.doesNotMatch(notes, /Older\./);
  const files = [
    'svrngs-linux-x86_64.tar.gz',
    'svrngs-linux-x86_64.tar.gz.sig',
    'SHA256SUMS',
    'SHA256SUMS.sig',
  ];
  files.push('install.sh', 'install.sh.sig', 'svrngs-release-key.asc');
  assert.deepEqual(
    gh.slice(6),
    files.map((n) => join(get, n)),
  );
});

// The fixture built, its sums replaced by `sums` and, if given, `sig`.
function built(t, sums, sig) {
  const f = fixture(t);
  assert.equal(f.release('build').status, 0);
  writeFileSync(join(f.site, 'get/SHA256SUMS'), sums);
  rmSync(join(f.site, 'get/SHA256SUMS.sig'), { force: true });
  if (sig) writeFileSync(join(f.site, 'get/SHA256SUMS.sig'), sig);
  return f;
}

const SUMS = readFileSync(join(SIGNED, 'SHA256SUMS'));
const SIG = readFileSync(join(SIGNED, 'SHA256SUMS.sig'));

test('page refuses an unsigned site, and deploys nothing', (t) => {
  const f = built(t, SUMS);
  const said = f.release('page');
  assert.equal(said.status, 1, said.stderr);
  assert.match(said.stderr, /has no valid signature by the release key/);
  assert.match(said.stderr, /scripts\/release\.sh sign/);
  assert.deepEqual(said.asked('vercel'), []);
});

test('page refuses sums changed after they were signed', (t) => {
  const f = built(t, Buffer.concat([SUMS, Buffer.from('\n')]), SIG);
  const said = f.release('page');
  assert.equal(said.status, 1, said.stderr);
  assert.deepEqual(said.asked('vercel'), []);
});

test('page deploys a site the release key signed, and leaves none of the Vercel CLI’s files in it', (t) => {
  const f = built(t, SUMS, SIG);
  const said = f.release('page');
  assert.equal(said.status, 0, said.stderr);
  assert.deepEqual(said.asked('vercel'), [
    `vercel link --yes --project svrngs-page --scope svrnmesh --cwd ${f.site}`,
    `vercel deploy --prod --yes --scope svrnmesh --cwd ${f.site}`,
  ]);
  for (const left of ['.vercel', '.env.local', '.gitignore'])
    assert.ok(!existsSync(join(f.site, left)), left);
});

test('a build never writes the index a hook names in GIT_INDEX_FILE', (t) => {
  const outer = process.env.GIT_INDEX_FILE;
  const index = join(realpathSync(mkdtempSync(join(tmpdir(), 'rel-index-'))), 'index');
  t.after(() => rmSync(dirname(index), { recursive: true, force: true }));
  writeFileSync(index, 'the commit being made\n');
  process.env.GIT_INDEX_FILE = index;
  try {
    assert.equal(fixture(t).release('build').status, 0);
  } finally {
    if (outer === undefined) delete process.env.GIT_INDEX_FILE;
    else process.env.GIT_INDEX_FILE = outer;
  }
  assert.equal(readFileSync(index, 'utf8'), 'the commit being made\n');
});

test('no step named refuses, naming the three', (t) => {
  const f = fixture(t);
  const said = f.release();
  assert.equal(said.status, 2, said.stderr);
  for (const step of ['build', 'sign', 'page'])
    assert.match(said.stderr, new RegExp(`  scripts/release\\.sh ${step} `));
  assert.deepEqual(said.asked('vercel'), []);
});
