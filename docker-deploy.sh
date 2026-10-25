#!/usr/bin/env bash
# Deploy prebuilt runtime images without compiling on Ubuntu.
set -euo pipefail
cd "$(dirname "${BASH_SOURCE[0]}")"
if [ "${1:-}" = '--full-stack' ]; then
  shift
  exec bash ./deploy-line.sh --skip-build --with-admin "$@"
fi
exec bash ./deploy-line.sh --skip-build "$@"
