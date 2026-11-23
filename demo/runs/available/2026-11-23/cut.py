"""Run in the guest so the cut timer does not include SSH."""

import select
import subprocess
import sys
import time


def as_user(user):
    return ["systemd-run", "-M", user + "@.host", "--user", "--wait", "--pipe", "-q"]


url, marker, name = sys.argv[1:]
with subprocess.Popen(as_user("alex") + ["curl", "--no-buffer", "-fsS", "--max-time", "10", url],
                      stdout=subprocess.PIPE, stderr=subprocess.PIPE) as stream:
    try:
        assert select.select([stream.stdout], [], [], 5)[0], "the member got no streaming bytes"
        assert stream.stdout.readline() == (marker + "\n").encode(), "wrong stream body"
        began = time.monotonic()
        if name != "--skip-withdrawal":
            subprocess.run(as_user("sam") + ["/home/sam/.local/bin/svrngs", "unshare", name],
                           check=True, capture_output=True, timeout=2)
        try:
            _, err = stream.communicate(timeout=max(0, 2 - (time.monotonic() - began)))
        except subprocess.TimeoutExpired:
            sys.exit("the member's answer still carries on after 2 s")
        elapsed = time.monotonic() - began
        print(f"member stream ended after {elapsed:.3f} s (exit {stream.returncode}): {err.decode().strip()}")
        assert elapsed <= 2 and stream.returncode != 0 and b"curl: (18)" in err
    finally:
        if stream.poll() is None:
            stream.terminate()
            stream.wait(3)
