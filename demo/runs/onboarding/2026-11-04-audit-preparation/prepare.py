"""Capture the authorized local instrument preparation and its actual verdicts."""

import hashlib
import os
from pathlib import Path
import sqlite3
import subprocess
import time

REPO = Path(__file__).resolve().parents[4]
OUT = Path(__file__).resolve().parent
TOOLS = Path('/Users/alexsbryan/dev/commonwealth-ai/target/debug')
AUDIT = str(TOOLS / 'sovereign-cli-dev')
ROOT = Path('/var/folders/qf/4ntssn_10d598hw_5rjhx3740000gp/T/opencode/onboarding-audit-029d4b84')
ENV = {**os.environ, 'SVRNMESH_DATA_DIR': str(ROOT)}


def run(log, args):
    log.write('$ ' + ' '.join(map(str, args)) + '\n')
    log.flush()
    started = time.monotonic()
    result = subprocess.run(args, cwd=REPO, env=ENV, stdout=subprocess.PIPE,
                            stderr=subprocess.STDOUT, text=True, check=False)
    log.write('\n'.join(line.rstrip() for line in result.stdout.splitlines()) + '\n')
    log.write(f'exit {result.returncode}; elapsed {time.monotonic() - started:.1f}s\n\n')
    log.flush()


with (OUT / 'preparation.txt').open('w') as log:
    log.write(f'cwd: {REPO}\nSVRNMESH_DATA_DIR={ROOT}\ncorpus-id: svrngs\n\n')
    for name in ('sovereign-cli-dev', 'svrn-ingest'):
        path = TOOLS / name
        log.write(f'SHA256 {hashlib.sha256(path.read_bytes()).hexdigest()} {path}\n')
    run(log, ['git', 'rev-parse', 'HEAD'])
    run(log, ['git', '-C', '/Users/alexsbryan/dev/commonwealth-ai', 'rev-parse', 'HEAD'])
    run(log, ['git', 'diff', '--stat', 'b463b29b', 'HEAD'])
    run(log, [AUDIT, 'code', 'index', str(REPO), '--corpus-id', 'svrngs',
              '--data-dir', str(ROOT / 'indexes'), '--full'])
    run(log, [AUDIT, 'refresh', '--local', '--data-dir', str(ROOT / 'indexes')])
    for crate in ('svrngs-core', 'svrngs-node', 'svrngs-runtime', 'svrngs'):
        run(log, [AUDIT, 'code', 'dry-report', '--corpus-id', 'svrngs',
                  '--scope', f'crates/{crate}/'])
    for noun in ('Call', 'Fault', 'Phase', 'Receipt', 'Request', 'Resolver', 'Welcome'):
        run(log, [AUDIT, 'code', 'converge', 'noun', noun, '--corpus-id', 'svrngs'])

with (OUT / 'coverage.txt').open('w') as log:
    db = ROOT / 'indexes/svrngs/scip_graph.db'
    log.write(f'SQLite read-only: {db}\nline_start values are raw SCIP coordinates\n\n')
    conn = sqlite3.connect(f'file:{db}?mode=ro', uri=True)
    queries = [
        'SELECT * FROM scip_meta ORDER BY key',
        'SELECT corpus_id, language, count(*) FROM symbols GROUP BY corpus_id, language',
        "SELECT name, kind, file_path, line_start FROM symbols WHERE name IN "
        "('Call','Fault','Phase','Receipt','Request','Resolver','Welcome') ORDER BY name,file_path,line_start",
        "SELECT file_path, count(*) FROM symbols WHERE file_path LIKE 'apps/%' GROUP BY file_path ORDER BY file_path",
        "SELECT substr(file_path,1,100), count(*) FROM symbols WHERE language='python' GROUP BY file_path ORDER BY count(*) DESC LIMIT 15",
    ]
    for query in queries:
        log.write(query + '\n')
        for row in conn.execute(query):
            log.write(repr(row) + '\n')
        log.write('\n')
    conn.close()

# Terminal progress uses CR and trailing spaces; normalize display whitespace
# without dropping any output lines, then leave one final newline for git.
for name in ('preparation.txt', 'coverage.txt'):
    path = OUT / name
    path.write_text(path.read_text().rstrip() + '\n')
