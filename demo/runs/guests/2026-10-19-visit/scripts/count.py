import sys,subprocess
def n(text): return sum(1 for l in text.splitlines() if l.strip() and not l.strip().startswith(("//","#","/*","*")))
for f in ["crates/svrngs-core/src/lib.rs","crates/svrngs-core/src/membership.rs","crates/svrngs-core/src/admit.rs"]:
    old=subprocess.run(["git","show",f"HEAD:{f}"],capture_output=True,text=True).stdout
    new=open(f).read()
    print(f, n(new)-n(old))
