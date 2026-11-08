"""`vibeos-cli build` — per-arch ninja + arm64 re-stage + binary-lints +
build-index + provenance stamp (plan §7/§8/§9).

warm by default; --clean nukes ALL discovered rust target dirs + lib*_rust.a
(prints the count); classifies CMAKE_CONFIGURE_FAIL / CARGO_CHECK_FAIL /
LINK_FAIL distinctly with i386 PE-decoration hints.
"""

from __future__ import annotations

import os
import subprocess
import sys
from pathlib import Path

from tools import build_infra_compat as native

from . import (
    archcfg,
    hostdisk,
    hostlock,
)
from . import cachecfg as cachecfg
from . import canon as canon
from . import diskspace as diskspace
from . import hostjobs as hostjobs
from . import ladder as ladder
from . import lints as lints
from . import provenance as provenance
from . import trim as trim
from . import warmth as warmth
from .artifacts import Run, Stage

ROOT = Path(__file__).resolve().parents[3]


def _mechanism(run, repo, operation, **arguments):
    from .build_mechanisms import call

    return call(run, repo, operation, **arguments)


def _execute(argv, *, cwd=ROOT, timeout=None, check=False, **_):
    from .gate import _execute as execute

    result = execute(argv, cwd=cwd, timeout=timeout)
    if check and result.returncode:
        raise subprocess.CalledProcessError(result.returncode, argv, result.stdout, result.stderr)
    return result


DEDUP_UNIT = Path("/etc/systemd/system/vibeos-dedup.service")


def kick_dedup() -> Stage:
    return Stage(**_mechanism(None, ROOT, "dedup", unit=DEDUP_UNIT)[0])


def _decoration_hint(sym):
    return native.call("cli_build_check_decoration", {"symbol": sym})


def source_date_epoch(repo: Path) -> str | None:
    """Commit-date epoch for reproducible builds (deterministic-builds-plan.md rung 2b).

    `git log -1 --format=%ct` → the HEAD commit's timestamp. Deriving from the *commit*
    (not wall-clock) makes the build byte-identical across worktrees/agents on the same
    commit while keeping a meaningful date. Pins both cmake's `string(TIMESTAMP)` in
    version.cmake (→ buildno.h version resources) and clang's `__DATE__`/`__TIME__`
    (e.g. ntoskrnl/ex/xipdisp.c) — both honor the SOURCE_DATE_EPOCH env var. Returns
    None outside a git repo (caller leaves the env untouched)."""
    return native.call("build_policy_source_date_epoch", {"repo": repo})


NINJA_TIMEOUT_ENV = "VIBEOS_BUILD_TIMEOUT"


#: Generated-header / host-tool barrier targets built before the boot set (ninja-prime).
_PRIME_TARGETS = ("host-tools", "asm", "psdk", "xdk", "bugcodes", "errcodes")


def _ninja_timeout():
    try:
        return native.call(
            "cli_build_check_timeout", {"raw": os.environ.get(NINJA_TIMEOUT_ENV, "")}
        )
    except native.NativeError as error:
        print(error, file=sys.stderr)
        raise SystemExit(2) from error


def _failed_edges(blob):
    return [(edge, lines) for edge, lines in native.call("cli_build_check_edges", {"text": blob})]


def _classify_build_failure(out, err, rc=None):
    disk = hostdisk.enospc_detail() if hostdisk.looks_like_enospc(out + "\n" + err) else []
    return tuple(
        native.call(
            "cli_build_check_classify",
            {"stdout": out, "stderr": err, "rc": rc, "timeout": _ninja_timeout(), "disk": disk},
        )
    )


def _ccache_already_enabled(build_dir: Path) -> bool:
    return native.call(
        "build_policy_cmake_value", {"build": build_dir, "key": "ENABLE_CCACHE"}
    ) in ("ON", "1", "TRUE")


def _setup_cache(run, repo, build_dir, cache, wrap=None):
    return _mechanism(
        run,
        repo,
        "setup_cache",
        build_dir=build_dir,
        cache=cache,
        wrapped=wrap is not None,
        wrap=wrap,
    )[0]


def _collect_cache_stats(run, cache, env, *, repo=None, wrap=None):
    _mechanism(
        run, repo or ROOT, "cache_stats", cache=cache, env=env, wrapped=wrap is not None, wrap=wrap
    )


