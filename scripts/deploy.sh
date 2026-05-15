#!/usr/bin/env bash
set -euo pipefail

TAG="${1:-$(date +%Y.%m.%d)}"
USER="${DOCKER_HUB_USER:-icedataforge}"
ROOT="$(cd "$(dirname "$0")/.." && pwd)"

echo "=== IceData Forge — Docker Hub Deploy ==="
echo "Tag:  $TAG"
echo "User: $USER"
echo ""

build_tag_push() {
  local name="$1" dockerfile="$2" context="$3" target="${4:-}"
  local img="${USER}/ice-data-${name}:${TAG}"
  local latest="${USER}/ice-data-${name}:latest"

  echo "--- Building ${img} ---"
  if [ -n "$target" ]; then
    docker build -f "$dockerfile" -t "$img" -t "$latest" --target "$target" "$context"
  else
    docker build -f "$dockerfile" -t "$img" -t "$latest" "$context"
  fi

  echo "--- Pushing ${img} ---"
  docker push "$img"
  docker push "$latest"
  echo "Done: $img"
  echo ""
}

build_tag_push "api"        "Dockerfile"            "."           "api"
build_tag_push "mcp"        "Dockerfile"            "."           "mcp"
build_tag_push "bot"        "chatbot/Dockerfile"    "chatbot"
build_tag_push "dashboard"  "dashboard/Dockerfile"  "dashboard"

echo "=== All images built and pushed ==="

# Optional SSH deploy
if [ -n "${SSH_HOST:-}" ] && [ -n "${SSH_KEY:-}" ]; then
  echo "--- Deploying to ${SSH_HOST} ---"
  ssh -i "$SSH_KEY" -o StrictHostKeyChecking=no "${SSH_USER}@${SSH_HOST}" "
    cd ${SSH_PATH}
    docker compose -f docker-compose.prod.yml pull
    docker compose -f docker-compose.prod.yml up -d
    docker image prune -f
  "
  echo "Deploy complete."
fi
