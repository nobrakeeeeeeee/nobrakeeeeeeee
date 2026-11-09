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

    # macOS updater bundles are <App>.app.tar.gz; the release workflow
    # renames them to <App>.<arch>.app.tar.gz so arm64/x64 don't collide.
    for name in sorted(os.listdir(directory)):
        # Matches both <App>.app.tar.gz and <App>.app.<arch>.tar.gz while
        # excluding unrelated tarballs like deb's data.tar.gz.
        if not (name.endswith(".tar.gz") and ".app." in name):
            continue
        if ".aarch64." in name:
            key, suffix = "darwin-aarch64", "aarch64.app.tar.gz"
        elif ".x86_64." in name:
            key, suffix = "darwin-x86_64", "x64.app.tar.gz"
        else:
            # Back-compat: untagged artifact comes from the arm64 runner.
            key, suffix = "darwin-aarch64", "aarch64.app.tar.gz"
        add_platform(platforms, directory, key, os.path.join(directory, name), version, base_url, suffix)

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
