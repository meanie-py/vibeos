#!/usr/bin/env python3
"""scan_bind_all_matches.py — find match arms that silently became catch-all bindings.

The bug class (found 2026-07-05 in usbport, cost USB enumeration entirely): a
translated `match` arm pattern names a constant that is NOT resolvable in scope
(`IRP_MN_QUERY_ID => {...}` with no such const imported). Rust silently treats
the name as a *binding pattern that matches everything*; rustc only warns.
Every arm after it is dead code, and the linker then garbage-collects the dead
arms' dependencies — hiding missing symbols and no-op stubs until someone makes
the code live.

Detection is compile-based (ground truth, not grep): run `cargo check` with
`--force-warn` for the three diagnostic signatures a bind-all arm produces —
overriding the crates' blanket #[allow]s — and flag:

  - unreachable_patterns   (arms after a bind-all; also plain overlap bugs)
  - unused_variables       where the "variable" is SCREAMING_CASE (the binder
                           itself — catches a bind-all even as the final arm)
  - non_snake_case         same SCREAMING_CASE filter, complementary coverage

Usage:
  .venv/bin/python tools/scan_bind_all_matches.py                  # whole workspace
  .venv/bin/python tools/scan_bind_all_matches.py -p usbport-rust  # one crate
  .venv/bin/python tools/scan_bind_all_matches.py --target aarch64-pc-windows-gnullvm

Non-default targets need (a) the matching build-<arch>/ dir fully built (crate
build.rs scripts read compile_commands.json + freeldr objs from it) and (b) the
canonical-path mount: `tools/canon-run.sh python3 tools/scan_bind_all_matches.py
--target …` — build dirs bake /tmp/vibeos paths. The default x86_64 target with
build-amd64 present works from the plain worktree. Arch-gated crates are
excluded per target below. The diagnostics this scan reads (name resolution)
are mostly target-independent, so the default-target run in `vibeos-cli check`
covers the bulk; alt-target runs are belt-and-braces for cfg'd code.

Exit 1 if any finding, 0 if clean. Output: one line per finding,
`file:line: [KIND] name — detail`.
"""

import argparse
import json
import os
import re
import subprocess
import sys
import tempfile
from pathlib import Path

REPO = Path(__file__).resolve().parent.parent
WORKSPACE = REPO / "sdk" / "lib" / "rust"

CONST_LIKE = re.compile(r"^[A-Z][A-Z0-9_]{2,}$")

# unused variable: `IRP_MN_QUERY_ID`
RE_UNUSED = re.compile(r"unused variable: `([^`]+)`")
# variable `IRP_MN_QUERY_ID` should have a snake case name
RE_SNAKE = re.compile(r"variable `([^`]+)` should have a snake case name")

FORCE_WARN = [
    "--force-warn",
    "unreachable_patterns",
    "--force-warn",
    "unused_variables",
    "--force-warn",
    "non_snake_case",
]


def primary_span(diag):
    for span in diag.get("spans", []):
        if span.get("is_primary"):
            return f"{span['file_name']}:{span['line_start']}"
    return "?:?"


def is_match_arm(loc: str) -> bool:
    """True if the diagnostic's source line looks like a match arm (`X =>`).
    Filters non_snake_case/unused_variables noise from fn params etc."""
    try:
        fname, line_s = loc.rsplit(":", 1)
        path = (WORKSPACE / fname) if not fname.startswith("/") else Path(fname)
        line = path.read_text(errors="replace").splitlines()[int(line_s) - 1]
        return "=>" in line
    except (OSError, IndexError, ValueError):
        return True  # can't read — keep the finding rather than hide it


