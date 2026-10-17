#!/usr/bin/env bash
# audit-captures.sh — captures(none)-class post-opt IR audit, end to end.
#
#   tools/audit-captures.sh <i386|amd64|arm64>
#
# Builds the arch with VIBEOS_EMIT_LLVM_IR=1 (reactos-link appends
# --emit=llvm-ir to every kernel crate's RUSTFLAGS, so each CGU's
# post-optimization IR lands next to the normal artifacts), then runs
# the native scan_captures plugin over every ntoskrnl-cargo rustlib deps dir for the
# arch's target triple(s). The class is toolchain-fixed on LLVM >= 21
# (rust-kernel-patterns.md "RESOLVED AT THE TOOLCHAIN") — run this after
# every rustc/LLVM bump to prove it stayed fixed.
set -euo pipefail

arch=${1:?usage: audit-captures.sh <i386|amd64|arm64> }
shift

root=$(cd "$(dirname "$0")/.." && pwd)
cd "$root"

case "$arch" in
i386) triple_glob="i686-*" ;;
amd64) triple_glob="x86_64-*" ;;
arm64) triple_glob="aarch64-*" ;;
*)
    echo "audit-captures: unknown arch '$arch'" >&2
    exit 2
    ;;
esac

# Env-only toggles don't re-run ntoskrnl-cargo's build.rs: the ninja edge's
# DEPENDS glob watches files, not env (field-manual §9.6b staleness class).
# Touch a globbed file so the cargo edge re-runs and sees the env.
touch sdk/lib/rust/ntoskrnl-cargo/build.rs
VIBEOS_EMIT_LLVM_IR=1 ./vibeos-cli build --arch "$arch"

# shellcheck disable=SC2206 # glob expansion is the point
dirs=(sdk/lib/rust/target/ntoskrnl-cargo-*-rustlib/$triple_glob/release/deps)
if [ ! -d "${dirs[0]}" ]; then
    echo "audit-captures: no rustlib deps dirs for $arch — did the build emit .ll?" >&2
    exit 1
fi
# The front door provisions the monolith; findings use gate semantics.
paths=()
for dir in "${dirs[@]}"; do paths+=(--path "$dir"); done
exec ./vibeos-cli lint --arch "$arch" --rules scan_captures "${paths[@]}" "$@"
