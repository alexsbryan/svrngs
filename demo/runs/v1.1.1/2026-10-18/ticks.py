"""Side instrument for the V1.1.1 smoke: every svrngs process of the kit's
binary on this machine, its utime+stime in clock ticks (100 Hz) and VmRSS,
once a second, from /proc: the precision ps's `time` (1 s) lacks."""
import json, os, sys, time, re
binary, out, secs = sys.argv[1], sys.argv[2], float(sys.argv[3])
f = open(out, "a", buffering=1)
end = time.time() + secs
while time.time() < end:
    t, rows = time.time(), {}
    for pid in os.listdir("/proc"):
        if not pid.isdigit():
            continue
        try:
            cmd = open(f"/proc/{pid}/cmdline", "rb").read().split(b"\0")
            if not cmd or cmd[0].decode() != binary or b"--as" not in cmd:
                continue
            who = cmd[cmd.index(b"--as") + 1].decode()
            st = open(f"/proc/{pid}/stat").read().rsplit(")", 1)[1].split()
            rss = int(re.search(r"VmRSS:\s+(\d+)", open(f"/proc/{pid}/status").read())[1])
            rows[who] = [int(st[11]) + int(st[12]), rss]
        except (OSError, IndexError, ValueError, TypeError):
            pass
    f.write(json.dumps({"t": round(t, 3), "rows": rows}) + "\n")
    time.sleep(max(0.0, t + 1 - time.time()))