def _ninja_keep_going(args=None):
    return native.call(
        "cli_build_check_keep", {"args": {"keep_going": bool(getattr(args, "keep_going", False))}}
    )


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
    return native.call(
        "build_policy_ninja_jobs",
        {
            "policy": hostlock.buildpolicy.read(hostlock.POLICY_DIR),
            "environment": dict(os.environ),
            "cpus": os.cpu_count() or 0,
        },
    )


def _policy_build_env(env: dict | None) -> dict | None:
    return native.call(
        "build_policy_build_env",
        {
            "policy": hostlock.buildpolicy.read(hostlock.POLICY_DIR),
            "env": env,
            "environment": dict(os.environ),
        },
    )


def build(args, repo):
    from .build_entry_bindings import call

    arch = archcfg.base_arch(args.arch)
    run = Run("build-headers" if getattr(args, "headers_only", False) else "build", arch)
    return call(
        args,
        repo,
        run,
        arch,
        archcfg.arch_build(arch),
        repo / f"build-{arch}",
        bool(getattr(args, "clean", False)),
        "run",
    )


def _trim_own_target(repo):
    _mechanism(None, repo, "trim")


def _failure_over_moved_tree(run, repo, start_tree):
    _mechanism(run, repo, "failure_moved", tree=start_tree)


def _inputs_moved(repo, start_tree):
    return _mechanism(None, repo, "inputs_moved", tree=start_tree)[0]


def _scheduled_store_gc(args):
    _mechanism(None, ROOT, "store_gc", args=args)


def _flag_in_cache(build_dir: Path, flag: str) -> bool:
    return _value_in_cache(build_dir, flag) in ("ON", "1", "TRUE")


def _value_in_cache(build_dir: Path, var: str) -> str:
    """The cached STRING value of `var`, or "" when absent. Mirrors
    _flag_in_cache; separate because an empty string and OFF are different
    answers and conflating them would make "clear the quarantine" a no-op."""
    return native.call("build_policy_cmake_value", {"build": build_dir, "key": var})


def _maybe_canonize(run, repo, build_dir, arch, instr_flags, sticky_on=(), instr_values=None):
    return _mechanism(
        run,
        repo,
        "canonize",
        build_dir=build_dir,
        arch=arch,
        flags=instr_flags,
        sticky=sticky_on,
        values=instr_values or {},
    )[0]


def _stage_payloads(run, repo, build_dir, arch):
    result, bindings = _mechanism(run, repo, "stage_payloads", arch=arch, build_dir=build_dir)
    return None if result is None else (result, bindings.payload_compile)


def _hold_wow64_source(run, repo, arch, args, reads):
    from .build_entry_bindings import call

    return call(
        args, repo, run, arch, None, repo / f"build-{arch}", False, "hold_source", reads=reads
    )


def _refresh_wow64_guests(run, repo, stage="wow64-guests"):
    return _mechanism(run, repo, "refresh_guests", stage=stage)[0]


def wowx64_cpu_stale(repo):
    return _mechanism(None, repo, "cpu_stale")[0]


# The out-of-tree FEX builds mkramdisk stages from, in dependency order: the vendored FEX
# (libFEXCore.a + the i386 guests' libwow64fex.dll, staged on EVERY arm64 image), then the
# x86-64 CPU layer and the probe harness, which both link that libFEXCore.a.
_CONSTANTS = _mechanism(None, ROOT, "constants")[0]
FEX_OUT_OF_TREE = _CONSTANTS["fex_out_of_tree"]
WOWX64_CPU_DLL = Path(_CONSTANTS["cpu_dll"])
WOWX64_CPU_SOURCES = tuple(_CONSTANTS["cpu_sources"])


def fex_prerequisites_missing(repo, environ=os.environ):
    return _mechanism(None, repo, "fex_prerequisites", env=dict(environ))[0]


def default_artifact_store():
    from tools import qemu_infra_compat

    return Path(
        qemu_infra_compat.call(
            "cli_build_entry_default_store", {"env": dict(os.environ), "home": Path.home()}
        )
    )


def _build_supervised(args, repo, run, arch, cfg, build_dir, cold):
    from .build_entry_bindings import call

    return call(args, repo, run, arch, cfg, build_dir, cold, "supervised")


def _build_locked(args, repo, run, arch, cfg, build_dir, cold, *, artifact_env=None):
    from .build_bindings import locked

    return locked(args, repo, run, arch, cfg, build_dir, cold, artifact_env=artifact_env)
