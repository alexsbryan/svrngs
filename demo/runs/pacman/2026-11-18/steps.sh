#!/bin/bash
# The pacman run's steps, run in an Omarchy guest as its person, cy:
#   steps.sh <step>
# from ~/svrngs-src, which holds `git archive <commit> svrngs-release-key.asc
# packaging/arch`. Each command is printed before it runs, then its output
# and exit status. --noconfirm is the only word added to README.md's lines:
# nobody is at the guest's terminal to answer pacman.
cd ~/svrngs-src || exit 1
run() { printf '$ %s\n' "$*"; bash -c "$*" 2>&1; printf '[exit %s]\n\n' "$?"; }
# A copy of the package's own files, none of its downloads or packages,
# with one value of the PKGBUILD changed by sed.
plant() {
  run "rm -rf ~/plant && mkdir ~/plant && cd packaging/arch && cp PKGBUILD *.install *.service *.desktop updated-by ~/plant && sed -i '$1' ~/plant/PKGBUILD && diff PKGBUILD ~/plant/PKGBUILD"
  run 'cd ~/plant && makepkg -f --noconfirm'
  run 'ls ~/plant/*.pkg.tar.zst'
}
case ${1:?step} in
lab)
  # The lab's, not the package's: sudo without a password for this boot,
  # since no one types at the guest; no idle lock; and pacman's sync
  # databases, which the lab's base image has none of.
  run 'echo omarchy | sudo -S -p "" sh -c "echo \"cy ALL=(ALL) NOPASSWD: ALL\" > /etc/sudoers.d/90-lab"'
  run 'pkill -x hypridle; pgrep -x hypridle || echo "hypridle stopped"'
  run 'sudo pacman -Sy'
  run 'pacman -Qu'
  ;;
route)
  # VM a's net.sh routed it through VM c, which is off: its default route
  # put back on its own NIC, QEMU's user network.
  run 'echo omarchy | sudo -S -p "" ip route add default via 10.0.2.2 dev enp0s4; ip route | head -2'
  ;;
before)
  run 'grep -E "^(NAME|BUILD_ID|VERSION_ID)=" /etc/os-release; pacman -Q omarchy pacman gnupg; uname -r'
  run 'command -v svrngs give podman; ls -A ~/.svrngs ~/.local/bin ~/.config/systemd/user'
  run 'sha256sum svrngs-release-key.asc packaging/arch/* packaging/arch/.gitignore'
  ;;
build)
  run 'gpg --import svrngs-release-key.asc'
  run 'gpg --fingerprint 46B4D67C5E6621093BB1AB2B057BF9082AA03CFB'
  run 'cd packaging/arch && makepkg --printsrcinfo'
  run 'cd packaging/arch && makepkg -si --noconfirm'
  ;;
version)
  run 'command -v svrngs give'
  run 'svrngs version'
  ;;
update)
  run 'sha256sum /usr/bin/svrngs; pacman -Qkk svrngs-bin; ls -A ~/.svrngs'
  run 'svrngs update'
  run 'sha256sum /usr/bin/svrngs; pacman -Qkk svrngs-bin; ls -A ~/.svrngs'
  ;;
service)
  run 'systemctl --user enable --now svrngs'
  run 'sleep 3; systemctl --user is-active svrngs; systemctl --user status svrngs --no-pager | head -6'
  run 'svrngs open --print'
  run 'curl -sS -o /dev/null -w "home page: %{http_code}\n" "$(cat ~/.svrngs/api)/"'
  run 'ls -A ~/.svrngs; svrngs update; ls -A ~/.svrngs; sha256sum /usr/bin/svrngs'
  run 'systemctl --user enable --now give'
  run 'sleep 3; systemctl --user is-active give; journalctl --user -u give -b --no-pager | tail -3'
  run 'svrngs restart; sleep 3; systemctl --user is-active svrngs'
  run 'systemd-analyze verify --user /usr/lib/systemd/user/svrngs.service /usr/lib/systemd/user/give.service'
  run 'desktop-file-validate /usr/share/applications/svrngs.desktop'
  run 'pacman -Qi svrngs-bin svrngs-give-bin'
  run 'pacman -Ql svrngs-bin svrngs-give-bin'
  run 'pacman -Qkk svrngs-bin svrngs-give-bin'
  ;;
namcap)
  run 'sudo pacman -S --noconfirm --needed namcap >/dev/null && cd packaging/arch && namcap PKGBUILD *.pkg.tar.zst'
  ;;
remove)
  run 'systemctl --user disable --now give svrngs'
  run 'sudo pacman -R --noconfirm svrngs-give-bin svrngs-bin'
  run 'pacman -Q svrngs-bin svrngs-give-bin; pacman -Qo /usr/bin/svrngs /usr/bin/give'
  run 'ls -d /usr/bin/svrngs /usr/bin/give /usr/lib/svrngs /usr/lib/systemd/user/svrngs.service /usr/lib/systemd/user/give.service /usr/share/applications/svrngs.desktop'
  run 'systemctl --user is-active svrngs give; ls ~/.config/systemd/user/default.target.wants; ls -A ~/.svrngs'
  run 'pacman -Qdtq'
  ;;
plant-key)
  # The release key's fingerprint, its last digit changed.
  plant 's/2AA03CFB/2AA03CFA/'
  ;;
plant-sum)
  # svrngs's x86_64 tarball sum, its first digit changed.
  plant 's/983a027f1c3a/883a027f1c3a/'
  ;;
before-move)
  run 'command -v svrngs; svrngs version 2>&1 | head -1; systemctl --user is-active svrngs; cat ~/.svrngs/api; echo; ls -A ~/.svrngs ~/svrngs'
  run 'ls ~/.local/bin/svrngs ~/.config/systemd/user/svrngs.service ~/.local/share/applications/svrngs.desktop; command -v podman'
  ;;
move)
  run 'systemctl --user disable --now svrngs'
  run 'mv ~/.local/bin/svrngs ~/.local/bin/svrngs.before-package'
  run 'mv ~/.config/systemd/user/svrngs.service ~/.config/systemd/user/svrngs.service.before-package'
  run 'mv ~/.local/share/applications/svrngs.desktop ~/.local/share/applications/svrngs.desktop.before-package'
  ;;
alone)
  run 'gpg --import svrngs-release-key.asc'
  run 'cd packaging/arch && makepkg'
  run 'cd packaging/arch && makepkg --packagelist'
  run 'cd packaging/arch && sudo pacman -U --noconfirm "$(makepkg --packagelist | grep /svrngs-bin-)"'
  run 'systemctl --user enable --now svrngs'
  run 'sleep 3; systemctl --user is-active svrngs; command -v svrngs; svrngs version; pacman -Q svrngs-bin svrngs-give-bin podman'
  run 'svrngs open --print; cat ~/.svrngs/api; echo; journalctl --user -u svrngs -b --no-pager | tail -6'
  run 'svrngs update; ls -A ~/.svrngs ~/svrngs'
  run 'systemctl --user disable --now svrngs'
  run 'sudo pacman -R --noconfirm svrngs-bin'
  run 'pacman -Q svrngs-bin; ls -d /usr/bin/svrngs /usr/lib/svrngs; ls -A ~/.svrngs ~/svrngs'
  ;;
*) echo "steps.sh: no step $1" >&2; exit 2 ;;
esac
