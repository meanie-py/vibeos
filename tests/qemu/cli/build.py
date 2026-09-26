"""`vibeos-cli build` — per-arch ninja + arm64 re-stage + binary-lints +
build-index + provenance stamp (plan §7/§8/§9).

warm by default; --clean nukes ALL discovered rust target dirs + lib*_rust.a
(prints the count); classifies CMAKE_CONFIGURE_FAIL / CARGO_CHECK_FAIL /
LINK_FAIL distinctly with i386 PE-decoration hints.
"""

from __future__ import annotations

import os
import re
import shutil
import subprocess
import time
from pathlib import Path

from . import archcfg, cachecfg, canon, diskspace, hostdisk, hostjobs, hostlock, lints, provenance
from .artifacts import Run, Stage

_UNDEF = re.compile(r"undefined (?:symbol|reference)[^\w]*[`']?([@\w]+)", re.I)
# ld.lld reports duplicate symbols as WARNINGS in this tree (the link runs
# with --allow-multiple-definition), so anchoring on "error:" is what keeps
# a warning from being read as the failure. Without the anchor a rustc
# compile error that merely co-occurred with those warnings classified as
# LINK_FAIL and listed three warnings as its salient — the real cause
# (error[E0308] in census.rs) appeared nowhere in the summary.
_DUP = re.compile(r"error:\s*duplicate symbol[:\s]+[`']?([@\w]+)", re.I)


def _decoration_hint(sym: str) -> str:
    if re.match(r"@\w+@\d+$", sym):
        return f"{sym}: i386 fastcall — wrong extern or missing #[no_mangle]"
    if re.match(r"_\w+@\d+$", sym):
        return f"{sym}: i386 stdcall — likely missing #[no_mangle] or wrong extern"
    if re.match(r"_\w+$", sym):
        return f"{sym}: i386 cdecl — missing symbol / wrong ABI"
    return f"{sym}: undefined"


def source_date_epoch(repo: Path) -> str | None:
    """Commit-date epoch for reproducible builds (deterministic-builds-plan.md rung 2b).

    `git log -1 --format=%ct` → the HEAD commit's timestamp. Deriving from the *commit*
    (not wall-clock) makes the build byte-identical across worktrees/agents on the same
    commit while keeping a meaningful date. Pins both cmake's `string(TIMESTAMP)` in
    version.cmake (→ buildno.h version resources) and clang's `__DATE__`/`__TIME__`
    (e.g. ntoskrnl/ex/xipdisp.c) — both honor the SOURCE_DATE_EPOCH env var. Returns
    None outside a git repo (caller leaves the env untouched)."""
    try:
        cp = subprocess.run(
            ["git", "log", "-1", "--format=%ct"],
            cwd=repo,
            capture_output=True,
            text=True,
            timeout=30,
        )
    except (OSError, subprocess.SubprocessError):
        return None
    ts = cp.stdout.strip()
    return ts if cp.returncode == 0 and ts.isdigit() else None


def _classify_build_failure(out: str, err: str) -> tuple[str, list[str]]:
    blob = out + "\n" + err
    low = blob.lower()
    # Host disk exhaustion FIRST: an ENOSPC surfaces as compiler/linker "error:"
    # text and would otherwise be filed as CARGO_CHECK_FAIL or BUILD_FAIL — i.e.
    # as a defect in the diff, which is exactly the wrong cause. It cost four
    # lanes a day on 2026-09-06. The detail always carries free bytes.
    if hostdisk.looks_like_enospc(blob):
        return "DISK_FULL", hostdisk.enospc_detail()
    if "cmake error" in low:
        return "CMAKE_CONFIGURE_FAIL", [
            ln.strip() for ln in blob.splitlines() if "CMake Error" in ln
        ][:3]
    # Link errors first (a duplicate/undefined symbol contains "error:" too, so
    # it must be matched before the generic cargo "error:" catch).
    undef = list(dict.fromkeys(_UNDEF.findall(blob)))
    dup = list(dict.fromkeys(_DUP.findall(blob)))
    # Require a real link ERROR, not merely ld.lld chatter: `ld.lld:` alone
    # matches its warnings too, which is the same false-positive as _DUP above.
    if undef or dup or "ld.lld: error" in low:
        sal = [_decoration_hint(s) for s in undef][:3]
        sal += [f"duplicate symbol: {s}" for s in dup][:3]
        return "LINK_FAIL", (sal or [ln.strip() for ln in blob.splitlines() if "ld.lld" in ln][:3])
    if "error[e" in low or ("error:" in low and "cargo" in low):
        return "CARGO_CHECK_FAIL", [
            ln.strip() for ln in blob.splitlines() if "error[" in ln or "error:" in ln
        ][:3]
    return "BUILD_FAIL", [ln.strip() for ln in blob.splitlines() if "error" in ln.lower()][:3]


def _clean(run: Run, repo: Path, arch: str) -> None:
    build_dir = repo / f"build-{arch}"
    targets = provenance.discover_rust_targets(build_dir)
    # The cargo-driven links (ntoskrnl.exe/win32k.sys/kernel32.dll/*-cargo) build
    # into the SHARED workspace cache sdk/lib/rust/target/<stem>, not build-<arch>,
    # so discover_rust_targets misses it — add it so --clean is genuinely cold
    # (automates the §9.6b manual `rm -rf sdk/lib/rust/target/<stem>`).
    targets = [*targets, *provenance.workspace_cargo_targets(repo, arch)]
    archives = provenance.discover_rust_archives(build_dir)
    # Cargo COPIES its build output into build-<arch>; nuking the caches above
    # leaves those copies newer than their .rs → ninja "no work" → stale binary
    # (det-plan §stale-cargo / task #41). Remove them so ninja re-runs the link.
    outputs = provenance.discover_rust_module_outputs(repo, arch)
    for p in targets:
        shutil.rmtree(p, ignore_errors=True)
    for a in archives + outputs:
        a.unlink(missing_ok=True)
    run.log(
        f"--clean removed {len(targets)} rust target dirs + {len(archives)} "
        f"lib*_rust.a + {len(outputs)} cargo-module outputs:\n"
        + "\n".join(str(p) for p in targets + archives + outputs),
        header="clean",
    )
    run.add(Stage("clean", status="ok", detail=f"{len(targets)}t+{len(archives)}a+{len(outputs)}o"))


