# Devcontainer setup

Minimal devcontainer for running Claude Code in an isolated environment, per
[code.claude.com/docs/en/devcontainer](https://code.claude.com/docs/en/devcontainer).

- Base image: `mcr.microsoft.com/devcontainers/base:ubuntu`
- Features: Node.js (required by the Claude Code feature; without it, the
  feature's own Node.js auto-install fails under apt — see gotchas), the
  official `github-cli` feature, and the official `claude-code` devcontainer
  feature
- Claude Code state persists across rebuilds via a named volume
  (`rusty-city-simulator-claude-config`) mounted at `~/.claude`, with
  `CLAUDE_CONFIG_DIR` pointed at the same path. This volume holds more than
  auth — conversation transcripts, file-edit history, and session state all
  live under `CLAUDE_CONFIG_DIR` too — so it's namespaced per repo rather than
  shared machine-wide. A container running for one repo can't read another
  repo's conversation history this way, the same reasoning behind scoping
  `GH_TOKEN` and the SSH keys below to this repo alone. The trade-off: you log
  in again the first time each repo's container comes up, not just once per
  machine.
- `gh` CLI auth comes from a `GH_TOKEN` env var supplied via a gitignored
  `.devcontainer/.env` file — see below
- Claude Code skills are wiped and reinstalled from the configured sources
  (`mattpocock/skills`, `ken-guru/skills`) on every container start — see
  "Skill management" below

## Opening it

1. Install Docker Desktop and VS Code's **Dev Containers** extension
   (`ms-vscode-remote.remote-containers`).
2. Copy `.devcontainer/.env.example` to `.devcontainer/.env` and paste in a
   GitHub token (a fine-grained PAT scoped to this repo). If you skip this,
   `initializeCommand` creates an empty `.env` for you so the build doesn't
   fail, but `gh` won't be authenticated until you fill in a real token and
   rebuild.
3. Open this repo in VS Code, then **Dev Containers: Reopen in Container**
   (Cmd+Shift+P).
4. Once built, open a terminal and run `claude`, then follow the login prompt.
   `gh auth status` should already show you as logged in — no `gh auth login`
   needed.

## Skill management

`post-start.sh` wipes `~/.claude/skills` and reinstalls the full set from each
configured source on every container start:

```
npx -y skills add mattpocock/skills --skill '*' -a claude-code -y --copy -g
npx -y skills add ken-guru/skills --skill '*' -a claude-code -y --copy -g
```

`~/.claude/skills` lives inside the same `rusty-city-simulator-claude-config`
volume already mounted for Claude Code state, so no separate volume is
needed — the wipe-and-reinstall just keeps the skill set in that volume
current with upstream on every start, rather than persisting a stale copy
across rebuilds. Add or remove a source by editing these lines directly.

## Gotchas fixed here (and why)

**`remoteUser` must match the base image's actual non-root user.** The
`mcr.microsoft.com/devcontainers/base:ubuntu` image's default non-root user is
`vscode`, not `node` — the example in Anthropic's own docs uses `/home/node`,
which only applies to Node-flavored base images. Using the wrong home path
silently creates an unused `/home/node` directory owned by `root`, and the
Claude session never persists because nothing is actually being read from or
written to it under the real user's `$HOME`. Fix: `remoteUser: "vscode"`, and
point the mount + `CLAUDE_CONFIG_DIR` at `/home/vscode/.claude`.

**A fresh named-volume mountpoint is always created `root:root`, regardless of
the parent directory's ownership** — even under `/home/vscode`, which is
otherwise fully owned by `vscode`. Docker does not inherit the parent
directory's ownership when it creates the mount target for a volume used for
the first time. Without a fix, `claude login` fails to persist anything: the
process (running as `vscode`) can't write into a directory it doesn't own,
even though the login flow itself completes successfully in the browser.
Fix: `postCreateCommand` runs `sudo chown -R vscode:vscode /home/vscode/.claude`
after the volume mounts.

**The `claude-code` feature installs the CLI into the nvm-managed global npm
tree as `root`**, even though the container runs as `vscode` afterward.
`vscode` is a member of the `nvm` group but only has read+execute (not write)
on that tree, so `claude update` (and the CLI's own auto-updater) fails with
"Insufficient permissions to install update." Fix: `postCreateCommand` also
chowns `$(npm config get prefix)/lib/node_modules/@anthropic-ai` and
`$(npm config get prefix)/bin/claude` to `vscode:nvm`.

**An outdated Claude Code CLI version can silently fail to persist login in a
container with no init system.** `claude doctor` reports a background daemon
that handles keychain sync and token refresh, managed via
launchd/systemd — but a bare devcontainer has no systemd running, so on an old
CLI build the daemon never starts, no persistence occurs, and `/login` reports
"Login successful" only to immediately fall back to logged out on the very
next check. Run `claude update` (needs the npm-permissions fix above to
succeed) to pick up a build new enough to run the daemon on-demand instead of
depending on a service manager. If login still doesn't persist after a
rebuild, run `claude doctor` first and check the "Background server" section
before assuming it's a mount or permissions problem again.

**There's no `docker-compose.yml` here, so `env_file` isn't available** —
this setup uses a plain `image`, and the devcontainer spec only supports
`env_file` under `dockerComposeFile`. Fix: `runArgs: ["--env-file", ...]`
passes `.devcontainer/.env` straight to `docker run` instead. Unlike
`env_file`'s `required: false`, `--env-file` errors if the file is missing,
so `initializeCommand` copies `.env.example` to `.env` on the host before the
build starts if `.env` doesn't exist yet — the container will build with
`gh` unauthenticated rather than failing outright.


## SSH deploy key and signing key

- Git push/pull and commit signing use two separate SSH keys, persisted
  across rebuilds in a named volume (`rusty-city-simulator-ssh-config`) mounted at
  `~/.ssh`.

Two separate ED25519 keys exist because GitHub rejects a public key as a
signing key once that same key is already registered as a deploy key.
`post-create.sh` generates `~/.ssh/id_ed25519` as the deploy key (git
transport: push/pull this repo, registered automatically against
`repos/ken-guru/rusty-city-simulator/keys` via the `gh` API) and `~/.ssh/id_ed25519_signing`
as the signing key (commit verification, registered manually once per machine
via the GitHub UI — there's no API-driven way to do this without granting the
token account-level `write:ssh_signing_key`, which would let it manage every
signing key on the account, not just this project's).

Both keys live in the `rusty-city-simulator-ssh-config` volume, so they and the
`~/.ssh/.signing-key-registered` marker survive container rebuilds. Only
wiping that volume regenerates the keys and resets the marker.

Deploy-key registration is checked by key **content**, not title — if the
volume is wiped and a new key is generated, the stale GitHub entry (same
title, old content) is deleted and replaced. `postAttachCommand` re-verifies
the deploy key on every attach so an accidental deletion on GitHub is caught
immediately instead of failing silently on the next `git push`.

`GH_TOKEN` needs the repo's **Administration (read/write)** permission to
list, register, and delete deploy keys via `gh api repos/.../keys` — this is
in addition to whatever else you use `gh` for (Issues, Pull requests,
Metadata). No account-level token permissions are needed for any of this.

Register the signing key: `postAttachCommand` prints a one-time prompt with a
public key to paste into <https://github.com/settings/ssh> as a **Signing
Key**. Do that, then dismiss the prompt with
`touch ~/.ssh/.signing-key-registered`.

**`GH_TOKEN` alone doesn't authenticate git push/pull, only the `gh` API.**
The `gh` CLI reads `GH_TOKEN` automatically for API calls, but `git` itself
has no idea it exists. If this repo's remote is an SSH URL (`git@github.com:...`),
the SSH deploy key set up in `post-create.sh` is what makes `git push`/`git
pull` work against `origin` — `git config --global credential.helper
'!gh auth setup-git'` (set in the baseline) only covers an HTTPS remote, and
does nothing for SSH transport.
