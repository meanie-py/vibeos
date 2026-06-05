"""vibeos-cli cargo-swap: the RUNTIME gate for the cargo-driven link PoCs.

`check --strict`'s cargo_link guard only RELINKS each cargo-driven module and
asserts a valid PE — it never boots, so a regression that links fine but boots
broken slips straight through. This verb closes that gap: it boot-swaps each
cargo-built DLL into the real livecd and boots it to the smoke marker (the full
kernel -> SMSS -> csrss -> win32k -> winlogon -> explorer chain, all of which
depends on ntdll), proving RUNTIME equivalence rather than just PE shape.

It wraps the per-module boot-swap-smoke.sh (which owns the careful build / swap /
cmp-assert / restore-on-exit mechanics and already boots through
`vibeos-cli smoke`), and aggregates one verdict + summary.json in the standard
artifact dir so the gate is a single discoverable command instead of an orphan
script nothing calls. x86 only — arm64 has no livecd for the swap.
"""
from __future__ import annotations

from pathlib import Path

from .archcfg import base_arch
from .artifacts import Run, Stage

# One script handles both DLLs (it takes a [arch] [module] pair); ntoskrnl-cargo
# has no boot-swap (reuse-19-.a relink only), so the runtime gate is the two
# cargo-deep ntdll DLLs.
_SCRIPT = "sdk/lib/rust/ntdll-cargo/boot-swap-smoke.sh"
_MODULES = ("ntdll", "ntdll_vista")


def _salient(stdout: str, stderr: str) -> list[str]:
    """Pull the few lines worth surfacing: the inner `vibeos-cli smoke` artifact
    pointer + the boot-swap verdict line. Keeps stdout bounded (plan §6)."""
    keep: list[str] = []
    for ln in (stdout or "").splitlines():
        s = ln.strip()
        if s.startswith(("full log:", "[5/5]", "FAIL")) or "confirmed: livecd carries" in s:
            keep.append(s)
    if not keep:
        keep = [ln.strip() for ln in (stderr or "").splitlines() if ln.strip()][-3:]
    return keep[-4:]


def cargo_swap(args, repo: Path) -> int:
    # boot-swap-smoke.sh + livecd media key off the bare arch (i386/amd64); the
    # -uefi aliases resolve to the same x86 livecd the inner `vibeos-cli smoke`
    # picks. arm64 has no livecd.
    arch = base_arch(args.arch)
    run = Run("cargoswap", arch)

    if arch not in ("i386", "amd64"):
        run.add(Stage(f"cargoswap:{arch}", status="skip",
                      detail="no livecd for boot-swap (x86 only)"))
        return run.finalize(repo)

    script = repo / _SCRIPT
    if not script.is_file():
        run.add(Stage("cargoswap", status="fail", detail=f"missing {_SCRIPT}"))
        return run.finalize(repo)

    modules = list(_MODULES) if args.module == "all" else [args.module]
    for mod in modules:
        stage = f"boot-swap:{mod}"
        cp = run.run(["bash", str(script), arch, mod], cwd=repo,
                     stage=stage, timeout=1800)
        salient = _salient(cp.stdout, cp.stderr)
        if cp.returncode == 0:
            run.add(Stage(stage, exit=0, status="ok",
                          detail="booted to smoke marker", salient=salient))
        else:
            run.add(Stage(stage, exit=cp.returncode, status="fail",
                          detail="cargo DLL relinked clean but failed RUNTIME boot",
                          salient=salient))

    return run.finalize(repo)