def _ccache_already_enabled(build_dir: Path) -> bool:
    # Both spellings count: an interactive `-DENABLE_CCACHE=1` caches as
    # UNINITIALIZED=1 (the canonize configure does this); option() sets BOOL=ON.
    # Missing either → a needless full reconfigure on EVERY build.
    cache = build_dir / "CMakeCache.txt"
    if not cache.is_file():
        return False
    try:
        text = cache.read_text(errors="ignore")
    except OSError:
        return False
    return any(
        line.startswith("ENABLE_CCACHE:") and line.split("=", 1)[-1] in ("ON", "1", "TRUE")
        for line in text.splitlines()
    )


def _setup_cache(
    run: Run, repo: Path, build_dir: Path, cache: cachecfg.CacheEnv, wrap=None
) -> dict | None:
    """Prepare caches + return the env overlay for ninja, or None if setup failed
    hard (ccache reconfigure error). C/C++ caching needs `-DENABLE_CCACHE` baked
    into build.ninja (ReactOS wires `RULE_LAUNCH_COMPILE ccache` behind it); Rust
    caching is pure env, no reconfigure."""
    cache.ccache_dir.mkdir(parents=True, exist_ok=True)
    cache.sccache_dir.mkdir(parents=True, exist_ok=True)
    env = {**os.environ, **cache.env()}

    if cache.use_ccache and not _ccache_already_enabled(build_dir):
        cmd = ["cmake", "-DENABLE_CCACHE=1", "."]
        cp = run.run(wrap(cmd) if wrap else cmd, cwd=build_dir, stage="ccache-config", timeout=600)
        if cp.returncode != 0:
            run.classification = "CMAKE_CONFIGURE_FAIL"
            run.add(
                Stage(
                    "ccache-config",
                    exit=cp.returncode,
                    status="fail",
                    detail="reconfigure for ENABLE_CCACHE failed",
                    salient=[ln.strip() for ln in cp.stderr.splitlines() if "Error" in ln][:3],
                )
            )
            return None
        run.add(Stage("ccache-config", status="ok", detail="ENABLE_CCACHE=1"))

    # Never reset shared counters: concurrent builds would erase each other's
    # measurements. Collection below is explicitly a shared snapshot, not a delta.
    run.add(Stage("cache", status="ok", detail=cache.summary()))
    return env


def _collect_cache_stats(
    run: Run, cache: cachecfg.CacheEnv, env: dict, *, repo: Path | None = None, wrap=None
) -> None:
    """Post-build compiler-cache telemetry, not a freshness verdict.

    A true no-op can have zero requests. Counters may include concurrent work:
    ccache is fleet-wide and canonical sccache is shared by this worktree's arches.
    """
    run.extra["cache_stats_scope"] = "shared cumulative snapshots; not per-build attribution"
    if cache.use_ccache:
        cp = run.run(
            [cache.ccache_bin or "ccache", "--print-stats"],
            cwd=run.dir,
            env=env,
            stage="ccache-stats",
            timeout=60,
        )
        st = cachecfg.parse_ccache_stats(cp.stdout)
        run.extra["ccache"] = st
        run.add(
            Stage(
                "ccache-stats",
                status="ok",
                detail=f"{st['hits']}h/{st['misses']}m  {st['hit_rate'] * 100:.0f}% hit",
            )
        )
    if cache.use_sccache:
        # The canonical wrapper selects the worktree's server and starts it in
        # the correct mount namespace. Outside it, stats describe another daemon.
        stats_cmd = [cache.sccache_bin or "sccache", "--show-stats", "--stats-format=json"]
        cp = run.run(
            wrap(stats_cmd) if wrap else stats_cmd,
            cwd=repo if wrap else run.dir,
            env=env,
            stage="sccache-stats",
            timeout=60,
        )
        st = cachecfg.parse_sccache_json(cp.stdout)
        run.extra["sccache"] = st
        if st:
            run.add(
                Stage(
                    "sccache-stats",
                    status="ok",
                    detail=f"{st['hits']}h/{st['misses']}m  {st['hit_rate'] * 100:.0f}% hit",
                )
            )


def _ninja_keep_going(args=None) -> list[str]:
    """`-k0` = don't stop at the first failure.

    Why it matters (2026-07-25): three independent latent breaks were sitting on mainline, and
    they were discovered ONE PER COLD BUILD — ninja halts at the first failing edge, so each
    ~6-minute build revealed exactly one, and the next only appeared after the previous was
    fixed. For a *verdict* build (the gate / the merge queue's batch) that is the wrong
    trade: we want the whole failure set in one pass, even though the log is longer. Local
    iteration keeps the default fail-fast, which is what you want when you are the one who
    just broke it."""
    return ["-k0"] if getattr(args, "keep_going", False) else []


def _ninja_did_work(stdout: str) -> int:
    """How many edges a ninja run actually executed, from its progress lines.

    Used by the re-affirm pass to say whether the optional targets invalidated
    anything. Counted rather than merely detected because "relinked 3" is the
    interesting number: a persistently non-zero count means a cohab cell is being
    rebuilt by two passes every single build, which is a wiring bug worth chasing,
    not just a hiccup this pass absorbed.

    Two things a naive `^\\[\\d+/\\d+\\]` count gets WRONG, both observed on a real
    no-op re-affirm that nonetheless reported "relinked 1 target(s)":

      [0/2] Re-checking globbed directories...
      ninja: no work to do.

    Nothing was rebuilt. `[0/...]` is a status pseudo-edge (the CONFIGURE_DEPENDS
    glob re-check), not a completed edge — real edges start at [1/n]. And when
    ninja says "no work to do" that is authoritative regardless of what it printed
    above. An always->=1 count would have made this signal useless in exactly the
    direction that hides the bug it exists to expose."""
    if "ninja: no work to do." in stdout:
        return 0
    return len(re.findall(r"^\[([1-9]\d*)/\d+\]", stdout, re.MULTILINE))


def _ninja_jobs() -> list[str]:
    """`-j N` cap on ninja parallelism. DEFAULT IS NOW UNCAPPED (reserve 0).

    The old default reserved 8 cores "so a co-running windows-host QEMU boot isn't
    starved into a false HANG_TIMEOUT" — a KVM-era vestige that doesn't survive scrutiny:
      * The smoke guest is UNIPROCESSOR: one TCG vCPU thread + an iothread, ~2 host
        threads total. On a many-core box CFS schedules those among a build's rustc
        fan-out without starving them; boot timeouts carry ~4× headroom besides.
      * The false-HANG incident that motivated the reserve (2026-06-19) was the NATIVE
        KVM-in-WSL backend (nested virt) starving on `/dev/kvm` + shared IDE between two
        KVM guests — an accelerator/disk fight, NOT CPU-core sharing of a TCG boot.
      * The DEFAULT windows-host TCG backend already runs guests off-host with no shared
        accelerator, which is exactly why `vm.py:per_arch_cap` BYPASSES the serialize cap
        and boots run in parallel. Reserving cores for one such boot contradicts that.
    So builds oversubscribe freely (compute-bound; the scheduler handles it). The knob
    survives for the one case it might matter again — build ∥ a WHPX boot, which is real
    hardware-virt and does contend — but even there the hostlock CPU-lease pool is the
    right throttle, not a blunt `-j` cap. Set VIBEOS_BUILD_CORE_RESERVE>0 to re-enable."""
    policy = hostlock.buildpolicy.read(hostlock.POLICY_DIR)
    if policy:
        return ["-j", str(policy["ninja_jobs"])]
    reserve = int(os.environ.get("VIBEOS_BUILD_CORE_RESERVE", "0"))
    n = os.cpu_count() or 0
    if reserve <= 0 or n <= reserve:
        return []
    return ["-j", str(n - reserve)]


