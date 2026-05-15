param(
  [string]$Tag = (Get-Date -Format "yyyy.MM.dd"),
  [string]$User = $(if ($env:DOCKER_HUB_USER) { $env:DOCKER_HUB_USER } else { "icedataforge" }),
  [string]$ApiPort = "8080",
  [string]$McpPort = "3001",
  [string]$DashboardPort = "80"
)

$ErrorActionPreference = "Stop"
$Root = Split-Path -Parent $PSScriptRoot
Set-Location $Root

Write-Output "=== IceData Forge — Docker Hub Deploy ==="
Write-Output "Tag:    $Tag"
Write-Output "User:   $User"
Write-Output ""

function Build-Tag-Push {
  param([string]$Name, [string]$Dockerfile, [string]$Context, [string]$Target)

  $img = "${User}/ice-data-${Name}:${Tag}"
  $latest = "${User}/ice-data-${Name}:latest"

  Write-Output "--- Building $img ---"
  if ($Target) {
    docker build -f $Dockerfile -t $img -t $latest --target $Target $Context
  } else {
    docker build -f $Dockerfile -t $img -t $latest $Context
  }
  if ($LASTEXITCODE -ne 0) { throw "Build failed for $Name" }

  Write-Output "--- Pushing $img ---"
  docker push $img
  docker push $latest
  Write-Output "Done: $img"
  Write-Output ""
}

# 1. Rust API binary
Build-Tag-Push "api" "Dockerfile" "." "api"

# 2. Rust MCP binary
Build-Tag-Push "mcp" "Dockerfile" "." "mcp"

# 3. Chatbot (Python)
Build-Tag-Push "bot" "chatbot/Dockerfile" "chatbot" $null

# 4. Dashboard (React + nginx)
Build-Tag-Push "dashboard" "dashboard/Dockerfile" "dashboard" $null

Write-Output "=== All images built and pushed successfully ==="

# Deploy to production via SSH (optional)
if ($env:SSH_HOST -and $env:SSH_KEY) {
  Write-Output "--- Deploying to $($env:SSH_HOST) ---"
  $remoteCmds = @(
    "cd $env:SSH_PATH",
    "docker compose -f docker-compose.prod.yml pull",
    "docker compose -f docker-compose.prod.yml up -d",
    "docker image prune -f"
  ) -join " && "
  ssh -i $env:SSH_KEY -o StrictHostKeyChecking=no "$($env:SSH_USER)@$($env:SSH_HOST)" $remoteCmds
  Write-Output "Deploy complete."
}
