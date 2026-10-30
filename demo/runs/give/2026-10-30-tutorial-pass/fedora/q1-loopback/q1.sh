#!/bin/bash
# Q1: is a loopback-only listener inside a container reachable through a published port?
PY=docker.io/library/python@sha256:1148dbb014615300fa7e713b2f94f550c0d1c69780c0805f84f093ceee34dd0b
SOCAT=docker.io/alpine/socat@sha256:603174b77eebaba25ba8f8ebb15e1234d6600409554e95d6fb203fd106150287
probe() { # name hostport
  sleep 3
  printf '%s: ' "$1"
  curl -sS -o /dev/null -m 5 -w 'http=%{http_code} size=%{size_download}\n' "http://127.0.0.1:$2/" 2>&1 | tr '\n' ' '; echo
}
run() { # name hostport netflag bindaddr
  podman run -d --name "svrngs-pass-q1-$1" $3 -p "127.0.0.1:$2:8000" "$PY" python3 -m http.server --bind "$4" 8000 >/dev/null
  probe "$1 (net=${3:-default}, bind=$4)" "$2"
  echo "  network mode: $(podman inspect --format '{{.HostConfig.NetworkMode}}' svrngs-pass-q1-$1)"
  echo "  listens (in container): $(podman exec svrngs-pass-q1-$1 python3 -c "print(open('/proc/net/tcp').read().splitlines()[1:])" | head -c 300)"
}
run default-lo   18001 ""                 127.0.0.1
run default-any  18002 ""                 0.0.0.0
run pasta-lo     18003 "--network pasta"  127.0.0.1
run pasta-any    18004 "--network pasta"  0.0.0.0
run bridge-lo    18005 "--network bridge" 127.0.0.1
run bridge-any   18006 "--network bridge" 0.0.0.0
# pods: forwarder beside the loopback-only program
pod() { # name hostport podnetflag
  podman pod create --name "svrngs-pass-q1-$1" $3 -p "127.0.0.1:$2:8001" >/dev/null
  podman run -d --pod "svrngs-pass-q1-$1" --name "svrngs-pass-q1-$1-app" "$PY" python3 -m http.server --bind 127.0.0.1 8000 >/dev/null
  podman run -d --pod "svrngs-pass-q1-$1" --name "svrngs-pass-q1-$1-fwd" "$SOCAT" TCP-LISTEN:8001,fork,reuseaddr TCP:127.0.0.1:8000 >/dev/null
  probe "pod $1 via forwarder 8001 (podnet=${3:-default})" "$2"
  echo "  pod infra network mode: $(podman inspect --format '{{.HostConfig.NetworkMode}}' $(podman pod inspect --format '{{.InfraContainerID}}' svrngs-pass-q1-$1))"
}
pod pod-default 18011 ""
pod pod-pasta   18012 "--network pasta"
pod pod-bridge  18013 "--network bridge"
# pod without forwarder: publish 8000 straight to the loopback-only app
podman pod create --name svrngs-pass-q1-podnofwd -p 127.0.0.1:18014:8000 >/dev/null
podman run -d --pod svrngs-pass-q1-podnofwd --name svrngs-pass-q1-podnofwd-app "$PY" python3 -m http.server --bind 127.0.0.1 8000 >/dev/null
probe "pod default, no forwarder, publish straight to loopback 8000" 18014
