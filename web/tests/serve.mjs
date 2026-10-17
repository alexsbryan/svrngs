// The built page served on localhost, for the app checks
// (apps/jellyfin/check/check.sh, apps/movie-night/check/check.sh): `serve`
// from U4's dry run's driver, demo/runs/u4/2026-10-05/drive.mjs:18-29,
// which stays there as it ran.
//
//   node serve.mjs <site dir>     serve the built page; prints `port <n>`
import { serve } from './page.mjs';

const [dir] = process.argv.slice(2);
if (!dir) {
  console.error('usage: node serve.mjs <dir>');
  process.exit(2);
}
// Each answer's path and bytes, a line each after the port: dry.sh sums
// what the origin sent during each run, the service worker's fetches too.
const server = await serve(dir);
server.on('request', (req, res) => {
  const end = res.end;
  res.end = function (body, ...rest) {
    console.log(`${res.statusCode} ${req.url} ${body?.length ?? 0}`);
    return end.call(this, body, ...rest);
  };
});
console.log(`port ${server.address().port}`);
