#!/usr/bin/env bash
# loadgit.sh - pull the latest code onto the Ubuntu server (run BEFORE docker-build.sh)
#
# Usage:
#   ./loadgit.sh              # pull branch "main"
#   ./loadgit.sh develop      # pull another branch
#   FORCE=1 ./loadgit.sh      # discard local changes on the server (git reset --hard origin/<branch>)
#
# First run on a fresh clone:  chmod +x loadgit.sh
set -euo pipefail

cd "$(dirname "${BASH_SOURCE[0]}")"

BRANCH="${1:-${BRANCH:-main}}"
REMOTE="${REMOTE:-origin}"

if ! git rev-parse --is-inside-work-tree >/dev/null 2>&1; then
  echo "[ERROR] $(pwd) is not a git repository. Clone the repo first." >&2
  exit 1
fi

# Windows commits do not carry the executable bit, so ignore file-mode changes on the server
git config core.fileMode false

echo "==> Fetching $REMOTE ..."
git fetch --prune "$REMOTE"

if ! git show-ref --verify --quiet "refs/remotes/$REMOTE/$BRANCH"; then
  echo "[ERROR] Branch '$BRANCH' not found on $REMOTE." >&2
  exit 1
fi

if [ -n "$(git status --porcelain)" ]; then
  if [ "${FORCE:-0}" = "1" ]; then
    echo "==> FORCE=1: discarding local changes"
    git reset --hard
  else
    echo "[ERROR] Local changes found on the server:" >&2
    git status --short >&2
    echo "Commit/stash them, or run: FORCE=1 ./loadgit.sh $BRANCH" >&2
    exit 1
  fi
fi

echo "==> Checking out $BRANCH ..."
git checkout "$BRANCH"

if [ "${FORCE:-0}" = "1" ]; then
  git reset --hard "$REMOTE/$BRANCH"
else
  git pull --ff-only "$REMOTE" "$BRANCH"
fi

chmod +x ./*.sh

echo
echo "==> Now at:"
git log -1 --format='    %h  %ad  %an%n    %s' --date=format:'%Y-%m-%d %H:%M'

if [ ! -f .env ]; then
  echo
  echo "[WARN] .env not found. Create it before deploying:  cp .env.example .env  &&  nano .env"
fi

echo
echo "Next:  ./docker-build.sh  &&  ./docker-deploy.sh"
