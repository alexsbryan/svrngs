"""Two scratch nodes and a launch agent on beefymac; run with -u under SSH."""
import hashlib
import http.client
import json
import os
from pathlib import Path
import plistlib
import pty
import re
import select
import socket
import subprocess as sp
import sys
import tempfile
import time
from urllib.parse import urlsplit

scratch = Path.home() / 'svrngs-keeper-scratch'
svr, keeper = (scratch / 'repo/target/debug' / n for n in ('svrngs', 'svrngs-keeper'))
name = 'test-mac-renew'
label = 'svrngs-keeper-' + name
target = f'gui/{os.getuid()}/{label}'
door_label = label + '-door'
domain = f'gui/{os.getuid()}'
door_target = f'{domain}/{door_label}'
cases = ('changed', 'holder', 'taker', 'held', 'withdrawn')
case = sys.argv[1]
assert case in ('all', *cases)
plant = sys.argv[2] if len(sys.argv) > 2 else ''
assert not plant or plant == case in cases


def until(test):
    for _ in range(100):
        if value := test():
            return value
        time.sleep(.2)
    raise AssertionError('condition did not become true')


with tempfile.TemporaryDirectory(dir=scratch) as tmp:
    work = Path(tmp)
    kids = []
    race = work / 'race'

    def env(who):
        home = work / who
        home.mkdir(exist_ok=True)
        return dict(os.environ, HOME=str(home), SVRNGS_DIR=str(home / 'node'),
                    XDG_DATA_HOME=str(home / 'data'), TOKIO_WORKER_THREADS='4',
                    RUST_LOG='error,door=debug,job=debug,cli=debug,plan=debug', SVRNGS_TEST_CONTROL=plant,
                    SVRNGS_TEST_RACE=str(race), SVRNGS_TEST_HANDOFF=str(work / 'fds'))

    def call(who, *args, check=True, answer=None):
        out = sp.run(args, cwd=work, env=env(who), text=True, input=answer,
                     capture_output=True, timeout=45)
        if check:
            assert out.returncode == 0, out.stdout + out.stderr
        return out

    def start(who, *args):
        with (work / f'{who}.log').open('a') as log:
            child = sp.Popen(args, cwd=work, env=env(who), stdout=log, stderr=log)
        kids.append(child)
        return child

    def confirm(*args):
        master, slave = pty.openpty()
        child = sp.Popen(args, cwd=work, env=env('sam'), stdin=slave, stdout=slave, stderr=slave)
        os.close(slave)
        output, replies = b'', 0
        try:
            while child.poll() is None:
                assert select.select([master], [], [], 20)[0], output.decode()
                try:
                    more = os.read(master, 4096)
                except OSError:
                    break
                if not more:
                    break
                output += more
                if output.count(b'Type y, then press Enter:') > replies:
                    os.write(master, b'y\n')
                    replies += 1
                if output.endswith(b'(Enter alone for none): '):
                    os.write(master, b'\n')
            assert child.wait(5) == 0, output.decode()
            return output.decode()
        finally:
            if child.poll() is None:
                child.terminate()
                child.wait(10)
            os.close(master)

    def show(job=target):
        return call('sam', '/bin/launchctl', 'print', job, check=False)

    def pid():
        found = re.search(r'^\tpid = (\d+)$', show().stdout, re.M)
        return int(found[1]) if found else None

    def ask(who, route, *args, check=True):
        out = call(who, svr, 'ask', route, *args, check=check)
        return json.loads(out.stdout) if out.returncode == 0 else None

    def request():
        to = urlsplit(origin)
        conn = http.client.HTTPConnection('127.0.0.1', to.port, timeout=12)
        try:
            conn.request('GET', '/same-link', headers={'Host': to.netloc})
            reply = conn.getresponse()
            return reply.status, reply.read().decode()
        finally:
            conn.close()

    with socket.socket() as probe:
        probe.bind(('127.0.0.1', 0))
        port = str(probe.getsockname()[1])

    def listening():
        try:
            with socket.create_connection(('127.0.0.1', int(port)), timeout=.1):
                return True
        except OSError:
            return False
    # Child holds the listener and takes the connection, not launchd's main PID.
    server = work / 'server.py'
    server.write_text('''
import socket, sys
from pathlib import Path

port, marker, data, route, role = sys.argv[1:]
mark = Path(marker)
def serve(conn):
    with conn:
        got = conn.recv(65536)
        if got:
            with mark.open('ab') as log: log.write(got)
            body = Path(data).read_bytes()
            conn.sendall(b'HTTP/1.1 200 OK\\r\\nContent-Length: ' + str(len(body)).encode()
                         + b'\\r\\nConnection: close\\r\\n\\r\\n' + body)
handoff = Path(route)
if role == 'taker':
    with socket.socket(socket.AF_UNIX) as channel:
        channel.bind(route)
        channel.listen()
        mark.touch()
        while True:
            link, _ = channel.accept()
            with link:
                _, fds, _, _ = socket.recv_fds(link, 1, 1)
                accepted = socket.socket(fileno=fds[0])
                link.sendall(b'1')
                serve(accepted)
else:
    s = socket.create_server(('0.0.0.0' if role == 'holder' else '127.0.0.1', int(port)), reuse_port=True)
    mark.touch()
    while True:
        conn, _ = s.accept()
        if role == 'app' and handoff.exists():
            with socket.socket(socket.AF_UNIX) as channel:
                channel.connect(route)
                with conn:
                    socket.send_fds(channel, [b'x'], [conn.fileno()])
                assert channel.recv(1) == b'1'
            handoff.with_suffix('.ready').touch()
        else:
            serve(conn)
''')
    wrapper = work / 'wrapper.sh'
    wrapper.write_text('/usr/bin/python3 "$@" &\nwait\n')
    mark, foreign = work / 'good.bytes', work / 'foreign.bytes'
    data, route = work / 'data', work / 'fds'
    data.write_text('the saved data')
    expected = (200, data.read_text())
    active = work / 'sam/Library/LaunchAgents' / f'{label}.plist'
    door_plist = active.with_name(f'{door_label}.plist')
    lend = work / 'sam/node/lends' / f'{name}.json'
    assert show().returncode == show(door_target).returncode == 113
    try:
        active.parent.mkdir(parents=True)
        registration = plistlib.dumps(dict(Label=door_label,
            ProgramArguments=[str(svr)], RunAtLoad=True,
            EnvironmentVariables={k: v for k, v in env('sam').items() if k in (
                'HOME', 'SVRNGS_DIR', 'XDG_DATA_HOME', 'TOKIO_WORKER_THREADS', 'RUST_LOG',
                'SVRNGS_TEST_CONTROL', 'SVRNGS_TEST_RACE', 'SVRNGS_TEST_HANDOFF')},
            StandardOutPath=str(work / 'sam.log'), StandardErrorPath=str(work / 'sam.log')))
        # Match web/install.sh's SVRNGS_DIR spelling, which CLI kept_over reads.
        door_plist.write_bytes(re.sub(rb'(<key>SVRNGS_DIR</key>)\s+(<string>)',
                                     rb'\1\2', registration))
        call('sam', '/bin/launchctl', 'bootstrap', domain, door_plist)
        start('alex', svr)
        for who in ('sam', 'alex'):
            until(lambda: ask(who, '/v1/groups', check=False))
        out = call('sam', svr, 'new', 'Scratch', '--as', 'Sam')
        link = re.search(r'svrngs://join/\S+', out.stdout + out.stderr)[0]
        call('alex', svr, 'join', link, '--as', 'Alex', '--no-run')
        group = ask('alex', '/v1/groups')['groups'][0]['id']
        call('sam', svr, 'run', name, '--port', port, '--folder', work, '--',
              '/bin/sh', wrapper, server, port, mark, data, route, 'app')
        first = pid()
        assert first and active.is_file()
        share = (svr, 'share', port, '--program', 'mac-fixture', '--name', name, '--group', 'Scratch')
        shared = confirm(*share)
        print(shared)
        assert 'closes until you approve' not in shared, 'Share gave conflicting restart guidance'
        before = json.loads(lend.read_text())
        original = active.read_bytes()
        got = before.get('keep')
        assert got and got['unit'] == label, 'first Share must grant the verified login service'
        assert got['files'] == hashlib.sha256(original).hexdigest()
        for job in (door_target, target):
            print('\n'.join(l.strip() for l in show(job).stdout.splitlines()
                            if l.startswith(('gui/', '\tpath = ', '\tproperties = ', '\tpid = '))))
        print(f'Share: label={got["unit"]}, plist hash matches={True}')

        def listed():
            rows = ask('alex', f'/v1/lends?namespace={group}')['lent']
            return next((row for row in rows if row['name'] == name), None)

        row = until(listed)
        origin = row['origin']
        ask('alex', '/v1/play', json.dumps({'on': {'key': row['key'], 'name': name}}))
        assert request() == expected

        def restart():
            was = pid()
            call('sam', keeper, 'restart', name)
            new = until(lambda: pid() if pid() != was else None)
            until(listening)
            return new

        successor = restart()
        reply = request()
        after = json.loads(lend.read_text())
        print(f'Restart: pid {first} -> {successor}; same address {reply[0]}; approval changed={after["approved"] != before["approved"]}')
        assert reply == expected and after['approved'] != before['approved']
        assert {k: v for k, v in after.items() if k != 'approved'} == {k: v for k, v in before.items() if k != 'approved'}

        def count():
            return tuple(f.stat().st_size if f.exists() else 0 for f in (mark, foreign))

        for control in cases if case == 'all' else (case,):
            restart()
            stray = None
            if control == 'changed':
                active.write_bytes(original + b'\n')
            if control in ('holder', 'taker'):
                foreign.unlink(missing_ok=True)
                stray = start('foreign', '/usr/bin/python3', server, port, foreign,
                              data, route, control)
                until(foreign.exists)
            if control == 'held':
                doc = json.loads(lend.read_text())
                doc['approved']['held'] = True
                lend.write_text(json.dumps(doc))
            saved = lend.read_bytes()
            received = count()
            if control == 'withdrawn':
                from concurrent.futures import ThreadPoolExecutor
                race.touch()
                with ThreadPoolExecutor(1) as pending:
                    reply = pending.submit(request)
                    until(race.with_suffix('.judged').exists)
                    call('sam', svr, 'unshare', name)
                    race.unlink()
                    try:
                        result = reply.result(timeout=15)
                    except http.client.RemoteDisconnected:
                        # Alex's Play address can close before the door answers.
                        result = ('disconnected', '')
                verdict = race.with_suffix('.decided')
                until(lambda: verdict.exists() and verdict.stat().st_size)
                decision = verdict.read_text()
                print(f'Withdrawal at door: {decision}')
                assert decision == ('carried' if plant else 'refused')
                if plant:
                    until(lambda: count()[0] > received[0])
                same = not lend.exists()
            else:
                result = request()
                same = lend.read_bytes() == saved
            got = tuple(a - b for a, b in zip(count(), received))
            status = 'disconnected' if result[0] == 'disconnected' else f'HTTP {result[0]}'
            print(f'{control}: {status}; bytes (app,foreign)={got}; state unchanged={same}')
            if plant:
                assert (result == expected or result[0] == 'disconnected') and sum(got) > 0, 'plant did not exercise its targeted refusal'
                print('RED: forwarding occurred despite the violated invariant')
                raise SystemExit(5)
            assert result[0] in (502, 'disconnected') and got == (0, 0) and same, 'refusal did not hold'
            if result[0] == 502:
                assert ('changed' if control == 'changed' else 'didn’t choose') in result[1], result
            if stray:
                stray.terminate()
                stray.wait(10)
                route.unlink(missing_ok=True)
                route.with_suffix('.ready').unlink(missing_ok=True)
            if control != 'withdrawn':
                active.write_bytes(original)
                lend.write_bytes(saved if control != 'held' else json.dumps(after).encode())
                assert request() == expected, 'control recovery'
        if case == 'all':
            print(confirm(*share))
            print(confirm(svr, 'approve', name))
            assert json.loads(lend.read_text()).get('keep') is None, 'Mac Approve granted renewal'
            print('Approve: instance-only')
    except Exception:
        for log in work.glob('*.log'):
            print(log.name, log.read_text()[-7000:])
        raise
    finally:
        for kid in kids:
            if kid.poll() is None:
                kid.terminate()
            kid.wait(10)
        for job, path in ((target, active), (door_target, door_plist)):
            if (registration := show(job)).returncode == 0:
                assert str(path) in registration.stdout, 'not our registration'
                call('sam', '/bin/launchctl', 'bootout', job)
                until(lambda: show(job).returncode == 113)
            print(f'Cleanup {job}: launchctl print exits {show(job).returncode}')
