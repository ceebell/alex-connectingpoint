#!/usr/bin/env bash
# deploy-all.sh - pull the latest release, package runtime images and deploy the full stack in one go.
# Same as running, in order:
#   bash ./loadgit.sh <branch>
#   sudo bash ./docker-build.sh --full-stack
#   sudo bash ./docker-deploy.sh --full-stack
#
# Usage (Ubuntu):
#   bash ./deploy-all.sh              # branch "main" (asks for the sudo password once)
#   bash ./deploy-all.sh develop      # another branch
#   FORCE=1 bash ./deploy-all.sh      # discard local server edits (see loadgit.sh)
# Never compiles on the server: it only uses the precompiled releases in dist/.
set -euo pipefail

# Wrapped in main so bash parses the whole file before loadgit.sh replaces it with a newer version.
main() {
  cd "$(dirname "${BASH_SOURCE[0]}")"

  if [ "$#" -gt 1 ]; then
    echo 'Usage: bash deploy-all.sh [branch]' >&2
    exit 1
  fi
  local branch="${1:-${BRANCH:-main}}"

  # Git runs as the repo owner (root-owned .git files break later pulls); Docker runs as root.
  local owner
  owner="$(stat -c %U .)"
  as_owner() {
    if [ "$(id -u)" -eq 0 ] && [ "$owner" != root ]; then
      sudo -u "$owner" env FORCE="${FORCE:-0}" REMOTE="${REMOTE:-origin}" "$@"
    else
      "$@"
    fi
  }
  as_root() {
    if [ "$(id -u)" -eq 0 ]; then "$@"; else sudo "$@"; fi
  }

  # Ask for the sudo password up front so the run is not interrupted after the pull.
  if [ "$(id -u)" -ne 0 ]; then sudo -v; fi

  echo "==> [1/3] Pulling $branch"
  as_owner bash ./loadgit.sh "$branch"

  echo
  echo "==> [2/3] Packaging runtime images (backend + frontend)"
  as_root bash ./docker-build.sh --full-stack

  echo
  echo "==> [3/3] Deploying full stack"
  as_root bash ./docker-deploy.sh --full-stack

  echo
  echo "[OK] Deploy complete"
  echo "    backend:  $(cat dist/line-linux/release.tag)"
  echo "    frontend: $(cat dist/frontend-linux/frontend.tag)"
}

main "$@"; exit
