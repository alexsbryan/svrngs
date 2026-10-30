#!/bin/bash
# Q3 model B, step 1: reinstall the observed env, pinned, inside the runtime's
# image, into a deps volume at the path the env had.
set -e
E=/home/alexbryan/.cache/uv/archive-v0/NGaSzeqy18D22xuYLd6WR
IMG=docker.io/library/python:3.11.15-slim@sha256:78b39ef14d8e2b4d71f8dc304f1328c37df95fe0ef99477c2ae6bd3d03784553
podman volume create svrngs-pass-owui-deps >/dev/null
# The install step is Give's, on the giver's computer, not the card: it may read
# the observed pin list from a file. uv is installed in the throwaway layer only.
time podman run --rm --name svrngs-pass-owui-install \
  -v svrngs-pass-owui-deps:$E -v $HOME/svrngs-pass/q3/freeze.txt:/req.txt:ro,Z \
  $IMG sh -c "pip install -q --root-user-action=ignore --target /opt/uv uv==0.11.7 \
    && /opt/uv/bin/uv venv --python /usr/local/bin/python3.11 $E \
    && /opt/uv/bin/uv pip install --python $E/bin/python --no-deps -r /req.txt \
    && $E/bin/python -c 'import open_webui, importlib.metadata as m; print(\"open-webui\", m.version(\"open-webui\"))' \
    && ls $E/bin | head -50 | tr '\n' ' '"