def _policy_build_env(env: dict | None) -> dict | None:
    policy = hostlock.buildpolicy.read(hostlock.POLICY_DIR)
    if policy is None:
        return env
    return dict(os.environ if env is None else env, CARGO_BUILD_JOBS=str(policy["cargo_jobs"]))


def _ninja_known_targets(repo: Path, build_dir: Path, wrap, env) -> set[str]:
    """All declared ninja target paths (`ninja -t targets all`), for filtering the
    mkramdisk boot manifest down to buildable file targets. Plain subprocess (not
    run.run) — the ~40K-line target list would bloat every full.log."""
    try:
        cp = subprocess.run(
            wrap(["ninja", "-t", "targets", "all"]),
            cwd=build_dir,
            env=env,
            capture_output=True,
            text=True,
            timeout=300,
        )
    except (OSError, subprocess.SubprocessError):
        return set()
    return {ln.split(":", 1)[0].strip() for ln in cp.stdout.splitlines() if ":" in ln}


def build(args, repo: Path) -> int:
    arch = archcfg.base_arch(args.arch)  # build side keys on the base arch
    cfg = archcfg.arch_build(arch)
    build_dir = repo / f"build-{arch}"
    run = Run("build", arch)
    cold = bool(args.clean)

    # (Historical: --lockdep/--cswitchdep used to fail-fast on arm64 — the
    # instrumentation needed nightly-only link_llvm_intrinsics and the arm64
    # kernel builds on stable (E0554). The culprit capture is now the stable
    # code_addr!() inline-asm macro (ke/lockdep.rs), so all three arches arm
    # on their native channels.)

    if not build_dir.is_dir() and not canon.available():
        # With userns available a missing build dir is auto-provisioned by the
        # canonize stage below (fresh canonical configure, fast on cache hits).
        run.classification = "NO_BUILD_DIR"
        run.add(
            Stage(
                "preflight",
                status="fail",
                detail="build dir missing",
                salient=[f"{build_dir} does not exist — configure it first"],
            )
        )
        return run.finalize(repo)

    # Directory serialization is a correctness lock. CPU admission belongs to the
    # compiler/linker processes, after Cargo and Ninja have resolved their own waits.
    #
    # Disk is claimed HERE rather than at process start, so a build that never runs is
    # never billed. That began as protection against the cpuheavy pool -- three arches
    # launched, two permits, and the third refused for space the broker would never let
    # it use -- and the per-job broker above has retired that particular shape. What is
    # left is the narrower guarantee that the claim tracks the build, not the process.
    # Safe here only because reserve() never BLOCKS: it admits or raises at once.
    try:
        with hostlock.build_lock(build_dir, arch) as admission:
            hostjobs.ensure(repo)
            run.extra["admission"] = admission
            with diskspace.reserve(repo, "build"):
                return _build_locked(args, repo, run, arch, cfg, build_dir, cold)
    except hostlock.buildpolicy.PolicyError as exc:
        run.classification = "BUILD_POLICY_REFUSED"
        run.add(Stage("admission", status="fail", detail=str(exc)))
        return run.finalize(repo)


def _flag_in_cache(build_dir: Path, flag: str) -> bool:
    cache = build_dir / "CMakeCache.txt"
    try:
        for line in cache.read_text(errors="ignore").splitlines():
            if line.startswith(f"{flag}:"):
                return line.split("=", 1)[-1].strip() in ("ON", "1", "TRUE")
    except OSError:
        pass
    return False


def _value_in_cache(build_dir: Path, var: str) -> str:
    """The cached STRING value of `var`, or "" when absent. Mirrors
    _flag_in_cache; separate because an empty string and OFF are different
    answers and conflating them would make "clear the quarantine" a no-op."""
    cache = build_dir / "CMakeCache.txt"
    try:
        text = cache.read_text(errors="ignore")
    except OSError:
        return ""
    for line in text.splitlines():
        if line.startswith(f"{var}:"):
            return line.split("=", 1)[1].strip() if "=" in line else ""
    return ""


