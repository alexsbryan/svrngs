#!/bin/bash
# Q3 route C: Open WebUI's own image, pinned, with the observed data dir as state.
E=/home/alexbryan/.cache/uv/archive-v0/NGaSzeqy18D22xuYLd6WR
D=$E/lib/python3.11/site-packages/open_webui/data
R=/home/alexbryan/svrngs-pass/q3/run
IMG=ghcr.io/open-webui/open-webui@sha256:0caf3d68199bcd8b7051775778691dfa5f3c8a355d1d6484000171b2974f40f9
set -x
# The data dir, plus the key file from the folder it ran in, into one volume.
podman volume create svrngs-pass-owui-c-data
( tar -C $D -cf - . ; ) | podman volume import svrngs-pass-owui-c-data -
# volume import replaces content, so add the key in a second pass through a container.
tar -C $R -cf - .webui_secret_key | podman run --rm -i -v svrngs-pass-owui-c-data:/d docker.io/library/python:3.11.15-slim@sha256:78b39ef14d8e2b4d71f8dc304f1328c37df95fe0ef99477c2ae6bd3d03784553 tar -C /d -xf -
podman run -d --name svrngs-pass-owui-c -p 127.0.0.1:18082:8080 \
  -e WEBUI_SECRET_KEY_FILE=/app/backend/data/.webui_secret_key \
  -v svrngs-pass-owui-c-data:/app/backend/data $IMG
