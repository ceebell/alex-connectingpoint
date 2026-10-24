#!/usr/bin/env bash
# Package locally compiled Linux release objects into runtime images. Never compile here.
set -euo pipefail
cd "$(dirname "${BASH_SOURCE[0]}")"
release_tag="$(bash scripts/verify-line-release.sh)"
if [ ! -f dist/frontend-linux/server.js ]; then
  echo "[ERROR] Full-stack packaging requires a frontend standalone release built LOCALLY for Linux in dist/frontend-linux." >&2
  echo "For the current backend-only LINE release, use sudo bash ./deploy-line.sh instead." >&2
  exit 1
fi
if [ -f .env ]; then
  set -a
  . ./.env
  set +a
fi
APP_NAME="${APP_NAME:-myapp}"
REGISTRY="${REGISTRY:-}"
TAG="$release_tag"
for service in backend frontend; do
  image="${REGISTRY}${APP_NAME}-${service}:${TAG}"
  echo "==> Packaging $image from precompiled objects"
  docker build -f "docker/${service}.Dockerfile" -t "$image" .
  if [ "${PUSH:-0}" = 1 ]; then docker push "$image"; fi
done
echo "[OK] Runtime images packaged. Next: sudo bash ./docker-deploy.sh"
