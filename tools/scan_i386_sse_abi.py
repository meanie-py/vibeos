#!/usr/bin/env python3
"""scan_i386_sse_abi.py — a loud tripwire for the i386 no-SSE build's ABI time bomb.

THE TIME BOMB
    54 i386 CMakeLists build their Rust against the BUILTIN target
    `i686-pc-windows-gnu` while passing `-C target-feature=-sse,-sse2`. SSE must be
    off: on i386 the kernel runs pool init in Phase 0 before CR4.OSFXSR is set, so
    any SSE instruction faults. But that builtin target's spec declares
    `rustc-abi: x86-sse2`, and disabling sse2 under it contradicts the declared ABI.
    rustc (1.92.0 and the pinned nightly) already emits:

        warning: target feature `sse2` must be enabled to ensure that the ABI of the
        current target can be implemented correctly ... this was previously accepted
        by the compiler but is being phased out; IT WILL BECOME A HARD ERROR IN A
        FUTURE RELEASE  (rust-lang#116344)

    It is only a WARNING today (exit 0), so nothing is broken. The trigger is a
    toolchain bump — exactly the moment the pin is raised — and when it fires it
    takes down all 54 i386 crates at once with a generic rustc error that points at
    none of this.

WHY A SCANNER AND NOT "the build will fail anyway"
    It will — cryptically, deep in a 54-target build, 54 times, naming neither the
    cause nor the fix. This tripwire runs in `check-tooling` (a fast land preflight,
    before any build) and turns that into ONE actionable red: the pinned rustc has
    turned the phase-out into a hard error, here is the migration. The person who
    bumps the pin is the person who needs to hear it, and this is where they look.

THE SANCTIONED FIX (when it trips)
    The softfloat ABI the kernel already uses elsewhere: a CUSTOM target json with
    `rustc-abi: x86-softfloat` (the i386 `*-cargo` cells are on `i686-kernel.json`).
    A custom json needs cargo to rebuild `core`, i.e. `-Z build-std` = nightly — the
    one sanctioned i386 nightly use (AGENTS.md). So the migration is: convert each of
    the 54 `add_custom_command` + `cargo build --target i686-pc-windows-gnu` crates
    into a build-std cargo cell on a softfloat json, the model `ntoskrnl-cargo` et al.
    already follow. It is deferred-BY-DESIGN while rustc only warns, because it is a
    large change that widens nightly surface for no live bug. Roadmap:
    "54 i386 crates build on an SSE/ABI combination rustc is phasing out".

WHAT COUNTS AS A DOOMED CMakeLists
    One that carries `-C target-feature=-sse,-sse2` AND the builtin `i686-pc-windows-gnu`
    triple AND no custom `.json` target. (A crate already on a softfloat json is safe.)
"""

from __future__ import annotations

import argparse
import subprocess
import tempfile
from pathlib import Path

SSE_FLAG = "target-feature=-sse,-sse2"
BUILTIN_TRIPLE = "i686-pc-windows-gnu"
PHASE_OUT_NOTE = "being phased out"

# Classification of the rustc probe.
OK_LATENT = "OK_LATENT"  # still a warning (exit 0) — expected today
TRIPPED = "TRIPPED_HARD_ERROR"  # now a hard error — the migration is DUE
ABI_CHANGED = "ABI_CHANGED"  # compiles with no phase-out note — target/ABI moved
UNAVAILABLE = "UNAVAILABLE"  # rustc or the i686 target not installed here


def repo_root() -> Path:
    here = Path(__file__).resolve()
    for p in (here, *here.parents):
        if (p / "AGENTS.md").exists() and (p / "sdk").is_dir():
            return p
    return here.parents[1]


def find_doomed_cmakelists(root: Path) -> list[Path]:
    """CMakeLists carrying the SSE flag + the builtin triple + no custom json target."""
    hits: list[Path] = []
    for path in root.rglob("CMakeLists.txt"):
        # skip anything under a build-* / target dir
        parts = set(path.parts)
        if any(p.startswith("build-") or p == "target" for p in parts):
            continue
        try:
            text = path.read_text(errors="replace")
        except OSError:
            continue
        if SSE_FLAG in text and BUILTIN_TRIPLE in text and "i686-kernel.json" not in text:
            hits.append(path)
    return sorted(hits)


