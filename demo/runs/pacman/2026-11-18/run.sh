#!/bin/bash
# The pacman run of 2026-11-18, driven from the Mac: packaging/arch at
# <commit>, on two Omarchy guests in the Fedora box's lab (~/omarchy-lab,
# lab.sh and g). Each step's command and output is <n>-<step>.log here.
#   demo/runs/pacman/2026-11-18/run.sh <commit>
set -euo pipefail
commit=${1:?commit}
here=$(cd "$(dirname "$0")" && pwd)
repo=$(git -C "$here" rev-parse --show-toplevel)
fedora() { ssh alexbryan@fedora "$@"; }
# boot <vm> <vnc> <ssh port>: -snapshot sends the guest's writes to a
# temporary file, so its disk is as it was once it stops; then the commit's
# files and steps.sh are put in the guest.
boot() {
  fedora "cd ~/omarchy-lab && ./lab.sh boot $1 $2 $3 -- -snapshot && for i in \$(seq 1 80); do ./g $3 -p true 2>/dev/null && break; sleep 3; done"
  git -C "$repo" archive "$commit" svrngs-release-key.asc packaging/arch |
    fedora "~/omarchy-lab/g $3 -p 'rm -rf ~/svrngs-src && mkdir ~/svrngs-src && tar -xf - -C ~/svrngs-src'"
  fedora "~/omarchy-lab/g $3 -p 'cat > ~/steps.sh && chmod +x ~/steps.sh'" <"$here/steps.sh"
}
# step <ssh port> <what the VM is> <log> <step>
step() {
  {
    printf '# steps.sh %s, %s\n# VM: %s\n# commit: %s (git archive %s svrngs-release-key.asc packaging/arch)\n\n' \
      "$4" "$(date -u +%FT%TZ)" "$2" "$commit" "$commit"
    fedora "~/omarchy-lab/g $1 -p '~/steps.sh $4'"
  } >"$here/$3.log" 2>&1
}

# A clean computer: an overlay of the lab's Omarchy 4.5.0 install with
# nothing of svrngs on it.
pac="pac, a new overlay on the lab's Omarchy 4.5.0 base.qcow2 (svrngs never installed), booted -snapshot; guest user cy"
boot pac 8 2333
n=0
for s in lab before build version update service namcap remove plant-key plant-sum; do
  step 2333 "$pac" "$(printf %02d $n)-$s" "$s"
  n=$((n + 1))
done
fedora "cd ~/omarchy-lab && ./lab.sh stop pac"

# A computer with svrngs from the install line: the lab's VM a.
a="a, the lab's VM a: Omarchy 4.5.0 with svrngs from the install line (6 November, an older svrngs), booted -snapshot; guest user cy"
boot a 6 2331
for s in route lab before-move move alone; do
  step 2331 "$a" "$(printf %02d $n)-$s" "$s"
  n=$((n + 1))
done
fedora "cd ~/omarchy-lab && ./lab.sh stop a"
