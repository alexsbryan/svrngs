"""Each plant applied alone, the guest tests run, the file restored: guests-door's checks red on their plants."""
import subprocess, sys
DOOR = "crates/svrngs-node/src/door.rs"
JOURNAL = "crates/svrngs-runtime/src/journal.rs"
REGISTER = "crates/svrngs-runtime/src/register.rs"
PLANTS = {
  "b": [
    ("the door carrying a guest as its host (GUESTS §7 bar 1)", [(DOOR,
      "        actor: PersonKey(pass.person.clone()),\n        device: DeviceKey(pass.person),\n        group,\n        person: guest.visit.guest,\n        host: Some(host),",
      "        actor: PersonKey(guest.host.clone()),\n        device: DeviceKey(guest.host.clone()),\n        group,\n        person: host,\n        host: None,")]),
    ("a visit opening a lend of Bo's (bar 2's first half)", [(DOOR,
      "let own = guest.host == me && matches!(lend.source, Source::Port(_));",
      "let own = true;")]),
    ("a lapsed visit still opening: the door not asking whether it is live (bar 4)", [(DOOR,
      "let live = guest.live(&register.cut, now);\n    let gone = register.cut.get(&guest.host).is_some_and(|c| c.at <= now);\n    let stopped = guest.stopped.is_some_and(|s| s <= now);",
      "let (live, gone, stopped) = (true, false, false);")]),
    ("the runtime's kept fold of standing keeping no Correct, so a stopped visit reads as live (bar 4, at Kim's Stop)", [(JOURNAL,
      "if read_by_standing(op) || self.named.contains(&op.actor) {",
      "if op.kind.act.moves_membership() || self.named.contains(&op.actor) {")]),
  ],
  "c": [
    ("the guest's key put in Standing (bar 3)", [(REGISTER,
      "    let removed = m\n",
      "    let mut members = members;\n    for g in m.visits.values() {\n        let host = members.get(&g.host).cloned();\n        host.map(|h| members.insert(g.visit.key.clone(), Member { person: g.visit.guest.clone(), ..h }));\n    }\n    let removed = m\n")]),
  ],
}
part = sys.argv[1]
for name, edits in PLANTS[part]:
    origs = {f: open(f).read() for f, _, _ in edits}
    try:
        print(f"## plant: {name}")
        for f, old, new in edits:
            text = open(f).read()
            assert text.count(old) == 1, (name, f)
            open(f, "w").write(text.replace(old, new))
            print(f"{f}\n-   " + old.replace("\n", "\n-   ") + "\n+   " + new.replace("\n", "\n+   "))
        r = subprocess.run(["cargo", "test", "-q", "-p", "svrngs-node", "--test", "door", "guests"], capture_output=True, text=True)
        out = r.stdout + r.stderr
        keep = [l for l in out.splitlines() if l.startswith(("test result", "error", "guests::")) or "panicked" in l
                or l.strip().startswith(("left:", "right:", "assertion")) or "x-svrngs-" in l or l.startswith("{") or l.strip().startswith('"')]
        print("\n".join(keep) or out[-2000:])
        print(f"exit {r.returncode}\n", flush=True)
    finally:
        for f, text in origs.items():
            open(f, "w").write(text)
