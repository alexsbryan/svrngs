"""Run the saved availability field proofs, one at a time, on one candidate."""
import argparse
from contextlib import closing
from enum import Enum
import fcntl
import hashlib
import json
import os
from pathlib import Path
import re
import shlex
import shutil
import socket
import subprocess as sp
import tempfile

HERE = Path(__file__).resolve().parent
ROOT = HERE.parents[2]
RUNS = Path('demo/runs')
KEEPER = RUNS / 'keeper/2026-11-23'
SPIKE = Path('/tmp/keeper-demo')
MODEL = 'qwen2.5-0.5b.gguf'
ARCHIVE = 'svrngs-linux-x86_64.tar.gz'
PAIR = ('svrngs', 'svrngs-keeper')
SSH = ['ssh', '-o', 'BatchMode=yes', '-o', 'ConnectTimeout=10',
       '-o', 'ServerAliveInterval=10', '-o', 'ServerAliveCountMax=3', 'beefymac']
LINUX_VM = (
    ('fedora-package', RUNS / 'available/2026-11-23/run.py', 2270, True),
    ('linux-native', Path(f'{KEEPER}-run-linux/run.py'), 2223, False),
    *((f'linux-container-{name}', Path(f'{KEEPER}-run-containers/{file}.py'), 2224, False)
      for name, file in (('plain', 'run'), ('recover', 'recover'), ('actual', 'actual'), ('stop-gap', 'stop_gap'))),
    ('linux-stdio', Path(f'{KEEPER}-run-stdio/cli.py'), 2223, False),
    ('linux-ship', Path(f'{KEEPER}-ship/run.py'), 2269, True),
)
MAC = ('mac-keeper-tests', 'mac-node-tests', 'mac-run',
       *(f'mac-proof-{case}' for case in ('lineage', 'ownership', 'managed')),
       'mac-renew', *(f'mac-plist-{case}' for case in
                     ('formatted', 'binary', 'absent', 'wrong-root', 'malformed', 'wrong-type')))


class Verdict(str, Enum):
    PASSED = 'passed'
    FAILED = 'failed'
    UNJUDGED = 'could-not-judge'
    MISSING = 'never-ran'


def git(*args):
    return sp.check_output(['git', *args], text=True).strip()


def digest(file):
    with file.open('rb') as stream:
        return hashlib.file_digest(stream, 'sha256').hexdigest()


def command(args, log, **kwargs):
    print(f'Running {log.name}: {shlex.join(map(str, args))}', flush=True)
    with log.open('wb') as out:
        try:
            return sp.run(args, stdout=out, stderr=sp.STDOUT, **kwargs).returncode
        except OSError as error:
            out.write(f'{error}\n'.encode())
            return 127


def identifies(version, short):
    return version.strip().endswith(f' ({short})')


def judge(code, text):
    cases = []
    for line in text.splitlines():
        if line.startswith('{'):
            try:
                case = json.loads(line)
            except json.JSONDecodeError:
                continue
            if case.get('event') == 'case':
                cases.append(case['verdict'])
    if Verdict.FAILED in cases:
        return Verdict.FAILED
    if Verdict.MISSING in cases or code == 127 or re.search(r'(scratch VM|guest) did not answer ssh', text, re.I):
        return Verdict.MISSING
    if code:
        return Verdict.FAILED
    if Verdict.UNJUDGED in cases:
        return Verdict.UNJUDGED
    return Verdict.PASSED


class Run:
    def __init__(self, logs, candidate):
        self.logs, self.candidate = logs, candidate
        self.verdicts = []

    def say(self, case, verdict, build, detail):
        self.verdicts.append(verdict)
        row = f'{verdict.value} {case}: commit={self.candidate} build={build}; {detail}'
        print(row, flush=True)
        with (self.logs / 'summary.txt').open('a') as out:
            print(row, file=out)

    def exit_code(self):
        return int(any(v in (Verdict.FAILED, Verdict.MISSING) for v in self.verdicts))

    def clean(self):
        return git('rev-parse', 'HEAD') == self.candidate and not git('status', '--porcelain')

    def step(self, case, build, args, **kwargs):
        log = self.logs / f'{case}.log'
        if not self.clean():
            self.say(case, Verdict.FAILED, build, 'checkout differs from the candidate')
            return
        code = command(args, log, **kwargs)
        text = log.read_text(errors='replace')
        verdict = judge(code, text)
        if not self.clean():
            verdict = Verdict.FAILED
        if case == 'linux-page-stdio' and verdict == Verdict.PASSED and not (
            'test result: ok. 1 passed;' in text and 'Page Run and controls: unanswered PTY' in text
        ):
            verdict = Verdict.MISSING
        self.say(case, verdict, build, f'exit={code}; {log.name}')


