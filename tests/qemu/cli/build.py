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

from . import archcfg, cachecfg, canon, hostlock, lints, provenance
from .artifacts import Run, Stage

_UNDEF = re.compile(r"undefined (?:symbol|reference)[^\w]*[`']?([@\w]+)", re.I)
_DUP = re.compile(r"duplicate symbol[:\s]+[`']?([@\w]+)", re.I)


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
    if "cmake error" in low:
        return "CMAKE_CONFIGURE_FAIL", [
            ln.strip() for ln in blob.splitlines() if "CMake Error" in ln
        ][:3]
    # Link errors first (a duplicate/undefined symbol contains "error:" too, so
    # it must be matched before the generic cargo "error:" catch).
    undef = list(dict.fromkeys(_UNDEF.findall(blob)))
    dup = list(dict.fromkeys(_DUP.findall(blob)))
    if undef or dup or "ld.lld:" in low or "duplicate symbol" in low:
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
    ws_target = provenance.workspace_cargo_target(repo, arch)
    if ws_target is not None:
        targets = [*targets, ws_target]
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

    # zero per-build counters so the post-build stats are this build's delta
    if cache.use_ccache:
        run.run(
            [cache.ccache_bin or "ccache", "-z"], cwd=repo, env=env, stage="ccache-zero", timeout=60
        )
    if cache.use_sccache:
        run.run(
            [cache.sccache_bin or "sccache", "--zero-stats"],
            cwd=repo,
            env=env,
            stage="sccache-zero",
            timeout=60,
        )
    run.add(Stage("cache", status="ok", detail=cache.summary()))
    return env


def _collect_cache_stats(run: Run, cache: cachecfg.CacheEnv, env: dict) -> None:
    """Post-build hit rates — also the determinism gauge: a no-source-change
    rebuild should be ~100% hits; any miss = something varied (plan §determinism)."""
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
        cp = run.run(
            [cache.sccache_bin or "sccache", "--show-stats", "--stats-format=json"],
            cwd=run.dir,
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


def _ninja_jobs() -> list[str]:
    """`-j N` args that cap ninja parallelism to leave host cores free for a concurrent
    windows-host QEMU boot. The build runs in WSL and the boot's `qemu.exe` runs on the
    Windows side, but they share the same physical CPU — and ninja's default is `nproc+2`
    (34 on a 32-thread box), which oversubscribes AND starves a co-running boot into a
    false HANG_TIMEOUT. Reserve cores (default 8) so build ∥ boot; VIBEOS_BUILD_CORE_RESERVE
    tunes it, 0 = uncapped (ninja's own default). Pairs with hostlock's separate boot pool."""
    reserve = int(os.environ.get("VIBEOS_BUILD_CORE_RESERVE", "8"))
    n = os.cpu_count() or 0
    if reserve <= 0 or n <= reserve:
        return []
    return ["-j", str(n - reserve)]


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


_DEDUP_INTERVAL = 24 * 3600  # hardlink-dedup the cache trees at most daily


def _maybe_dedup(run: Run, repo: Path) -> None:
    """Opportunistic disk reclaim: hardlink byte-identical build artifacts across
    the ~100 per-consumer cargo target dirs + worktrees (tools/dedup-build-cache.sh
    — the deliberate price of per-crate target isolation; see that script's
    header). Detached so the build verdict never waits on a multi-GB scan, and
    throttled via a stamp under VIBEOS_ROOT. The script itself skips trees with
    writes in the last 180s (live builds), so a detached run is safe."""
    script = repo / "tools" / "dedup-build-cache.sh"
    if not script.is_file():
        return
    stamp = cachecfg.vibeos_root(repo) / ".dedup-last"
    try:
        if stamp.is_file() and time.time() - stamp.stat().st_mtime < _DEDUP_INTERVAL:
            return
        stamp.touch()
    except OSError:
        return
    log = run.dir / "dedup.log"
    with open(log, "ab") as fh:
        subprocess.Popen(
            ["bash", str(script), "--apply"],
            stdout=fh,
            stderr=fh,
            start_new_session=True,
            cwd=repo,
        )
    run.add(Stage("dedup", status="ok", detail=f"detached, ≤1/day → {log}"))


def build(args, repo: Path) -> int:
    arch = archcfg.base_arch(args.arch)  # build side keys on the base arch
    cfg = archcfg.arch_build(arch)
    build_dir = repo / f"build-{arch}"
    run = Run("build", arch)
    cold = bool(args.clean)

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

    # Host CPU broker: a build is the writer in the writer-takes-all CPU pool — it grabs
    # ALL leases (exclusive host, so no concurrent TCG boot starves into a false
    # HANG_TIMEOUT) AND the per-build-dir lock (so two builds never corrupt the same dir,
    # the determinism-verify-vs-gate incident). Held around the whole build; on a stuck
    # holder it warns + proceeds unlocked, and it's a no-op when the broker is disabled.
    with hostlock.build_lock(build_dir, arch):
        return _build_locked(args, repo, run, arch, cfg, build_dir, cold)


def _maybe_canonize(run, repo: Path, build_dir: Path, arch: str) -> bool | None:
    """Ensure the build dir is configured at the CANONICAL path (see canon.py) so
    caches share across worktrees. Returns True → wrap build cmds in canon-run,
    False → legacy real-path passthrough (userns unavailable), None → hard fail.

    A legacy (real-path) build dir is nuked + fresh-configured canonically — a
    LOUD one-time full rebuild that repopulates the shared cache; every later
    build (any worktree) hits it."""
    if canon.is_canonical(build_dir):
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


def _build_locked(args, repo: Path, run, arch: str, cfg, build_dir: Path, cold: bool) -> int:
    wrapped = _maybe_canonize(run, repo, build_dir, arch)
    if wrapped is None:
        return run.finalize(repo)

    def wrap(cmd: list[str]) -> list[str]:
        return canon.wrap(repo, cmd) if wrapped else cmd

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
        wrap(["ninja", *_ninja_jobs(), *targets, *cohab]),
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

    # optional arch-specific targets (best-effort)
    if cfg.ninja_targets_optional:
        run.run(
            wrap(["ninja", *cfg.ninja_targets_optional]),
            cwd=build_dir,
            env=build_env,
            stage="ninja-optional",
            timeout=900,
        )

    # --- cache hit-rate stats (also the determinism gauge) ---
    if caching and build_env is not None:
        _collect_cache_stats(run, cache, build_env)

    # --- arm64 re-stage into ramdisk/boot.img ---
    if cfg.needs_mkramdisk:
        mk = repo / "qemu" / "mkramdisk.sh"
        ramdisk = repo / "qemu" / "ramdisk.img"
        flag = "--refresh" if ramdisk.is_file() else ""
        cmd = ["bash", str(mk)] + ([flag] if flag else [])
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

    # --- auto-check (advisory) unless --no-check ---
    if not getattr(args, "no_check", False):
        lints.run_source_lints(run, repo, strict=False)

    # --- write provenance stamp ---
    stamp = provenance.make_stamp(repo, arch, cold=cold, ts=int(time.time()))
    stamp.write(build_dir)
    run.add(Stage("stamp", status="ok", detail="cold" if cold else "warm"))

    # --- opportunistic hardlink dedup of cache trees (detached, throttled) ---
    _maybe_dedup(run, repo)
    if not cold:
        run.add(
            Stage(
                "warm-build", status="warn", detail="cargo incremental may mask; --clean to verify"
            )
        )
    return run.finalize(repo)
