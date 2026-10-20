import subprocess, sys
F = "crates/svrngs-core/src/membership.rs"
PLANTS = [
  ("the fold counting no guests",
   "let is = !used && timely && on.len() + 1 < members && theirs < GUESTS_PER_HOST;",
   "let is = !used && timely;"),
  ("until unchecked",
   "let timely = (1..=VISIT_LONGEST).contains(&v.until.saturating_sub(at));",
   "let timely = true;"),
  ("a host's leave leaving the visit live",
   "let cut = cut.get(&self.host).map(|c| c.at);",
   "let cut = None;"),
  ("a bound read from the page rather than the fold: the length the page chose is the bound",
   "let timely = (1..=VISIT_LONGEST).contains(&v.until.saturating_sub(at));",
   "let timely = (1..=v.until.saturating_sub(at)).contains(&v.until.saturating_sub(at));"),
  ("the Stop unread",
   "visit.map(|g| g.stopped.get_or_insert(op.ts_unix));",
   "visit.map(|g| g.stopped);"),
  ("anyone's Correct a Stop",
   "let visit = visits.get_mut(corrects).filter(|g| g.host == op.actor);",
   "let visit = visits.get_mut(corrects);"),
  ("the key's freshness unread",
   "visits.values().any(|g| g.visit.key == v.key) || bindings.contains_key(&v.key);",
   "false;"),
  ("the register's fold leaving out the Stop",
   "let kept = |a: &RailAct| a.moves_membership() || matches!(a, RailAct::Correct { .. });",
   "let kept = |a: &RailAct| a.moves_membership();"),
  ("the guest's key put in standing",
   "            visits.insert(id.clone(), g);",
   "            live.insert(g.visit.key.clone(), g.visit.guest.clone());\n            visits.insert(id.clone(), g);"),
]
orig = open(F).read()
try:
    for name, old, new in PLANTS:
        assert orig.count(old) == 1, name
        open(F, "w").write(orig.replace(old, new))
        r = subprocess.run(["cargo", "test", "-q", "-p", "svrngs-core", "--lib", "visit_tests"], capture_output=True, text=True)
        out = r.stdout + r.stderr
        lines = [l for l in out.splitlines() if l.startswith(("test membership", "test result", "error")) or "panicked" in l or l.strip().startswith(("left:", "right:")) or "failed:" in l]
        print(f"## plant: {name}\n-   {old}\n+   {new}")
        print("\n".join(lines) or out[-2000:])
        print(f"exit {r.returncode}\n")
finally:
    open(F, "w").write(orig)
