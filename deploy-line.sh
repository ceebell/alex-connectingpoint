#!/usr/bin/env bash
# Deploy the LINE backend and MongoDB beside an existing host Caddy service.
# Usage: sudo bash ./deploy-line.sh --init-swarm  (first deployment)
#        sudo bash ./deploy-line.sh               (subsequent deployments)
set -euo pipefail
cd "$(dirname "${BASH_SOURCE[0]}")"

# Fail before Docker/Swarm mutations if the LOCAL release is missing or invalid.
release_tag="$(bash scripts/verify-line-release.sh)"

init_swarm=0
skip_build=0
with_admin=0
while [ "$#" -gt 0 ]; do
  case "$1" in
    --init-swarm) init_swarm=1 ;;
    --skip-build) skip_build=1 ;;
    --with-admin) with_admin=1 ;;
    *) echo 'Usage: bash deploy-line.sh [--init-swarm] [--skip-build] [--with-admin]' >&2; exit 1 ;;
  esac
  shift
done
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

stack_args=(-c docker-stack.line.yml)
expected_nodes=2
if [ "$with_admin" -eq 1 ]; then
  bash scripts/verify-frontend-release.sh
  FRONTEND_TAG="$(cat dist/frontend-linux/frontend.tag)"
  FRONTEND_PORT="${FRONTEND_PORT:-18081}"
  admin_password="${ADMIN_PASSWORD:-}"
  ADMIN_USERNAME="${ADMIN_USERNAME:-admin}"
  export ADMIN_USERNAME
  if [ "${#admin_password}" -lt 16 ]; then
    echo '[ERROR] Set ADMIN_USERNAME and ADMIN_PASSWORD (at least 16 characters) in the server .env.' >&2
    exit 1
  fi
  if [[ ! "$FRONTEND_PORT" =~ ^[0-9]+$ ]] || (( FRONTEND_PORT < 1 || FRONTEND_PORT > 65535 )) || [ "$FRONTEND_PORT" = "$LINE_HTTP_PORT" ]; then
    echo '[ERROR] FRONTEND_PORT must be valid and differ from LINE_HTTP_PORT.' >&2
    exit 1
  fi
  export FRONTEND_TAG FRONTEND_PORT
  stack_args+=(-c docker-stack.admin.yml)
  expected_nodes=3
fi


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
if [[ ! "$LINE_NODE_ID" =~ ^[a-z0-9]+$ ]]; then
  echo "[ERROR] Docker did not return a valid local Swarm node ID." >&2
  exit 1
fi
node_status="$(docker node inspect "$LINE_NODE_ID" --format '{{.Status.State}} {{.Spec.Availability}}')"
if [ "$node_status" != 'ready active' ]; then
  echo "[ERROR] The local Swarm node must be ready and active before deployment (current: $node_status)." >&2
  exit 1
fi
export APP_NAME STACK REGISTRY TAG LINE_NODE_ID LINE_HTTP_PORT
image="${REGISTRY}${APP_NAME}-backend:${TAG}"

# Legacy Swarm interpolation differs from Compose. Verify the rendered values,
# without printing or saving the configuration that contains credentials.
rendered_stack="$(docker stack config "${stack_args[@]}")"
node_matches="$(printf '%s\n' "$rendered_stack" | grep -Fc -- "          - node.id == $LINE_NODE_ID" || true)"
if [ "$node_matches" -ne "$expected_nodes" ] \
  || ! printf '%s\n' "$rendered_stack" | grep -Fxq -- "    image: $image"; then
  echo "[ERROR] Rendered stack does not match the local node ID or release image. No deployment was performed." >&2
  exit 1
fi
if [ "$with_admin" -eq 1 ] && ! printf '%s\n' "$rendered_stack" | grep -Fxq -- "    image: ${REGISTRY}${APP_NAME}-frontend:${FRONTEND_TAG}"; then
  echo '[ERROR] Rendered frontend image does not match the local release. No deployment was performed.' >&2
  exit 1
fi
unset rendered_stack
echo "==> Verified target node $LINE_NODE_ID and image $image"
if [ "$skip_build" -eq 0 ]; then
  if [ "$with_admin" -eq 1 ]; then bash ./docker-build.sh --full-stack; else bash ./docker-build.sh; fi
else
  expected_hash="$(cut -d ' ' -f 1 dist/line-linux/backend.sha256)"
  image_hash="$(docker image inspect "$image" --format '{{ index .Config.Labels "com.alex.release.sha256" }}' 2>/dev/null || true)"
  if [ "$image_hash" != "$expected_hash" ]; then
    echo "[ERROR] The runtime image is missing or does not match this release. Run sudo bash ./docker-build.sh first." >&2
    exit 1
  fi
  echo "==> Using the existing runtime image for release $TAG"
fi
if [ "$with_admin" -eq 1 ]; then
  frontend_image="${REGISTRY}${APP_NAME}-frontend:${FRONTEND_TAG}"
  expected_hash="$(sha256sum dist/frontend-linux/frontend.sha256 | cut -d ' ' -f 1)"
  image_hash="$(docker image inspect "$frontend_image" --format '{{ index .Config.Labels "com.alex.release.sha256" }}' 2>/dev/null || true)"
  if [ "$image_hash" != "$expected_hash" ]; then
    echo '[ERROR] Frontend image does not match the release. Run sudo bash ./docker-build.sh --full-stack first.' >&2
    exit 1
  fi
fi
if [ -n "$REGISTRY" ]; then
  docker push "$image"
  if [ "$with_admin" -eq 1 ]; then docker push "$frontend_image"; fi
fi
docker pull mongo:7
echo "==> Deploying stack $STACK"
# All services are pinned to this node, where the local image has just been built.
docker stack deploy "${stack_args[@]}" --resolve-image never --with-registry-auth "$STACK"

echo "==> Waiting for backend health on port $LINE_HTTP_PORT"
for ((attempt=0; attempt<120; attempt++)); do
  if response="$(curl --noproxy '*' -fsS --max-time 3 "http://127.0.0.1:$LINE_HTTP_PORT/api/health" 2>/dev/null)" \
    && [ "$response" = '{"status":"ok"}' ]; then
    docker stack services "$STACK"
    echo "[OK] Backend is healthy: http://127.0.0.1:$LINE_HTTP_PORT/api/health"
    echo "Webhook: https://hermes.salepie.online/api/line/webhook"
    if [ "$with_admin" -eq 1 ]; then
      echo "==> Waiting for admin frontend on port $FRONTEND_PORT"
      for ((front_attempt=0; front_attempt<60; front_attempt++)); do
        if curl --noproxy '*' -fsS --max-time 3 "http://127.0.0.1:$FRONTEND_PORT/" >/dev/null; then
          echo '[OK] Admin frontend is healthy. Route its HTTPS hostname through Caddy.'
          exit 0
        fi
        sleep 2
      done
      echo '[ERROR] Admin frontend did not become healthy.' >&2
      docker service ps --no-trunc "${STACK}_frontend"
      exit 1
    fi
    exit 0
  fi
  sleep 2
done
echo "[ERROR] Backend did not become healthy. Inspect the service tasks below." >&2
docker service ps --no-trunc "${STACK}_backend"
docker service ps --no-trunc "${STACK}_mongo"
exit 1
