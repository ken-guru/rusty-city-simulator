#!/bin/bash
set -euo pipefail

# Bevy's Linux backends (audio, input, windowing) link against these system
# libraries even for a headless `cargo build`/`cargo test` — mirrors the apt
# package list in .github/workflows/ci.yml so a local build matches CI.
sudo apt-get update
sudo apt-get install -y \
  libwayland-dev \
  libxkbcommon-dev \
  libudev-dev \
  libasound2-dev \
  pkg-config

# Add YOLO alias for fast development — skips permission prompts, runs in an
# isolated worktree, and enables remote control.
cat >> ~/.bashrc << 'EOF'
alias claude-yolo="claude --dangerously-skip-permissions --worktree --remote-control"
EOF

# Fix ownership on mounted volumes.
# The claude-code devcontainer feature installs as root into the nvm tree;
# hand it back to vscode so the manifest-managed startup update works without
# sudo.
sudo chown -R vscode:vscode /home/vscode/.claude
NVM_NODE_PREFIX=$(npm config get prefix)
sudo chown -R vscode:nvm "${NVM_NODE_PREFIX}/lib/node_modules/@anthropic-ai"
sudo chown vscode:nvm "${NVM_NODE_PREFIX}/bin/claude"

# Git identity — read from .devcontainer/.env (GIT_USER_EMAIL / GIT_USER_NAME).
git config --global credential.helper '!gh auth setup-git'
git config --global user.email "${GIT_USER_EMAIL:-ken.paulsen@gmail.com}"
git config --global user.name "${GIT_USER_NAME:-Ken Sørevåge}"

# SSH identity — two keys per host machine, both persisted in the
# rusty-city-simulator-ssh-config volume.
#   id_ed25519         — deploy key: scoped auth for this repo (registered automatically)
#   id_ed25519_signing — signing key: commit verification (registered manually once)
#
# Two separate keys because GitHub rejects a public key as a signing key once
# that same key is already registered as a deploy key.
sudo mkdir -p /home/vscode/.ssh
sudo chown -R vscode:vscode /home/vscode/.ssh
chmod 700 /home/vscode/.ssh

if [ -z "${DEVCONTAINER_HOST:-}" ]; then
  echo "ERROR: DEVCONTAINER_HOST is not set." >&2
  echo "       Copy .devcontainer/.env.example to .devcontainer/.env and set DEVCONTAINER_HOST." >&2
  exit 1
fi
DEPLOY_KEY_TITLE="rusty-city-simulator-devcontainer@${DEVCONTAINER_HOST}"
SIGNING_KEY_TITLE="rusty-city-simulator-devcontainer-signing@${DEVCONTAINER_HOST}"

# Deploy key — used for git transport (push/pull)
if [ ! -f ~/.ssh/id_ed25519 ]; then
  ssh-keygen -t ed25519 -C "$DEPLOY_KEY_TITLE" -f ~/.ssh/id_ed25519 -N ""
fi
chmod 600 ~/.ssh/id_ed25519
chmod 644 ~/.ssh/id_ed25519.pub

# Signing key — used only for commit signing, not for git transport
if [ ! -f ~/.ssh/id_ed25519_signing ]; then
  ssh-keygen -t ed25519 -C "$SIGNING_KEY_TITLE" -f ~/.ssh/id_ed25519_signing -N ""
fi
chmod 600 ~/.ssh/id_ed25519_signing
chmod 644 ~/.ssh/id_ed25519_signing.pub

# SSH client config — deploy key for GitHub transport only, no agent.
if ! grep -q "Host github.com" ~/.ssh/config 2>/dev/null; then
  cat >> ~/.ssh/config << 'EOF'
Host github.com
  IdentityFile ~/.ssh/id_ed25519
  IdentitiesOnly yes
  User git
EOF
  chmod 600 ~/.ssh/config
fi

# Trust GitHub's host key without an interactive prompt.
if ! grep -q "github.com" ~/.ssh/known_hosts 2>/dev/null; then
  ssh-keyscan -H github.com >> ~/.ssh/known_hosts 2>/dev/null
fi

DEPLOY_PUBKEY=$(cat ~/.ssh/id_ed25519.pub)
DEPLOY_KEY_BODY=$(echo "$DEPLOY_PUBKEY" | awk '{print $1, $2}')
REPO="ken-guru/rusty-city-simulator"

ALL_DEPLOY_KEYS=$(gh api "repos/${REPO}/keys")

# Check by key content — a title match with different content means the key was
# rotated (e.g. the ssh-config volume was wiped). In that case remove the
# stale entry and re-register with the new key.
existing_id=$(echo "$ALL_DEPLOY_KEYS" | jq -r \
  --arg body "$DEPLOY_KEY_BODY" \
  '.[] | select((.key | split(" ")[:2] | join(" ")) == $body) | .id')

if [ -n "$existing_id" ]; then
  echo "Deploy key already registered: $DEPLOY_KEY_TITLE"
else
  stale_id=$(echo "$ALL_DEPLOY_KEYS" | jq -r \
    --arg title "$DEPLOY_KEY_TITLE" \
    '.[] | select(.title == $title) | .id')
  if [ -n "$stale_id" ]; then
    gh api "repos/${REPO}/keys/${stale_id}" -X DELETE
    echo "Removed stale deploy key (volume was rotated): $DEPLOY_KEY_TITLE"
  fi
  gh api "repos/${REPO}/keys" -X POST \
    -f title="$DEPLOY_KEY_TITLE" -f key="$DEPLOY_PUBKEY" -F read_only=false
  echo "Deploy key registered: $DEPLOY_KEY_TITLE"
fi

# Signing key is separate from the deploy key so it can be registered on GitHub
# without hitting the "key is already in use" constraint.
git config --global gpg.format ssh
git config --global user.signingkey ~/.ssh/id_ed25519_signing.pub
git config --global commit.gpgsign true
