#!/usr/bin/env bash
# The synthetic HLS VOD asset of SUPPORT_LINK's playback PoC (svrn-docs
# rings/current/SUPPORT_LINK.md, "Fixture and qualification"): 14 minutes,
# aligned 2 s H.264 segments at 540p, 720p and 1080p, valid playlists. The
# breakage is the origin's (origin.mjs), never the files'. Also fetches the
# pinned hls.js. Both land in git-ignored folders; this script is how to
# get them back.
set -euo pipefail
cd "$(dirname "$0")"
FIXTURE_REV=1   # bump when anything below changes; origin.mjs reports it

mkdir -p vendor
if [ ! -f vendor/hls-1.7.4.min.js ]; then
  t=$(mktemp -d); (cd "$t" && npm pack hls.js@1.7.4 --silent && tar xzf hls.js-1.7.4.tgz)
  cp "$t/package/dist/hls.min.js" vendor/hls-1.7.4.min.js; rm -rf "$t"
fi
echo "cfc70cc9e9fc63d071919e087670abc760c8fd1bc889b88588026e4d22193bbe  vendor/hls-1.7.4.min.js" | sha256sum -c

[ -f media/master.m3u8 ] && { echo "media/ exists (rev $(cat media/REV))"; exit 0; }
rm -rf media; mkdir -p media
# 24 fps, a GOP of exactly 48 frames: every segment starts on a key frame at
# an even second, so the three renditions' segment n all cover [2n, 2n+2).
ffmpeg -hide_banner -loglevel warning -f lavfi -i "testsrc2=size=1920x1080:rate=24:duration=840" \
  -filter_complex "[0:v]drawtext=text='%{pts\:hms}':fontsize=96:fontcolor=white:box=1:boxcolor=black@0.6:x=60:y=60,split=3[a][b][c];[a]scale=960:540[v0];[b]scale=1280:720[v1];[c]copy[v2]" \
  -map "[v0]" -map "[v1]" -map "[v2]" -c:v libopenh264 -g 48 -keyint_min 48 \
  -force_key_frames "expr:gte(t,n_forced*2)" \
  -b:v:0 1500k -b:v:1 3000k -b:v:2 6000k \
  -f hls -hls_time 2 -hls_playlist_type vod -hls_segment_type fmp4 \
  -hls_segment_filename "media/v%v/seg%03d.m4s" -hls_fmp4_init_filename init.mp4 \
  -master_pl_name master.m3u8 -var_stream_map "v:0,name:540p v:1,name:720p v:2,name:1080p" \
  "media/v%v/index.m3u8"
echo "$FIXTURE_REV" > media/REV
ls media; grep -c EXTINF media/v1080p/index.m3u8