def _maybe_canonize(
    run,
    repo: Path,
    build_dir: Path,
    arch: str,
    instr_flags: dict[str, bool],
    sticky_on: tuple[str, ...] = (),
    instr_values: dict[str, str] | None = None,
) -> bool | None:
    """Ensure the build dir is configured at the CANONICAL path (see canon.py) so
    caches share across worktrees. Returns True → wrap build cmds in canon-run,
    False → legacy real-path passthrough (userns unavailable), None → hard fail.

    A legacy (real-path) build dir is nuked + fresh-configured canonically — a
    LOUD one-time full rebuild that repopulates the shared cache; every later
    build (any worktree) hits it.

    `sticky_on` names cmake options to turn ON and then LEAVE ON, the opposite
    discipline from instr_flags: they are one-directional, so a later plain build
    never flips one back OFF. Cache state a caller opted into (the kmtest corpus)
    is expensive to rebuild and harmless to keep, unlike instrumentation, which a
    gate must never silently inherit."""
    if canon.is_canonical(build_dir):
        # Instrumentation flags (lockdep §4 / cswitchdep) are per-BUILD state,
        # not sticky cache state — reconcile every build so a plain (no-flag)
        # build deterministically flips a stale ON back OFF and the gate never
        # inherits an instrumented kernel.
        stale = {
            flag: on for flag, on in instr_flags.items() if on != _flag_in_cache(build_dir, flag)
        }
        # Sticky opt-ins ride the same reconfigure, but only ever ON -> they are
        # absent from `stale` when already set, so a no-op costs nothing.
        # Deliberately an exact per-flag check, not the OR that modules/CMakeLists.txt
        # applies (kmtests build under `ENABLE_ROSTESTS OR ENABLE_ROSTESTS_KMTESTS`):
        # a dir with only the parent ON therefore takes one redundant reconfigure,
        # which is cached and harmless, and that is worth more than teaching this
        # generic mechanism which cmake options imply which others.
        stale.update({f: True for f in sticky_on if not _flag_in_cache(build_dir, f)})
        # STRING-valued instrumentation reconciles on the same pass, and has to
        # take part in the "is anything stale" decision or it never applies: the
        # bools were already at their target, so nothing reconfigured and the
        # new value sat unused. That is how a quarantine list silently does
        # nothing while the build reports success.
        stale_values = {
            k: v for k, v in (instr_values or {}).items() if v != _value_in_cache(build_dir, k)
        }
        incomplete = not (build_dir / "build.ninja").is_file()
        if stale or stale_values or incomplete:
            defs = [f"-D{flag}={'ON' if on else 'OFF'}" for flag, on in stale.items()]
            defs += [f"-D{k}={v}" for k, v in stale_values.items()]
            cp = run.run(
                canon.wrap(repo, ["cmake", *defs, "."]),
                cwd=build_dir,
                stage="instr-config",
                timeout=600,
            )
            if cp.returncode != 0:
                run.classification = "CMAKE_CONFIGURE_FAIL"
                run.add(
                    Stage(
                        "instr-config",
                        exit=cp.returncode,
                        status="fail",
                        detail=f"cmake {' '.join(defs)} reconfigure failed",
                        salient=[
                            ln.strip()
                            for ln in (cp.stdout + cp.stderr).splitlines()
                            if "Error" in ln
                        ][:3],
                    )
                )
                return None
            if not (build_dir / "build.ninja").is_file():
                run.classification = "CMAKE_CONFIGURE_FAIL"
                run.add(
                    Stage("instr-config", status="fail", detail="cmake produced no build.ninja")
                )
                return None
            run.add(
                Stage(
                    "instr-config",
                    status="ok",
                    detail=" ".join(defs) or "recovered incomplete configure",
                )
            )
        return True
    if not canon.available():
        run.add(
            Stage(
                "canon",
                status="warn",
                detail="userns unavailable — legacy real-path build (no cross-worktree cache)",
            )
        )
        return False
    if not canon.ensure_toolchain(repo):
        run.classification = "CMAKE_CONFIGURE_FAIL"
        run.add(
            Stage(
                "canonize",
                status="fail",
                detail="no llvm-mingw toolchain found",
                salient=[".build-tools/llvm-mingw missing and no sibling llvm-mingw-* dir"],
            )
        )
        return None
    carried = canon.carried_options(build_dir)
    carried = [*carried, *(f"-D{flag}=ON" for flag, on in instr_flags.items() if on)]
    # STRING-valued instrumentation cannot ride instr_flags (bools only). These
    # reconcile for free: canon._CARRY does not list them, so a build that omits
    # the flag reconfigures without it. For the winetest quarantine that is the
    # safe direction — a stale skip list would silently subtract subtests from a
    # sweep whose whole job is to run them all.
    for _k, _v in (instr_values or {}).items():
        if _v:
            carried = [*carried, f"-D{_k}={_v}"]
    # A dir that does not exist yet carries nothing, so a sticky opt-in has to be
    # named on the FRESH configure too — otherwise the caller's request is silently
    # dropped exactly once, on the one run that cannot detect it.
    carried = [*carried, *(f"-D{flag}=ON" for flag in sticky_on if f"-D{flag}=ON" not in carried)]
    fresh = not (build_dir / "CMakeCache.txt").is_file()
    if build_dir.exists():
        shutil.rmtree(build_dir)  # legacy caches bake real paths into every generated file
    build_dir.mkdir(parents=True)
    cp = run.run(
        canon.wrap(repo, canon.configure_cmd(arch, carried)),
        cwd=build_dir,
        stage="canonize",
        timeout=1800,
    )
    if cp.returncode != 0:
        run.classification = "CMAKE_CONFIGURE_FAIL"
        run.add(
            Stage(
                "canonize",
                exit=cp.returncode,
                status="fail",
                detail="fresh canonical configure failed",
                salient=[
                    ln.strip() for ln in (cp.stdout + cp.stderr).splitlines() if "Error" in ln
                ][:3],
            )
        )
        return None
    run.add(
        Stage(
            "canonize",
            status="ok",
            detail=(
                f"{'fresh configure' if fresh else 'migrated legacy dir'} @ "
                f"{canon.canon_path()}" + ("" if fresh else " (one-time full rebuild)")
            ),
        )
    )
    return True


