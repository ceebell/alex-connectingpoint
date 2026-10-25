#!/usr/bin/env bash
# Package the locally built connector and run a private companion container.
set -euo pipefail
cd "$(dirname "${BASH_SOURCE[0]}")"
if [ "$#" -ne 0 ]; then echo 'Usage: sudo bash ./deploy-oa-connector.sh' >&2; exit 1; fi
if [ -f .env ]; then set -a; . ./.env; set +a; fi
for key in LINE_DESTINATION_ID INQUIRY_ADMIN_TOKEN ADMIN_PASSWORD; do
  if [ -z "${!key:-}" ]; then echo "[ERROR] Set $key in the server .env." >&2; exit 1; fi
done
if [ "${#ADMIN_PASSWORD}" -lt 16 ]; then echo '[ERROR] ADMIN_PASSWORD must have at least 16 characters.' >&2; exit 1; fi
release=dist/oa-connector-linux
if [ ! -f "$release/connector.sha256" ] || [ ! -f "$release/connector.tag" ]; then
  echo '[ERROR] Run build-oa-connector.bat on LOCAL, commit the release, and pull Git first.' >&2; exit 1
fi
(cd "$release" && sha256sum --quiet -c connector.sha256)
tag="$(cat "$release/connector.tag")"
if [[ ! "$tag" =~ ^oa-[0-9]{14}-[0-9a-f]{12}$ ]]; then echo '[ERROR] Invalid connector release tag.' >&2; exit 1; fi
if [ "$(cat "$release/playwright.version")" != '1.64.1' ]; then echo '[ERROR] Playwright/browser version mismatch.' >&2; exit 1; fi
name="${APP_NAME:-alex-line}-oa-connector"
if [[ ! "$name" =~ ^[a-zA-Z0-9][a-zA-Z0-9_.-]+$ ]]; then echo '[ERROR] Invalid APP_NAME.' >&2; exit 1; fi
control_port="${OA_CONTROL_HOST_PORT:-18082}"
if [[ ! "$control_port" =~ ^[0-9]+$ ]] || ((control_port<1 || control_port>65535)); then echo '[ERROR] Invalid OA_CONTROL_HOST_PORT.' >&2; exit 1; fi
export OA_BACKEND_URL="${OA_BACKEND_URL:-http://host.docker.internal:${LINE_HTTP_PORT:-18080}}"
export OA_POLL_SECONDS="${OA_POLL_SECONDS:-30}" ADMIN_USERNAME="${ADMIN_USERNAME:-admin}"
hash="$(sha256sum "$release/connector.sha256" | cut -d ' ' -f 1)"
if [ "${tag##*-}" != "${hash:0:12}" ]; then echo '[ERROR] Connector tag does not match its release manifest.' >&2; exit 1; fi
if [[ ! "$OA_POLL_SECONDS" =~ ^[0-9]+$ ]] || ((OA_POLL_SECONDS<15 || OA_POLL_SECONDS>300)); then echo '[ERROR] OA_POLL_SECONDS must be 15..300.' >&2; exit 1; fi
image="${REGISTRY:-}${APP_NAME:-alex-line}-oa-connector:$tag"
docker build -f docker/oa-connector.Dockerfile --label "com.alex.release.sha256=$hash" -t "$image" .
docker volume create "$name-profile" >/dev/null
if docker container inspect "$name" >/dev/null 2>&1; then docker rm -f "$name" >/dev/null; fi
docker run -d --name "$name" --restart unless-stopped --init --shm-size 512m --memory "${OA_MEMORY:-1g}" \
  --add-host host.docker.internal:host-gateway -p "127.0.0.1:$control_port:8082" -v "$name-profile:/profile" \
  --env LINE_DESTINATION_ID --env INQUIRY_ADMIN_TOKEN --env ADMIN_USERNAME --env ADMIN_PASSWORD \
  --env OA_BACKEND_URL --env OA_POLL_SECONDS "$image" >/dev/null
for attempt in $(seq 1 45); do
  if curl --noproxy '*' -fsS --max-time 2 "http://127.0.0.1:$control_port/health" >/dev/null; then
    echo '[OK] Connector container is running. This does not yet confirm LINE sync.'
    echo 'Next on Windows: login-oa.bat (log in to LINE in the private browser).'
    exit 0
  fi
  sleep 1
done
echo "[ERROR] Connector did not start. Inspect: sudo docker logs --tail 30 $name" >&2
exit 1
