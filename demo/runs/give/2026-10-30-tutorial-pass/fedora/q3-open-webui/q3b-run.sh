#!/bin/bash
# Q3 model B, step 2: copy the state, start the card's equivalent by hand.
E=/home/alexbryan/.cache/uv/archive-v0/NGaSzeqy18D22xuYLd6WR
D=$E/lib/python3.11/site-packages/open_webui/data
R=/home/alexbryan/svrngs-pass/q3/run
IMG=docker.io/library/python:3.11.15-slim@sha256:78b39ef14d8e2b4d71f8dc304f1328c37df95fe0ef99477c2ae6bd3d03784553
SOCAT=docker.io/alpine/socat@sha256:603174b77eebaba25ba8f8ebb15e1234d6600409554e95d6fb203fd106150287
set -x
# The copy: the data dir (sqlite + WAL, copied live; the original was idle) and
# the folder it ran in (.webui_secret_key).
podman volume create svrngs-pass-owui-data; tar -C $D -cf - . | podman volume import svrngs-pass-owui-data -
podman volume create svrngs-pass-owui-home; tar -C $R -cf - . | podman volume import svrngs-pass-owui-home -
podman volume create svrngs-pass-owui-hf
podman pod create --name svrngs-pass-owui-b -p 127.0.0.1:18081:18090
podman run -d --pod svrngs-pass-owui-b --name svrngs-pass-owui-b-app \
  -v svrngs-pass-owui-deps:$E -v svrngs-pass-owui-data:$D -v svrngs-pass-owui-home:$R \
  -v svrngs-pass-owui-hf:/root/.cache/huggingface -w $R \
  $IMG $E/bin/python $E/bin/open-webui serve --host 127.0.0.1 --port 18080
podman run -d --pod svrngs-pass-owui-b --name svrngs-pass-owui-b-door $SOCAT TCP-LISTEN:18090,fork,reuseaddr TCP:127.0.0.1:18080
