# Install svrngs with pacman

This folder builds svrngs as a pacman package from its signed GitHub
release, so pacman installs, updates and removes it like the rest of your
system. It builds Give beside it, for a computer that keeps apps for its
groups.

## Build and install

You need this repository and `base-devel`. From the repository's root,
import the key svrngs releases are signed with:

    gpg --import svrngs-release-key.asc

Its fingerprint is `46B4D67C5E6621093BB1AB2B057BF9082AA03CFB`, as
[RELEASES.md](../../RELEASES.md) says. Then build svrngs, install it, and
keep it running whenever you log in:

    cd packaging/arch
    makepkg
    sudo pacman -U "$(makepkg --packagelist | grep /svrngs-bin-)"
    systemctl --user enable --now svrngs

makepkg downloads svrngs from the release and refuses it unless its
signature is by that key and its checksum is the one the release lists.
`svrngs open`, or svrngs in your applications menu, opens your home page.

To keep apps for your groups too, run `makepkg -si` instead of the two
lines after `cd`. It installs svrngs and Give, and Podman, which Give runs
apps in. Then run `systemctl --user enable --now svrngs give`.

## What pacman owns

The `svrngs-bin` package owns `/usr/bin/svrngs`, its service
`/usr/lib/systemd/user/svrngs.service`, its entry in your applications
menu `/usr/share/applications/svrngs.desktop`, and
`/usr/lib/svrngs/updated-by`. That last file tells svrngs that pacman
updates it, so svrngs never replaces itself. That's the trade:
`svrngs update` says how to update instead of updating.

The `svrngs-give-bin` package owns `/usr/bin/give` and
`/usr/lib/systemd/user/give.service`.

Your groups, lends and keys in `~/.svrngs`, and your groups' folders in
`~/svrngs`, are yours, not pacman's. Installing, updating and removing
leave them as they are.

## Update

Pull this repository, build and install the same way, then run
`svrngs restart`.

## Remove

    systemctl --user disable --now svrngs
    sudo pacman -R svrngs-bin

With Give, run `systemctl --user disable --now svrngs give`, then
`sudo pacman -Rs svrngs-give-bin svrngs-bin`. The `-s` also removes
Podman, if nothing else needs it.

## If the install line put svrngs here first

Move that svrngs aside before you install the package, so the packaged one
runs in its place, at the same home page address:

    systemctl --user disable --now svrngs
    mv ~/.local/bin/svrngs ~/.local/bin/svrngs.before-package
    mv ~/.config/systemd/user/svrngs.service ~/.config/systemd/user/svrngs.service.before-package
    mv ~/.local/share/applications/svrngs.desktop ~/.local/share/applications/svrngs.desktop.before-package

If you added Give with `--give`, also run
`systemctl --user disable --now give`, then move `~/.local/bin/give` and
`~/.config/systemd/user/give.service` aside the same way.