def _isolation_preflight(run, repo: Path, build_dir: Path) -> bool:
    """Footgun guard (the 2026-07-04 dedup incident, both failure modes): a
    multi-link file inside a build tree means some cross-worktree hardlink
    tool has fused this tree's artifacts with another's. In-place writers
    (spec2def, `cmake -E copy`, ccache-MISS clang) then corrupt the sibling
    through the shared inode — SIGBUS under a running ld.lld — and shared
    mtimes silently defeat cargo invalidation (stale build scripts re-run
    with no error anywhere). Refuse to build until the tree is private again.

    Thresholds: cmake outputs never legitimately hardlink → `-links +1`
    fails. CARGO target dirs DO hardlink internally (deps/ ↔ final artifact,
    2 links, same tree) → only `-links +2` fails there (a 2-way cross-tree
    link slips through; the mass-dedup signature is 3+, observed up to 1003).
    Cargo dirs are detected by their CACHEDIR.TAG marker — several nest
    INSIDE build-<arch> (rust-kernel-target, rust-wmilib-target, …), so the
    cmake scan prunes them and they get the cargo threshold instead. See
    tools/dedup-build-cache.sh SAFETY for history."""

    def _cargo_roots(root: Path) -> list[Path]:
        try:
            cp = subprocess.run(
                ["find", str(root), "-name", "CACHEDIR.TAG"],
                capture_output=True,
                text=True,
                timeout=300,
            )
        except (OSError, subprocess.SubprocessError):
            return []
        dirs = sorted(Path(ln).parent for ln in cp.stdout.splitlines() if ln.strip())
        # keep only top-most roots (cargo stamps per-triple subdirs too)
        tops: list[Path] = []
        for d in dirs:
            if not any(str(d).startswith(str(t) + os.sep) for t in tops):
                tops.append(d)
        return tops

    def _probe_cmd(root: Path, links: str, prune: list[Path], *, all_hits: bool) -> list[str]:
        cmd: list[str] = ["find", str(root)]
        for p in prune:
            cmd += ["-path", str(p), "-prune", "-o"]
        cmd += ["-type", "f", "-links", links, "-print"]
        if not all_hits:
            cmd += ["-quit"]
        return cmd

    def _probe(root: Path, links: str, prune: list[Path]) -> str | None:
        try:
            cp = subprocess.run(
                _probe_cmd(root, links, prune, all_hits=False),
                capture_output=True,
                text=True,
                timeout=300,
            )
        except (OSError, subprocess.SubprocessError):
            run.add(Stage("isolation", status="warn", detail=f"probe failed for {root} — skipped"))
            return None
        return cp.stdout.strip() or None

    def _probe_all(root: Path, links: str, prune: list[Path]) -> list[str]:
        try:
            cp = subprocess.run(
                _probe_cmd(root, links, prune, all_hits=True),
                capture_output=True,
                text=True,
                timeout=300,
            )
        except (OSError, subprocess.SubprocessError):
            run.add(Stage("isolation", status="warn", detail=f"probe failed for {root} — skipped"))
            return []
        return [ln for ln in cp.stdout.splitlines() if ln.strip()][:200]

    def _foreign_links(cands: list[str]) -> list[str]:
        """Filter multi-link cargo files down to the ones whose links are NOT
        all inside this worktree. Cargo's incremental sessions legitimately
        hardlink metadata.rmeta into deps/ (nlink = 1 + N session dirs, all
        in-tree), so nlink>2 alone is NOT the dedup signature — a link path
        OUTSIDE the worktree is. One find walk accounts every candidate inode."""
        infos: dict[int, list] = {}
        for h in cands:
            try:
                st = os.stat(h)
            except OSError:
                continue
            infos.setdefault(st.st_ino, [st.st_nlink, h])
        if not infos:
            return []
        cond: list[str] = []
        for ino in infos:
            cond += ["-inum", str(ino), "-o"]
        cmd = ["find", str(repo), "-type", "f", "(", *cond[:-1], ")", "-printf", "%i\n"]
        try:
            cp = subprocess.run(cmd, capture_output=True, text=True, timeout=300)
        except (OSError, subprocess.SubprocessError):
            # Accounting failed — fail SAFE (treat as foreign; a spurious
            # refusal is recoverable, a corrupted sibling build is not).
            return [h for _, h in infos.values()]
        seen: dict[int, int] = {}
        for ln in cp.stdout.splitlines():
            if ln.strip().isdigit():
                ino = int(ln)
                seen[ino] = seen.get(ino, 0) + 1
        return [h for ino, (nlink, h) in infos.items() if seen.get(ino, 0) < nlink]

    hits: list[str] = []
    if build_dir.is_dir():
        nested_cargo = _cargo_roots(build_dir)
        hit = _probe(build_dir, "+1", nested_cargo)  # cmake outputs: any sharing is foreign
        if hit:
            hits.append(f"{hit} (links 2+ in a cmake tree)")
        for c in nested_cargo:
            for f in _foreign_links(_probe_all(c, "+2", [])):
                hits.append(f"{f} (links 3+ in a cargo dir, link escapes this worktree)")
    ws = repo / "sdk" / "lib" / "rust" / "target"
    if ws.is_dir():
        for f in _foreign_links(_probe_all(ws, "+2", [])):
            hits.append(f"{f} (links 3+ in a cargo dir, link escapes this worktree)")
    if not hits:
        return True
    run.classification = "ISOLATION_FAIL"
    run.add(
        Stage(
            "isolation",
            status="fail",
            detail="build tree shares inodes with another worktree — refusing to build",
            salient=[f"first hit: {h}" for h in hits]
            + [
                "fix: find <dir> -type f -links +N -print0 | xargs -0 -P8 -I{} "
                'sh -c \'cp -p "{}" "{}.t" && mv "{}.t" "{}"\'  (or delete + rebuild)',
                "cause: a cross-worktree hardlink dedup ran — see the SAFETY header of "
                "tools/dedup-build-cache.sh",
            ],
        )
    )
    return False


# T1.2 flag→artifact verification: a requested build feature must PROVE itself
# in the output binary, and must be ABSENT when not requested — the --lockdep
# flag was silently dead for 3 builds (canonize early-return) and only a manual
# `llvm-nm` caught it. Table rows: (cache flag, requested?, artifact, symbol).
_FLAG_SIGNATURES = [
    (
        "VIBEOS_LOCKDEP",
        # cswitchdep IMPLIES the lockdep cargo feature (held-set mirror), so its
        # symbol is legitimately present in a --cswitchdep build too.
        lambda args: (
            bool(getattr(args, "lockdep", False)) or bool(getattr(args, "cswitchdep", False))
        ),
        Path("ntoskrnl") / "ntoskrnl.exe",
        "VIBEOS_LOCKDEP",
    ),
    (
        "VIBEOS_CSWITCHDEP",
        lambda args: bool(getattr(args, "cswitchdep", False)),
        Path("ntoskrnl") / "ntoskrnl.exe",
        "VIBEOS_CSWITCHDEP",
    ),
]


def _verify_flag_artifacts(run, repo: Path, build_dir: Path, args) -> bool:
    nm = repo / ".build-tools" / "llvm-mingw" / "bin" / "llvm-nm"
    ok = True
    for flag, wanted_fn, rel, symbol in _FLAG_SIGNATURES:
        artifact = build_dir / rel
        if not nm.is_file() or not artifact.is_file():
            continue  # ninja already failed / non-standard tree; nothing to verify
        wanted = wanted_fn(args)
        try:
            cp = subprocess.run(
                [str(nm), str(artifact)], capture_output=True, text=True, timeout=120
            )
            # i386 decorates C-ABI statics with a leading underscore
            # (_VIBEOS_LOCKDEP) — match both forms or every i386 armed build
            # false-fails (found arming i386 for the first time, 2026-07-07).
            present = any(
                ln.endswith(f" {symbol}") or ln.endswith(f" _{symbol}")
                for ln in cp.stdout.splitlines()
            )
        except (OSError, subprocess.SubprocessError):
            run.add(Stage("flag-verify", status="warn", detail=f"llvm-nm failed for {rel}"))
            continue
        if present == wanted:
            run.add(
                Stage(
                    "flag-verify",
                    status="ok",
                    detail=f"{flag}={'ON' if wanted else 'OFF'} ↔ {symbol} "
                    f"{'present' if present else 'absent'} in {rel.name}",
                )
            )
        else:
            ok = False
            run.classification = run.classification or "FLAG_VERIFY_FAIL"
            run.add(
                Stage(
                    "flag-verify",
                    status="fail",
                    detail=(
                        f"{flag} requested but {symbol} ABSENT from {rel} — the flag is "
                        f"silently dead (plumbing broke somewhere between cmake and cargo)"
                        if wanted
                        else f"{flag} OFF but {symbol} PRESENT in {rel} — instrumentation "
                        f"leaked into a production build"
                    ),
                    salient=[f"artifact: {artifact}"],
                )
            )
    return ok


