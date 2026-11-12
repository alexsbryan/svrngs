// Adapted from HolyArrs/HolyArrs app/tests/fakes/{tmdb,indexer}.mjs
// @7c4c22a266d9098acbea96602fec9ab03f365a16 (AGPL-3.0).
// One fictional film; the source returns a real torrent of generated media.
import { createHash } from 'node:crypto';
import { createServer } from 'node:http';
import { appendFileSync, readFileSync, writeFileSync } from 'node:fs';

export const film = {
  id: 900008,
  title: 'Copper Lighthouse',
  release_date: '2017-04-04',
  imdb_id: 'tt9000080',
  overview: 'A generated audiovisual test pattern for the Friends scene.',
  original_language: 'en',
  runtime: 1,
  release_dates: {
    results: [
      {
        iso_3166_1: 'US',
        release_dates: [{ type: 4, release_date: '2017-06-01T00:00:00.000Z', certification: '' }],
      },
    ],
  },
};
export const release = 'Copper.Lighthouse.2017.720p.WEB-DL.H264-SVRNGS.mp4';

// BitTorrent v1's canonical byte encoding. Binary piece hashes stay bytes.
export function bencode(value) {
  if (Buffer.isBuffer(value)) return Buffer.concat([Buffer.from(`${value.length}:`), value]);
  if (typeof value === 'string') return bencode(Buffer.from(value));
  if (typeof value === 'number') return Buffer.from(`i${value}e`);
  if (Array.isArray(value))
    return Buffer.concat([Buffer.from('l'), ...value.map(bencode), Buffer.from('e')]);
  return Buffer.concat([
    Buffer.from('d'),
    ...Object.keys(value)
      .sort()
      .flatMap((k) => [bencode(k), bencode(value[k])]),
    Buffer.from('e'),
  ]);
}

