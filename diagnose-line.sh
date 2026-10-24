#!/usr/bin/env bash
# Read-only LINE deployment diagnostics. Never print configuration secrets.
set -euo pipefail
cd "$(dirname "${BASH_SOURCE[0]}")"
if [ ! -f .env ]; then
  echo '[ERROR] Server .env is missing.' >&2
  exit 1
fi
set -a
. ./.env
set +a
mkdir -p data
report_file="$(mktemp data/line-diagnostics.XXXXXX)"
# This report contains only the whitelisted, sanitized diagnostic fields.
chmod 644 "$report_file"
status=0
python3 scripts/diagnose-line.py | tee "$report_file" || status=$?
mv -f "$report_file" data/line-diagnostics-latest.txt
exit "$status"