class Probe:
    def __init__(self, verdict: str, returncode: int | None, phased_out: bool, stderr: str):
        self.verdict = verdict
        self.returncode = returncode
        self.phased_out = phased_out
        self.stderr = stderr


def probe_rustc(target: str = BUILTIN_TRIPLE) -> Probe:
    """Compile a trivial crate with the doomed flag+triple and classify the result."""
    try:
        installed = subprocess.run(
            ["rustc", "--print", "target-list"],
            capture_output=True,
            text=True,
            check=False,
        )
    except FileNotFoundError:
        return Probe(UNAVAILABLE, None, False, "rustc not found")
    if installed.returncode != 0 or target not in installed.stdout.split():
        return Probe(UNAVAILABLE, None, False, f"target {target} not available to rustc")

    with tempfile.TemporaryDirectory() as td:
        src = Path(td) / "t.rs"
        src.write_text('#![crate_type = "lib"]\npub fn f() {}\n')
        out = Path(td) / "t.rlib"
        proc = subprocess.run(
            [
                "rustc",
                "--edition",
                "2021",
                "--target",
                target,
                "-C",
                f"{SSE_FLAG}",
                str(src),
                "-o",
                str(out),
            ],
            capture_output=True,
            text=True,
            check=False,
        )
        stderr = proc.stderr
        phased = PHASE_OUT_NOTE in stderr
        # A no-std target with no prebuilt core would fail for an unrelated reason;
        # i686-pc-windows-gnu ships std, so a nonzero rc here is the ABI hard error.
        if proc.returncode != 0:
            # Distinguish the ABI error from an environment failure (missing std, etc.).
            abi_ish = ("sse2" in stderr and "ABI" in stderr) or "x86-softfloat" in stderr
            if abi_ish:
                return Probe(TRIPPED, proc.returncode, phased, stderr)
            return Probe(UNAVAILABLE, proc.returncode, phased, stderr)
        if phased:
            return Probe(OK_LATENT, 0, True, stderr)
        return Probe(ABI_CHANGED, 0, False, stderr)


def main(argv: list[str] | None = None) -> int:
    ap = argparse.ArgumentParser(
        description=__doc__, formatter_class=argparse.RawDescriptionHelpFormatter
    )
    ap.add_argument("--list", action="store_true", help="list the doomed CMakeLists and exit")
    args = ap.parse_args(argv)

    root = repo_root()
    doomed = find_doomed_cmakelists(root)

    if args.list:
        for p in doomed:
            print(p.relative_to(root))
        print(f"\n{len(doomed)} CMakeLists on the builtin i686 triple + -sse,-sse2.")
        return 0

    probe = probe_rustc()
    print(
        f"i386 no-SSE ABI tripwire — {len(doomed)} doomed CMakeLists, "
        f"rustc verdict: {probe.verdict}"
    )

    if probe.verdict == TRIPPED:
        print(
            "\nTRIPPED: the pinned rustc now makes `-C target-feature=-sse,-sse2` on the\n"
            f"builtin `{BUILTIN_TRIPLE}` target a HARD ERROR (rust-lang#116344). All\n"
            f"{len(doomed)} i386 crates below will fail to build.\n\n"
            "Fix: migrate them to a softfloat custom target json (rustc-abi: x86-softfloat)\n"
            "built with `-Z build-std` (nightly), the model the i386 `*-cargo` cells already\n"
            "use (sdk/lib/rust/ntoskrnl-cargo/i686-kernel.json + gcc.cmake add_cargo_module).\n"
            'Roadmap: "54 i386 crates build on an SSE/ABI combination rustc is phasing out".\n'
        )
        for p in doomed:
            print(f"  {p.relative_to(root)}")
        return 1

    if probe.verdict == ABI_CHANGED:
        print(
            "\nNOTE: rustc compiled the doomed flag+triple with NO phase-out warning. The\n"
            "builtin target's ABI may have changed upstream; re-evaluate whether the\n"
            "`-sse,-sse2` flag is still needed, or whether a softfloat ABI is now builtin.\n"
        )
        return 0

    if probe.verdict == UNAVAILABLE:
        print("\nNOTE: rustc or the i686 target is unavailable here; tripwire not evaluated.")
        return 0

    print("\nOK: still a warning (latent). No action until the pin is bumped.")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
