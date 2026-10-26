# Factorio

A factory your group builds together, kept by one of your computers at a
time. Each player needs their own copy of Factorio, at the version the card
names (2.1.23). On Steam, a player can stay on that version through the
game's betas.

The card runs `factoriotools/factorio` in its rootless build, pinned by
digest. The map is the image's `/factorio`. The image has no health
command, so the card checks that the game's port is bound.

On a computer with an arm64 processor, such as a Mac with Apple silicon,
the card runs the image's amd64 build. The arm64 build segfaulted making
the map (`demo/runs/give/2026-10-26-cards/`). On a Mac, Podman's machine
runs the amd64 build under Rosetta, and the Mac this was checked on had
Rosetta turned on in Podman. Podman 5.6 and later turn it off by default.
