#!/usr/bin/env bash
# docker-deploy.sh - deploy / update the Docker Swarm stack (run on the Swarm manager)
#
# Usage:
#   ./docker-deploy.sh                # deploy the LINE image built by docker-build.sh
#   ./docker-deploy.sh --init-swarm   # first LINE deployment
#   ./docker-deploy.sh --full-stack   # legacy backend + frontend stack
#   STACK=myapp-staging ./docker-deploy.sh
#
# Requires: a swarm (docker swarm init) and a server-side .env (see .env.example)
set -euo pipefail

cd "$(dirname "${BASH_SOURCE[0]}")"

# The current application is the LINE backend. Build its image separately first.
if [ "${1:-}" != '--full-stack' ]; then
  exec bash ./deploy-line.sh --skip-build "$@"
fi
shift
if [ "$#" -ne 0 ]; then
  echo 'Usage: bash docker-deploy.sh [--init-swarm] | --full-stack' >&2
  exit 1
fi

if [ ! -f .env ]; then
  echo "[ERROR] .env not found. Create it:  cp .env.example .env  &&  nano .env" >&2
  exit 1
fi

set -a
# shellcheck disable=SC1091
. ./.env
set +a

APP_NAME="${APP_NAME:-myapp}"
REGISTRY="${REGISTRY:-}"
TAG="$(bash scripts/verify-line-release.sh)"
bash scripts/verify-frontend-release.sh
FRONTEND_TAG="$(cat dist/frontend-linux/frontend.tag)"
for name in MONGO_USER MONGO_PASSWORD LINE_SHOP_ID LINE_CHANNEL_ID LINE_DESTINATION_ID \
  LINE_CHANNEL_SECRET LINE_CHANNEL_ACCESS_TOKEN INQUIRY_ADMIN_TOKEN; do
  if [ -z "${!name:-}" ]; then
    echo "[ERROR] Missing configuration: $name" >&2
    exit 1
  fi
done
STACK="${STACK:-$APP_NAME}"
export APP_NAME REGISTRY TAG FRONTEND_TAG

for service in backend frontend; do
  service_tag="$TAG"
  expected_hash="$(cut -d ' ' -f 1 dist/line-linux/backend.sha256)"
  if [ "$service" = frontend ]; then
    service_tag="$FRONTEND_TAG"
    expected_hash="$(sha256sum dist/frontend-linux/frontend.sha256 | cut -d ' ' -f 1)"
  fi
  image="${REGISTRY}${APP_NAME}-${service}:${service_tag}"
  image_hash="$(docker image inspect "$image" --format '{{ index .Config.Labels "com.alex.release.sha256" }}' 2>/dev/null || true)"
  if [ "$image_hash" != "$expected_hash" ]; then
    echo '[ERROR] A runtime image is missing or does not match the release. Run sudo bash ./docker-build.sh --full-stack first.' >&2
    exit 1
  fi
done

if [ "$(docker info --format '{{.Swarm.LocalNodeState}}' 2>/dev/null)" != "active" ]; then
  echo "[ERROR] This node is not part of a swarm. Run once:  docker swarm init" >&2
  echo "        (add --advertise-addr <server-ip> if the server has several network interfaces)" >&2
  exit 1
fi

echo "==> Deploying stack '${STACK}' (tag: ${TAG})"
docker stack deploy \
  -c docker-stack.yml \
  --with-registry-auth \
  --prune \
  "$STACK"

echo
docker stack services "$STACK"
echo
echo "Watch rollout :  docker service ps ${STACK}_backend"
echo "Logs          :  docker service logs -f ${STACK}_backend"
echo "Roll back     :  docker service rollback ${STACK}_backend"
