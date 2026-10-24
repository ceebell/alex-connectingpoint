#!/usr/bin/env bash
# Deploy the LINE backend and MongoDB beside an existing host Caddy service.
# Usage: sudo bash ./deploy-line.sh --init-swarm  (first deployment)
#        sudo bash ./deploy-line.sh               (subsequent deployments)
set -euo pipefail
cd "$(dirname "${BASH_SOURCE[0]}")"

# Fail before Docker/Swarm mutations if the LOCAL release is missing or invalid.
release_tag="$(bash scripts/verify-line-release.sh)"

init_swarm=0
if [ "${1:-}" = "--init-swarm" ]; then
  init_swarm=1
  shift
fi
if [ "$#" -ne 0 ]; then
  echo "Usage: bash deploy-line.sh [--init-swarm]" >&2
  exit 1
fi
if [ ! -f .env ]; then
  echo "[ERROR] Create the server .env before deploying." >&2
  exit 1
fi
set -a
# shellcheck disable=SC1091
. ./.env
set +a

for name in LINE_SHOP_ID LINE_CHANNEL_ID LINE_DESTINATION_ID LINE_CHANNEL_SECRET \
  LINE_CHANNEL_ACCESS_TOKEN INQUIRY_ADMIN_TOKEN MONGO_USER MONGO_PASSWORD; do
  if [ -z "${!name:-}" ]; then
    echo "[ERROR] Missing configuration: $name" >&2
    exit 1
  fi
done
if [[ ! "$MONGO_USER" =~ ^[a-zA-Z0-9]+$ ]] || [[ ! "$MONGO_PASSWORD" =~ ^[a-zA-Z0-9]+$ ]]; then
  echo "[ERROR] MONGO_USER and MONGO_PASSWORD must be alphanumeric for this URI format." >&2
  exit 1
fi
APP_NAME="${APP_NAME:-alex-line}"
STACK="${STACK:-$APP_NAME}"
REGISTRY="${REGISTRY:-}"
LINE_HTTP_PORT="${LINE_HTTP_PORT:-18080}"
if [[ ! "$STACK" =~ ^[a-zA-Z0-9][a-zA-Z0-9_-]*$ ]]; then
  echo "[ERROR] Invalid stack name." >&2
  exit 1
fi
if [[ ! "$LINE_HTTP_PORT" =~ ^[0-9]+$ ]] || (( LINE_HTTP_PORT < 1 || LINE_HTTP_PORT > 65535 )); then
  echo "[ERROR] Invalid LINE_HTTP_PORT." >&2
  exit 1
fi

TAG="$release_tag"

swarm_state="$(docker info --format '{{.Swarm.LocalNodeState}}')"
if [ "$swarm_state" = inactive ] && [ "$init_swarm" -eq 1 ]; then
  if [ -z "${SWARM_ADVERTISE_ADDR:-}" ]; then
    echo "[ERROR] Set SWARM_ADVERTISE_ADDR in .env before initializing Swarm." >&2
    exit 1
  fi
  # Swarm init prints join tokens; suppress them from the deployment log.
  docker swarm init --advertise-addr "$SWARM_ADVERTISE_ADDR" >/dev/null
elif [ "$swarm_state" != active ]; then
  echo "[ERROR] Swarm is not active. Use --init-swarm for the first deployment." >&2
  exit 1
fi
if [ "$(docker info --format '{{.Swarm.ControlAvailable}}')" != true ]; then
  echo "[ERROR] Run this script on the Swarm manager that hosts Caddy." >&2
  exit 1
fi
LINE_NODE_ID="$(docker info --format '{{.Swarm.NodeID}}')"
export APP_NAME STACK REGISTRY TAG LINE_NODE_ID LINE_HTTP_PORT
image="${REGISTRY}${APP_NAME}-backend:${TAG}"

# Validate without printing interpolated credentials.
docker stack config -c docker-stack.line.yml >/dev/null
echo "==> Packaging precompiled local release $TAG into $image (no compilation)"
docker build -f docker/backend.Dockerfile -t "$image" .
if [ -n "$REGISTRY" ]; then
  docker push "$image"
fi
docker pull mongo:7
echo "==> Deploying stack $STACK"
# All services are pinned to this node, where the local image has just been built.
docker stack deploy -c docker-stack.line.yml --resolve-image never --with-registry-auth "$STACK"

echo "==> Waiting for backend health on port $LINE_HTTP_PORT"
for ((attempt=0; attempt<120; attempt++)); do
  if response="$(curl --noproxy '*' -fsS --max-time 3 "http://127.0.0.1:$LINE_HTTP_PORT/api/health" 2>/dev/null)" \
    && [ "$response" = '{"status":"ok"}' ]; then
    docker stack services "$STACK"
    echo "[OK] Backend is healthy: http://127.0.0.1:$LINE_HTTP_PORT/api/health"
    echo "Webhook: https://hermes.salepie.online/api/line/webhook"
    exit 0
  fi
  sleep 2
done
echo "[ERROR] Backend did not become healthy. Inspect the service tasks below." >&2
docker service ps --no-trunc "${STACK}_backend"
docker service ps --no-trunc "${STACK}_mongo"
exit 1
