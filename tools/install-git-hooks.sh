#!/usr/bin/env bash
#
# Install the repo's git hooks into the shared common git dir, so they apply to
# the main checkout AND every linked worktree (hooks live in the common dir).
#
# Idempotent — safe to re-run. Run once per clone (the swarm onboarding does this;
# see AGENTS.md "Mainline protection"). The marquee hook is the mainline-rewind
# guard (tools/git-hooks/reference-transaction).
set -euo pipefail

repo_root="$(git rev-parse --show-toplevel)"
hooks_dir="$(git -C "$repo_root" rev-parse --git-common-dir)/hooks"
src_dir="$repo_root/tools/git-hooks"

mkdir -p "$hooks_dir"
for hook in "$src_dir"/*; do
    name="$(basename "$hook")"
    install -m 0755 "$hook" "$hooks_dir/$name"
    echo "installed: $hooks_dir/$name"
done
echo "git hooks installed (mainline-rewind guard active across all worktrees)."
