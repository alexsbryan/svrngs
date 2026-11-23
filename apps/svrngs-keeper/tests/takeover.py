"""Run on the Linux host: python3 apps/svrngs-keeper/tests/takeover.py target/debug/svrngs-keeper."""

import errno
import json
import os
from pathlib import Path
import signal
import secrets
import socket
import stat
import subprocess as sp
import sys
import tempfile
import time
import urllib.request


BIN = Path(sys.argv[1]).resolve()
PYTHON = Path(sys.executable).resolve()
STAGES = ('prepared', 'staged', 'stopping', 'stopped', 'started', 'answered', 'enabled', 'verified', 'rollback', 'restored')
PRE_STOP = ('prepared', 'staged', 'stopping')


def ctl(*args, **kw):
    return sp.run(['systemctl', '--user', *args], text=True, capture_output=True, **kw)


def until(read, timeout=15):
    end = time.monotonic() + timeout
    while time.monotonic() < end:
        try:
            if result := read():
                return result
        except OSError:
            pass
        time.sleep(0.05)
    raise AssertionError(f'timed out: {read.__name__}')


def free_port():
    # Don't release a port back into the kernel's ephemeral allocator.
    for _ in range(20):
        port = 20000 + secrets.randbelow(10000)
        with socket.socket() as sock:
            try:
                sock.bind(('127.0.0.1', port))
                return port
            except OSError as why:
                if why.errno != errno.EADDRINUSE:
                    raise
    raise AssertionError('no scratch port found')


