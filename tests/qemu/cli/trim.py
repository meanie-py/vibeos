"""`vibeos-cli prune --trim` — cut a cargo target dir back to what a build can still read.

Cargo never deletes anything. A lane's `sdk/lib/rust/target` therefore only grows: a fresh
three-arch build is ~32 G (the merge queue's scrubbed integration trees), and long-lived lanes
measured 35-45 G on 2026-10-06. The excess has two sources, and neither is anything the build
can use again:

  * SUPERSEDED UNITS. Cargo names a unit's outputs `<crate>-<hash>` where the hash covers the
    toolchain, the dependency graph, rustflags and features, but NOT the source. An ordinary
    edit rebuilds into the same name; a rustc bump, a dependency change or a flag change mints
    a new hash and orphans the old outputs forever. 1-4 G per lane.
  * AD-HOC TARGET TREES. The CMake build only writes the per-module `<pkg>-...-rustlib` /
    `-cargo-link` dirs and the host `release/`. A hand-run `cargo check --target x86_64-...`,
    `cargo test` or clippy lands in `target/<triple>/` or `target/debug/` instead — up to 8.4 G
    in one lane, re-creatable by re-running whatever made it.

  * DEAD CUSTOM TARGETS. A per-module dir holds one subdir per target the module was built
    for, named after the triple or, for a custom target, after its json (`i686-kernel`). When
    the cell's `<name>.json` goes (a cell moved onto the builtin triple), that subdir is dead:
    no build of the current source reads it. 215 of them held 27 G in one lane on 2026-10-15,
    after the cargo-drive waves moved the i386 kernel cells off `i686-kernel.json`; each one
    holds a build-std `core` (~106 M) plus ntkrnl-sys.

  * RETIRED LAYOUTS. Two private layouts are left behind when a lane's build moves on.
    A `build-<arch>/**/rust-*-target` dir that the arch's build.ninja no longer names
    belonged to a module whose Rust build moved to a cargo cell (5.1 G of 6.8 G in one
    amd64 build dir, 2026-10-16). And once every configured arch shares nested body
    builds (VIBEOS_RUST_BODY_SHARDS non-zero), each cell's old private
    `<cell>-<body>-rustlib` dir is read by nothing (54 G in one lane); a diagnostic
    IR build still uses them, which the idle window covers.
    A `body-storage-v1-<arch>-<config>-<n>-rustlib` shard that no build has acquired for
    three days goes too: since the shared artifact store builds every eligible body,
    only the ineligible ones still build into shards, and the rest are dead weight
    (13 G of shards in one lane on 2026-10-16). Its `.body-storage.lock` mtime is the
    last use (reactos-link stamps it on every acquire); a shard locked right now is
    never a candidate.

Unlike `prune`, this never costs a warm build: everything it removes is unreachable by the
current build configuration, so it can run on live lanes. Two guards keep that true:

  * Units are compared by cargo's own fingerprint (`.fingerprint/<pkg>-<hash>/*.json`), keyed by
    package, target, source path, profile, features and compile kind. Two units that differ in
    any of those are BOTH live — one host `release/` legitimately holds `crt-shared-rust` with
    two feature sets, and five difftest packages each own a test target named `diff`. Only
    units with an identical key and an older fingerprint are superseded.
  * Nothing younger than `idle_hours` (default 24) is touched, so a lane toggling between two
    configurations within a day keeps both, and a unit cargo is writing right now is never
    a candidate.

Removing a still-wanted unit is not a correctness hazard either way: cargo checks that a
unit's outputs exist before calling it fresh, so the worst case is one recompile (an sccache
hit).

It also trims the rustconv PORT STAGING caches: every `port check` / `crate-repair` gives each
ported module its own `~/vibeos-rust-staging/port/<stem>/target-<arch>` so lanes never share a
cargo lock, and nothing ever deletes them — 316 idle dirs held 45 G on 2026-10-12, a third of
that tree. They are pure cargo output: the paid artifacts beside them (`state.json`, banked
hand fixes, responses) are files, never a `target-*` dir, and are never touched. An idle one
goes whole; the next `port check` of that module rebuilds it.
"""

from __future__ import annotations

import os
import time
from dataclasses import dataclass, field
from pathlib import Path

from tools import build_infra_compat as native

# The planner and the deleter are native: tools/vibeos-build/src/trim.rs. This module is the
# Python-facing adapter for `prune --trim` and keeps the TrimPlan shape its callers print.

TARGET_RELDIRS = ("sdk/lib/rust/target", "ntoskrnl/rust-kernel-target")
# rustconv `port`'s state root (rustconv/port.py STATE_ROOT, same env override). Shared by every
# lane, so it is planned once, not per worktree.
PORT_STATE_ROOT = Path(
    os.environ.get("RUSTCONV_PORT_STATE", Path.home() / "vibeos-rust-staging" / "port")
)
BODY_SHARD_IDLE_SECONDS = 72 * 3600


@dataclass
class TrimPlan:
    paths: list[tuple[Path, int]] = field(default_factory=list)  # (path, bytes)
    superseded_units: int = 0
    adhoc_dirs: int = 0
    dead_targets: int = 0
    retired_dirs: int = 0

    @property
    def total(self) -> int:
        return sum(sz for _, sz in self.paths)


def _plan(value: dict) -> TrimPlan:
    return TrimPlan(
        paths=[(Path(p), int(s)) for p, s in value["paths"]],
        superseded_units=value["superseded_units"],
        adhoc_dirs=value["adhoc_dirs"],
        dead_targets=value["dead_targets"],
        retired_dirs=value["retired_dirs"],
    )


def _now(now: float | None) -> float:
    return time.time() if now is None else now


def plan_target(target_root: Path, *, now: float, idle_hours: float) -> TrimPlan:
    kwargs = {"target_root": str(target_root), "now": now, "idle_hours": idle_hours}
    return _plan(native.call("trim_plan_target", kwargs))


def plan_worktree(worktree: Path, *, now: float | None = None, idle_hours: float = 24) -> TrimPlan:
    kwargs = {"worktree": str(worktree), "now": _now(now), "idle_hours": idle_hours}
    return _plan(native.call("trim_plan_worktree", kwargs))


def plan_port_staging(
    root: Path | None = None, *, now: float | None = None, idle_hours: float = 24
) -> TrimPlan:
    """Idle `<stem>/target-*` cargo dirs under the port state root, each going whole."""
    kwargs = {
        "root": str(PORT_STATE_ROOT if root is None else root),
        "now": _now(now),
        "idle_hours": idle_hours,
    }
    return _plan(native.call("trim_plan_port_staging", kwargs))


def apply(worktree: Path, plan: TrimPlan) -> tuple[int, int]:
    """Delete a plan's paths. Returns (bytes freed, failures). Never follows a symlink out
    of the worktree."""
    plan_json = {
        "paths": [[str(p), s] for p, s in plan.paths],
        "superseded_units": plan.superseded_units,
        "adhoc_dirs": plan.adhoc_dirs,
        "dead_targets": plan.dead_targets,
        "retired_dirs": plan.retired_dirs,
    }
    result = native.call("trim_apply", {"root": str(worktree), "plan": plan_json})
    for line in result["failures"]:
        print(f"  FAILED {line}")
    return int(result["freed"]), len(result["failures"])
