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

import json
import os
import re
import shutil
import time
from collections import defaultdict
from dataclasses import dataclass, field
from pathlib import Path

# Cargo target roots inside a worktree (the non-build-* half of prune.PRUNE_RELDIRS).
TARGET_RELDIRS = ("sdk/lib/rust/target", "ntoskrnl/rust-kernel-target")

# rustconv `port`'s state root (rustconv/port.py STATE_ROOT, same env override). Shared by every
# lane, so it is planned once, not per worktree.
PORT_STATE_ROOT = Path(
    os.environ.get("RUSTCONV_PORT_STATE", Path.home() / "vibeos-rust-staging" / "port")
)

# A target-root entry named for a rustc triple, or cargo's default dev profile, is an ad-hoc
# `cargo check/test --target ...` tree: the CMake build gives every cargo invocation its own
# `<pkg>-...-rustlib` / `-cargo-link` dir and builds host tools into `release/`.
_ADHOC_TRIPLE = re.compile(
    r"^(x86_64|i[3-6]86|aarch64|arm64ec|armv7|thumbv7a)-[a-z0-9_]+-[a-z0-9_]+(-[a-z0-9_]+)?$"
)
_ADHOC_NAMES = {"debug"}

_HASH = re.compile(r"-([0-9a-f]{16})(?:\.|$)")
# The fingerprint fields that make two units DIFFERENT things rather than versions of one.
# Left out on purpose: rustc, deps, rustflags, config, local — exactly what changes when a
# unit is superseded.
_IDENTITY = ("target", "path", "profile", "features", "compile_kind")
# Subdirs of a profile dir whose entries carry a unit hash.
_HASHED_SUBDIRS = ("deps", "build", "incremental", ".fingerprint")


@dataclass
class TrimPlan:
    paths: list[tuple[Path, int]] = field(default_factory=list)  # (path, bytes)
    superseded_units: int = 0
    adhoc_dirs: int = 0
    dead_targets: int = 0

    @property
    def total(self) -> int:
        return sum(sz for _, sz in self.paths)


def _size(path: Path) -> int:
    try:
        st = path.lstat()
    except FileNotFoundError:
        return 0
    if not path.is_dir() or path.is_symlink():
        return st.st_blocks * 512
    total = st.st_blocks * 512
    for parent, dirs, files in os.walk(path):
        for name in dirs + files:
            try:
                total += os.lstat(os.path.join(parent, name)).st_blocks * 512
            except FileNotFoundError:
                continue
    return total


def _newest_mtime(path: Path) -> float:
    best = path.lstat().st_mtime
    for parent, dirs, files in os.walk(path):
        for name in dirs + files:
            try:
                best = max(best, os.lstat(os.path.join(parent, name)).st_mtime)
            except FileNotFoundError:
                continue
    return best


def unit_identity(fp_dir: Path) -> tuple | None:
    """(pkg, unit kind/name, *identity fields) for one `.fingerprint/<pkg>-<hash>` dir, or None
    when it holds no readable fingerprint json (cargo mid-write, or a layout we do not know —
    either way, not ours to judge)."""
    pkg = fp_dir.name.rsplit("-", 1)[0]
    keys = []
    for js in sorted(fp_dir.glob("*.json")):
        try:
            data = json.loads(js.read_text())
        except (OSError, ValueError):
            return None
        if not isinstance(data, dict) or any(k not in data for k in _IDENTITY):
            return None
        keys.append((js.name, *(json.dumps(data[k], sort_keys=True) for k in _IDENTITY)))
    return (pkg, *sorted(keys)) if keys else None


def superseded_hashes(profile_dir: Path, *, now: float, idle_seconds: float) -> set[str]:
    """Unit hashes in one profile dir (`.../release`, `.../<triple>/release`) that a newer unit
    with the same identity has replaced and that are older than the idle window."""
    groups: dict[tuple, list[tuple[float, str]]] = defaultdict(list)
    fp_root = profile_dir / ".fingerprint"
    for fp_dir in fp_root.iterdir():
        m = _HASH.search(fp_dir.name)
        if not m or not fp_dir.is_dir():
            continue
        ident = unit_identity(fp_dir)
        if ident is None:
            continue
        try:
            groups[ident].append((_newest_mtime(fp_dir), m.group(1)))
        except FileNotFoundError:
            continue
    dead: set[str] = set()
    for units in groups.values():
        if len(units) < 2:
            continue
        units.sort()
        newest = units[-1][0]
        for mtime, h in units[:-1]:
            if mtime < newest and now - mtime >= idle_seconds:
                dead.add(h)
    return dead


def _profile_dirs(target_root: Path):
    """Every dir under a cargo target root that directly holds a `.fingerprint/`."""
    for parent, dirs, _files in os.walk(target_root):
        if ".fingerprint" in dirs:
            yield Path(parent)
        dirs[:] = [d for d in dirs if d not in _HASHED_SUBDIRS]