def scan(target: str, packages: list[str], quiet: bool) -> list[str]:
    findings = []
    seen = set()

    cmd = [
        # --release matches the production cfg (kernel/freeldr builds are
        # release; some crates have debug_assertions-only code that no longer
        # compiles and would abort a dev-profile check).
        "cargo",
        "check",
        "--release",
        "--message-format=json",
        "--target",
        target,
        "--manifest-path",
        str(WORKSPACE / "Cargo.toml"),
    ]
    if packages:
        for p in packages:
            cmd += ["-p", p]
    else:
        cmd += ["--workspace"]
        # Arch-gated crates: hard-error on the wrong target under --workspace.
        excludes = {
            # kdvm/hal: x86-only. difftests: build.rs needs the aarch64 cross-cc
            # on PATH, absent in the plain scan env (they're host-test crates;
            # their arm64 coverage is marginal).
            "aarch64-pc-windows-gnullvm": [
                "kdvm-cargo",
                "hal-rust",
                "rtl-difftest",
                "cnt-difftest",
            ],
        }
        for p in excludes.get(target, []):
            cmd += ["--exclude", p]

    env = dict(os.environ)
    # Default toolchain (has the windows-gnu targets installed); the kernel
    # cargo-links pin nightly separately, driver crates build on default.
    env["RUSTC_BOOTSTRAP"] = "1"
    # Own scratch target dir: --force-warn changes flags, so never pollute the
    # shared build caches with force-warn artifacts.
    env["CARGO_TARGET_DIR"] = env.get(
        "SCAN_BINDALL_TARGET_DIR",
        str(Path(tempfile.gettempdir()) / "scan-bind-all-target"),
    )
    env["RUSTFLAGS"] = (env.get("RUSTFLAGS", "") + " " + " ".join(FORCE_WARN)).strip()

    proc = subprocess.run(cmd, capture_output=True, text=True, env=env, cwd=WORKSPACE)

    for line in proc.stdout.splitlines():
        try:
            msg = json.loads(line)
        except json.JSONDecodeError:
            continue
        if msg.get("reason") != "compiler-message":
            continue
        diag = msg.get("message", {})
        code = (diag.get("code") or {}).get("code", "")
        text = diag.get("message", "")
        loc = primary_span(diag)
        # External registry crates (e.g. smoltcp) are not ours to fix and
        # carry benign cfg-dependent unreachable arms — skip them.
        if "/.cargo/registry/" in loc:
            continue

        finding = None
        if code == "unreachable_patterns":
            finding = f"{loc}: [UNREACHABLE] {text}"
        elif code == "unused_variables":
            m = RE_UNUSED.search(text)
            if m and CONST_LIKE.match(m.group(1)) and is_match_arm(loc):
                finding = (
                    f"{loc}: [BIND-ALL?] pattern binds `{m.group(1)}` "
                    f"(const-like name, unused) — likely unresolved constant"
                )
        elif code == "non_snake_case":
            m = RE_SNAKE.search(text)
            if m and CONST_LIKE.match(m.group(1)) and is_match_arm(loc):
                finding = (
                    f"{loc}: [CONST-BINDING] variable `{m.group(1)}` is "
                    f"SCREAMING_CASE — check for unresolved constant in pattern"
                )

        if finding and finding not in seen:
            seen.add(finding)
            findings.append(finding)

    if proc.returncode != 0 and not findings:
        # compile failed before diagnostics of interest — surface loudly.
        # Also emit a finding-style line on STDOUT: the vibeos-cli check gate
        # diffs stdout lines vs baseline (exit code ignored), so a silent
        # stderr-only failure would read as "clean". Fail loud, never degrade.
        tail = "\n".join(proc.stderr.splitlines()[-15:])
        print(f"ERROR: cargo check failed (target {target}):\n{tail}", file=sys.stderr)
        print(
            f"scan_bind_all_matches: [SCAN-ERROR] cargo check failed (target {target})"
            " — scan did not run"
        )
        sys.exit(2)

    if not quiet:
        for f in findings:
            print(f)
    return findings


def main():
    ap = argparse.ArgumentParser()
    ap.add_argument(
        "-p",
        "--package",
        action="append",
        default=[],
        help="limit to package(s); default whole workspace",
    )
    ap.add_argument("--target", default="x86_64-pc-windows-gnu")
    ap.add_argument("-q", "--quiet", action="store_true")
    args = ap.parse_args()

    findings = scan(args.target, args.package, args.quiet)
    print(f"\n{len(findings)} finding(s) [{args.target}]", file=sys.stderr)
    sys.exit(1 if findings else 0)


if __name__ == "__main__":
    main()
