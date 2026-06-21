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

# Repair the fetch refspec. This clone shipped with a master-ONLY refspec
# (+refs/heads/master:refs/remotes/origin/master), so `git fetch` never updated
# refs/remotes/origin/continue-rust-port — it sat frozen at the ancient fork point,
# which made agents read "origin is stale/behind" and reset mainline back to it (the
# ~81-commit rewind). Restore the normal all-branches refspec so fetch tracks c-r-p.
want_refspec="+refs/heads/*:refs/remotes/origin/*"
if [ "$(git -C "$repo_root" config --get-all remote.origin.fetch || true)" != "$want_refspec" ]; then
    git -C "$repo_root" config --unset-all remote.origin.fetch 2>/dev/null || true
    git -C "$repo_root" config --add remote.origin.fetch "$want_refspec"
    echo "fixed remote.origin.fetch -> $want_refspec (was master-only; broke origin/continue-rust-port tracking)"
fi