def plan_target(target_root: Path, *, now: float, idle_hours: float) -> TrimPlan:
    plan = TrimPlan()
    if not target_root.is_dir():
        return plan
    idle = idle_hours * 3600
    adhoc: list[Path] = []
    for entry in target_root.iterdir():
        named_adhoc = entry.name in _ADHOC_NAMES or _ADHOC_TRIPLE.match(entry.name)
        if named_adhoc and entry.is_dir() and not entry.is_symlink():
            try:
                if now - _newest_mtime(entry) >= idle:
                    adhoc.append(entry)
            except FileNotFoundError:
                continue
    for d in adhoc:
        plan.paths.append((d, _size(d)))
        plan.adhoc_dirs += 1
    dead_targets = _dead_custom_targets(target_root, now=now, idle_seconds=min(idle, 3600))
    for d in dead_targets:
        plan.paths.append((d, _size(d)))
        plan.dead_targets += 1
    adhoc += dead_targets
    for prof in _profile_dirs(target_root):
        if any(prof == d or d in prof.parents for d in adhoc):
            continue  # already going whole
        dead = superseded_hashes(prof, now=now, idle_seconds=idle)
        if not dead:
            continue
        plan.superseded_units += len(dead)
        for sub in _HASHED_SUBDIRS:
            sd = prof / sub
            if not sd.is_dir():
                continue
            for p in sd.iterdir():
                m = _HASH.search(p.name)
                if m and m.group(1) in dead:
                    plan.paths.append((p, _size(p)))
    return plan


# The per-module dirs the CMake build gives each cargo invocation: `<cell>-<arch>-cargo-link`
# (the cell's own link) and `<cell>-<body>-rustlib` (its body crate's nested build).
_CELL_LINK = re.compile(r"^(?P<cell>.+?)-(?:i386|amd64|arm64)-cargo-link$")
_CELL_RUSTLIB = re.compile(r"^(?P<cell>.+?-cargo)-.+-rustlib$")
# What a per-module dir holds besides its target subdirs.
_MODULE_DIR_OTHER = {"release", "debug", "CACHEDIR.TAG", "tmp"}


def _dead_custom_targets(target_root: Path, *, now: float, idle_seconds: float) -> list[Path]:
    """Custom-target subdirs (`i686-kernel`) of per-module dirs whose cell no longer has the
    target's json. Read against THIS worktree's source only; a dir whose cell cannot be found
    is not judged. The idle window is capped at an hour: the rule is exact, the window only
    keeps clear of a build that started before the json went."""
    rust = target_root.parent
    if rust.name != "rust" or not target_root.is_dir():
        return []
    dead = []
    for entry in target_root.iterdir():
        m = _CELL_LINK.match(entry.name) or _CELL_RUSTLIB.match(entry.name)
        if not m or not entry.is_dir() or entry.is_symlink():
            continue
        cell = rust / m.group("cell")
        if not (cell / "Cargo.toml").is_file():
            continue
        for sub in entry.iterdir():
            if sub.name in _MODULE_DIR_OTHER or _ADHOC_TRIPLE.match(sub.name):
                continue
            if not sub.is_dir() or sub.is_symlink() or (cell / f"{sub.name}.json").exists():
                continue
            try:
                if now - _newest_mtime(sub) >= idle_seconds:
                    dead.append(sub)
            except FileNotFoundError:
                continue
    return dead


def plan_worktree(worktree: Path, *, now: float | None = None, idle_hours: float = 24) -> TrimPlan:
    now = time.time() if now is None else now
    plan = TrimPlan()
    for rel in TARGET_RELDIRS:
        sub = plan_target(worktree / rel, now=now, idle_hours=idle_hours)
        plan.paths += sub.paths
        plan.superseded_units += sub.superseded_units
        plan.adhoc_dirs += sub.adhoc_dirs
        plan.dead_targets += sub.dead_targets
    return plan


def plan_port_staging(
    root: Path | None = None, *, now: float | None = None, idle_hours: float = 24
) -> TrimPlan:
    """Idle `<stem>/target-*` cargo dirs under the port state root, each going whole."""
    root = PORT_STATE_ROOT if root is None else root
    now = time.time() if now is None else now
    plan = TrimPlan()
    if not root.is_dir():
        return plan
    for stem in sorted(root.iterdir()):
        if not stem.is_dir() or stem.is_symlink():
            continue
        for t in sorted(stem.glob("target-*")):
            if not t.is_dir() or t.is_symlink():
                continue
            try:
                if now - _newest_mtime(t) < idle_hours * 3600:
                    continue
            except FileNotFoundError:
                continue
            plan.paths.append((t, _size(t)))
            plan.adhoc_dirs += 1
    return plan


def apply(worktree: Path, plan: TrimPlan) -> tuple[int, int]:
    """Delete a plan's paths. Returns (bytes freed, failures). Never follows a symlink out
    of the worktree."""
    root = worktree.resolve()
    freed = failures = 0
    for p, sz in plan.paths:
        try:
            if root not in p.resolve().parents:
                raise OSError(f"resolves outside {root}")
            if p.is_dir() and not p.is_symlink():
                shutil.rmtree(p)
            else:
                p.unlink()
            freed += sz
        except FileNotFoundError:
            continue
        except OSError as exc:
            failures += 1
            print(f"  FAILED {p}: {exc}")
    return freed, failures
