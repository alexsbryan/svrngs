#!/usr/bin/env python3
"""js-read (a)'s instrument: the four lists SURVEY.md reads, over the pages'
code (ralph/next/js/CHARTER.md, "Scope"). Run from the repo root:

    python3 -I demo/runs/economize/2026-11-03-js-read/survey.py

It prints, per file:
  views     each `view` entry and each top-level helper that answers a
            template (html`...`), and whether it is a named component
            (a `function Capitalized`) or an expression;
  machine   whether the file holds a machine (`takes`, `step`, `take`)
            beside io and views;
  comments  each comment block whose words, once its citations (a row's
            letter, a `file@commit`, a bare `file:line`) are taken out,
            are fewer than six: a comment with no why left;
  dupes     each run of four normalized lines found in two places;
  names     each top-level name defined in more than one file;
  iife      each arrow function called where it is written, `((a) => …)(x)`;
  defaults  each parameter whose default reads an earlier one, a local
            written as a default;
  cites     per file, the comment lines citing a row's letter, a
            `file@commit` or v1, the page before the switchover.

The parenthesized sequences, `(a(), b)`, are oxlint's, not this script's:
`no-sequences` with `allowInParentheses: false` (SURVEY.md gives the line).
"""
import hashlib
import re
import sys
from collections import defaultdict
from pathlib import Path

ROOTS = ['web/page', 'crates/svrngs-node/src/home/page', 'crates/svrngs-node/src/home/ui']
TOPS = ['web', 'crates/svrngs-node/src/home']


def files():
    out = []
    for top in TOPS:
        out += sorted(Path(top).glob('*.js'))
    for root in ROOTS:
        out += sorted(Path(root).glob('*.js'))
    return out


CITE = [
    re.compile(r'\b[\w./-]+@[0-9a-f]{7,}(:[\d,\s:-]+)?'),  # file@commit:lines
    re.compile(r'\b[a-z][\w-]*-[\w-]+ \([a-z]\)(, \([a-z]\))*'),  # row (a)
    re.compile(r'\b[\w./-]+\.(js|rs|html|md|mjs|css)(:[\d,\s:-]+)?'),  # file:line
    re.compile(r'\b[A-Z_]+\.md\b( §[\d.]+)?'),
    re.compile(r'(AGENTS|DESIGN|CHARTER|ARCH)[^;)]*?§?\s?\d+'),
    re.compile(r'`[^`]*`'),
    re.compile(r'\bsettled \d+\b|\bv1\b|#\d+'),
]


def words_left(text):
    for pattern in CITE:
        text = pattern.sub(' ', text)
    return [w for w in re.findall(r"[A-Za-z’']+", text) if len(w) > 1]


def comment_blocks(lines):
    """Each run of `//` lines (not SPDX), with its first line number."""
    block, start = [], 0
    for number, line in enumerate(lines, 1):
        stripped = line.strip()
        if stripped.startswith('//') and 'SPDX' not in stripped:
            if not block:
                start = number
            block.append(stripped[2:].strip())
        elif block:
            yield start, ' '.join(block)
            block = []
    if block:
        yield start, ' '.join(block)


TOP_TEMPLATE = re.compile(r'^(export )?(const|function) (\w+)')
VIEW_ENTRY = re.compile(r'^  (\w+): (\(|async |[a-z]\w* =>|\w+,)')


def views(lines):
    out = []
    in_view = False
    for number, line in enumerate(lines, 1):
        if re.match(r'^export const view\b', line):
            in_view = not line.rstrip().endswith('};')
            if not in_view:
                out.append((number, 'view (one line)', 'expression'))
            continue
        if in_view:
            if line.startswith('};'):
                in_view = False
                continue
            match = VIEW_ENTRY.match(line)
            if match:
                out.append((number, f'view.{match.group(1)}', 'expression'))
            continue
        match = TOP_TEMPLATE.match(line)
        if match:
            # its body: until the next top-level line
            body = []
            for later in lines[number:]:
                if later and not later[0].isspace() and not later.startswith(('}', ')', ']')):
                    break
                body.append(later)
            text = line + '\n'.join(body)
            if 'html`' in text:
                kind = 'component' if match.group(2) == 'function' and match.group(3)[0].isupper() else 'expression'
                out.append((number, match.group(3), kind))
    return out


