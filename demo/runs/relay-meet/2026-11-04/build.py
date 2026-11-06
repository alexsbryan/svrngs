#!/usr/bin/env python3
"""Bounded two-platform local demo build; no deployment or full-gate claim."""
import datetime
import hashlib
import json
import os
from pathlib import Path
import subprocess
import tarfile

root = Path(__file__).resolve().parents[4]
out = root / 'target/relay-demo'
out.mkdir(exist_ok=True)
env = dict(os.environ, CARGO_BUILD_JOBS='1', CARGO_INCREMENTAL='0')
source = subprocess.check_output(['git', 'rev-parse', 'HEAD'], cwd=root, text=True).strip()

def run(name, command):
    print(f'{name}: starting', flush=True)
    with (out / f'{name}.log').open('w') as log:
        result = subprocess.run(command, cwd=root, env=env, stdout=log, stderr=subprocess.STDOUT)
    print(f'{name}: exit {result.returncode}', flush=True)
    if result.returncode:
        raise SystemExit(f'{name} failed; inspect {out / (name + ".log")}')

run('build-web', ['bash', 'scripts/build-web.sh', '--out', 'target/relay-demo/site'])
get = out / 'site/get'
get.mkdir(exist_ok=True)
targets = [('darwin-aarch64', 'aarch64-apple-darwin', ['cargo', 'build']),
           ('linux-x86_64', 'x86_64-unknown-linux-musl', ['cargo', 'zigbuild'])]
for name, triple, command in targets:
    run(name, [*command, '--locked', '--release', '-p', 'svrngs', '--target', triple])
    with tarfile.open(get / f'svrngs-{name}.tar.gz', 'w:gz') as archive:
        archive.add(root / f'target/{triple}/release/svrngs', arcname='svrngs')
sums = []
for path in sorted(get.glob('*.tar.gz')):
    sums.append(f'{hashlib.sha256(path.read_bytes()).hexdigest()}  {path.name}')
(get / 'SHA256SUMS').write_text('\n'.join(sums) + '\n')
manifest = {'source': source, 'built_at': datetime.datetime.now(datetime.timezone.utc).isoformat(),
            'platforms': [t[0] for t in targets], 'scope': 'local relay/onboarding demo; no scoped lending',
            'sha256': {str(p.relative_to(out / 'site')): hashlib.sha256(p.read_bytes()).hexdigest()
                       for p in sorted((out / 'site').rglob('*')) if p.is_file()}}
(out / 'site/source.json').write_text(json.dumps(manifest, indent=2) + '\n')
print(f'Local demo site ready: {out / "site"}; source {source}', flush=True)
