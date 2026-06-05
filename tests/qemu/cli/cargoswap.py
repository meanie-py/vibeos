"""vibeos-cli cargo-swap: the RUNTIME gate for the cargo-driven link PoCs.

`check --strict`'s cargo_link guard only RELINKS each cargo-driven module and
asserts a valid PE — it never boots, so a regression that links fine but boots
broken slips straight through. This verb closes that gap: it boot-swaps each
cargo-built module into the real livecd and boots it to the smoke marker (the
full kernel -> SMSS -> csrss -> win32k -> winlogon -> explorer chain), proving
RUNTIME equivalence rather than just PE shape.

Covers the cargo-deep ntdll DLLs (x86) AND the cargo-driven ntoskrnl.exe kernel
(i386 — its boot-swap also applies the kernel post-link: native-pefixup --kernel
+ the i386 forwarder fix). It wraps each module's boot-swap-smoke.sh (which owns
the build / swap / cmp-assert / restore-on-exit mechanics and boots through
`vibeos-cli smoke`), and aggregates one verdict + summary.json. x86 only — arm64
has no livecd for the swap.
"""
from __future__ import annotations

from pathlib import Path

from .archcfg import base_arch
from .artifacts import Run, Stage

# Per-module boot-swap: module -> (script, passes (arch,module) args, arches).
# ntdll/ntdll_vista share one script that takes an [arch] [module] pair (x86).
# ntoskrnl has its own i386-only kernel boot-swap (no args; standalone PoC).
_SWAPS = {
    "ntdll":       ("sdk/lib/rust/ntdll-cargo/boot-swap-smoke.sh",    True,  ("i386", "amd64")),
    "ntdll_vista": ("sdk/lib/rust/ntdll-cargo/boot-swap-smoke.sh",    True,  ("i386", "amd64")),
    "ntoskrnl":    ("sdk/lib/rust/ntoskrnl-cargo/boot-swap-smoke.sh", False, ("i386",)),
}
_ORDER = ("ntdll", "ntdll_vista", "ntoskrnl")


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
    # boot-swap scripts + livecd media key off the bare arch (i386/amd64); the
    # -uefi aliases resolve to the same x86 livecd the inner `vibeos-cli smoke`
    # picks. arm64 has no livecd.
    arch = base_arch(args.arch)
    run = Run("cargoswap", arch)

    if arch not in ("i386", "amd64"):
        run.add(Stage(f"cargoswap:{arch}", status="skip",
                      detail="no livecd for boot-swap (x86 only)"))
        return run.finalize(repo)

    sel = list(_ORDER) if args.module == "all" else [args.module]
    for mod in sel:
        script_rel, takes_module, arches = _SWAPS[mod]
        stage = f"boot-swap:{mod}"
        if arch not in arches:
            run.add(Stage(stage, status="skip",
                          detail=f"{mod} is {'/'.join(arches)}-only"))
            continue
        script = repo / script_rel
        if not script.is_file():
            run.add(Stage(stage, status="fail", detail=f"missing {script_rel}"))
            continue
        cmd = ["bash", str(script), arch, mod] if takes_module else ["bash", str(script)]
        cp = run.run(cmd, cwd=repo, stage=stage, timeout=1800)
        salient = _salient(cp.stdout, cp.stderr)
        if cp.returncode == 0:
            run.add(Stage(stage, exit=0, status="ok",
                          detail="booted to smoke marker", salient=salient))
        else:
            run.add(Stage(stage, exit=cp.returncode, status="fail",
                          detail="relinked clean but failed RUNTIME boot",
                          salient=salient))

    return run.finalize(repo)