def linux(run, work, short):
    site, bin_dir = work / 'site', work / 'bin'
    log = run.logs / 'linux-build.log'
    archive = site / 'get' / ARCHIVE
    code = command(['toolbox', 'run', '-c', 'sovereign-vulkan', 'bash', '-c',
                    f'cd {shlex.quote(str(ROOT))} && scripts/build-release.sh --only linux-x86_64 --out {shlex.quote(str(site))}'], log)
    build = 'linux-x86_64 release'
    if code or not run.clean():
        run.say('linux-build', Verdict.FAILED, build, f'exit={code}; {log.name}')
        build = None
    else:
        bin_dir.mkdir()
        try:
            sp.run(['tar', '-xzf', archive, '-C', bin_dir, *PAIR], check=True)
            version = sp.check_output([bin_dir / 'svrngs', 'version'], text=True)
            if not identifies(version, short):
                raise ValueError(f'candidate mismatch: {version.strip()} expected {short}')
            build = f'{ARCHIVE}@{digest(archive)}'
            run.say('linux-build', Verdict.PASSED, build,
                    f'{version.strip()}; {log.name}; ' + ' '.join(f'{p}={digest(bin_dir / p)}' for p in PAIR))
        except (OSError, sp.CalledProcessError, ValueError) as error:
            run.say('linux-build', Verdict.FAILED, build, str(error))
            build = None

    vm_inputs = [SPIKE / p for p in ('prepared.qcow2', 'key', 'seed.iso', MODEL)]
    for name, driver, port, packaged in LINUX_VM:
        missing = next((p for p in [Path('/dev/kvm'), *vm_inputs] if not os.access(p, os.R_OK)), None)
        if not build or missing or not shutil.which('qemu-system-x86_64'):
            run.say(name, Verdict.MISSING, build or 'none', f'VM input unavailable: {missing or "candidate build or QEMU"}')
            continue
        with closing(socket.socket()) as probe:
            if probe.connect_ex(('127.0.0.1', port)) == 0:
                run.say(name, Verdict.MISSING, build, f'VM SSH port {port} already occupied')
                continue
        scratch = work / name
        args = ['python3', '-Bu', driver] + ([site] if packaged else [])
        env = dict(os.environ, W=str(scratch), BIN=str(bin_dir))
        run.step(name, build, args, env=env)
        if scratch.exists():
            shutil.rmtree(scratch)
            print(f'Cleanup {name}: removed VM overlay, guest units/state and local scratch {scratch}', flush=True)
    run.step('linux-page-stdio', 'svrngs-gate/debug run-page-gate browser and keeper',
             ['bash', f'{KEEPER}-run-page/gate.sh', '--tty'],
             env=dict(os.environ, SVRNGS_COMMIT=short, SOURCE_DATE_EPOCH=git('log', '-1', '--format=%ct')))


def mac_verdicts(code, text, candidate):
    seen = re.findall(r'^release-candidate (\S+)$', text, re.M)
    results = {}
    for name, status, build in re.findall(r'^release-case (\S+) (\d+) (\S+)$', text, re.M):
        verdict = {0: Verdict.PASSED, 125: Verdict.MISSING}.get(int(status), Verdict.FAILED)
        if name in results or seen != [candidate]:
            verdict = Verdict.FAILED
        results[name] = (verdict, build)
    for name in (*MAC, 'mac-cleanup'):
        yield name, *results.get(name, (Verdict.MISSING, 'none'))
    if (seen and seen != [candidate]) or set(results) - {*MAC, 'mac-cleanup'}:
        yield 'mac-candidate', Verdict.FAILED, 'mismatch'
    elif code not in (0, 125, 255):
        yield 'mac-session', Verdict.FAILED, 'beefymac'
    elif code in (125, 255):
        yield 'mac-session', Verdict.MISSING, 'beefymac'


def mac(run, work, short):
    log = run.logs / 'beefymac.log'
    if not run.clean():
        run.say('mac-candidate', Verdict.FAILED, 'none', 'checkout differs from the candidate')
        return
    if not (SPIKE / MODEL).is_file():
        for name in MAC:
            run.say(name, Verdict.MISSING, 'none', f'model unavailable at {SPIKE / MODEL}')
        return
    src = work / 'source.tar'
    sp.run(['git', 'archive', '--format=tar', '-o', src, run.candidate], check=True)
    sqlite = git('rev-parse', f'{run.candidate}:vendor/cr-sqlite')
    setup = shlex.join(['/usr/bin/lockf', '-k', '/tmp/svrngs-gate.lock',
                        '/bin/bash', '-c', (HERE / 'mac.sh').read_text(),
                        'release-tier', run.candidate, short, git('log', '-1', '--format=%ct'), sqlite])
    print('Staging candidate archive and model on beefymac', flush=True)
    with sp.Popen(['tar', '-cf', '-', '-C', work, src.name, '-C', SPIKE, MODEL], stdout=sp.PIPE) as bundle:
        with log.open('wb') as out:
            code = sp.run([*SSH, setup], stdin=bundle.stdout, stdout=out, stderr=sp.STDOUT).returncode
        bundle.stdout.close()
        sent = bundle.wait()
    for name, verdict, build in mac_verdicts(code, log.read_text(errors='replace'), run.candidate):
        run.say(name, verdict, build, f'SSH exit={code}; transfer exit={sent}; {log.name}')
    if sent and not code:
        run.say('mac-transfer', Verdict.FAILED, 'none', f'tar exit={sent}; {log.name}')


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('logs', type=Path, help='new evidence directory outside the checkout')
    args = parser.parse_args()
    os.chdir(ROOT)
    logs = args.logs.resolve()
    if logs.is_relative_to(ROOT) or git('status', '--porcelain'):
        parser.error('need a clean checkout and an evidence directory outside it')
    logs.mkdir(mode=0o700)
    candidate, short = git('rev-parse', 'HEAD'), git('rev-parse', '--short', 'HEAD')
    run = Run(logs, candidate)
    with open('/tmp/svrngs-keeper-release.lock', 'a') as lock:
        fcntl.flock(lock, fcntl.LOCK_EX)
        with tempfile.TemporaryDirectory(prefix='keeper-release-', dir=logs.parent) as tmp:
            linux(run, Path(tmp), short)
            mac(run, Path(tmp), short)
        print(f'Cleanup: removed local build staging and all replay folders under {tmp}', flush=True)
    if not run.clean():
        run.say('candidate', Verdict.FAILED, 'mixed', 'checkout changed during the run')
    print(f'Release tier exit={run.exit_code()}; {logs / "summary.txt"}', flush=True)
    return run.exit_code()


if __name__ == '__main__':
    raise SystemExit(main())