def normalized(lines):
    for number, line in enumerate(lines, 1):
        text = re.sub(r'\s+', ' ', line.strip())
        if len(text) < 24 or text.startswith(('//', '*', '/**', 'import ', '@')):
            continue
        yield number, text


def main():
    all_files = files()
    windows = defaultdict(list)
    for path in all_files:
        lines = path.read_text().splitlines()
        print(f'## {path}')
        machine = any(re.match(r'^export const takes\b|^  step: |^  take\(|^export (const|function) (take|step)\b', l) for l in lines)
        io = any(re.match(r'^export const io\b|^  io: \{|^const io\b', l) for l in lines)
        drawn = views(lines)
        print(f'machine={machine} io={io} templates={sum(l.count("html`") for l in lines)} '
              f'long={sum(len(l) > 100 for l in lines)}')
        for number, name, kind in drawn:
            print(f'  view {path}:{number} {name} {kind}')
        for number, text in comment_blocks(lines):
            left = words_left(text)
            if len(left) < 6:
                print(f'  comment {path}:{number} left={len(left)} {text[:110]}')
        kept = list(normalized(lines))
        for i in range(len(kept) - 3):
            key = hashlib.sha1('\n'.join(t for _, t in kept[i:i + 4]).encode()).hexdigest()
            windows[key].append((str(path), kept[i][0]))
    print('## dupes')
    seen = set()
    for key, places in windows.items():
        if len(places) < 2:
            continue
        sig = tuple(sorted({p for p, _ in places}))
        first = tuple(places)
        if any((p, n - d) in seen for p, n in first for d in range(1, 4)):
            seen.update(first)
            continue
        seen.update(first)
        print('  dupe ' + ' = '.join(f'{p}:{n}' for p, n in places))


def iifes(text):
    out = []
    for match in IIFE.finditer(text):
        depth = 0
        for end in range(match.start(), len(text)):
            if text[end] == '(':
                depth += 1
            elif text[end] == ')':
                depth -= 1
                if depth == 0:
                    break
        rest = text[end + 1:].lstrip(' \n')
        if rest.startswith('('):
            out.append(text.count('\n', 0, match.start()) + 1)
    return out


IIFE = re.compile(r"\(\s*\((?:[\w\s,=.?{}\[\]'`$:]|\([^()]*\))*\)\s*=>")
DEFAULT = re.compile(
    r'(?:const \w+ = |function \w+|\w+: )\(\s*\{?\s*(\w+)[^;]*?,\s*(?:\[[\w, ]+\]|\w+)\s*=\s*([^;]*?)\)\s*(?:=>|\{)',
    re.S,
)
ROW = re.compile(r'\b[a-z][a-z0-9]*(?:-[a-z0-9]+)+ \([a-z]\)|\b(?:u4|page|keys|capability)-[a-z0-9-]+\b')
COMMIT = re.compile(r'@[0-9a-f]{7,}')
V1 = re.compile(r'\bv1\b')


def defaults(text):
    out = []
    for match in DEFAULT.finditer(text):
        head = text[match.start():match.end()]
        if head.count('\n') > 8:
            continue
        before = head[:head.find('=', head.find('('))]
        names = set(re.findall(r'\w+', before))
        if re.search(r'=\s*[^=>]*\b(%s)\b' % '|'.join(names), head):
            out.append(text.count('\n', 0, match.start()) + 1)
    return out


def cites(lines):
    out = []
    for number, line in enumerate(lines, 1):
        stripped = line.strip()
        if stripped.startswith(('//', '*', '/**')):
            if ROW.search(stripped) or COMMIT.search(stripped) or V1.search(stripped):
                out.append(number)
    return out


def more():
    names = defaultdict(list)
    for path in files():
        text = path.read_text()
        lines = text.splitlines()
        print(f'## {path}')
        print(f'  iife {iifes(text)}')
        print(f'  defaults {defaults(text)}')
        found = cites(lines)
        print(f'  cites {len(found)} {found}')
        for number, line in enumerate(lines, 1):
            match = re.match(r'^(?:export )?(?:async )?(?:const|function) ([A-Za-z_]\w*)', line)
            if match:
                names[match.group(1)].append(f'{path}:{number}')
    print('## names')
    for name, places in sorted(names.items(), key=lambda item: -len(item[1])):
        if len(places) > 1:
            print(f'  name {name} {len(places)} {" ".join(places)}')


if __name__ == '__main__':
    if sys.argv[1:] == ['more']:
        sys.exit(more())
    sys.exit(main())
