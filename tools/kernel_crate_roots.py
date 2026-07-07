"""Single source of truth for the "kernel crate" source roots.

The ntoskrnl Rust translation started as one monolithic crate
(`ntoskrnl-rust`). As subsystems are peeled off into their own crates for
compile-time isolation (the crate-split effort — see
docs/engineering/ntoskrnl-crate-split-plan.md), any lint/verify tool that
hardcoded `sdk/lib/rust/ntoskrnl-rust/src` as its sole scan root would silently
DROP the extracted subsystem from coverage. That's a silent-degradation trap.

So kernel-scoped tools import this list instead of hardcoding a root. When a
subsystem is extracted into a new crate, add its `src` dir here ONCE and every
consumer keeps full coverage.

Consumers: tools/lint_kernel.py, tools/verify_signatures.py. (verify_ffi.py
already scans every `*/src` in the workspace, so it needs no entry.) Migration/
codegen tools that still hardcode ntoskrnl-rust (gen_global_barrier.py,
apply_opaque_ptr.py, fix_abi_mismatch.py, …) are one-shot and out of gate scope;
migrate them to this list if/when they're re-run against a split kernel.
"""

from pathlib import Path

# Each entry is a crate `src` dir, relative to the repo root. ntoskrnl-rust is
# the (shrinking) monolith; the rest are extracted subsystems.
KERNEL_CRATE_SRC_ROOTS = [
    "sdk/lib/rust/ntoskrnl-rust/src",
    "sdk/lib/rust/lpc-rust/src",  # extracted 2026-07 (crate-split PoC)
    "sdk/lib/rust/fsrtl-rust/src",  # extracted 2026-07 (crate-split)
    "sdk/lib/rust/dbgk-rust/src",  # extracted 2026-07 (crate-split)
    "sdk/lib/rust/wmi-rust/src",  # extracted 2026-07 (crate-split)
    "sdk/lib/rust/dbgprint-rust/src",  # print-infra leaf hoisted 2026-07 (crate-split Phase 1b)
    "sdk/lib/rust/cc-rust/src",  # extracted 2026-07 (crate-split Phase 1b)
    "sdk/lib/rust/iomgr-rust/src",  # extracted 2026-07 (crate-split Phase 1b)
    "sdk/lib/rust/kdbg-rust/src",  # extracted 2026-07 (crate-split Phase 1b, x86-only)
    "sdk/lib/rust/config-rust/src",  # extracted 2026-07 (crate-split Phase 1b)
    "sdk/lib/rust/kd-rust/src",  # extracted 2026-07 (crate-split Phase 1b)
    "sdk/lib/rust/kd64-rust/src",  # extracted 2026-07 (crate-split Phase 1b)
    "sdk/lib/rust/vf-rust/src",  # extracted 2026-07 (crate-split Phase 1b)
    "sdk/lib/rust/rtlk-rust/src",  # extracted 2026-07 (crate-split Phase 1b)
    "sdk/lib/rust/po-rust/src",  # extracted 2026-07 (crate-split Phase 1b)
    "sdk/lib/rust/inbv-rust/src",  # extracted 2026-07 (crate-split Phase 1b)
    "sdk/lib/rust/vdm-rust/src",  # extracted 2026-07 (crate-split Phase 1b, i386-only)
    "sdk/lib/rust/fstub-rust/src",  # extracted 2026-07 (crate-split Phase 1b)
    "sdk/lib/rust/se-rust/src",  # extracted 2026-07 (crate-split Phase 1b)
    "sdk/lib/rust/pnpmgr-rust/src",  # extracted 2026-07 (crate-split Phase 1b)
]


def kernel_rs_files(repo_root: Path = Path(".")):
    """Yield every .rs file across all kernel-crate roots, sorted, existing-only."""
    files = []
    for rel in KERNEL_CRATE_SRC_ROOTS:
        d = repo_root / rel
        if d.exists():
            files.extend(sorted(d.rglob("*.rs")))
    return files
