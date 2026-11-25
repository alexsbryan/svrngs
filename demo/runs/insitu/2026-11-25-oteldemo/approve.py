"""Approves a restarted lend at a terminal, as the site's person types y.
  python3 -I approve.py <svrngs binary> <lend>   (HOME and SVRNGS_DIR set)"""
import os, pty, sys, time
pid, fd = pty.fork()
if pid == 0:
    os.execv(sys.argv[1], [sys.argv[1], "approve", sys.argv[2]])
out, t = b"", time.time()
while time.time() - t < 20:
    try:
        b = os.read(fd, 4096)
    except OSError:
        break
    if not b:
        break
    out += b
    if b"Type y" in out and b"y\n" not in out[-3:]:
        os.write(fd, b"y\n"); out += b"y\n"
print(out.decode(errors="replace").strip().splitlines()[-1])
