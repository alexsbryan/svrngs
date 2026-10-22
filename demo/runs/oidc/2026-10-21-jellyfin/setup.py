#!/usr/bin/env python3
"""What the run was made of, beside its data: pinned versions, the trust
floor, the lean check and the plugin settings' hash.

    setup.py <data dir> <unchanged|changed>
"""
import hashlib
import json
import sys
from pathlib import Path

out = Path(sys.argv[1])
floor = sys.argv[2]
setup = {
    "jellyfin": {
        "version": "10.11.11",
        "tar": "https://repo.jellyfin.org/files/server/macos/stable/v10.11.11/arm64/jellyfin_10.11.11-arm64.tar.xz",
        "sha256": "27c0bcbeac7a698cf0fd034e347f124f69083bc4463816099b52bebead614a47",
    },
    "plugin": {
        "name": "SSO-Auth",
        "version": "4.0.0.4",
        "zip": "https://github.com/9p4/jellyfin-plugin-sso/releases/download/v4.0.0.4/sso-authentication_4.0.0.4.zip",
        "sha256": "c09f16ba31059a434ddd7f811e4f9608d4b4c4514cc80a5bf1ca33bee61e1107",
    },
    "trust_floor": floor,
    "lean": (out / "lean.txt").read_text() if (out / "lean.txt").exists() else None,
    "plugin_settings_sha256": hashlib.sha256(
        (out / "plugin-settings.json").read_bytes()
    ).hexdigest()
    if (out / "plugin-settings.json").exists()
    else None,
}
(out / "setup.json").write_text(json.dumps(setup, indent=2) + "\n")
