#!/usr/bin/env bash
# Check the frontend that was built on the LOCAL computer for Linux.
set -euo pipefail
release_root="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
release_dir="$release_root/dist/frontend-linux"
for file in server.js frontend.platform frontend.tag frontend.sha256 frontend-release.json; do
  if [ ! -f "$release_dir/$file" ]; then
    echo "[ERROR] Missing frontend Linux release: $file. Run build-frontend.bat on the LOCAL computer first." >&2
    exit 1
  fi
done
if [ "$(cat "$release_dir/frontend.platform")" != linux/amd64 ]; then
  echo '[ERROR] Frontend release must target Linux/amd64.' >&2
  exit 1
fi
if [[ ! "$(cat "$release_dir/frontend.tag")" =~ ^front-[0-9]{14}-[a-f0-9]{12}$ ]]; then
  echo '[ERROR] Frontend release tag is invalid.' >&2
  exit 1
fi
if ! (cd "$release_dir" && sha256sum --check --status frontend.sha256); then
  echo '[ERROR] Frontend checksum mismatch. Build locally and push the complete release again.' >&2
  exit 1
fi
echo '[OK] Frontend Linux release checksums verified.'
