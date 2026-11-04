#!/usr/bin/env python3
"""Recorded-run driver; reuses prior survey instruments, not a product gate.

Run from the repository root with `survey`, `public`, or `close`.
The public pass demotes only an archived scratch tree, never the checkout.
"""
import io
import json
import os
from pathlib import Path
import re
import runpy
import subprocess
import sys
import tarfile

ROOT = Path(__file__).resolve().parents[4]
HERE = Path(__file__).resolve().parent
DATA = HERE / 'data'
OLD = ROOT / 'demo/runs/economize/2026-10-25-give/scripts'
JS = ROOT / 'demo/runs/economize/2026-11-03-js-read/survey.py'
SCRATCH = ROOT / 'target/onboarding-econ/scratch'


def run(name, command, cwd=ROOT, env=None):
    result = subprocess.run(command, cwd=cwd, env=env, text=True, capture_output=True)
    (DATA / name).write_text(
        '$ ' + ' '.join(map(str, command)) + '\n'
        + result.stdout + result.stderr + f'\nexit: {result.returncode}\n')
    print(f'{name}: exit {result.returncode}', flush=True)
    return result


def survey():
    run('lean-before.txt', ['python3', 'scripts/lean_check.py'])
    run('delta.txt', ['git', 'diff', '--stat', 'dece85b1..HEAD', '--',
                      'crates', 'web', 'apps', 'scripts'])
    run('clone-availability.txt', ['zsh', '-c',
        'command -v sovereign-cli; for p in '
        '$HOME/dev/commonwealth-ai/target/debug/sovereign-cli '
        '$HOME/dev/commonwealth-ai/target/release/sovereign-cli '
        '$HOME/dev/commonwealth-ai/sovereign/target/release/sovereign-cli '
        '$HOME/dev/commonwealth-ai/dist/sovereign-aarch64-apple-darwin/sovereign-cli '
        '$HOME/dev/wt-rel070/dist/sovereign-aarch64-apple-darwin/sovereign-cli; '
        'do if test -x "$p"; then "$p" code dry-report --help; '
        'else printf "unavailable: %s\\n" "$p"; fi; done'])
    paths = subprocess.check_output(['git', 'ls-files', '-z'], cwd=ROOT).decode().split('\0')
    product = [p for p in paths if p.endswith(('.rs', '.js', '.sh', '.py'))
               and p.startswith(('crates/', 'web/', 'scripts/'))
               and '/tests/' not in p and not re.search(r'_tests?\.rs$|/tests?(_\w+)?\.rs$|\.test\.m?js$', p)
               and '/vendor/' not in p]
    run('windows-4.txt', ['python3', str(OLD / 'windows.py'), '4', *product])
    run('cfgtest.txt', ['python3', str(OLD / 'cfgtest.py')])
    run('js-survey.txt', ['python3', '-I', str(JS)])
    run('js-more.txt', ['python3', '-I', str(JS), 'more'])
    run('check-workspace.txt', ['cargo', 'check', '--workspace', '--offline'])


def public():
    if not SCRATCH.exists():
        SCRATCH.mkdir(parents=True)
        archive = subprocess.check_output(['git', 'archive', '75f9fd3b'])
        with tarfile.open(fileobj=io.BytesIO(archive)) as source:
            source.extractall(SCRATCH, filter='data')
        run('demote.txt', ['python3', str(OLD / 'demote.py'), str(SCRATCH)])
        # Prior pass's two cross-module exports whose name lookup cannot restore
        # visibility, and the command's binary-only public method (give SURVEY §1).
        for path, name in [('crates/svrngs-core/src/membership.rs', 'standing_reads'),
                           ('crates/svrngs-core/src/synced.rs', 'NotASyncPath'),
                           ('crates/svrngs/src/cli.rs', 'exit_code')]:
            at = SCRATCH / path
            at.write_text(re.sub(r'pub\(crate\)([^\n]*\b' + name + r'\b)',
                                 r'pub\1', at.read_text()))
    env = dict(os.environ, CARGO_INCREMENTAL='0', CARGO_TARGET_DIR=str(ROOT / 'target'))
    for label, directory, args in [('native', SCRATCH, ['--workspace']),
                                   ('wasm', SCRATCH / 'web', ['--target', 'wasm32-unknown-unknown'])]:
        env['DIR'] = '.' if label == 'native' else 'web'
        for number in range(1, 26):
            name = f'public-{label}-{number}.txt'
            result = run(name, ['cargo', 'check', *args, '--offline', '--message-format=json'], directory, env)
            raw = DATA / f'public-{label}.jsonl'
            raw.write_text(result.stdout)
            if result.returncode == 0:
                run(f'warnings-{label}.txt', ['python3', str(OLD / 'cargo_warnings.py'), str(raw)])
                break
            fixed = run(f'repub-{label}-{number}.txt',
                        ['python3', str(OLD / 'repub.py'), str(SCRATCH), str(raw)], env=env)
            if 'fixed 0' in fixed.stdout:
                print(f'{label}: could-not-judge until scratch diagnostics are resolved')
                break


def close():
    run('lean-after.txt', ['python3', 'scripts/lean_check.py'])
    result = run('check.txt', ['lockf', '-k', '/tmp/svrngs-gate.lock', 'bash', 'scripts/check.sh'])
    print((DATA / 'check.txt').read_text())
    sys.exit(result.returncode)


def compact():
    # Retain exact rendered diagnostics and build verdicts, not repeated cargo
    # artifact metadata. The original run is not re-executed or reinterpreted.
    for path in sorted(DATA.glob('public-*')):
        lines = []
        for line in path.read_text().splitlines():
            try:
                message = json.loads(line)
            except ValueError:
                lines.append(line)
                continue
            if message.get('reason') == 'compiler-message':
                lines.append(message['message']['rendered'])
            elif message.get('reason') == 'build-finished':
                lines.append(line)
        path.write_text('\n'.join(lines) + '\n')
        print(f'{path.name}: retained diagnostics and build verdict')


def sites():
    sys.argv = [str(ROOT / 'scripts/lean_check.py')]
    lean = runpy.run_path(sys.argv[0])
    paths = [
        'crates/svrngs-node/src/invite.rs', 'crates/svrngs-core/src/app.rs',
        'crates/svrngs-node/src/http.rs', 'crates/svrngs-node/src/declared.rs',
        'crates/svrngs-core/src/project.rs', 'web/install.sh',
        'crates/svrngs-node/src/home/page/group-link.js',
        'crates/svrngs-node/src/home/page/group-links.js',
        'crates/svrngs-node/src/home/ui/copy.js', 'web/page/name-form.js',
        'web/page/computer.js', 'web/page/computer-line.js',
        'crates/svrngs-node/src/home/page/folder.js',
        'crates/svrngs-node/src/home/page/group-folder.js',
    ]
    lines = ['lean_check.code_lines per reused file (inline tests included):']
    for path in paths:
        kinds = {'.js', '.mjs'} if path.endswith('.js') else lean['CODE']
        lines.append(f"{path}: {lean['code_lines'](ROOT / path, kinds)}")
    text = '\n'.join(lines) + '\n'
    (DATA / 'sites.txt').write_text(text)
    print(text)


if __name__ == '__main__':
    DATA.mkdir(exist_ok=True)
    {'survey': survey, 'public': public, 'close': close, 'compact': compact,
     'sites': sites}[sys.argv[1]]()
