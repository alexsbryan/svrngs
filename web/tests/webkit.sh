#!/bin/sh
# SPDX-License-Identifier: AGPL-3.0-or-later
# Playwright's WebKit on macOS, as page.mjs `starting` has it start: with
# AppKit's window animations off. Each page WebKit opens there animates its
# window in on a thread of a pool of 64, and with the display off the thread
# never comes back, so a browser's 64th page never loads
# (demo/runs/webkit-wall/2026-11-03). Playwright refuses a browser argument
# that doesn't start with '-', so the default's `NO` is added here, after
# Playwright's own arguments. Playwright.app opens its last argument that
# doesn't start with "--" as its first page, so `about:blank` ends the line:
# the first page Playwright gives a profile's browser, where `NO` would be
# opened and a profile's launch would wait on it for ever.
# SVRNGS_WEBKIT is Playwright's own launcher.
exec "$SVRNGS_WEBKIT" "$@" -NSAutomaticWindowAnimationsEnabled NO about:blank