def _build_locked(args, repo: Path, run, arch: str, cfg, build_dir: Path, cold: bool) -> int:
    if not _isolation_preflight(run, repo, build_dir):
        return run.finalize(repo)
    wrapped = _maybe_canonize(
        run,
        repo,
        build_dir,
        arch,
        {
            "VIBEOS_LOCKDEP": bool(getattr(args, "lockdep", False)),
            "VIBEOS_CSWITCHDEP": bool(getattr(args, "cswitchdep", False)),
            # SMP bring-up media knob: livecd freeldr.ini defaults to the
            # LiveCD_SMP entry (ntkrnlmp.exe + halmp.dll). Reconciled every build
            # like the instrumentation flags so a plain build flips it back OFF.
            # Since halmp.dll became its own cell this knob rewrites `DefaultOS=`
            # and NOTHING ELSE — both kernels and both HALs are on every media
            # regardless, so flipping it no longer relinks the HAL (it used to
            # cost 9m25s i386 / 12m22s amd64 warm, which is why there has never
            # been an MP gate leg).
            "VIBEOS_UP_DEFAULT_BOOT": bool(getattr(args, "up_kernel", False)),
            # i386 HAL flavor: APIC (LAPIC/IOAPIC/TPR) is the DEFAULT as of the
            # i386-apic-hal lane landing. The legacy 8259 PIC is the opt-out
            # escape hatch (`--hal-pic` → -DVIBEOS_PIC_HAL=ON); reconciled every
            # build so a plain build reverts to the APIC HAL. i386 only — amd64
            # hal is always APIC.
            "VIBEOS_PIC_HAL": bool(getattr(args, "hal_pic", False)),
            # The full winetest corpus (`--rostests`). RECONCILED, not sticky,
            # which is the opposite call from ENABLE_ROSTESTS_KMTESTS below and
            # for a reason: kmtests are cheap to keep and something (conformance)
            # always wants them, whereas ENABLE_ROSTESTS adds ~128 executables
            # that no default build has any use for. Sticky would mean one
            # opt-in silently bloats every later i386 build in the worktree --
            # the exact surprise this flag exists to avoid. Toggling is cheap
            # after the first build: turning it OFF only drops the targets from
            # build.ninja, so the objects survive and turning it back ON is an
            # up-to-date ninja pass.
            "ENABLE_ROSTESTS": bool(getattr(args, "rostests", False)),
            # The winetest SWEEP marker (`--winetest-sweep`). Reconciled for
            # the same reason as ENABLE_ROSTESTS and then some: a sweep is
            # minutes of boot, and leaving it latched on would quietly turn
            # every later boot in the worktree into one. It only stages a
            # marker file, so toggling costs a media rebuild and nothing else.
            "VIBEOS_WINETEST_SWEEP": bool(getattr(args, "winetest_sweep", False)),
        },
        # Sticky, never reconciled OFF: the kmtest corpus is what `conformance`
        # sweeps, and both cmake options gating it default OFF, so a build dir
        # this worktree just created has no tests in it. Turning it on here is
        # the whole fix for a fresh worktree building a full media and only
        # finding out at `[KMTEST-SWEEP] launch FAILED`, minutes later and a
        # layer away from the cause.
        sticky_on=tuple(
            opt
            for opt, flag in (
                ("ENABLE_ROSTESTS_KMTESTS", "kmtests"),
                # Sticky for the same reason as kmtests: the apitest payload
                # sweeps this corpus, the cmake option defaults OFF, and a fresh
                # worktree that silently built no apitests only says so minutes
                # later at `[APITEST-SWEEP] launch FAILED`.
                ("ENABLE_ROSTESTS_APITESTS", "apitests"),
            )
            if getattr(args, flag, False)
        ),
        instr_values={
            "VIBEOS_WINETEST_SKIP": (getattr(args, "winetest_skip", "") or "").strip(),
        },
    )
    if wrapped is None:
        return run.finalize(repo)

    def wrap(cmd: list[str]) -> list[str]:
        return canon.wrap(repo, cmd) if wrapped else cmd

    shards = getattr(args, "rust_storage_shards", None)
    if shards is not None:
        cp = run.run(
            wrap(["cmake", f"-DVIBEOS_RUST_STORAGE_SHARDS={shards}", "."]),
            cwd=build_dir,
            stage="storage-config",
            timeout=600,
        )
        if cp.returncode:
            run.classification = "CMAKE_CONFIGURE_FAIL"
            run.add(Stage("storage-config", status="fail", exit=cp.returncode))
            return run.finalize(repo)

    # --- payload selection stamp (docs/engineering/payload-harness-design.md) ---
    # Persist the env's VIBEOS_PAYLOAD/VIBEOS_DOOM selection into the build dir
    # BEFORE ninja runs the media steps, so env-less rebuilds (conftest's
    # clean-build `ninja livecd`, raw canon-run ninja, smoke's net stage)
    # reproduce the same media instead of silently reverting the payload set
    # (the 2026-07-07 doom-clobber incident). Front door sets or CLEARS it:
    # a build with no selection in env stamps <none>.
    import importlib.util

    _spec = importlib.util.spec_from_file_location(
        "payload_compile", repo / "tools" / "payloads" / "compile.py"
    )
    assert _spec is not None and _spec.loader is not None
    _payload_compile = importlib.util.module_from_spec(_spec)
    _spec.loader.exec_module(_payload_compile)

    stamped = _payload_compile.write_stamp(build_dir)
    if stamped:
        run.add(Stage("payloads", status="ok", detail=",".join(stamped)))

    if cold:
        _clean(run, repo, arch)

    # --- reproducible builds: pin SOURCE_DATE_EPOCH (rung 2b) ---
    # Set it in our env so every subprocess (cmake configure inside _setup_cache,
    # ninja compile, cargo) inherits it — no per-call threading. setdefault so an
    # externally pinned value (CI) wins. Honored by cmake string(TIMESTAMP) and clang
    # __DATE__/__TIME__; verify the effect with `vibeos-cli repro report`.
    sde = source_date_epoch(repo)
    if sde:
        os.environ.setdefault("SOURCE_DATE_EPOCH", sde)

    # --- build cache (mandatory; opt-out: --no-cache / VIBEOS_BUILD_CACHE=0) ---
    cache = cachecfg.make(repo)
    caching = cachecfg.cache_enabled(args)
    build_env: dict | None = None
    if caching:
        build_env = _setup_cache(run, repo, build_dir, cache, wrap=wrap)
        if build_env is None:
            return run.finalize(repo)  # ccache reconfigure failed hard

    policy = hostlock.buildpolicy.read(hostlock.POLICY_DIR)
    if policy:
        build_env = _policy_build_env(build_env)
        run.extra["build_policy"] = policy

    # --- fast-iterate: rustc incremental (opt-in; NOT for gate/determinism) ---
    # On a kernel .rs edit, `ntoskrnl-rust` is ONE 451K-LOC crate → rustc recompiles
    # the whole crate (measured ~13s vs ~5s with incremental). sccache CAN'T help a
    # changed crate (input hash differs → miss), and sccache + incremental are mutually
    # exclusive, so this trades the Rust sccache wrapper for rustc's within-crate reuse.
    # ccache (C/C++) stays on. Incremental artifacts are non-reproducible, so this is a
    # LOCAL fast-iterate lever only — the gate/CI path keeps sccache (default off).
    incremental = getattr(args, "incremental", False) or os.environ.get(
        "VIBEOS_RUST_INCREMENTAL", ""
    ).lower() in ("1", "true", "yes")
    if incremental and build_env is not None:
        build_env["CARGO_INCREMENTAL"] = "1"
        build_env["RUSTC_WRAPPER"] = ""  # disable sccache for rustc (incompatible)
        build_env["CARGO_BUILD_RUSTC_WRAPPER"] = ""  # also override a config-set wrapper
        run.add(
            Stage(
                "incremental",
                status="warn",
                detail="rustc INCREMENTAL on, Rust sccache OFF (fast-iterate; non-reproducible)",
            )
        )

    build_env = hostjobs.environment(
        repo, build_env, canonical=canon.is_canonical(build_dir), run_dir=run.dir
    )

    # --- generated-header barrier ---
    # On a FRESH tree (canonize/provision) some RC + cc-rs compiles race ahead of
    # the psdk/xdk winnt.h generators — dep-edge rot that months of copied build
    # dirs masked (winnt.h always pre-existed). Prime the generator targets as an
    # explicit barrier; warm trees no-op in ~1s.
    cp = run.run(
        wrap(["ninja", *_ninja_jobs(), "host-tools", "asm", "psdk", "xdk", "bugcodes", "errcodes"]),
        cwd=build_dir,
        env=build_env,
        stage="ninja-prime",
        timeout=1800,
    )
    if cp.returncode != 0:
        cls, salient = _classify_build_failure(cp.stdout, cp.stderr)
        run.classification = cls
        run.add(
            Stage(
                "ninja-prime",
                exit=cp.returncode,
                status="fail",
                detail=cls,
                salient=salient or ["see full log"],
            )
        )
        return run.finalize(repo)
    run.add(Stage("ninja-prime", status="ok", detail="generated headers"))

    # --- ninja the boot target(s) ---
    # arm64 (mkramdisk arches): force-build the ENTIRE boot manifest (mkramdisk
    # REQUIRED_FILES — file paths are valid ninja targets), not just the curated
    # ninja_targets list. Two bugs this kills: (a) the cohab-flip false-green —
    # a Rust .sys/.dll missing from ninja_targets left a STALE C binary that
    # mkramdisk staged (fastfat/ramdisk/advapi32 class); (b) the fresh-tree gap —
    # a canonized/provisioned build-arm64 had never built smss.exe etc. at all
    # (the curated list assumed a previously-full tree). Deriving from the boot
    # manifest keeps it self-maintaining; ninja dedups anything a phony target
    # already covers. x86 builds the full `livecd` DAG and needs neither.
    targets = list(cfg.ninja_targets)
    if cfg.needs_mkramdisk:
        # Not every staged file is a ninja target (e.g. media/nls/l_intl.nls is
        # copied by an aggregate rule) — intersect with the known target list or
        # ninja hard-errors on the first unknown.
        known = _ninja_known_targets(repo, build_dir, wrap, build_env)
        cohab = [
            r for r in provenance.mkramdisk_required_rels(repo) if r not in targets and r in known
        ]
    else:
        cohab = [r for r in provenance.boot_set_cohab_rels(repo, arch) if r not in targets]
    if cohab:
        run.log(
            "force-building the boot-manifest set (self-heal):\n" + "\n".join(cohab),
            header="cohab",
        )
    cp = run.run(
        wrap(["ninja", *_ninja_jobs(), *_ninja_keep_going(args), *targets, *cohab]),
        cwd=build_dir,
        env=build_env,
        stage="ninja",
        timeout=3600,
    )
    if cp.returncode != 0:
        cls, salient = _classify_build_failure(cp.stdout, cp.stderr)
        run.classification = cls
        run.add(
            Stage(
                "ninja",
                exit=cp.returncode,
                status="fail",
                detail=cls,
                salient=salient or ["see full log"],
            )
        )
        return run.finalize(repo)
    run.add(Stage("ninja", status="ok", detail="+".join(targets)))

    # --- flag→artifact verification (T1.2): requested features must PROVE
    # themselves in the output; unrequested ones must be absent. ---
    if not _verify_flag_artifacts(run, repo, build_dir, args):
        return run.finalize(repo)

    # optional arch-specific targets — ONE ninja per target, and every outcome
    # REPORTED. "Optional" means the target may be ABSENT (not in this build's
    # ninja graph); it has never meant "ignore whether it built".
    #
    # Two ways this used to lie, both of which stage a STALE binary into the
    # arm64 ramdisk and still go green — the exact false-green class archcfg's
    # comment block exists to prevent:
    #   1. Batching (`ninja a b c`): ninja rejects the WHOLE command on the first
    #      unknown target, so one absent target silently skipped the present
    #      ones. kmtest_all only exists under -DENABLE_ROSTESTS(_KMTESTS)=ON
    #      (both default OFF), so a kmtests-off dir took arm64input/arm64vid —
    #      and ntkrnlmp — down with it.
    #   2. Discarding the result: a target that EXISTS but FAILS TO COMPILE was
    #      swallowed whole. arm64input could stop building entirely and the build
    #      stayed ✓ while mkramdisk staged the last-good .sys.
    #
    # So: absence is a VISIBLE skip; a real build failure FAILS the build. And
    # the polarity is fail-loud — only ninja's recognised "unknown target" counts
    # as absence. Anything we can't classify is a failure, never a shrug.
    for target in cfg.ninja_targets_optional:
        cp = run.run(
            wrap(["ninja", *_ninja_jobs(), target]),
            cwd=build_dir,
            env=build_env,
            stage=f"ninja-optional:{target}",
            timeout=900,
        )
        if cp.returncode == 0:
            run.add(Stage(f"opt:{target}", status="ok"))
            continue
        blob = (cp.stdout or "") + (cp.stderr or "")
        if "unknown target" in blob.lower():
            run.add(Stage(f"opt:{target}", status="skip", detail="absent from ninja graph"))
            continue
        cls, salient = _classify_build_failure(cp.stdout or "", cp.stderr or "")
        run.classification = cls
        run.add(
            Stage(
                f"opt:{target}",
                exit=cp.returncode,
                status="fail",
                detail=f"optional target FAILED to build ({cls}) — not absent, broken",
                salient=salient,
            )
        )
        return run.finalize(repo)

    # --- re-affirm the main targets (the optional passes can invalidate them) ---
    # An optional target shares cargo cells with the main graph: `ntkrnlmp` and
    # `ntoskrnl` both build `-p kernel-rust` into ONE CARGO_TARGET_DIR and one
    # copied `libkernel_rust.a`. So `ninja ntkrnlmp`, running as its own pass
    # AFTER the main one, can re-emit that staticlib with new bytes — and nothing
    # relinks ntoskrnl.exe, because the main pass is already over. Measured on
    # amd64: libkernel_rust.a rebuilt at 02:08:11, ntkrnlmp.exe linked against it
    # at 02:08:23, ntoskrnl.exe left at 00:27:35 — 1h40m older than the archive it
    # links. That is a genuinely stale binary, and mkramdisk (below) would stage
    # it, so this MUST run before the staging step, not after.
    #
    # One more `ninja` over the same target list closes it by construction: a
    # no-op in the normal case (~1s), a real relink when a cohab lib moved. It is
    # not a retry — a target that FAILED already returned above — so a non-zero
    # exit here means the re-affirm itself broke something, and it fails the build.
    cp = run.run(
        wrap(["ninja", *_ninja_jobs(), *_ninja_keep_going(args), *targets, *cohab]),
        cwd=build_dir,
        env=build_env,
        stage="ninja-reaffirm",
        timeout=1800,
    )
    if cp.returncode != 0:
        cls, salient = _classify_build_failure(cp.stdout, cp.stderr)
        run.classification = cls
        run.add(
            Stage(
                "ninja-reaffirm",
                exit=cp.returncode,
                status="fail",
                detail=f"relink after the optional passes failed ({cls})",
                salient=salient or ["see full log"],
            )
        )
        return run.finalize(repo)
    relinked = _ninja_did_work(cp.stdout or "")
    run.add(
        Stage(
            "ninja-reaffirm",
            status="ok",
            detail=(
                f"relinked {relinked} target(s) invalidated by the optional passes"
                if relinked
                else "no-op — optional passes invalidated nothing"
            ),
        )
    )

    # --- cache hit-rate stats (also the determinism gauge) ---
    if caching and build_env is not None:
        _collect_cache_stats(run, cache, build_env, repo=repo, wrap=wrap)

    # --- arm64 re-stage into ramdisk/boot.img ---
    if cfg.needs_mkramdisk:
        mk = repo / "qemu" / "mkramdisk.sh"
        ramdisk = repo / "qemu" / "ramdisk.img"
        flag = "--refresh" if ramdisk.is_file() else ""
        cmd = ["bash", str(mk)] + ([flag] if flag else [])
        # Both pairs are staged unconditionally and freeldr.ini ships both entries;
        # the MP one is the DEFAULT since 2026-09-12. `--up` here moves `DefaultOS=`
        # back to the UP entry at BUILD time, which is the only way to get UP media.
        # Nothing automated passes it -- gate, smoke and conformance all test the MP
        # pair, because UP is a dead branch we are not shipping.
        if getattr(args, "up_kernel", False):
            cmd.append("--up")
        cp = run.run(cmd, cwd=repo, stage="mkramdisk", timeout=900)
        if cp.returncode != 0:
            run.classification = "RESTAGE_FAIL"
            run.add(
                Stage(
                    "mkramdisk",
                    exit=cp.returncode,
                    status="fail",
                    detail="ramdisk re-stage",
                    salient=cp.stderr.splitlines()[-3:],
                )
            )
            return run.finalize(repo)
        run.add(Stage("mkramdisk", status="ok", detail=flag or "full"))

    # --- binary lints (need the built binary) ---
    lints.run_binary_lints(run, repo, arch, strict=bool(getattr(args, "strict", False)))

    # --- symbol index for triage ---
    bi = repo / "qemu" / "build-index.sh"
    if bi.is_file():
        cp = run.run(["bash", str(bi), arch], cwd=repo, stage="build-index", timeout=300)
        run.add(
            Stage("build-index", exit=cp.returncode, status="ok" if cp.returncode == 0 else "warn")
        )

    # Source analysis belongs to check/gate; local build opts in explicitly.
    if getattr(args, "check", False):
        lints.run_source_lints(run, repo, strict=False)

    # --- write provenance stamp ---
    stamp = provenance.make_stamp(repo, arch, cold=cold, ts=int(time.time()))
    stamp.write(build_dir)
    run.add(Stage("stamp", status="ok", detail="cold" if cold else "warm"))

    # --- artifact self-check: a "successful" build must end with real, fresh
    # boot media. Catches the silent-false-success class at the source instead
    # of at smoke time (or never): sandbox/overlay runs that report rc=0 with no
    # persisted artifacts (previously tribal knowledge in AGENTS.md only), and a
    # ninja run that skipped a stale cargo-linked PE / cohab binary (the same
    # mtime nets smoke's preflight uses — the 2026-07-04 ntoskrnl cargo-link
    # false-green class, now caught before anyone boots it).
    st = provenance.check_staleness(repo, arch)
    if st.media_missing or st.stale:
        run.classification = "BUILD_INCOMPLETE"
        run.add(
            Stage(
                "artifact-check",
                status="fail",
                detail="boot media missing or stale after a 'successful' build",
                salient=st.reasons[:4],
            )
        )
        return run.finalize(repo)
    run.add(Stage("artifact-check", status="ok", detail="media present + fresh"))

    # NOTE: auto hardlink-dedup of cache trees was RETIRED 2026-07-04 (unsound on
    # live trees — SIGBUS + mtime-poisoned false-green; see tools/dedup-build-cache.sh
    # SAFETY + AGENTS.md "Fleet isolation law"). Disk reclaim is manual: `vibeos-cli prune`.
    if not cold:
        run.add(
            Stage(
                "warm-build", status="warn", detail="cargo incremental may mask; --clean to verify"
            )
        )
    return run.finalize(repo)
