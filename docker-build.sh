#!/usr/bin/env bash
# Package locally compiled Linux release objects into runtime images. Never compile here.
set -euo pipefail
cd "$(dirname "${BASH_SOURCE[0]}")"
full_stack=0
if [ "${1:-}" = '--full-stack' ]; then full_stack=1; shift; fi
if [ "$#" -ne 0 ]; then
  echo 'Usage: bash docker-build.sh [--full-stack]' >&2
  exit 1
fi
release_tag="$(bash scripts/verify-line-release.sh)"
if [ "$full_stack" -eq 1 ]; then
  bash scripts/verify-frontend-release.sh
  frontend_tag="$(cat dist/frontend-linux/frontend.tag)"
fi
if [ -f .env ]; then
  set -a
  . ./.env
  set +a
fi
APP_NAME="${APP_NAME:-alex-line}"
REGISTRY="${REGISTRY:-}"
TAG="$release_tag"
services=(backend)
if [ "$full_stack" -eq 1 ]; then services+=(frontend); fi
binary_hash="$(cut -d ' ' -f 1 dist/line-linux/backend.sha256)"
for service in "${services[@]}"; do
  service_tag="$TAG"
  release_hash="$binary_hash"
  if [ "$service" = frontend ]; then
    service_tag="$frontend_tag"
    release_hash="$(sha256sum dist/frontend-linux/frontend.sha256 | cut -d ' ' -f 1)"
  fi
  image="${REGISTRY}${APP_NAME}-${service}:${service_tag}"
  echo "==> Packaging $image from precompiled objects"
  docker build -f "docker/${service}.Dockerfile" \
    --label "com.alex.release.sha256=$release_hash" -t "$image" .
  if [ "${PUSH:-0}" = 1 ]; then docker push "$image"; fi
done
if [ "$full_stack" -eq 1 ]; then
  echo "[OK] Runtime images packaged. Next: sudo bash ./docker-deploy.sh --full-stack"
else
  echo "[OK] Backend image packaged. Next: sudo bash ./docker-deploy.sh"
fi
