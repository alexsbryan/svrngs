"""Test fixture: the gate's delegated cgroup, a scratch home and a private user manager."""
import argparse
from contextlib import contextmanager
from functools import partial
import json
import os
from pathlib import Path
import shutil
import subprocess as sp
import tempfile
import time


def built_test(env, *target):
    built = sp.run(
        ['cargo', 'test', '--locked', '-p', 'svrngs-node', *target,
         '--no-run', '--message-format=json-render-diagnostics'],
        env=env, text=True, stdout=sp.PIPE, check=True,
    )
    name = 'browser' if target == ('--test', 'browser') else 'svrngs_node'
    (binary,) = (Path(item['executable']) for line in built.stdout.splitlines()
                 if (item := json.loads(line)).get('executable')
                 and item['profile']['test'] and item['target']['name'] == name)
    return binary


def require_gate():
    if not Path('/run/.containerenv').exists() or Path.cwd() != Path('/work'):
        raise SystemExit('never-ran: keeper tests need the disposable svrngs-gate container')
    if not Path('/run/systemd/system').is_dir():
        raise SystemExit('never-ran: rebuild svrngs-gate for its private systemd user manager')


@contextmanager
def user_manager():
    require_gate()
    with tempfile.TemporaryDirectory(prefix='svrngs-keeper-test-') as scratch:
        home = Path(scratch)
        runtime = home / 'runtime'
        runtime.mkdir(mode=0o700)
        (home / '.config/systemd/user').mkdir(parents=True)
        bus_address = f'unix:path={runtime}/bus'
        env = dict(os.environ, HOME=scratch, XDG_CONFIG_HOME=str(home / '.config'),
                   XDG_RUNTIME_DIR=str(runtime), SVRNGS_DIR=str(home / 'node'),
                   DBUS_SESSION_BUS_ADDRESS=bus_address, TOKIO_WORKER_THREADS='4')
        # Name the subtree as a real login manager's, as job::unit_of reads it.
        cg = Path('/sys/fs/cgroup') / home.name / f'user@{os.getuid()}.service'
        cg.mkdir(parents=True)
        with sp.Popen(['dbus-daemon', '--session', '--nofork', '--address=' + bus_address], env=env) as bus:
            try:
                with sp.Popen(['systemd', '--user'], env=env,
                              preexec_fn=lambda: (cg / 'cgroup.procs').write_text(str(os.getpid()))) as manager:
                    try:
                        for _ in range(100):
                            if manager.poll() is not None:
                                raise RuntimeError(f'private user manager exited {manager.returncode}')
                            if (runtime / 'systemd/private').exists():
                                status = sp.run(['systemctl', '--user', 'is-system-running'], env=env,
                                                capture_output=True, text=True)
                                if status.stdout.strip() in ('running', 'degraded'):
                                    break
                            time.sleep(.05)
                        else:
                            raise RuntimeError('private user manager did not start')
                        yield home, env
                    finally:
                        if manager.poll() is None:
                            manager.terminate()
            finally:
                bus.terminate()


def in_unit(home, env, binary, name, extra=''):
    # Share judges the door's sign-in chain, so this is a real enabled unit,
    # not a transient systemd-run job, and it runs only when we start it.
    unit = 'svrngs-keeper-test-gate.service'
    log = home / 'test.log'
    keys = ('HOME', 'XDG_RUNTIME_DIR', 'SVRNGS_DIR', 'DBUS_SESSION_BUS_ADDRESS',
            'TOKIO_WORKER_THREADS', 'PATH', 'PLAYWRIGHT_BROWSERS_PATH')
    settings = ''.join(f'Environment={k}={env[k]}\n' for k in keys)
    units = home / '.config/systemd/user'
    (units / unit).write_text(
        f'[Service]\nType=oneshot\nWorkingDirectory=/work\n'
        f'{settings}Environment=SVRNGS_RUN_PAGE_DEMO=scratch\n{extra}'
        f'ExecStart={json.dumps(str(binary))} --ignored --exact {name} --nocapture\n'
        f'StandardOutput=append:{log}\nStandardError=inherit\n'
        'TimeoutStartSec=180\n[Install]\nWantedBy=default.target\n'
    )
    ctl = partial(sp.run, env=env)
    try:
        ctl(['systemctl', '--user', 'enable', unit], check=True)
        out = ctl(['systemctl', '--user', 'start', unit], check=False)
        text = log.read_text() if log.exists() else ''
        print(text, end='', flush=True)
        return out, text
    finally:
        coder = 'svrngs-keeper-test-page-coder.service'
        running = [coder, unit] if (units / coder).exists() else [unit]
        ctl(['systemctl', '--user', 'stop', *running], check=False)


def exact(home, env, binary, name):
    print(f'keeper e2e: {name}', flush=True)
    out, text = in_unit(home, env, binary, name)
    if out.returncode:
        raise SystemExit(out.returncode)
    if not any(line.startswith('test result: ok. 1 passed; 0 failed; 0 ignored;')
               for line in text.splitlines()):
        raise SystemExit(f'never-ran: expected exactly one passing test: {name}')


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--test', choices=['browser'], required=True)
    parser.add_argument('browser', nargs=2)
    parser.add_argument('--lib', required=True)
    cases = parser.parse_args()
    env = dict(os.environ, TOKIO_WORKER_THREADS='4')
    require_gate()
    renewal = built_test(env, '--lib')
    with user_manager() as (home, private):
        exact(home, private, renewal, cases.lib)
    sp.run(['cargo', 'build', '--quiet', '--locked', '-p', 'svrngs-keeper'], env=env, check=True)
    keeper = Path(env.get('CARGO_TARGET_DIR', 'target')) / 'debug/svrngs-keeper'
    browser = built_test(env, '--test', cases.test)
    for name in cases.browser:
        # Each WebKit case runs and keeps the same named card; never share that home.
        with user_manager() as (home, private):
            shutil.copy2(browser, home / 'browser')
            shutil.copy2(keeper, home)
            exact(home, private, home / 'browser', name)


if __name__ == '__main__':
    main()
