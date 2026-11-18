// SPDX-License-Identifier: AGPL-3.0-or-later
// spot's Markdown (markdown.js): what a chat model writes, as the tree the page builds.
import assert from 'node:assert/strict';
import test from 'node:test';
import { TAGS, inline, parse } from './markdown.js';

const t = (tag, children = [], attrs = {}) => ({ tag, children, ...attrs });

test('a paragraph keeps its line breaks; a blank line starts another', () => {
  assert.deepEqual(parse('one\ntwo\n\nthree'), [
    t('p', ['one', t('br'), 'two']),
    t('p', ['three']),
  ]);
});

test('headings, and a rule of three or more', () => {
  assert.deepEqual(parse('# A\n### B ###\n---\n* * *'), [
    t('h1', ['A']),
    t('h3', ['B']),
    t('hr'),
    t('hr'),
  ]);
});

test('a fence is code as written, its language kept, Markdown inside untouched', () => {
  assert.deepEqual(parse('```js\nconst a = **b**;\n\n<x>\n```\nafter'), [
    t('pre', [t('code', ['const a = **b**;\n\n<x>'], { lang: 'js' })]),
    t('p', ['after']),
  ]);
});

test('an unclosed fence, as an answer mid-stream has, runs to the end as code', () => {
  assert.deepEqual(parse('see:\n```\nfn main() {'), [
    t('p', ['see:']),
    t('pre', [t('code', ['fn main() {'])]),
  ]);
});

test('inline spans, nested, and code spans take nothing inside them', () => {
  assert.deepEqual(inline('**bold *both*** and `a*b*c` and ~~gone~~'), [
    t('strong', ['bold ', t('em', ['both'])]),
    ' and ',
    t('code', ['a*b*c']),
    ' and ',
    t('del', ['gone']),
  ]);
});

test('snake_case and a lone asterisk stay text', () => {
  assert.deepEqual(inline('my_var_name is 5 * 3'), ['my_var_name is 5 * 3']);
});

test('_x_ is emphasis outside a word', () => {
  assert.deepEqual(inline('_keep_ it, (_so_) and call_me_maybe'), [
    t('em', ['keep']),
    ' it, (',
    t('em', ['so']),
    ') and call_me_maybe',
  ]);
});

test('an escape is its character', () => {
  assert.deepEqual(inline('\\*not em\\*'), ['*not em*']);
});

test('a link goes only to http, https or mailto; anything else is shown as written', () => {
  assert.deepEqual(inline('[a](https://x.org/p) [b](javascript:alert(1)) [c](mailto:c@d.e)'), [
    t('a', ['a'], { href: 'https://x.org/p' }),
    ' [b](javascript:alert(1)) ',
    t('a', ['c'], { href: 'mailto:c@d.e' }),
  ]);
});

test("a bare address is a link, without the sentence's last stop", () => {
  assert.deepEqual(inline('see https://deepinfra.com/models.'), [
    'see ',
    t('a', ['https://deepinfra.com/models'], { href: 'https://deepinfra.com/models' }),
    '.',
  ]);
});

test("a tight list is its items' text; an ordered one keeps where it starts", () => {
  assert.deepEqual(parse('- one\n- **two**\n\n3. c\n4. d'), [
    t('ul', [t('li', ['one']), t('li', [t('strong', ['two'])])]),
    t('ol', [t('li', ['c']), t('li', ['d'])], { start: 3 }),
  ]);
});

test('a nested list, and an item running onto the next line', () => {
  assert.deepEqual(parse('1. first\n   still first\n   - inner\n2. second'), [
    t('ol', [
      t('li', [t('p', ['first', t('br'), 'still first']), t('ul', [t('li', ['inner'])])]),
      t('li', ['second']),
    ]),
  ]);
});

test('a table, aligned as its divider says, an escaped bar kept', () => {
  assert.deepEqual(parse('| Model | Cost |\n|:--|--:|\n| a\\|b | **$1** |'), [
    t('table', [
      t('thead', [
        t('tr', [t('th', ['Model'], { align: 'left' }), t('th', ['Cost'], { align: 'right' })]),
      ]),
      t('tbody', [
        t('tr', [
          t('td', ['a|b'], { align: 'left' }),
          t('td', [t('strong', ['$1'])], { align: 'right' }),
        ]),
      ]),
    ]),
  ]);
});

test('a quote holds blocks of its own', () => {
  assert.deepEqual(parse('> # Note\n> keep **this**\nlazy'), [
    t('blockquote', [t('h1', ['Note']), t('p', ['keep ', t('strong', ['this']), t('br'), 'lazy'])]),
  ]);
});

test('every node is a known tag with only known attributes, whatever the text', () => {
  const corpus = [
    '<script>alert(1)</script> <img src=x onerror=alert(1)>',
    '[x](  javascript:alert(1) ) [y](data:text/html,hi) [z](//evil.example)',
    '| <b> | x |\n|---|---|\n| [a](vbscript:x) | `<i>` |',
    '- [ ] task\n  > quote in item\n  ```\n  code in item',
    '***\n___\n**unclosed *mixed _styles ~~here',
  ];
  const attrs = new Set(['tag', 'children', 'href', 'lang', 'align', 'start']);
  const walk = (n) => {
    if (typeof n === 'string') return;
    assert.ok(TAGS.has(n.tag), `tag ${n.tag}`);
    for (const k of Object.keys(n)) assert.ok(attrs.has(k), `attribute ${k}`);
    if (n.href) assert.match(n.href, /^(https?:\/\/|mailto:)/i);
    n.children.forEach(walk);
  };
  for (const text of corpus) parse(text).forEach(walk);
});
