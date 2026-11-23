"""Two scratch nodes and a launch agent on beefymac; run with -u under SSH."""
import hashlib
import http.client
import json
import os
from pathlib import Path
import pty
import re
import select
import socket
import subprocess as sp
import tempfile
import time
from urllib.parse import urlsplit

scratch = Path.home() / 'svrngs-keeper-scratch'
svr, keeper = (scratch / 'repo/target/debug' / n for n in ('svrngs', 'svrngs-keeper'))
name = 'test-mac-renew'
label = 'svrngs-keeper-' + name
target = f'gui/{os.getuid()}/{label}'


def until(test):
    for _ in range(100):
        if value := test():
            return value
        time.sleep(.2)
    raise AssertionError('condition did not become true')


with tempfile.TemporaryDirectory(dir=scratch) as tmp:
    work = Path(tmp)
    kids = []

    def env(who):
        home = work / who
        home.mkdir(exist_ok=True)
        return dict(os.environ, HOME=str(home), SVRNGS_DIR=str(home / 'node'),
                    XDG_DATA_HOME=str(home / 'data'), TOKIO_WORKER_THREADS='4', RUST_LOG='error,door=debug,job=debug')

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

    def show():
        return call('sam', '/bin/launchctl', 'print', target, check=False)

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
import os, socket, sys
from pathlib import Path
s = socket.create_server(('127.0.0.1', int(sys.argv[1])))
mark = Path(sys.argv[2]); mark.touch()
while True:
    conn, _ = s.accept()
    with conn:
        got = conn.recv(65536)
        if got:
            with mark.open('ab') as log: log.write(got)
            conn.sendall(b'HTTP/1.1 200 OK\\r\\nContent-Length: 6\\r\\nConnection: close\\r\\n\\r\\nkeeper')
''')
    wrapper = work / 'wrapper.sh'
    wrapper.write_text('/usr/bin/python3 "$@" &\nwait\n')
    mark, foreign = work / 'good.bytes', work / 'foreign.bytes'
    active = work / 'sam/Library/LaunchAgents' / f'{label}.plist'
    lend = work / 'sam/node/lends/coder.json'
    assert show().returncode == 113
    try:
        start('sam', svr)
        start('alex', svr)
        for who in ('sam', 'alex'):
            until(lambda: ask(who, '/v1/groups', check=False))
        out = call('sam', svr, 'new', 'Scratch', '--as', 'Sam')
        link = re.search(r'svrngs://join/\S+', out.stdout + out.stderr)[0]
        call('alex', svr, 'join', link, '--as', 'Alex', '--no-run')
        group = ask('alex', '/v1/groups')['groups'][0]['id']
        call('sam', svr, 'run', name, '--port', port, '--folder', work, '--',
             '/bin/sh', wrapper, server, port, mark)
        first = pid()
        assert first and active.is_file()
        print(confirm(svr, 'share', port, '--program', 'mac-fixture', '--name', 'coder', '--group', 'Scratch'))
        # The scratch node is not the installed login service. Reapprove the
        # running share so this run can test renewal without claiming login.
        print(confirm(svr, 'approve', 'coder'))
        before = json.loads(lend.read_text())
        original = active.read_bytes()
        got = before.get('keep')
        print(f'Approve: launchd pid={first} grant label={got and got["unit"]}')
        assert got and got['unit'] == label, 'the observed launch agent must be granted'
        assert got['files'] == hashlib.sha256(original).hexdigest()

        def listed():
            rows = ask('alex', f'/v1/lends?namespace={group}')['lent']
            return next((row for row in rows if row['name'] == 'coder'), None)

        row = until(listed)
        origin = row['origin']
        ask('alex', '/v1/play', json.dumps({'on': {'key': row['key'], 'name': 'coder'}}))
        assert request() == (200, 'keeper')
        call('sam', keeper, 'restart', name)
        successor = until(lambda: pid() if pid() != first else None)
        until(listening)
        reply = request()
        after = json.loads(lend.read_text())
        print(f'Restart: pid {first} -> {successor}; same address {reply[0]}; approval changed={after["approved"] != before["approved"]}')
        assert reply == (200, 'keeper') and after['approved'] != before['approved']
        assert {k: v for k, v in after.items() if k != 'approved'} == {k: v for k, v in before.items() if k != 'approved'}

        def refused(case, counter, words):
            before_count = counter.stat().st_size
            saved = lend.read_bytes()
            status, text = request()
            n = counter.stat().st_size - before_count
            print(f'{case}: HTTP {status}; bytes={n}; share unchanged={lend.read_bytes() == saved}; {text[:130]}')
            assert status == 502 and words in text and n == 0 and lend.read_bytes() == saved

        call('sam', keeper, 'restart', name)
        until(lambda: pid() if pid() != successor else None)
        active.write_bytes(original + b'\n')
        until(listening)
        refused('Changed plist', mark, 'changed')
        active.write_bytes(original)
        assert request() == (200, 'keeper')

        call('sam', keeper, 'stop', name)
        stray = start('foreign', '/usr/bin/python3', server, port, foreign)
        until(foreign.exists)
        refused('Unrelated holder', foreign, 'didn’t choose')
        stray.terminate()
        stray.wait(10)
        call('sam', keeper, 'start', name)
        until(listening)
        doc = json.loads(lend.read_text())
        doc['approved']['held'] = True
        lend.write_text(json.dumps(doc))
        refused('Already held', mark, 'didn’t choose')
        call('sam', svr, 'unshare', 'coder')
        call('sam', keeper, 'restart', name)
        status, _ = request()
        print(f'Withdrawn: HTTP {status}; no declaration={not lend.exists()}')
        assert status != 200 and not lend.exists() and pid()
    except BaseException:
        for log in work.glob('*.log'):
            print(log.name, log.read_text()[-7000:])
        raise
    finally:
        for kid in kids:
            if kid.poll() is None:
                kid.terminate()
            kid.wait(10)
        registration = show()
        if registration.returncode == 0:
            assert str(active) in registration.stdout, 'not our registration'
            call('sam', '/bin/launchctl', 'bootout', target)
            until(lambda: show().returncode == 113)
        print(f'Cleanup: launchctl print exits {show().returncode}')
