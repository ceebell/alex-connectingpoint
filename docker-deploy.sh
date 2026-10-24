#!/usr/bin/env bash
# docker-deploy.sh - deploy / update the Docker Swarm stack (run on the Swarm manager)
#
# Usage:
#   ./docker-deploy.sh                # deploy using the locally compiled release tag
#   STACK=myapp-staging ./docker-deploy.sh
#
# Requires: a swarm (docker swarm init) and a server-side .env (see .env.example)
set -euo pipefail

cd "$(dirname "${BASH_SOURCE[0]}")"

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
STACK="${STACK:-$APP_NAME}"
export APP_NAME REGISTRY TAG

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