if (process.argv[2] === 'serve') {
  const media = readFileSync(`/seed/${release}`);
  const piece = 256 * 1024;
  const pieces = [];
  for (let i = 0; i < media.length; i += piece)
    pieces.push(
      createHash('sha1')
        .update(media.subarray(i, i + piece))
        .digest(),
    );
  const info = {
    length: media.length,
    name: release,
    'piece length': piece,
    pieces: Buffer.concat(pieces),
    private: 1,
  };
  const hash = createHash('sha1').update(bencode(info)).digest('hex');
  const torrent = bencode({ announce: 'http://127.0.0.1:18080/announce', info });
  writeFileSync(
    '/data/release.json',
    JSON.stringify({
      hash,
      bytes: media.length,
      sha256: createHash('sha256').update(media).digest('hex'),
    }),
  );
  const calls = [];
  const unexpected = [];
  const server = createServer((req, res) => {
    const url = new URL(req.url, 'http://127.0.0.1:18080');
    const path = url.pathname;
    const json = (status, body) => {
      res.writeHead(status, { 'content-type': 'application/json' });
      res.end(JSON.stringify(body));
    };
    if (path === '/evidence') return json(200, { calls, unexpected });
    calls.push({ method: req.method, path, query: url.search, at: new Date().toISOString() });
    appendFileSync('/data/fixture.jsonl', `${JSON.stringify(calls.at(-1))}\n`);
    if (path === '/announce') {
      // The two real qBittorrents share the pod's loopback; no public peers.
      const other = Number(url.searchParams.get('port')) === 6881 ? 6882 : 6881;
      const peers = Buffer.from([127, 0, 0, 1, other >> 8, other & 255]);
      res.end(bencode({ interval: 1, complete: 1, incomplete: 1, peers }));
      return;
    }
    if (path === '/release.torrent') {
      res.writeHead(200, { 'content-type': 'application/x-bittorrent' });
      res.end(torrent);
      return;
    }
    if (path === '/readyz' || path === '/healthz') return json(200, { ok: true });
    if (path === '/v1/installations' && req.method === 'POST')
      return json(201, { schema_version: 1, token: 'hai_svrngs_scene_fixture_0123456789' });
    if (path === '/v1/trailers/movie/900008') return json(404, { error: { code: 'not_found' } });
    const item = {
      source: 'tmdb',
      source_id: String(film.id),
      kind: 'movie',
      title: film.title,
      source_data: film,
    };
    const envelope = (data, id = '') => ({
      schema_version: 1,
      source: 'tmdb',
      source_id: id,
      fetched_at: new Date().toISOString(),
      expires_at: new Date(Date.now() + 86_400_000).toISOString(),
      completeness: 'complete',
      stale: false,
      data,
    });
    if (path === '/v1/search') {
      const items =
        url.searchParams.get('kind') === 'movie' && /copper/i.test(url.searchParams.get('q') ?? '')
          ? [item]
          : [];
      return json(
        200,
        envelope({ items, pagination: { page: 1, total_pages: 1, total_results: items.length } }),
      );
    }
    if (path === '/v1/movies/tmdb/900008') return json(200, envelope(item, String(film.id)));
    if (path === '/v1/music/packages') return json(200, { packages: [], next_cursor: '' });
    if (path === '/releases/latest')
      return json(200, { tag_name: 'v1.0.10', html_url: 'https://github.com/HolyArrs/HolyArrs' });
    if (path === '/source') {
      const t = url.searchParams.get('t');
      let body;
      if (t === 'caps')
        body =
          '<caps><limits max="50"/><searching><search available="yes" supportedParams="q"/><movie-search available="yes" supportedParams="q,imdbid"/></searching><categories><category id="2000" name="Movies"/></categories></caps>';
      else if (t === 'movie' || t === 'search') {
        const match =
          url.searchParams.get('imdbid') === '9000080' ||
          /copper/i.test(url.searchParams.get('q') ?? '');
        const item = `<item><title>${release.slice(0, -4)}</title><guid>${hash}</guid><pubDate>Mon, 01 Jan 2024 00:00:00 +0000</pubDate><enclosure url="http://127.0.0.1:18080/release.torrent" length="${media.length}" type="application/x-bittorrent"/><torznab:attr name="seeders" value="1"/><torznab:attr name="peers" value="2"/><torznab:attr name="infohash" value="${hash}"/></item>`;
        body = `<rss version="2.0" xmlns:torznab="http://torznab.com/schemas/2015/feed"><channel><title>Scene source</title>${match ? item : ''}</channel></rss>`;
      }
      if (body) {
        res.writeHead(200, { 'content-type': 'application/xml' });
        res.end(`<?xml version="1.0"?>${body}`);
        return;
      }
    }
    const tmdb = path.replace(/^\/v1\/tmdb/, '');
    if (tmdb === '/search/movie')
      return json(200, {
        page: 1,
        total_pages: 1,
        results: /copper/i.test(url.searchParams.get('query') ?? '') ? [film] : [],
      });
    if (tmdb === '/movie/900008') return json(200, film);
    if (tmdb === '/movie/900008/release_dates')
      return json(200, { id: film.id, ...film.release_dates });
    if (tmdb === '/genre/movie/list' || tmdb === '/genre/tv/list') return json(200, { genres: [] });
    if (tmdb === '/search/keyword' || /^\/movie\/900008\/(reviews|recommendations)$/.test(tmdb))
      return json(200, { page: 1, total_pages: 1, results: [] });
    if (tmdb === '/movie/900008/watch/providers') return json(200, { id: film.id, results: {} });
    // Explicitly absent optional catalogue features; never a catch-all answer.
    if (
      [
        '/v1/music/pools',
        '/v1/music/charts',
        '/v1/music/lists/shelves',
        '/v1/music/genres',
        '/v1/lists/music-trending',
        '/v1/music/new',
        '/v1/control',
        '/v1/ratings/movie/900008',
        '/v1/wikidata/title/movie/900008',
        '/v1/art/movie/900008',
      ].includes(path)
    )
      return json(404, { error: { code: 'not_found' } });
    if (tmdb === '/tv/on_the_air') return json(200, { page: 1, total_pages: 1, results: [] });
    if (/^\/(trending|discover|movie\/(popular|top_rated|now_playing|upcoming|changes))/.test(tmdb))
      return json(200, { page: 1, total_pages: 1, results: [] });
    unexpected.push(calls.at(-1));
    json(404, { error: `Unexpected fixture route: ${req.method} ${req.url}` });
  });
  server.listen(18080, '0.0.0.0');
}
