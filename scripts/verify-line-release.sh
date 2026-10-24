#!/usr/bin/env bash
# Verify precompiled input before contacting Docker or changing Swarm state.
set -euo pipefail
release_root="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
release_dir="$release_root/dist/line-linux"
for file in backend backend.sha256 release.tag; do
  if [ ! -f "$release_dir/$file" ]; then
    echo "[ERROR] Missing Linux release: $file. Run build-line.bat on the LOCAL computer and upload the release archive first." >&2
    exit 1
  fi
done
release_tag="$(cat "$release_dir/release.tag")"
if [[ ! "$release_tag" =~ ^rel-[0-9]{14}-[a-f0-9]{12}$ ]]; then
  echo "[ERROR] Invalid release tag." >&2
  exit 1
fi
checksum_line="$(cat "$release_dir/backend.sha256")"
if [[ ! "$checksum_line" =~ ^[a-f0-9]{64}[[:space:]][[:space:]]backend$ ]] \
  || ! (cd "$release_dir" && sha256sum --check --status backend.sha256); then
  echo "[ERROR] Release checksum mismatch. Upload the complete local release again." >&2
  exit 1
fi
elf_header="$(od -An -tx1 -N20 "$release_dir/backend" | tr -d ' \n')"
if [[ "$elf_header" != 7f454c46020101* ]] || [ "${elf_header:36:4}" != 3e00 ]; then
  echo "[ERROR] Expected Linux x86_64 ELF release, not a Windows .exe or ARM binary." >&2
  exit 1
fi
if [ "$(uname -m)" != x86_64 ]; then
  echo "[ERROR] This release requires an x86_64 server." >&2
  exit 1
fi
printf '%s\n' "$release_tag"