def case(stage):
    with tempfile.TemporaryDirectory(prefix='svrngs-keeper-test-') as tmp:
        root = Path(tmp)
        name = f'test-takeover-{root.name.rsplit("-", 1)[-1].replace("_", "a")}'
        unit = f'svrngs-keeper-{name}.service'
        scope = f'svrngs-keeper-{name}-rollback.scope'
        port = str(free_port())
        data = root / 'data'
        journal = data / 'svrngs-keeper' / name / 'takeover.json'
        env = dict(os.environ, XDG_DATA_HOME=str(data), RUST_LOG='svrngs_keeper=debug')
        # Don't export the original's terminal setting to the keeper or systemd.
        program = root / 'serve.py'
        program.write_text('''import os, sys
from http.server import BaseHTTPRequestHandler, HTTPServer
if sys.argv[2] in ('rollback', 'restored') and os.getenv('KEEPER_T1') != 'original':
    sys.exit('the copy lacks its terminal setting')
class Handler(BaseHTTPRequestHandler):
    def do_GET(self):
        self.send_response(200)
        self.end_headers()
        self.wfile.write(f"{os.getpid()}:{os.getenv('KEEPER_T1', 'kept')}".encode())
HTTPServer(('127.0.0.1', int(sys.argv[1])), Handler).serve_forever()
''')
        children = []

        def command(*args, check=True):
            out = sp.run([BIN, *args], env=env, cwd=root, capture_output=True, text=True, timeout=70)
            assert not check or out.returncode == 0, (args, out.returncode, out.stderr, out.stdout)
            return out

        def response():
            with urllib.request.urlopen(f'http://127.0.0.1:{port}/', timeout=0.5) as http:
                pid, marker = http.read().decode().split(':')
                return int(pid), marker

        def trial(marker):
            child = sp.Popen([PYTHON, program, port, stage], start_new_session=True,
                             env=dict(env, KEEPER_T1=marker), cwd=root,
                             stdout=sp.DEVNULL, stderr=sp.DEVNULL)
            children.append(child)
            assert until(response) == (child.pid, marker)
            return child

        try:
            before = trial('original')
            facts = json.loads(command('facts', port, '--name', name).stdout)
            assert facts['kind'] == 'copyable' and facts['port'] == int(port), facts
            assert 'KEEPER_T1' in facts['omitted_settings'] and 'original' not in facts.values(), facts
            assert until(response) == (before.pid, 'original') and not journal.exists()
            take = ['keep', port, '--name', name, '--yes', '--token', facts['token']]
            if stage == 'confirmation':
                assert command('keep', port, '--name', name, '--yes', check=False).returncode != 0
                before.terminate()
                before.wait(timeout=5)
                other = trial('replacement')
                assert json.loads(command('facts', port, '--name', name).stdout)['token'] != facts['token']
                assert command(*take, check=False).returncode != 0, 'stale confirmation was accepted'
                assert until(response) == (other.pid, 'replacement') and not journal.exists()
                print('passed  stale or missing token cannot stop a replacement')
                return

            point = 'stopped' if stage in ('collision', 'new-boot', 'boot-port-taken') else stage
            keeper = sp.Popen([BIN, *take], env=dict(env, SVRNGS_KEEPER_TEST_PAUSE=point), cwd=root,
                              stdout=sp.DEVNULL, stderr=sp.DEVNULL)
            children.append(keeper)

            def paused():
                assert keeper.poll() is None, f'keeper exited before {point}: {keeper.returncode}'
                return any(line.startswith('State:\tT') for line in Path(f'/proc/{keeper.pid}/status').read_text().splitlines())

            until(paused)
            assert stat.S_IMODE(journal.stat().st_mode) == 0o600, 'journal exposes the original environment'
            if point not in ('enabled', 'verified'):
                assert ctl('is-enabled', unit).stdout.strip() != 'enabled', 'enabled before verification'
            if stage == 'staged':
                locked = command('facts', port, '--name', name + '-other', check=False)
                assert locked.returncode != 0 and 'keeper lock port-' in locked.stderr, locked
                assert until(response) == (before.pid, 'original')
            keeper.kill()
            assert keeper.wait(timeout=5) == -signal.SIGKILL
            if stage in ('new-boot', 'boot-port-taken'):
                record = json.loads(journal.read_text())
                record['boot'] = 'test-previous-boot'
                journal.write_text(json.dumps(record))
            if stage == 'collision':
                other = trial('replacement')
                refused = command(*take, check=False)
                assert refused.returncode != 0 and 'another process' in refused.stderr, refused
                assert until(response) == (other.pid, 'replacement') and journal.exists()
                other.terminate()
                other.wait(timeout=5)
            if stage == 'boot-port-taken':
                other = trial('replacement')
                listed = json.loads(command('list', '--json').stdout)
                assert listed == [dict(kind='interrupted', name=name, port=int(port), stage='stopped')], listed
                assert until(response) == (other.pid, 'replacement') and not journal.exists()
                assert ctl('is-enabled', unit).stdout.strip() == 'disabled'
                other.terminate()
                other.wait(timeout=5)
                command('start', name)
                assert until(response)[1] == 'kept'
                print('passed  boot-port-taken: recorded interrupted, left replacement alone, explicit Start worked')
                return

            listing = command('list', '--json')
            listed = json.loads(listing.stdout)
            assert not journal.exists(), 'recovery left the transaction pending'
            now, marker = until(response)
            if stage in PRE_STOP:
                assert (now, marker) == (before.pid, 'original') and listed == [], listed
                assert ctl('is-enabled', unit).stdout.strip() == 'not-found'
            elif stage in ('stopped', 'collision', 'rollback', 'restored'):
                assert now != before.pid and marker == 'original', (now, marker)
                assert scope in Path(f'/proc/{now}/cgroup').read_text(), 'rollback is not in its own scope'
            else:
                assert now != before.pid and marker == 'kept', (now, marker, listing.stderr,
                    sp.run(['journalctl', '--user', '-u', unit, '-n', '12'], text=True, capture_output=True).stdout)
                assert unit in Path(f'/proc/{now}/cgroup').read_text()
                assert ctl('is-enabled', unit).stdout.strip() == 'enabled'
                assert os.getpriority(os.PRIO_PROCESS, now) == 10
                assert listed[0]['kind'] == 'kept' and listed[0]['port'] == int(port), listed
            if stage not in PRE_STOP:
                assert command(*take, check=False).returncode != 0, 'stale retry succeeded'
                assert until(response) == (now, marker), 'stale retry stopped a second process'
            if stage == 'verified':
                command('restart', name)
                assert until(response)[0] != now, 'restart did not make a new process'
                command('stop', name)
                command('restart', name)
                assert ctl('is-active', unit).stdout.strip() == 'inactive', 'restart undid stop'
                command('start', name)
                until(response)
                for _ in range(4):
                    if ctl('restart', unit).returncode:
                        break
                assert ctl('show', unit, '-p', 'Result').stdout.strip() == 'Result=start-limit-hit'
                command('start', name)
                assert until(response)[1] == 'kept'
            print(f'passed  {stage}: recovered {marker}; replacement was not stopped')
        finally:
            ctl('disable', '--now', unit)
            ctl('stop', scope)
            config = Path(os.environ.get('XDG_CONFIG_HOME') or Path.home() / '.config')
            (config / 'systemd/user' / unit).unlink(missing_ok=True)
            ctl('daemon-reload')
            ctl('reset-failed', unit, scope)
            for child in children:
                if child.poll() is None:
                    child.kill()
                child.wait(timeout=5)


if __name__ == '__main__':
    failures = []
    for item in (sys.argv[2:] or ['confirmation', *STAGES, 'collision', 'new-boot', 'boot-port-taken']):
        try:
            case(item)
        except AssertionError as why:
            failures.append(item)
            print(f'failed  {item}: {why}')
    assert not failures, failures
