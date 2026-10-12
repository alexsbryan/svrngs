#!/usr/bin/env bash
# The media run.py reads from MEDIA, made here (PREDICTIONS.md, "The
# media"): the dry run made its own outside its driver and did not keep
# them. photos/: thirty 4032×3024 JPEGs, IMG_0001.jpg to IMG_0030.jpg, each
# its own; after.jpg, Sam's photo after the removal; music/: three albums of
# three tagged tracks, Ana's. By Jellyfin's ffmpeg, as the onboarding run's
# photo was.
#
#   media.sh <MEDIA>
set -euo pipefail
M="${1:?media.sh <MEDIA>}"
FF=/Applications/Jellyfin.app/Contents/MacOS/ffmpeg
rm -rf "$M"; mkdir -p "$M/photos" "$M/music"
photo() { "$FF" -v error -y -f lavfi -i "testsrc2=size=4032x3024:rate=1,noise=alls=17:allf=t:all_seed=$1,hue=h=$(($1 * 11))" -frames:v 1 -q:v 3 "$2"; }
for i in $(seq 1 30); do photo "$i" "$M/photos/$(printf 'IMG_%04d.jpg' "$i")"; done
photo 99 "$M/after.jpg"
for a in 1 2 3; do
  album="Lake Songs $a"; mkdir -p "$M/music/$album"
  for t in 1 2 3; do
    "$FF" -v error -y -f lavfi -i "sine=frequency=$((220 * a + 55 * t)):duration=40" \
      -metadata album="$album" -metadata album_artist=Ana -metadata artist=Ana \
      -metadata title="Track $t" -metadata track="$t" -metadata date=2026 \
      -c:a libmp3lame -q:a 6 "$M/music/$album/0$t Track $t.mp3"
  done
done
echo "photos: $(ls "$M/photos" | wc -l | tr -d ' '), $(cat "$M"/photos/*.jpg | wc -c | tr -d ' ') bytes; music: $(find "$M/music" -name '*.mp3' | wc -l | tr -d ' ') tracks"
