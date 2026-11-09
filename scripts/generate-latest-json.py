#!/usr/bin/env python3
"""Generate latest.json for the Tauri updater from built bundle artifacts.

Usage: generate-latest-json.py <release-files-dir>

Expects the directory to already contain the installer artifacts (flat).
For each updater-supported platform it copies the artifact to a
space-free filename and writes latest.json alongside it.
"""

import json
import os
import shutil
import sys
from datetime import datetime, timezone

APP = "no-brakeeeeeeee"


def find_artifact(directory: str, *suffixes: str) -> str | None:
    for name in sorted(os.listdir(directory)):
        if name.endswith(suffixes):
            return os.path.join(directory, name)
    return None


def add_platform(
    platforms: dict,
    directory: str,
    key: str,
    artifact: str,
    version: str,
    base_url: str,
    asset_suffix: str,
) -> None:
    sig_path = artifact + ".sig"
    if not os.path.exists(sig_path):
        print(f"warning: missing signature for {artifact}", file=sys.stderr)
        return

    asset_name = f"{APP}_{version}_{asset_suffix}"
    dest = os.path.join(directory, asset_name)
    if os.path.abspath(artifact) != os.path.abspath(dest):
        shutil.copyfile(artifact, dest)

    with open(sig_path, encoding="utf-8") as f:
        signature = f.read().strip()

    platforms[key] = {
        "signature": signature,
        "url": f"{base_url}/{asset_name}",
    }
    print(f"{key}: {asset_name}")


def main() -> int:
    if len(sys.argv) != 2:
        print("usage: generate-latest-json.py <release-files-dir>", file=sys.stderr)
        return 1

    directory = sys.argv[1]
    tag = os.environ.get("GITHUB_REF_NAME", "")
    repo = os.environ.get("GITHUB_REPOSITORY", "")
    if not tag or not repo:
        print("GITHUB_REF_NAME and GITHUB_REPOSITORY must be set", file=sys.stderr)
        return 1

    version = tag.lstrip("v")
    base_url = f"https://github.com/{repo}/releases/download/{tag}"
    platforms: dict = {}

    artifact = find_artifact(directory, ".AppImage")
    if artifact:
        add_platform(platforms, directory, "linux-x86_64", artifact, version, base_url, "amd64.AppImage")

    artifact = find_artifact(directory, "-setup.exe")
    if artifact:
        add_platform(platforms, directory, "windows-x86_64", artifact, version, base_url, "x64-setup.exe")

    # macOS updater bundle is <App>.app.tar.gz; filenames don't carry the
    # arch. The release matrix currently builds macos-latest (arm64) only,
    # so map it to darwin-aarch64. If an Intel macOS target is added,
    # distinguish by runner/arch here.
    artifact = find_artifact(directory, ".app.tar.gz")
    if artifact:
        add_platform(platforms, directory, "darwin-aarch64", artifact, version, base_url, "aarch64.app.tar.gz")

    if not platforms:
        print("no updater artifacts found", file=sys.stderr)
        return 1

    manifest = {
        "version": version,
        "notes": "",
        "pub_date": datetime.now(timezone.utc).strftime("%Y-%m-%dT%H:%M:%SZ"),
        "platforms": platforms,
    }
    out = os.path.join(directory, "latest.json")
    with open(out, "w", encoding="utf-8") as f:
        json.dump(manifest, f, indent=2)
    print(f"wrote {out}")
    return 0


if __name__ == "__main__":
    sys.exit(main())
