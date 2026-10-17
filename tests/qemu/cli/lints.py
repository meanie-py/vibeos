"""Source + binary lint runners with baseline-diff (plan §20.3).

Gate = zero NEW findings vs a checked-in baseline (tools/lint-baseline/<tool>.txt),
never absolute zero — the tree carries known pre-existing findings. Advisory
(warn) in `build`'s auto-check; blocking-on-NEW only under `check --strict`.
"""

from __future__ import annotations

import os
import re
import shutil
import subprocess
import sys
from collections import Counter
from pathlib import Path

from . import rustlints
from .artifacts import Run, Stage

BASELINE_DIR = "tools/lint-baseline"


def _shellcheck_bin() -> str | None:
    b = shutil.which("shellcheck")
    if b:
        return b
    # venv bin dir (shellcheck-py installs here; not necessarily on PATH when
    # invoked via the wrapper's explicit interpreter)
    cand = Path(sys.executable).parent / "shellcheck"
    if cand.exists():
        return str(cand)
    try:
        import shellcheck_py  # type: ignore

        if shellcheck_py.__file__:
            cand = Path(shellcheck_py.__file__).parent / "resources" / "shellcheck"
            if cand.exists():
                return str(cand)
    except Exception:
        pass
    return None


def _baseline(repo: Path, tool: str) -> set[str]:
    p = repo / BASELINE_DIR / f"{tool}.txt"
    try:
        return {ln.rstrip() for ln in p.read_text().splitlines() if ln.strip()}
    except OSError:
        return set()


# Count-bearing SUMMARY lines are not findings: they drift whenever a count
# changes (a crate add/remove moves 'TOTAL: … across N files'; a bindgen regen
# moves 'Bindgen catalog: N functions'), which made every cross-lane merge
# spuriously LINT_FAIL with "1 new" — 2026-07-06 hit the same TOTAL line twice
# in one night. Findings are the per-site lines; tallies are presentation.
# (Stale tally lines already in baselines are harmless — they simply never
# match anything — and the next --update-baseline drops them.)
_SUMMARY_LINE = re.compile(
    r"(^\s*TOTAL:"  # 'TOTAL: 160 issues across 2348 files'
    r"|^\s*SUMMARY:"  # verify_match_consts tally
    r"|^\s*[A-Z_]+: \d+$"  # per-category counters ('C_MACRO: 94')
    r"|catalog: \d+"  # 'Spec/Bindgen catalog: N …'
    r"|^\s*-> .*: *\d+$"  # verify_tables tallies
    r"|^\s*matched to a C array\s*: *\d+$"
    r"|^\s*no C counterpart\s*: *\d+$"  # verify_tables: moves whenever a table is added
    r"|^\s*Rust integer tables found\s*: *\d+$"
    r"|^\s*Found \d+ unprotected"  # lint_kernel header tally
    r"|^\s*Total: \d+ violations"  # lint_kernel footer tally (mixed-case)
    # verify_signatures' two header tallies and its whole '── coverage ──' block.
    # These move by ONE whenever a commit adds a single #[no_mangle] fn, which is
    # the most ordinary thing a translation lane does: adding CmpFreeKcbCachedSecurity
    # bumped 'exported Rust fns', 'exported fns checked' and 'no C proto (internal)'
    # by one each and bounced the batch as LINT_FAIL "3 new" (2026-07-31). The
    # per-site [HIGH]/[MEDIUM] blocks below them are the real findings and are
    # untouched by this. Enumerated rather than a generic '<label>: <number>' rule
    # because _SUMMARY_LINE applies to EVERY tool, and a rule broad enough to catch
    # any tally is broad enough to silently swallow some other scanner's finding.
    r"|^\s*C prototypes: \d+"
    r"|^\s*exported Rust fns: \d+"
    r"|^\s*exported fns checked\s*: *\d+$"
    r"|^\s*matched to a C proto\s*: *\d+$"
    r"|^\s*no C proto \(internal\)\s*: *\d+$"
    r"|^\s*signature OK\s*: *\d+$"
    r"|^\s*mismatches\s*: *\d+)"
)


def _finding_lines(out: str) -> set[str]:
    return {ln.rstrip() for ln in out.splitlines() if ln.strip() and not _SUMMARY_LINE.search(ln)}


def _scanner_error(cp: subprocess.CompletedProcess, findings_from: str) -> bool:
    """Crashes outrank partial/baselined output: the scan never completed."""
    if cp.returncode < 0 or re.search(
        r"^(?:Traceback \(most recent call last\):|FATAL:)", cp.stderr or "", re.M
    ):
        return True
    text = cp.stdout or ""
    if findings_from != "stdout":
        text += "\n" + (cp.stderr or "")
    return cp.returncode != 0 and not _finding_lines(text)


# Finding identity WITHOUT the line/column, so LINE DRIFT is not a new finding.
# Same disease as the count-bearing summary lines above, one level deeper: the baseline
# stored 'qemu/lib/common.sh:90:17: … SLOT_INDEX …', a 3-line COMMENT was inserted earlier
# in the file, and the identical finding at :93:17 read as "1 new" → gate RED (2026-07-25).
# There is nothing an author can do about that except re-bless the baseline, which re-blesses
# whatever ELSE drifted in the same pass — so the mechanism actively teaches people to
# launder findings. Compare on (file, message) instead, as a MULTISET so a genuinely
# additional occurrence still counts: 2 hits where the baseline recorded 1 is NEW, the same
# hit at a different line is not. Lines with no `file:line:` prefix are compared verbatim,
# exactly as before.
_LOCATION = re.compile(r"^(?P<file>[^\s:]+):\d+(?::\d+)?:")
# lint_kernel does not use a `file:line:` prefix — it prints a file HEADER and then indented
# per-site lines that carry the line number INSIDE the message: `    L 309 RtlpCheckDeviceName():
# *NameInvalid = ...`. So the prefix-stripper above cannot see it, and the drift immunity it
# provides silently did not apply to this one scanner. Adding 4-line banners to 12 files
# reported 35 "new" findings that were the same 35 sites (gate WARN, 2026-07-25). The function
# name stays in the key, and _new_findings is a multiset, so two real hits still need two
# baseline entries.
# The number is RIGHT-ALIGNED (`L 309` at 3 digits, `L1062` at 4 — no space), so the
# separator between `L` and the digits is optional. Requiring one matched only the
# narrow findings and left the wide ones drifting.
_EMBEDDED_LINE = re.compile(r"^(?P<indent>\s*)L\s*\d+\s")
# Several scanners print a CLASS TAG before the location — `[C_MACRO] path:38: msg`
# (verify_ffi, 159/160 baseline lines) and `[TYPED_PTR_HEX_ARITH] path:223: msg`
# (scan_magic_offsets, 46/46). _LOCATION is start-anchored, so it never saw the path and
# the drift immunity silently did not apply: a ONE-LINE comment added to fastfat's extern
# block moved an unrelated finding 177 -> 178 and the merge queue rejected the batch as
# LINT_FAIL (2026-07-25). The tag is semantic — a [C_MACRO] and a [FLEX_ARRAY] at the same
# site are different findings — so it stays in the key; only line:col is dropped.
_TAGGED_LOCATION = re.compile(r"^(?P<tags>\s*(?:\[[^\]]+\]\s*)+)(?P<file>[^\s:]+):\d+(?::\d+)?:")
# verify_signatures puts the location LAST and in parens, after the symbol it is talking
# about: `[MEDIUM] NtSetLdtEntries  (sdk/lib/rust/.../stubs.rs:210)`.
_TRAILING_LOCATION = re.compile(r"^(?P<head>.*)\((?P<file>[^\s:()]+):\d+(?::\d+)?\)\s*$")


# And locations INSIDE the message: scan_dropped_asserts lists the Rust twins as
# `… Rust has 0 (sdk/lib/rust/…/section.rs:252, …/shim_exports.rs:1454)` after its own
# `file:line:` prefix. Only the prefix was stripped, so a 6-line fix in section.rs read as
# "2 new" for two findings nobody touched (2026-09-26). A path with a real source extension
# followed by `:digits` is a location wherever it sits.
_INNER_LOCATION = re.compile(
    r"(?P<f>[\w./+-]+\.(?:rs|c|h|cpp|hpp|py|md|txt|cmake|spec|toml|sh|S|s|asm|inc)):\d+(?::\d+)?\b"
)


def _finding_key(line: str) -> str:
    return _INNER_LOCATION.sub(r"\g<f>", _finding_key_outer(line))


def _finding_key_outer(line: str) -> str:
    m = _LOCATION.match(line)
    if m:
        return f"{m.group('file')}: {line[m.end() :].strip()}"
    m = _TAGGED_LOCATION.match(line)
    if m:
        return f"{m.group('tags').strip()} {m.group('file')}: {line[m.end() :].strip()}"
    m = _TRAILING_LOCATION.match(line)
    if m:
        return f"{m.group('head').strip()} ({m.group('file')})"
    m = _EMBEDDED_LINE.match(line)
    if m:
        return f"{m.group('indent')}L {line[m.end() :].strip()}"
    return line


def _new_findings(cur: set[str], base: set[str]) -> list[str]:
    """Current finding lines whose (file, message) is not covered by the baseline, keeping
    the real line numbers for the report. Surplus occurrences of an already-baselined key
    are reported too (that is what makes this a multiset and not just a set of keys)."""
    budget = Counter(_finding_key(ln) for ln in base)
    out: list[str] = []
    for ln in sorted(cur):
        k = _finding_key(ln)
        if budget.get(k, 0) > 0:
            budget[k] -= 1
        else:
            out.append(ln)
    return out


def _rel(text: str, repo: Path) -> str:
    """Strip the worktree's absolute repo prefix from tool output so findings
    are repo-relative — otherwise a baseline captured in one worktree never
    matches another (the abs path differs: vibeos vs vibeos-smoke-harness),
    making every finding spuriously 'new'. Makes baselines cross-worktree."""
    return text.replace(str(repo) + "/", "").replace(str(repo), "")


def _lint_stage(
    run: Run,
    repo: Path,
    name: str,
    cmd: list[str],
    *,
    cwd: Path | None = None,
    strict: bool,
    findings_from: str = "stdout",
    by_exit_code: bool = False,
    cp: subprocess.CompletedProcess | None = None,
    new_fail: bool | None = None,
) -> Stage:
    """Run one lint; classify NEW-vs-baseline. Advisory unless strict.

    new_fail: whether a NEW finding fails the stage, separately from `strict`, which
    then governs only COVERAGE (a missing tool, a scanner that exits without findings).
    None means "same as strict". Plain `check` passes True: a new finding is exactly
    what `land` and the gate refuse, so a local check that only warned about one was
    green-lighting the submission it would bounce.

    by_exit_code: judge purely on the tool's --check exit status instead of
    line-baseline-diff. Required for BINARY lints (scan_ud2, sacred_primitives):
    their output is ARCH-SPECIFIC (i386 ud2 lines ≠ arm64 branch-to-self spins ≠
    amd64), so a single arch-agnostic baseline file can never match across arches
    — every line reads as "new". These tools already encode the correct per-arch
    pass/fail in their exit code (their own EXPECTED-site classification), so the
    exit code is the right signal."""
    fail_new = strict if new_fail is None else new_fail
    if cmd[0] == "NOCHECK":
        # Genuinely nothing to lint (no matching sources). Not a coverage loss.
        return run.add(Stage(name, status="skip", detail="no sources to check"))
    if cmd[0] == "MISSING":
        # A missing TOOL is MISSING COVERAGE, not a pass. Silently skipping it
        # means that the day shellcheck/ruff/etc falls out of the image, the gate
        # keeps printing green while the check it advertises simply isn't running
        # — the check most likely to vanish unnoticed is the one nobody sees fail.
        # strict (the gate) must have the coverage it claims → fail; a casual
        # local `check` warns, so it stays usable without every tool installed.
        st = "fail" if strict else "warn"
        return run.add(
            Stage(name, status=st, detail="tool unavailable — LINT NOT RUN (no coverage)")
        )
    # `cp` pre-supplied means the subprocess already ran (see `run_source_lints`,
    # which executes the scanners concurrently and then classifies them in order).
    if cp is None:
        cp = run.run(cmd, cwd=cwd or repo, stage=f"lint:{name}")
    if _scanner_error(cp, "both" if by_exit_code else findings_from):
        run.classification = "SCANNER_ERROR"
        return run.add(
            Stage(
                name,
                exit=cp.returncode,
                status="fail",
                detail="SCANNER_ERROR: scanner did not complete; coverage unavailable",
                salient=_rel(
                    (cp.stderr or cp.stdout or f"exit {cp.returncode}"), repo
                ).splitlines()[-4:],
            )
        )
    if by_exit_code:
        text = _rel((cp.stdout or "") + "\n" + (cp.stderr or ""), repo)
        if cp.returncode == 0:
            return run.add(Stage(name, exit=0, status="ok"))
        st = "fail" if fail_new else "warn"
        salient = [ln for ln in text.splitlines() if "*** BUG" in ln or "FAIL" in ln][:3]
        return run.add(
            Stage(
                name,
                exit=cp.returncode,
                status=st,
                detail=f"exit {cp.returncode}",
                salient=salient or text.splitlines()[:3],
            )
        )
    text = cp.stdout if findings_from == "stdout" else (cp.stdout + "\n" + cp.stderr)
    text = _rel(text, repo)
    base = _baseline(repo, name)
    cur = _finding_lines(text)
    new = _new_findings(cur, base)
    # The gate is NEW findings vs baseline — NOT the exit code. Tools like
    # shellcheck/verify_* exit nonzero whenever ANY finding exists, but those
    # are baselined/accepted; only new ones matter.
    if new:
        st = "fail" if fail_new else "warn"
        return run.add(
            Stage(
                name,
                exit=cp.returncode,
                status=st,
                detail=f"{len(new)} new",
                salient=new[:3],
            )
        )
    if cur:
        # findings exist but all are in the baseline → accepted
        return run.add(Stage(name, exit=cp.returncode, status="ok", detail="baselined"))
    if cp.returncode != 0:
        # no parseable findings yet nonzero exit → the tool itself failed to run
        return run.add(
            Stage(
                name,
                exit=cp.returncode,
                status=("fail" if strict else "warn"),
                detail=f"exit {cp.returncode}",
                salient=[ln for ln in text.splitlines()[:3]],
            )
        )
    return run.add(Stage(name, exit=0, status="ok"))


def source_lint_specs(repo: Path) -> list[tuple]:
    """(name, cmd, findings_from) for every source lint. Single source of truth
    shared by run_source_lints and write_baselines."""
    py = sys.executable
    sc = _shellcheck_bin() or "MISSING"
    sh_files = sorted(str(p) for p in (repo / "qemu").glob("*.sh"))
    sh_files += sorted(str(p) for p in (repo / "qemu" / "lib").glob("*.sh"))
    rc = repo / "tools" / "rustconv" / "rustconv"
    lk = repo / "tools" / "lint_kernel.py"
    specs = [
        # Two very different reasons shellcheck might not run, which the single
        # ["MISSING"] sentinel used to conflate — and they deserve OPPOSITE
        # verdicts: "no shell sources to lint" is genuinely nothing to do, while
        # "shellcheck is not installed" is COVERAGE SILENTLY LOST and must be
        # loud (see _lint_stage). Keep them distinct so the honest failure can't
        # be mistaken for the benign one.
        (
            "shellcheck",
            [sc, "-S", "warning", "-f", "gcc", *sh_files]
            if sc != "MISSING" and sh_files
            else (["MISSING"] if sc == "MISSING" else ["NOCHECK"]),
            "stdout",
        ),
        ("py-syntax", [py, "-m", "compileall", "-q", "tests/qemu", "tools"], "both"),
    ]
    # verify_ffi is a strict SUPERSET of verify_abi: its ABI-mismatch check
    # (extern "C" vs "system" on i386) is byte-for-byte the same, plus it adds
    # param-count, macro-as-extern, and flex-array-UB checks (verified 2026-07-05:
    # verify_abi's 62 findings ⊆ verify_ffi's 135, zero coverage gap). The
    # redundant verify_abi lint run was folded out here (T1) + its baseline
    # dropped. verify_abi.py STAYS — abi_map.py imports its parse_spec /
    # parse_bindgen_conventions, so it's a library, not just a lint.
    ffi = rc / "verify_ffi.py"
    specs.append(("verify_ffi", [py, str(ffi)] if ffi.is_file() else ["MISSING"], "both"))
    # verify_constants reads stdout only: it now scans the whole rust workspace
    # (driver crates included), so its volatile catalog/file-count summary goes to
    # stderr to keep the baseline stable — only `MISMATCH:` finding lines on stdout.
    vc = rc / "verify_constants.py"
    specs.append(("verify_constants", [py, str(vc)] if vc.is_file() else ["MISSING"], "stdout"))
    # Enum-member drift — the AdjustUnwait class: a mistranslated C enum value
    # silently deforms dispatch/scheduling and compiles clean on every arch (19
    # such drifts found in the 2026-07-04 dispatcher audit; none were gated).
    # `--enums` is FP-prone on generic names, which is exactly what the baseline
    # mechanism is for (same pattern as verify_signatures): current accepted
    # findings live in the baseline, the gate is NEW-only.
    specs.append(
        (
            "verify_constants_enums",
            [py, str(vc), "--enums"] if vc.is_file() else ["MISSING"],
            "stdout",
        )
    )
    specs.append(("lint_kernel", [py, str(lk)] if lk.is_file() else ["MISSING"], "stdout"))
    # Stable-toolchain RATCHET: no `#![feature(...)]` in the workspace, no
    # RUSTC_BOOTSTRAP in the build graph. The i386 `-Z build-std` cell (gcc.cmake,
    # RUSTUP_TOOLCHAIN from .rust-nightly) is the ONE sanctioned nightly user.
    # Baseline is EMPTY, so any finding is NEW and fails. Exists because the lane
    # drove this to zero and a concurrent lane re-introduced it the same day.
    ns = repo / "tools" / "scan_nightly_surface.py"
    specs.append(("scan_nightly_surface", [py, str(ns)] if ns.is_file() else ["MISSING"], "stdout"))
    # MSVC is not a supported toolchain (roadmap section 11): every CMake `if()` that tests
    # for it was resolved by tools/strip_msvc_cmake.py on 2026-10-03. Baseline is EMPTY, so
    # an upstream merge that brings a new one fails here with the command that resolves it.
    smc = repo / "tools" / "strip_msvc_cmake.py"
    specs.append(
        ("strip_msvc_cmake", [py, str(smc), "--check"] if smc.is_file() else ["MISSING"], "stdout")
    )
    # shell32's zero-C++ lane has an exhaustive live-CMake ledger. A newly added
    # source, a claimed removal that remains compiled, or an expected source that
    # silently fell out of CMake is a hard mismatch rather than baseline debt.
    sti = repo / "tools" / "rustconv" / "rustconv" / "shell32_tail_inventory.py"
    specs.append(
        (
            "shell32_tail_inventory",
            [py, str(sti), "--check"] if sti.is_file() else ["MISSING"],
            "stdout",
        )
    )
    # C↔Rust differential oracles in tools/ (offsets are a compile-time gate, not
    # here). tables + match_consts are clean today (empty baseline = strict);
    # signatures carries known dual-cfg-shim FPs, so the baseline gates NEW only.
    for tool, rel in (
        ("verify_tables", "verify_tables.py"),
        ("verify_match_consts", "verify_match_consts.py"),
        ("verify_signatures", "verify_signatures.py"),
    ):
        t = repo / "tools" / rel
        specs.append((tool, [py, str(t)] if t.is_file() else ["MISSING"], "stdout"))
    # Calling-convention alias table: every macro in the SDK headers that resolves to a
    # convention (WSAAPI -> FAR PASCAL -> WINAPI -> __stdcall) must be known to
    # rustconv's abi_enforce. It reads RAW C, so an UNKNOWN alias does not fail — it
    # silently derives **cdecl**, which on i386 is a stack imbalance on every call the OS
    # makes through the fn pointer, and it compiles/links/boots. 53 aliases today; the
    # tool knew 9 until ws2_32 forced the issue (it would have mis-derived all 187
    # exports). Baseline EMPTY = strict: a header that grows a new alias fails here.
    vcm = repo / "tools" / "verify_conv_macros.py"
    specs.append(("verify_conv_macros", [py, str(vcm)] if vcm.is_file() else ["MISSING"], "stdout"))
    # ucrtbase.dll is vendored Microsoft UCRT plus OUR msvcrt_shared, so the Rust archive it
    # links must cover msvcrt_shared's surface and nothing else. The failure this catches has
    # no diagnostic of its own: ucrtbase.spec and msvcrt's cohab archive collide on 891 names
    # of which only 35 are msvcrt_shared's, and the module links with
    # --allow-multiple-definition, so linking the wrong archive silently replaces 856 genuine
    # UCRT bodies with msvcrt-contract ones and still builds, boots and looks fine. Baseline
    # EMPTY = strict.
    vuc = repo / "tools" / "verify_ucrtbase_cohab.py"
    specs.append(
        ("verify_ucrtbase_cohab", [py, str(vuc)] if vuc.is_file() else ["MISSING"], "stdout")
    )
    # vendor.yaml — which C is NOT ours to translate. Every progress number in this project
    # is a fraction whose denominator is "C that is ours", and before the registry that set
    # was prose in rust-port-status.md, a one-tree manifest under external/, and a hardcoded
    # tuple in verify_conv_macros — three lists, three contents. This checks the registry
    # against the tree: registered paths exist, every version PROBE still resolves (a probe
    # is stored instead of a copied version so it cannot rot silently), the VENDORED stamps
    # are fresh, and — the ratchet — no unregistered directory has appeared inside a vendor
    # container. That last one is why this is a gate and not a doc: a library dropped into
    # dll/3rdparty/ otherwise joins the translation denominator with nobody noticing.
    vnd = repo / "tools" / "vendored.py"
    specs.append(("vendored_registry", [py, str(vnd)] if vnd.is_file() else ["MISSING"], "stdout"))
    # msvcrt's two 257-entry ctype tables are GENERATED from dll/win32/msvcrt/ctype.c, not
    # copied — 514 numbers with no compiler check is the one shape where a single wrong
    # entry misclassifies exactly one character and nothing anywhere says so. This is the
    # drift half: if ctype.c or the msvcrt-sys island moves, the committed Rust block is
    # stale and that is a NEW finding here. The stronger half — every value proven equal to
    # the bytes in ctype.c.obj's `.data` — needs an i386 build, so it runs whenever this
    # worktree has one and announces on stderr when it cannot. Baseline EMPTY = strict.
    # Every whole-file msvcrt translation, string literals compared with its C as a
    # MULTISET. The only oracle that sees a literal which is real, correctly spelled and in
    # the WRONG SLOT -- `undname.c`'s `case 'C': "pascal"` shipped as `"__stdcall"`, a
    # genuine member of that same switch, and `time.c`'s `L"HH':'mm':'ss"` shipped with the
    # quotes transposed. Neither fails to compile, fails a sealed-constant check, or fails a
    # boot. Baselined because a few differences are representation, not content (asm text
    # spelled `global_asm!`, a `static char[64]` built rather than written): those are
    # recorded and visible, and anything NEW is a finding.
    vtl = repo / "tools" / "verify_translation_literals.py"
    specs.append(
        ("verify_translation_literals", [py, str(vtl)] if vtl.is_file() else ["MISSING"], "stdout")
    )
    gct = repo / "tools" / "gen_ctype_tables.py"
    ctype_rs = "sdk/lib/rust/msvcrt-crt-rust/src/msvcrt/ctype.rs"
    specs.append(
        (
            "gen_ctype_tables",
            [py, str(gct), "--check", str(repo / ctype_rs)] if gct.is_file() else ["MISSING"],
            "stdout",
        )
    )
    # A cohab flip that DROPS a function removes the DLL's export silently: rustc is happy
    # (nothing calls it), the link is happy (a .spec export with no impl is not an error —
    # the DLL just ships a smaller export table), and the boot is happy unless the boot path
    # happens to call that exact API. ws2_32 lost `bind`, `connect`, `sendto`, `WSAConnect`,
    # `WSAJoinLeaf` and `WSASendTo` this way — tree-sitter cannot parse `IN CONST struct
    # sockaddr *`, so the scanner never put them in the plan and the crate compiled clean on
    # all three arches. A socket client cannot bind or connect. Baseline EMPTY = strict.
    vle = repo / "tools" / "verify_lost_exports.py"
    specs.append(
        ("verify_lost_exports", [py, str(vle)] if vle.is_file() else ["MISSING"], "stdout")
    )
    # Each translated fn's signature vs the C prototype it came from. The model reaches for
    # the POINTER flavour of a scalar with real regularity (INT -> LPINT, SIZE_T -> PSIZE_T,
    # HANDLE -> LPHANDLE, DWORD -> DWORD_PTR). Nothing else can see it: rustc never sees the
    # C prototype, verify_ffi only checks conventions, and on 64-bit the wrong typedef is the
    # same WIDTH — so it links, runs, and returns the right answer, then breaks on i386.
    vcs = repo / "tools" / "verify_c_signatures.py"
    specs.append(
        ("verify_c_signatures", [py, str(vcs), "--all"] if vcs.is_file() else ["MISSING"], "stdout")
    )
    # Inlined flag/constant-literal lint (AST/tree-sitter). verify_constants only
    # checks named `const` defs; this catches bare `0x2 /* TOP_LEVEL */` literals.
    # Carries residual FP classes (mask unions, informational args) → baseline gates NEW.
    sif = repo / "tools" / "scan_inline_flag_literals.py"
    specs.append(
        (
            "scan_inline_flag_literals",
            [py, str(sif), "--lint"] if sif.is_file() else ["MISSING"],
            "stdout",
        )
    )
    # Silent-failure shapes (AST/tree-sitter): bare `loop {}` spins, unwrap/
    # expect in no-std kernel code, swallowed Err arms. The arm64 explorer
    # regression hid for weeks behind exactly this (win32k's bare-loop panic
    # handler = silent spin), and the POWNED paint-mask panic WAS an unwrap.
    # Stock is large (~2.1K, mostly unwraps) → baseline gates NEW only; the
    # baseline doubles as the fix-campaign work queue.
    # A forward declaration whose PARAMETER COUNT differs from the definition it links to.
    # The 2026-07-17 clashing_extern triage sorted ~1458 findings and called this bucket "the
    # highest-value needles" — every other bucket is a spelling or width difference that some
    # ABI makes harmless, but a different arg count is the caller and callee disagreeing about
    # the shape of the call. It found classpnp's ScsiGetNextSenseDescriptorByType, whose 4th
    # parameter was `found: *mut u8` while callers pass `typeList.len() as ULONG` there — a
    # write to address 0x2, gated off only by IsDescriptorSenseDataFormat(). Baseline holds 6
    # verified-benign entries (address-only forward decls + the SSDT's uniform 6-arg shape).
    sea = repo / "tools" / "scan_extern_arity.py"
    specs.append(("scan_extern_arity", [py, str(sea)] if sea.is_file() else ["MISSING"], "stdout"))
    # Translated .rs that no `mod` chain reaches. cargo never opens such a file, so
    # `cargo check` is green whatever is in it and every source gate here skips it —
    # while grep still finds it, so the next reader takes a dead copy for the truth
    # (classpnp's class_gap3.rs was "fixed" that way while the link kept failing).
    # Parking a translation is legitimate; an UNDECLARED park is not, so the gate is
    # satisfied by an `//! ORPHAN-BY-DESIGN: <reason>` banner in the file itself.
    # Baseline EMPTY = strict: all 12 of today's orphans carry one.
    som = repo / "tools" / "scan_orphan_modules.py"
    specs.append(
        ("scan_orphan_modules", [py, str(som)] if som.is_file() else ["MISSING"], "stdout")
    )
    # A cohab flip is held in place by a single `#` in a CMake source list
    # (`#${REACTOS_SOURCE_DIR}/ntoskrnl/lpc/reply.c`). Drop it and the dead C goes live
    # alongside its Rust twin — a partial flip, which is what froze the guest on
    # 2026-09-19 with SMOKE GREEN ON ALL THREE ARCHES (the hang is after `shell ready`,
    # so no boot test can see it). The upstream sync makes that reachable by accident:
    # upstream has touched 400 CMake files, 197 of ours carry a comment-out, and the
    # 80-file intersection can be disturbed by a CLEAN auto-merge with no conflict
    # marker to review. This is a DIFFER, not a classifier — the pattern deliberately
    # also matches prose mentioning a filename, because stable prose is stable in the
    # snapshot while a false negative costs a frozen guest. Snapshot lives in
    # tools/cohab-census.txt; re-run `--write` in the SAME commit as a deliberate flip.
    # Baseline EMPTY = strict.
    ccen = repo / "tools" / "cohab_census.py"
    specs.append(("cohab_census", [py, str(ccen)] if ccen.is_file() else ["MISSING"], "stdout"))
    # Cohab stubs that bugcheck on entry AND are declared by a Rust module — i.e.
    # unimplemented and reachable, so they crash whenever their path first runs. The
    # intersection is computable from the tree, and paying a build+boot per entry to
    # rediscover it is waste: fastfat cost three such cycles in one night. The
    # MACRO_AS_EXTERN subclass is strictly a translation defect — an `extern fn` naming
    # a `#define` can never resolve. Stock (18) is the work queue; baseline gates NEW.
    sbs = repo / "tools" / "scan_bugcheck_stubs.py"
    specs.append(
        ("scan_bugcheck_stubs", [py, str(sbs)] if sbs.is_file() else ["MISSING"], "stdout")
    )
    ssf = repo / "tools" / "scan_silent_failures.py"
    specs.append(
        (
            "scan_silent_failures",
            [py, str(ssf), "--lint"] if ssf.is_file() else ["MISSING"],
            "stdout",
        )
    )
    # Hand-written numeric struct offsets (the MMPFN pfn_offsets shift + the
    # KTHREAD.Teb 0x1e8 class, both real kernel bugs 2026-07-05). verify_offsets
    # proves bindgen DECLS match C; this catches USAGE-site numerology — byte
    # arithmetic into structs that should be core::mem::offset_of!(bindgen, f).
    # Stock (~47) is the conversion work queue; baseline gates NEW only.
    smo = repo / "tools" / "scan_magic_offsets.py"
    specs.append(
        (
            "scan_magic_offsets",
            [py, str(smo), "--lint"] if smo.is_file() else ["MISSING"],
            "stdout",
        )
    )
    # Numeric bugcheck stop codes (§9.113). The LLM invented the number for 59 of
    # ~75 translated `KeBugCheck(SYMBOL)` calls — plausible, internally consistent,
    # and naming the wrong subsystem (INVALID_AFFINITY_SET -> an IRP bugcheck,
    # IRQL_GT_ZERO_AT_SYSTEM_SERVICE -> MEMORY_MANAGEMENT). Nothing else can see it:
    # rustc accepts any ULONG and the code is only read once the kernel is dying.
    # Fix = ntkrnl_sys::bugcodes::NAME (tools/gen_bugcodes.py). Baseline is EMPTY.
    # Field-manual section numbering. The `### 9.NNN` counter is RETIRED (2026-07-25):
    # every lane appended at the tail, so (a) every merge conflicted on a purely
    # positional collision, and (b) same-day lanes picked the same number independently
    # — five collisions in one week, plus five pairs that merged CLEANLY and left a
    # number meaning two different bugs (9.27/9.36/9.41/9.43/9.44), which nothing else
    # can detect. Entries now live in the topical section their symptom belongs to and
    # are titled by the symptom; surviving `§9.N` tokens are frozen inline tags so old
    # cross-references still grep. The lint fails on a NEWLY MINTED number (field manual
    # only — other docs may number their sections) and on any duplicate (any doc).
    # Baseline is EMPTY: the 5 historical dupes were merged away in the refresh.
    sds = repo / "tools" / "scan_doc_sections.py"
    specs.append(
        (
            "scan_doc_sections",
            [py, str(sds), "--lint"] if sds.is_file() else ["MISSING"],
            "stdout",
        )
    )
    # Registry hives the livecd stages into system32\config but the arm64 ramdisk does
    # not. The two media are assembled by unrelated machinery (add_cd_file vs a hand-
    # written mkramdisk.sh), so they drift, and the drift is SILENT BY CONSTRUCTION: a
    # hive that fails to open is not an error, it is fabricated — CmpLinkHiveToMaster
    # conjures an empty volatile one whose root takes CmpHiveRootSecurityDescriptor().
    # arm64 booted with an empty SAM for months and read as *more* correct than x86,
    # because the fabricated descriptor is exactly what the CmSecurity kmtest expects,
    # which sent the investigation at the wrong arch. Static, free, and it names the
    # missing hive. Baseline EMPTY.
    sbm = repo / "tools" / "scan_boot_media_drift.py"
    specs.append(
        (
            "scan_boot_media_drift",
            [py, str(sbm), "--lint"] if sbm.is_file() else ["MISSING"],
            "stdout",
        )
    )
    # A bare store to the PL011 data register (Arm64UartKernelBase). The FIFO holds
    # 16-32 bytes and drains at 115200 baud, so a CPU writing faster than that loses
    # every byte after the FIFO fills - and QEMU's model drains instantly, so it only
    # shows on hardware. On a physical Pi 4 it cut each Phase-1 trace to its first
    # `[` and made a hang look like it stopped earlier than it did. 87 private copies
    # in six crates when found. Baseline is EMPTY.
    sru = repo / "tools" / "scan_raw_uart_writes.py"
    specs.append(
        (
            "scan_raw_uart_writes",
            [py, str(sru), "--lint"] if sru.is_file() else ["MISSING"],
            "stdout",
        )
    )
    sbc = repo / "tools" / "scan_bugcheck_codes.py"
    specs.append(
        (
            "scan_bugcheck_codes",
            [py, str(sbc), "--lint"] if sbc.is_file() else ["MISSING"],
            "stdout",
        )
    )
    # Sibling of the above, same root cause, different lowering (§9.120). C's
    # PAGED_CODE/ASSERT/NT_ASSERT are DIAGNOSTICS — KdPrint+RtlAssert under DBG,
    # ((void)0) in retail — and never __assume(0). Translating one to
    # core::hint::unreachable_unchecked() makes it UB, licensing LLVM to delete
    # the guarding check and poison what follows: strictly worse than the C in
    # every build config, since retail C does nothing at all. Found live in
    # cdrom (PAGED_CODE) and fastfat (NT_ASSERT on an allocation result), plus a
    # third that had escalated to a BUGCHECK with an invented code in classpnp.
    # Had already recurred once (see the fatinit.rs __int2c comment), which is
    # why a gate and not another one-off fix. Baseline is EMPTY.
    sau = repo / "tools" / "scan_assert_to_ub.py"
    specs.append(
        (
            "scan_assert_to_ub",
            [py, str(sau), "--lint"] if sau.is_file() else ["MISSING"],
            "stdout",
        )
    )
    # C's `!x` is LOGICAL not; Rust's `!x` on an integer is BITWISE. So a translated
    # `!(flags & FLAG)` becomes `!(flags & FLAG) != 0`, which is true for every value a
    # flag test can produce — a CONSTANT condition that types fine, compiles clean and
    # trips no rustc lint. Eight sites on 2026-07-26, and only three were assertions:
    # the rest were live branches, including classpnp's ADJUST_FUA_FLAG collapsing to
    # its first term (forcing FUA onto devices that had reported it unsupported) and an
    # always-true arm that disabled win32k's CJK line-break rule. Nothing but reading
    # the C finds these, which is exactly why it is a gate. Baseline is EMPTY.
    sbn = repo / "tools" / "scan_bitwise_not_bool.py"
    specs.append(
        (
            "scan_bitwise_not_bool",
            [py, str(sbn), "--lint"] if sbn.is_file() else ["MISSING"],
            "stdout",
        )
    )
    # An array FIELD passed by value to a C variadic (`note("%s", (*ip).name)`): C decays
    # it to a pointer, Rust passes the array, and only i386 copies the bytes onto the stack,
    # so `%s` reads the string's first four bytes as a pointer. The i386 DHCP client died on
    # 0x7374656e ("nets") every boot; msgina's "logged on as DOMAIN\\User" line had the same
    # shape (2026-10-04). Fields are resolved through each variable's declared type, because
    # the same field name is an array in one struct and a pointer in the next. Baseline is EMPTY.
    sva = repo / "tools" / "scan_variadic_array_arg.py"
    specs.append(
        (
            "scan_variadic_array_arg",
            [py, str(sva), "--lint"] if sva.is_file() else ["MISSING"],
            "stdout",
        )
    )
    # Four bug CLASSES that rustconv difftest-contract found in shipping Rust on
    # 2026-09-26, each now a $0 check over every translated body. None was visible to
    # smoke; all compile, link and boot. The two C-oracle scanners pair a Rust function
    # with its C twin by NAME (tools/c_twin.py, index cached in tools/.cache/).
    #   ptr_byte_diff: C's `p - q` on typed pointers counts ELEMENTS; the translation
    #     subtracted addresses (version.dll, kernel32 vdm/move, shell32 wildcard_to_file).
    #   misaligned_cast: a `[u8; N]` cast to *mut u32 and dereferenced (cryptlib sha1 +18).
    #   dropped_null_guard: the C null-tests an out-param, the Rust stores through it
    #     unconditionally (rtl compress, ole32, mlang, msvcrt, fastfat).
    #   store_on_failure: the C writes an out-param only after its last failure return,
    #     the Rust writes it and then returns a status that can fail (rtl compress).
    #   bool_width (older, wired 2026-09-26): a u8/BOOLEAN return where the other side
    #     reads a BOOL — 56 definitions across userenv, win32k, shell32, winsrv, ks, msvcrt.
    #   cfg_vs_c: a per-arch `#[cfg]` whose arch set the C twin's `#if`s never draw — e.g.
    #     `_WIN64` translated as `target_arch = "x86_64"` (user32 GetClassLongA/W on arm64).
    #   constant_placement: the C's constants all present but in each other's places
    #     (`a = 10; b = 20;` -> `a = 20; b = 10;`), which the fact seal's SET check passes.
    # All were driven to zero before wiring; baselines are EMPTY.
    for lint_name in (
        "scan_ptr_byte_diff",
        "scan_misaligned_cast",
        "scan_dropped_null_guard",
        "scan_store_on_failure",
        "scan_bool_width",
        "scan_cfg_vs_c",
        "scan_constant_placement",
    ):
        script = repo / "tools" / f"{lint_name}.py"
        specs.append(
            (lint_name, [py, str(script), "--lint"] if script.is_file() else ["MISSING"], "stdout")
        )
    # One constant name, two values, both ours (§9.113b). The ORACLE-FREE half of
    # the invented-constant hunt: verify_constants needs the right C header in its
    # catalog, and the 18 findings of 2026-07-24 hid precisely because theirs
    # weren't in it. This needs no header — if our own tree spells one name two
    # ways in one arch bucket, one of them is wrong regardless of what C says.
    # (It found UNICODE_STRING_MAX_BYTES wrong in two BOUNDS CHECKS, one of them
    # RPC input validation.) Stock is large and mixed with legitimately
    # driver-local scales → baseline gates NEW only, and doubles as the
    # hoist-to-one-definition work queue.
    scc = repo / "tools" / "scan_const_contradictions.py"
    specs.append(
        (
            "scan_const_contradictions",
            [py, str(scc), "--lint"] if scc.is_file() else ["MISSING"],
            "stdout",
        )
    )
    # Same scanner, other oracle: a hand copy whose value matches NO bindgen island's value
    # for that name. A lone invented copy is invisible above (it has nothing of ours to
    # disagree with); fsrtl's swapped GenericEqual/GreaterThan was exactly that, and hung
    # every file open on a FAT volume. Stock is mixed (freeldr's ARC status codes share
    # errno NAMES) → baseline gates NEW only, and is the triage queue.
    specs.append(
        (
            "scan_const_vs_island",
            [py, str(scc), "--vs-island"] if scc.is_file() else ["MISSING"],
            "stdout",
        )
    )
    # Same scanner, the crate's OWN island: a hand constant there shadows the island's glob
    # import with no warning, and the any-island oracle above accepts a value copied from
    # another crate. atapi's MAX_SLOTS came from usbxhci (16 for 32) and pciidex's reset loops
    # from legide (1000x). One right answer per crate, so the baseline only shrinks.
    specs.append(
        (
            "scan_const_vs_own_island",
            [py, str(scc), "--vs-own-island"] if scc.is_file() else ["MISSING"],
            "stdout",
        )
    )
    # Same scanner, narrowest scope: one NAME with two VALUES inside ONE crate. One copy is
    # wrong by construction. userenv's IDS_PROFILEPATH (0 vs 1) sat in the cross-crate
    # baseline above while it stopped GUI setup on every install, so this half is gated
    # against a short reviewed baseline of per-file meanings only.
    specs.append(
        (
            "scan_const_within_crate",
            [py, str(scc), "--within-crate"] if scc.is_file() else ["MISSING"],
            "stdout",
        )
    )
    # The ORACLE-BEARING half, and the half the above is blind to by construction:
    # a constant with exactly ONE definition in the tree cannot contradict itself,
    # so a lone wrong value is invisible there forever. This one needs no inference
    # because the translator labelled the population itself -- "// placeholder",
    # "// guessed value", "// assumed" -- and then resolves each name against a
    # NARROWLY scoped oracle (the C module the crate translates, plus the SDK
    # headers), because a repo-wide catalog collides on short names and its
    # findings are overwhelmingly false (§9.113b). fastfat's
    # IRP_CONTEXT_FLAG_DISABLE_DIRTY was one of these, at 0x100 against
    # fatstruc.h's 0x01, and it cost a BUGCHECK 0x50 on arm64 through an infinite
    # recursion whose guard could never fire. MISMATCH is a defect; UNRESOLVED is
    # a review queue, so the baseline gates NEW only.
    sic = repo / "tools" / "scan_invented_constants.py"
    specs.append(
        (
            "scan_invented_constants",
            [py, str(sic), "--lint"] if sic.is_file() else ["MISSING"],
            "stdout",
        )
    )
    # Its unlabelled sibling: a BARE `0x8003xxxxu32 as HRESULT` has no name to
    # resolve, so only the C function it translates can say it is wrong -- the
    # literal must be a value that function returns. ole32's storage32.rs carried
    # seven that were not (STG_E_TOOMANYOPENFILES for STG_E_READFAULT and
    # STG_E_WRITEFAULT, STG_E_INVALIDHANDLE for STG_E_INVALIDNAME, ...).
    shl = repo / "tools" / "scan_hresult_literals.py"
    specs.append(
        (
            "scan_hresult_literals",
            [py, str(shl), "--lint"] if shl.is_file() else ["MISSING"],
            "stdout",
        )
    )
    # A C do-while's `continue` evaluates the condition; the translated `loop { ..;
    # if !(cond) { break; } }` jumps over it. Four shipped (an endless setup dialog
    # loop, a win32k use-after-free re-peek, a temp-name spin, a scrollbar that kept
    # tracking). The shape also fits a faithful `for (;;)`, so the baseline holds the
    # sites read against their C and found OK; a NEW one means: go read the C.
    # `(base as *mut PBYTE).offset(n)` steps in POINTER units: the C's `(PBYTE)base + n`
    # turned into 8n on 64-bit. oleaut32's RecordCopy read every field past offset 0
    # from the wrong address; win32k's 16bpp transparent blit wrote 8x past its target.
    sbs = repo / "tools" / "scan_byteptr_stride.py"
    specs.append(
        (
            "scan_byteptr_stride",
            [py, str(sbs), "--lint"] if sbs.is_file() else ["MISSING"],
            "stdout",
        )
    )
    # HEAP_ZERO_MEMORY handed to GlobalAlloc: not a GMEM_ flag, so GlobalAlloc refuses it
    # outright. oleaut32's create_stream did this and no picture ever loaded from a file.
    sgf = repo / "tools" / "scan_gmem_flags.py"
    specs.append(
        (
            "scan_gmem_flags",
            [py, str(sgf), "--lint"] if sgf.is_file() else ["MISSING"],
            "stdout",
        )
    )
    # A C assignment translated as a nested `let _x`: the underscore hides the unused
    # binding and the outer variable keeps its old value. freeldr reported 0 BIOS disks.
    sls = repo / "tools" / "scan_let_shadow.py"
    specs.append(
        (
            "scan_let_shadow",
            [py, str(sls), "--lint"] if sls.is_file() else ["MISSING"],
            "stdout",
        )
    )
    # A C bitfield reached by bit index (`_bitfield_1.set(0, 1, v)`) instead of bindgen's named
    # accessor: atapi passed a page code as the width (a panic), the ARM3 session code swapped
    # Initialized and DeletePending at all four sites.
    srb = repo / "tools" / "scan_raw_bitfield.py"
    specs.append(
        (
            "scan_raw_bitfield",
            [py, str(srb), "--lint"] if srb.is_file() else ["MISSING"],
            "stdout",
        )
    )
    # A const named after a winerror.h code whose value is another code's. oleaut32's
    # TYPE_E_CANTLOADLIBRARY held TYPE_E_AMBIGUOUSNAME's value; nothing else judges a
    # const whose comment does not admit a guess.
    swc = repo / "tools" / "scan_winerror_consts.py"
    specs.append(
        (
            "scan_winerror_consts",
            [py, str(swc), "--lint"] if swc.is_file() else ["MISSING"],
            "stdout",
        )
    )
    # C `case A|B:` is one label, the bitwise OR; the same text in a Rust arm is an
    # or-pattern matching EITHER. comctl32's TREEVIEW_SelectItem shipped it.
    scb = repo / "tools" / "scan_case_bitor_pattern.py"
    specs.append(
        (
            "scan_case_bitor_pattern",
            [py, str(scb), "--lint"] if scb.is_file() else ["MISSING"],
            "stdout",
        )
    )
    # A wine/list.h walk lowered to start AT the head it stops at runs zero times: rpcss's
    # running-object table never found a registered object (2026-10-11).
    slw = repo / "tools" / "scan_list_walk_from_head.py"
    specs.append(
        (
            "scan_list_walk_from_head",
            [py, str(slw), "--lint"] if slw.is_file() else ["MISSING"],
            "stdout",
        )
    )
    sdc = repo / "tools" / "scan_dowhile_continue.py"
    specs.append(
        (
            "scan_dowhile_continue",
            [py, str(sdc), "--lint"] if sdc.is_file() else ["MISSING"],
            "stdout",
        )
    )
    # The type-level sibling of the above, and the reason it exists is that a width
    # disagreement does not merely compare wrong, it CORRUPTS MEMORY. fastfat's VBO
    # (task #46) was `ULONG32` in the generated islands, `LARGE_INTEGER` in dirsup.rs
    # and `u64` in dirctrl.rs, so FatLocateDirent stored 8 bytes through an out-param
    # three callers backed with a `u32` stack local — 4 bytes of the caller's frame,
    # on every file open, on the arch that boots FAT. Nothing else was positioned to
    # see it: every single-arch build is green because each file is internally
    # consistent, and verify_c_signatures compared `PVBO` against `*mut VBO` and found
    # the NAMES agreeing. Only a whole-tree read catches it. Resolves aliases to a
    # width and reports only two KNOWN widths in one arch bucket; unresolvable RHS and
    # `#[cfg]`-gated pairs are skipped rather than guessed at (§9.113b's lesson).
    # The oracle-FREE sibling of scan_inline_flag_literals: rustconv keeps the
    # original C as trailing comments, so `X &= ~NAME` in the C with no live
    # `!NAME` is a mask this file said it would apply and then didn't. Needs no
    # header and no C build — a `verify_constants` catalog miss cannot hide a
    # finding, which is how the 18 of 2026-07-24 hid. It found E21 unprompted
    # (NtOpenKey masked 0x0100 where the C masks KEY_WOW64_RES = 0x0300 → every
    # KEY_WOW64_32KEY open in the OS returned ACCESS_DENIED), i386
    # KeFlushCurrentTb having lost its whole CR4.PGE branch, and then four wrong
    # HIVE_* bits in config-rust where CmUnloadKey set 0x40 while CmpParseKey
    # tested 0x4 — a guard against touching an unloading hive that could not fire.
    # Baseline holds the DROPPED/OPENCODED residue (a dropped mask is often a
    # deliberate reimplementation); MISMATCH is currently ZERO tree-wide and a new
    # one is a value bug by construction. `--self-test` proves it still detects
    # all four historical bugs — run it if you touch the patterns.
    smd = repo / "tools" / "scan_mask_drift.py"
    specs.append(
        (
            "scan_mask_drift",
            [py, str(smd), "--lint"] if smd.is_file() else ["MISSING"],
            "stdout",
        )
    )
    # Same oracle, opposite question: not "did the Rust contradict its C" but
    # "did it stay silent where its C spoke". Upstream prints on nearly every
    # kernel failure path and ~90% of those were dropped in translation, so a
    # failure reports NOTHING on serial — `CmpLinkKeyToHive` losing all three of
    # its DPRINT1s cost the \\REGISTRY casing hunt two build+boot cycles. Only
    # the macros upstream ACTUALLY emits on a default build count (DPRINT1,
    # DbgPrint*, wine ERR/FIXME); DPRINT/TRACE/WARN are excluded on purpose. The
    # baseline holds the backlog and is burned down subsystem by subsystem; a NEW
    # row means a translation landed a failure path the OS cannot report. NOOP is
    # the sharp one — a debug macro lowered to an out-of-line empty `fn`, which
    # AGENTS.md forbids outright because it is also an ABI-mismatch uncloaker.
    sdp = repo / "tools" / "scan_dropped_prints.py"
    specs.append(
        (
            "scan_dropped_prints",
            [py, str(sdp), "--lint"] if sdp.is_file() else ["MISSING"],
            "stdout",
        )
    )
    # The same oracle as scan_dropped_prints, opposite stakes: not "did it stay silent
    # where its C spoke" but "did it dereference a user pointer where its C validated
    # one". A dropped print costs a debugging cycle; a dropped probe is a security bug,
    # and under DUMMY_PSEH there is no SEH to contain it. NtSetValueKey (#79) shipped a
    # bare `Status = STATUS_SUCCESS;` where ProbeForRead belongs and copied straight off
    # the caller's raw pointer -- so a Data pointing into KERNEL SPACE was copied into a
    # registry value the same unprivileged caller reads back with NtQueryValueKey.
    # Nothing else is positioned to see it: the raw-pointer deref is ordinary correct
    # code in any other context, and only the commented-out C saying "I probed this
    # first" makes it a bug, so rustc, the build and the boot are all silent. Every
    # remaining row is an Nt* syscall entry point. ALL is the high-confidence kind;
    # PARTIAL is advisory (the C often probes a struct field-by-field where the Rust
    # probes the buffer once). A false positive is settled by FOLLOWING THE POINTER to
    # the caller -- a callee whose buffer was already captured into pool is genuinely
    # safe -- and then suppressed with that reason via `nolint: dropped-probe`.
    sdpr = repo / "tools" / "scan_dropped_probes.py"
    specs.append(
        (
            "scan_dropped_probes",
            [py, str(sdpr), "--lint"] if sdpr.is_file() else ["MISSING"],
            "stdout",
        )
    )
    # The COHAB-FLIP half of the dropped-handler class, which scan_dropped_probes above
    # structurally cannot see. That one reads the commented-out C rustconv leaves inline in
    # the same .rs, so it needs both sides in one file. A cohab flip puts them in two: the
    # .c stays on disk, gets commented out of the CMake source list, and a separate .rs
    # takes over under #[no_mangle]. lpc-rust was the proof -- zero inline C, zero _SEH2
    # text, and six files in which every ProbeForRead/Write had lost its _SEH2_EXCEPT, so
    # any unprivileged process passing a bad pointer to NtReplyPort/NtRequestPort/
    # NtSecureConnectPort/NtAcceptConnectPort bugchecked the kernel.
    #
    # Gated at ZERO rather than against a baseline, because the class was driven to zero
    # when it was found; a baseline here would be a place for the next one to hide. The
    # lower-confidence pairs (basename-only, or C still compiled) are printed to stderr and
    # deliberately NOT gated -- basename matching pairs fastfat/create.c with
    # lpc-rust/create.rs, and a human has to read those.
    # A local or parameter the C declares as a pointer TYPEDEF (`PFILEPATHS_W x`) that the
    # Rust declares one level deeper (`x: *mut PFILEPATHS_W`), then reads `(*(*x)).Field` --
    # the struct's first field taken as its address. usetup shipped five (every keypress
    # and copy-progress message read garbage); rustconv's purchase audit now rejects the
    # shape in a new body, and this holds the bodies already in the tree to the same check.
    # Gated at ZERO: the sweep that introduced it found two (pcix, ramdisk), both fixed.
    sptd = repo / "tools" / "scan_ptr_typedef_indirection.py"
    specs.append(
        (
            "scan_ptr_typedef_indirection",
            [py, str(sptd)] if sptd.is_file() else ["MISSING"],
            "stdout",
        )
    )
    sdsp = repo / "tools" / "scan_dropped_seh_pairs.py"
    specs.append(
        (
            "scan_dropped_seh_pairs",
            [py, str(sdsp), "--lint"] if sdsp.is_file() else ["MISSING"],
            "stdout",
        )
    )
    # The KEPT-probe half: the probe is still there, but it is the RAISING one. The C
    # catches it in a _SEH2_EXCEPT; with DUMMY_PSEH nothing does, so one bad pointer
    # from user mode is BUGCHECK 0x1E. About 160 such calls (ProbeForRead/Write and the
    # typed probe.h helpers) were converted to ntkrnl_sys::probe's Result probes on
    # 2026-10-04. Gated at ZERO for the same reason as the pairs scanner above.
    srp = repo / "tools" / "scan_raising_probes.py"
    specs.append(
        (
            "scan_raising_probes",
            [py, str(srp), "--lint"] if srp.is_file() else ["MISSING"],
            "stdout",
        )
    )
    # The access AFTER the probe: an Nt* syscall that loads or stores through its
    # caller's pointer with a plain deref or copy. The probe proves where the buffer
    # is, never that it stays mapped, and without SEH a fault there is a bugcheck any
    # process can trigger. Gated against a BASELINE, not zero: the scanner cannot tell
    # a kernel-mode-only branch from a user one, and the inherited rows are owed work
    # (vibeos-roadmap "Kernel user-pointer probes"). A NEW row fails.
    srua = repo / "tools" / "scan_raw_user_access.py"
    specs.append(
        (
            "scan_raw_user_access",
            [py, str(srua), "--lint"] if srua.is_file() else ["MISSING"],
            "stdout",
        )
    )
    # The SAME user address read twice (a TOCTOU double-fetch): one snapshot validates
    # a size and a second, re-read snapshot sizes the copy, so on SMP a thread on
    # another CPU changes the value in between. Gated against a BASELINE of reviewed
    # two-stage reads (header then body) and one owed low-severity case; a NEW row —
    # a fresh same-source pair reachable on one control-flow path — fails.
    sdf = repo / "tools" / "scan_double_fetch.py"
    specs.append(
        (
            "scan_double_fetch",
            [py, str(sdf), "--lint"] if sdf.is_file() else ["MISSING"],
            "stdout",
        )
    )
    # A user pointer validated by a NON-raising range probe (SafeProbeForRead /
    # SafeProbeForWrite, or a crate's Ks* mirror) and then dereferenced RAW. The probe
    # proves the range, never that the page stays mapped, so under DUMMY_PSEH a raw
    # touch after it is an any-process BUGCHECK 0x1E where the C's _SEH2_TRY returned a
    # status (NtUserScrollDC / NtUserSetSysColors / NtGdiGetDIBits were this). The class
    # was driven to zero, so this gates at ZERO (empty baseline): a baseline would be a
    # place for the next one to hide.
    sspd = repo / "tools" / "scan_safe_probe_deref.py"
    specs.append(
        (
            "scan_safe_probe_deref",
            [py, str(sspd), "--lint"] if sspd.is_file() else ["MISSING"],
            "stdout",
        )
    )
    # The same NOOP shape scan_dropped_prints finds, but keyed on a property of the CRATE
    # ("you already define this as a macro_rules!") instead of a hardcoded print list — so it
    # covers the names that emit nothing and were therefore outside every scanner here.
    # UNREFERENCED_PARAMETER was one: shell32-rust had the correct #[macro_export] macro in
    # win32.rs and two modules carried an empty `fn` shadow anyway, because both are declared
    # BEFORE win32 in lib.rs and macro_rules! textual scoping only runs forward. The baseline
    # is EMPTY (driven to zero 2026-10-09, tools/fn_call_to_macro.py), so this gates at zero.
    smfs = repo / "tools" / "scan_macro_fn_shadow.py"
    specs.append(
        (
            "scan_macro_fn_shadow",
            [py, str(smfs), "--lint"] if smfs.is_file() else ["MISSING"],
            "stdout",
        )
    )
    stac = repo / "tools" / "scan_type_alias_contradictions.py"
    specs.append(
        (
            "scan_type_alias_contradictions",
            [py, str(stac), "--lint"] if stac.is_file() else ["MISSING"],
            "stdout",
        )
    )
    # msvcrt's printf `...` entry points are the one part of the family that cannot be
    # Rust (stable has no va_start), so they are a hand transcription of wcs.c's shells
    # in their own TU — their own TU because left in wcs.c clang keeps inlining wcs.c's
    # C engine into them and the flip is silently undone. Nothing can grade them by
    # execution: msvcrt's printf plumbing is only coherent where a va_list IS a pointer,
    # which no SysV host is (see the scanner's header). So the transcription is checked
    # at the source, and every tolerated rewrite is proven from wcs.c's own forwarders.
    # Baseline is EMPTY and must stay so — a finding here is a live printf bug.
    svsp = repo / "tools" / "scan_varargs_shim_parity.py"
    specs.append(
        (
            "scan_varargs_shim_parity",
            [py, str(svsp), "--lint"] if svsp.is_file() else ["MISSING"],
            "stdout",
        )
    )
    # Hand-encoded UTF-16 string literals decoded against their `// L"..."` intent
    # comment. The SERVICES_KEY class (2026-07-10): `static X: [u16; 33]` sized by
    # hand, one element short, silently dropped the last character -> services-rust
    # opened `...\Service` and every registration failed ERROR_FILE_NOT_FOUND. rustc
    # cannot see it (the array is well-formed at any length) and the comment above it
    # says the RIGHT string, so review slides off. Baseline is EMPTY — the one real
    # survivor (hal kdpci KEY_NAME, truncated AND dead) is deleted; this keeps it at 0.
    swl = repo / "tools" / "scan_wide_literals.py"
    specs.append(
        (
            "scan_wide_literals",
            [py, str(swl)] if swl.is_file() else ["MISSING"],
            "stdout",
        )
    )
    # Dangling pointers to transmute temporaries: `let p = transmute::<[u8;N],
    # [u16;M]>(x).as_ptr()` (or the same as a struct field) — a rustconv artifact
    # where C's stack-backed WCHAR[] became a storageless temporary, so the stored
    # pointer dangles the instant the statement ends. Confirmed real 2026-07-12
    # (disk-rust symbolic-link printf fmt; lsasrv RTL_QUERY_REGISTRY_TABLE Name).
    # Baseline is EMPTY — the class is fully fixed; this gate keeps it at zero.
    sdt = repo / "tools" / "scan_dangling_transmute.py"
    specs.append(
        (
            "scan_dangling_transmute",
            [py, str(sdt), "--lint"] if sdt.is_file() else ["MISSING"],
            "stdout",
        )
    )
    # Pointer arithmetic on a struct field bindgen typed as an OPAQUE 1-byte stub
    # steps one byte per element whatever the code casts to afterwards. consrv's
    # handle table overlapped its entries this way and crashed csrss on the first
    # console close (BUGCHECK 0xF4). Baseline EMPTY.
    sopa = repo / "tools" / "scan_opaque_ptr_arith.py"
    specs.append(
        (
            "scan_opaque_ptr_arith",
            [py, str(sopa), "--lint"] if sopa.is_file() else ["MISSING"],
            "stdout",
        )
    )
    # A Rust-ABI fn transmuted into a C callback slot (`transmute(f as *const ())`)
    # erases the ABI rustc would otherwise check. It works by accident on
    # amd64/arm64 and breaks on i386: consrv's GuiVtbl (22 of 25 slots) killed
    # csrss with BUGCHECK 0xF4 on the first i386 GUI console. Baseline EMPTY.
    sfa = repo / "tools" / "scan_fnptr_abi_transmute.py"
    specs.append(
        (
            "scan_fnptr_abi_transmute",
            [py, str(sfa), "--lint"] if sfa.is_file() else ["MISSING"],
            "stdout",
        )
    )
    # A bare cdecl i386 symbol ALIASED (`.set _X, _X@N`) onto a stdcall body: links,
    # then every cdecl call pops the arguments twice. fastfat's DriverEntry drifted
    # ESP by 8 and returned into its caller's stack (BUGCHECK 0x1E) the first time a
    # FAT volume loaded it on i386. Only @0 is sound. Baseline EMPTY.
    sca = repo / "tools" / "scan_cdecl_alias.py"
    specs.append(
        (
            "scan_cdecl_alias",
            [py, str(sca), "--lint"] if sca.is_file() else ["MISSING"],
            "stdout",
        )
    )
    # A C byte count (RtlCopyMemory) kept as the ELEMENT count of a u16-typed
    # core::ptr::copy moves twice the bytes. fastfat's FatQueryDirectory copied
    # every long name at double length and faulted on arm64 once winsxs names
    # were long enough (BUGCHECK 0x50 at smss); mountmgr and the registry value
    # cache had the same shape. Baseline EMPTY.
    swb = repo / "tools" / "scan_wide_byte_copy.py"
    specs.append(
        (
            "scan_wide_byte_copy",
            [py, str(swb), "--lint"] if swb.is_file() else ["MISSING"],
            "stdout",
        )
    )
    # Its general form: the byte count kept as the element count of a copy typed as a
    # STRUCT, often through `as *const _`. partmgr's IOCTL_DISK_GET_DRIVE_LAYOUT_EX copied
    # size * 192 bytes into the caller's buffer; diskpart's first query bugchecked 0x19
    # (found by the modrun payload). Baseline EMPTY.
    stb = repo / "tools" / "scan_typed_byte_copy.py"
    specs.append(
        (
            "scan_typed_byte_copy",
            [py, str(stb), "--lint"] if stb.is_file() else ["MISSING"],
            "stdout",
        )
    )
    # A C `goto` translated as `'x: { break 'x; }` -- a block that jumps to its own end, so
    # control falls into the code the C skipped. setuplib's ParseArcName did it for
    # `ramdisk(0)` and no ramdisk ARC path resolved (found by the modrun payload). Baseline:
    # the 23 sites found 2026-10-07, each a vestige or a live bug still to be read.
    sng = repo / "tools" / "scan_noop_goto.py"
    specs.append(
        (
            "scan_noop_goto",
            [py, str(sng), "--lint"] if sng.is_file() else ["MISSING"],
            "stdout",
        )
    )
    # A `#[no_mangle]` printf whose parameters end at the format: it cannot read its
    # arguments, so it prints the format verbatim or nothing. freeldr's RtlStringCbPrintfW
    # registered every bootcd driver as "%S\\%S" and panicked the setup loader; classpnp's
    # pair left DEVICEMAP's key path empty. Baseline: none.
    sfp = repo / "tools" / "scan_formatless_printf.py"
    specs.append(
        (
            "scan_formatless_printf",
            [py, str(sfp), "--lint"] if sfp.is_file() else ["MISSING"],
            "stdout",
        )
    )
    # `(p as *mut u8).add(1)` for the C's `(PUCHAR)(p + 1)`: one byte instead of one
    # struct. usetup's cabinet reader read its folder table from inside the header (the
    # install smoke's first file copy faulted), and setuplib/recyclebin had it too.
    # Baseline: the four sites where `p` really is a byte pointer, each read against its C.
    sbs = repo / "tools" / "scan_byte_step_one.py"
    specs.append(
        (
            "scan_byte_step_one",
            [py, str(sbs), "--lint"] if sbs.is_file() else ["MISSING"],
            "stdout",
        )
    )
    # `(*(x as *mut PFOO).read()).F` for the C's `((PFOO)x)->F`: the cast form of
    # scan_ptr_typedef_indirection. usetup's unattended language lookup used L"0000" as a
    # pointer and the bootcd's smss.exe died (0x71). Gated at zero.
    spc = repo / "tools" / "scan_ptr_typedef_cast_read.py"
    specs.append(
        (
            "scan_ptr_typedef_cast_read",
            [py, str(spc), "--lint"] if spc.is_file() else ["MISSING"],
            "stdout",
        )
    )
    # A bare integer where a FILE_INFORMATION_CLASS belongs. Mm extended files with class 2
    # (FileFullDirectoryInformation) instead of FileEndOfFileInformation (20), so setup's
    # cabinet extraction lost 964 files; apphelp passed 0. Gated at zero.
    sic = repo / "tools" / "scan_info_class_literals.py"
    specs.append(
        (
            "scan_info_class_literals",
            [py, str(sic), "--lint"] if sic.is_file() else ["MISSING"],
            "stdout",
        )
    )
    # A named constant cast to a type too narrow for it: rustc's overflow lint covers
    # literals only. disk.sys set SRB_CLASS_FLAGS_PERSISTANT `as USHORT` (= 0), classpnp freed
    # an SRB inside the device extension, 0xC2 on the first flush. Gated at zero.
    scn = repo / "tools" / "scan_const_narrowing_cast.py"
    specs.append(
        (
            "scan_const_narrowing_cast",
            [py, str(scn), "--lint"] if scn.is_file() else ["MISSING"],
            "stdout",
        )
    )
    # A statement a bulk edit folded into a trailing comment: `{ // DPRINT removed return X; }`.
    # cmlib's HvLoadHive lost six of them and fell through on a file that was not a hive.
    # Gated at zero.
    scs = repo / "tools" / "scan_comment_swallowed_code.py"
    specs.append(
        (
            "scan_comment_swallowed_code",
            [py, str(scs), "--lint"] if scs.is_file() else ["MISSING"],
            "stdout",
        )
    )
    # A call passing a different local than its own C at the same call (k-th call of a callee,
    # same arity). userenv's InitializeProfiles loaded the profiles path into the buffer it
    # then expanded an empty string over: GUI setup stopped at "InitializeProfiles() failed".
    # Baseline: reviewed look-alikes (renamed buffers, an inlined Status, a deliberate WDF fix).
    sas = repo / "tools" / "scan_call_arg_swap.py"
    specs.append(
        (
            "scan_call_arg_swap",
            [py, str(sas), "--lint"] if sas.is_file() else ["MISSING"],
            "stdout",
        )
    )
    # A narrow b"..." passed where the file's own C passes L"...". kernel32's FindFirstFileExW
    # compared its UTF-16 pattern with b"*.*", never shortened it, and "*.*" stopped matching
    # "." and "..": FindFirstFileW failed on every empty directory. Gated at zero.
    snw = repo / "tools" / "scan_narrow_vs_wide_literal.py"
    specs.append(
        (
            "scan_narrow_vs_wide_literal",
            [py, str(snw), "--lint"] if snw.is_file() else ["MISSING"],
            "stdout",
        )
    )
    # A re-ported mm.h section-segment-entry helper (`IS_DIRTY_SSE` & co.) evaluated against
    # the macro on sample entries. ntkrnl-sys's IS_DIRTY_SSE tested a share-count bit, so
    # MiShutdownSystem looped forever and the install never rebooted. Gated at zero.
    sse = repo / "tools" / "scan_sse_helpers.py"
    specs.append(
        (
            "scan_sse_helpers",
            [py, str(sse), "--lint"] if sse.is_file() else ["MISSING"],
            "stdout",
        )
    )
    # A location read right after InterlockedExchange*(P, 0) cleared it, instead of the
    # returned old value. i386/amd64 MmDeleteVirtualMapping lost every PTE dirty bit that way:
    # mapped-file writes never reached the disk. Gated at zero.
    src = repo / "tools" / "scan_read_after_clear.py"
    specs.append(
        (
            "scan_read_after_clear",
            [py, str(src), "--lint"] if src.is_file() else ["MISSING"],
            "stdout",
        )
    )
    # A kernel object initialized through MaybeUninit::as_mut_ptr() and then moved out with
    # assume_init(): its self-pointing list heads still name the old slot. IoShutdownSystem's
    # event did it (0x109 at the end of every install). Gated at zero.
    sim = repo / "tools" / "scan_init_then_move.py"
    specs.append(
        (
            "scan_init_then_move",
            [py, str(sim), "--lint"] if sim.is_file() else ["MISSING"],
            "stdout",
        )
    )
    # CONTAINING_RECORD with the field passed as a null pointer: offset 0, right only for a
    # first field. po's volume flush took the LAST field that way (0x1E at setup's reboot).
    # Gated at zero.
    scr = repo / "tools" / "scan_containing_record_null.py"
    specs.append(
        (
            "scan_containing_record_null",
            [py, str(scr), "--lint"] if scr.is_file() else ["MISSING"],
            "stdout",
        )
    )
    # A call passing a number where the C kept in the same file passes a named constant.
    # CmLoadKey passed check flag 1 for CM_CHECK_REGISTRY_PURGE_VOLATILES (2): every NtLoadKey
    # of a hive file failed REGISTRY_CORRUPT. Its first sweep also found win32k's GDI reuse
    # counter, cdrom's media-change IOCTL, desk/netapi32/userenv arguments. Baseline: the
    # pool-tag pairs (allocated and freed with the same tag) and the UNRESOLVED names.
    scl = repo / "tools" / "scan_call_literal_vs_c.py"
    specs.append(
        (
            "scan_call_literal_vs_c",
            [py, str(scl), "--lint"] if scl.is_file() else ["MISSING"],
            "stdout",
        )
    )
    # A struct the C declares under pshpack that its Rust port lays out unpacked. diskpart read
    # the MBR magic from past the sector (no disk looked partitioned); setuplib's boot sectors
    # shifted every BPB field. Baseline: freeldr's PNP_DOCK_INFO and ACPI_BIOS_DATA, whose
    # field offsets are identical either way (only trailing padding differs).
    spd = repo / "tools" / "scan_pack_drift.py"
    specs.append(
        (
            "scan_pack_drift",
            [py, str(spd), "--lint"] if spd.is_file() else ["MISSING"],
            "stdout",
        )
    )
    # A UTF-16 literal spelled as a byte string (`b"L\0i\0b\0..."`) and cast to a wide
    # pointer has alignment 1; any link change can move it to an odd address, which
    # ProbeForRead(.., sizeof(WCHAR)) rejects. ws2_32's L"LibraryPath" did exactly that
    # when msvcrtex became Rust and bugchecked i386 (0x1E). Baseline EMPTY: use
    # win32macros_rust::wide_bytes!.
    suw = repo / "tools" / "scan_unaligned_wide.py"
    specs.append(
        (
            "scan_unaligned_wide",
            [py, str(suw), "--lint"] if suw.is_file() else ["MISSING"],
            "stdout",
        )
    )
    # The WoW64 win32k thunk table must be indexed by the GUEST's (i386) syscall
    # numbers, not the host's. w32ksvc32.h gates NtUserSetWindowLongPtr on
    # `#ifdef _WIN64`, so parsing it as flat text on a 64-bit host shifts every
    # service number after it by one — 195 syscalls silently misrouted onto their
    # neighbours (no fault: the signatures are plausible, the calls just do the
    # wrong thing). x86 Doom's ShowWindow became ShowScrollBar. Field manual
    # §9.71. This compares the generated table against the REAL i386 stubs
    # disassembled out of the built user32/gdi32 — hard fail on any divergence.
    vsn = repo / "tools" / "wow64gen" / "verify_svc_numbering.py"
    specs.append(
        (
            "verify_svc_numbering",
            [py, str(vsn)] if vsn.is_file() else ["MISSING"],
            "stdout",
        )
    )
    # The WoW64 flat-thunk blind spot. wow64gen types its thunks from the
    # .spec, which says only "this arg is a pointer" — never what it points AT.
    # When the real prototype takes a pointer to a payload whose 32- and 64-bit
    # layouts differ, the generated FLAT thunk hands 64-bit win32k the guest's
    # 32-bit struct and the kernel reads it with 64-bit eyes. Because such a
    # call is never flagged NEEDS_MANUAL, `marshalgen report` cannot see it —
    # so the class was found three times the expensive way instead, each as a
    # live BUGCHECK 0x1E from a garbage UNICODE_STRING.Buffer (E6 GetAtomName /
    # GetClassName, E8 UnregisterClass, E9 FindWindowEx), plus E7's HWND-array
    # heap overrun. `marshalgen audit` cross-checks every LIVE flat thunk in
    # the generated table against its real header prototype using clang's own
    # 32/64 record layouts, so a new one is caught at build time instead of by
    # the next applet to call it. Baseline gates NEW.
    #
    # The table is derived from CHECKED-IN sources (.spec + sysfuncs.h + the
    # manual lists), NOT from build-*/...gen.c, so this baseline is portable
    # across worktrees; the built artifact is cross-checked and a divergence
    # HARD FAILS as "stale build dir". That is not defensive coding: reading the
    # artifact directly made a stale build re-report 33 already-FIXED syscalls,
    # and another lane re-blessed them into the baseline as accepted debt
    # (field manual §9.115a). A lint whose output gets committed must be a pure
    # function of committed state.
    mga = repo / "tools" / "wow64gen" / "marshalgen.py"
    specs.append(
        (
            "marshalgen_audit",
            [py, str(mga), "audit"] if mga.is_file() else ["MISSING"],
            "stdout",
        )
    )
    # Same-crate extern-decl vs #[no_mangle]-def calling-convention drift: UB
    # that LLVM resolves by silently DELETING the calling branch on i386 (no
    # ud2, no link error — the desktop-icons U32AccelCacheLock bug, field
    # manual §9.30). Stock (~330, mostly driver crates) is the burn-down
    # queue; baseline gates NEW only.
    ccm = repo / "tools" / "scan_cc_mismatch.py"
    specs.append(
        (
            "scan_cc_mismatch",
            [py, str(ccm)] if ccm.is_file() else ["MISSING"],
            "stdout",
        )
    )
    # The OTHER half of that pair: a cross-MODULE extern declaration, checked against the
    # .spec of the DLL that exports it. `cargo check` never links, so nothing in `check`,
    # `check-tooling` or `difftest` can see a wrong-ABI import — it survives to the PE link,
    # which is the merge-queue gate. Wave 18c bounced that way (errno.rs declared kernel32's
    # WINAPI SetErrorMode `extern "C"`, so the linker wanted `_SetErrorMode` and the import
    # lib had `_SetErrorMode@4`), and the tree carries three older instances that were papered
    # over with hand-written stdcall trampolines instead. Baseline holds 4 generated-island
    # findings where the C HEADER and the .spec genuinely disagree upstream.
    # A raw-dylib link name that resolves to a file that is not there. rustc appends
    # `.dll` to the name UNCONDITIONALLY — it does not look for an existing extension —
    # so `name = "ntoskrnl"` names `ntoskrnl.dll`, and ks/pcix/ramdisk shipped that import
    # descriptor for months while freeldr logged `PeLdrLoadImage(...ntoskrnl.dll) failed`
    # into a debug channel that was itself silent. The reason this is a LINT and not a
    # lesson is that the obvious fix is also wrong and also shipped: bare
    # `name = "ntoskrnl.exe"` yields `ntoskrnl.exe.dll`, and nothing catches it — raw-dylib
    # invents the descriptor, so there is no unresolved symbol and the link succeeds. It
    # took llvm-readobj on the rebuilt binary to see it. Correct is the name WITH its real
    # extension plus modifiers = "+verbatim". Note the scanner does NOT simply demand
    # +verbatim everywhere: bare `ntdll` -> `ntdll.dll` is correct, and flagging it would
    # teach the next agent to write `ntdll.dll.dll`. Baseline is EMPTY and should stay that way.
    srd = repo / "tools" / "scan_rawdylib_linkname.py"
    specs.append(
        (
            "scan_rawdylib_linkname",
            [py, str(srd)] if srd.is_file() else ["MISSING"],
            "stdout",
        )
    )
    sic = repo / "tools" / "scan_import_cc.py"
    specs.append(
        (
            "scan_import_cc",
            [py, str(sic)] if sic.is_file() else ["MISSING"],
            "stdout",
        )
    )
    # A translated ASSERT whose guard IS the asserted condition: it fires on every
    # legitimate path and never on the one it guards. pcix's PciBuildHackTable printed
    # `ASSERT FAILED: !NT_SUCCESS(Status)` on every boot. Baseline is EMPTY.
    sap = repo / "tools" / "scan_assert_polarity.py"
    specs.append(
        (
            "scan_assert_polarity",
            [py, str(sap)] if sap.is_file() else ["MISSING"],
            "stdout",
        )
    )
    # A C octal literal that became a Rust DECIMAL one. `0444` is octal 292 in C and four
    # hundred and forty-four in Rust, which has no C-style octal at all — so the mistake
    # compiles, links, produces a plausible value, and no rustc lint has an opinion about
    # it. msvcrt shipped `st_mode` 0o776 instead of 0o666 through six `_fstat*` exports on
    # all three arches until a difftest row diverged on it. That is the point of scanning
    # for it: msvcrt/cnt/rtl are the only translated modules with a differential harness,
    # the other ~40 crates have no oracle at all, and every C source we translate is full
    # of octal mode bits. Baseline is EMPTY and should stay that way.
    # A FOURCC the model folded to an integer itself, keeping `// MAKEFOURCC('I','M','C','1')`
    # beside a number that is not those four characters. evr's MFIsFormatYUV had three of
    # nine wrong; rustc saw two only because they collided with right ones (unreachable
    # arms). Held to its own comment, so no C is needed. Baseline is EMPTY.
    sfc = repo / "tools" / "scan_fourcc_literals.py"
    specs.append(
        (
            "scan_fourcc_literals",
            [py, str(sfc)] if sfc.is_file() else ["MISSING"],
            "stdout",
        )
    )
    # A C switch's `case X: ...; break;` inside a loop, translated to a match arm whose
    # trailing `break;` now leaves the LOOP: mf's session_check_topology leaked a node
    # reference per SetTopology, welcome's TranslateEscapes stopped at the first `\x`.
    # The baseline is 19 triaged deliberate exits (goto-to-exit in the C, hand-written Rust).
    smb = repo / "tools" / "scan_match_arm_break.py"
    specs.append(
        (
            "scan_match_arm_break",
            [py, str(smb)] if smb.is_file() else ["MISSING"],
            "stdout",
        )
    )
    # `calloc(n, size_of::<OLECHAR>()) as *mut BSTR` for the C's `calloc(n, sizeof(*strs))`:
    # the element's element. wscript's Host_Echo overran its array on every multi-argument
    # Echo, dropped the second argument and crashed the host (modrun's cscript suite).
    # Baseline is EMPTY.
    sce = repo / "tools" / "scan_calloc_elem_type.py"
    specs.append(
        (
            "scan_calloc_elem_type",
            [py, str(sce)] if sce.is_file() else ["MISSING"],
            "stdout",
        )
    )
    # A variadic extern whose fixed arity is not its .spec export's. The C's count-form
    # `swprintf(buf, n, fmt, ...)` is a <stdio.h> inline over _vsnwprintf; bound to msvcrt's
    # legacy `swprintf(ptr wstr)` export, `n` became the format pointer (cscript crashed on
    # every script error; msiexec, ole32 and the recycle bin had it too). Baseline is EMPTY.
    svs = repo / "tools" / "scan_varargs_spec_arity.py"
    specs.append(
        (
            "scan_varargs_spec_arity",
            [py, str(svs)] if svs.is_file() else ["MISSING"],
            "stdout",
        )
    )
    sol = repo / "tools" / "scan_octal_literals.py"
    specs.append(
        (
            "scan_octal_literals",
            [py, str(sol)] if sol.is_file() else ["MISSING"],
            "stdout",
        )
    )
    # `ptr & 0xFFFF0000 == 0` as "the pointer is at most 0xFFFF" tests bits 16-31 only on
    # 64-bit. RtlpCheckIntegerAtom did that, so a pool-captured atom name at ...0000_xxxx
    # became integer atom xxxx for every name in the boot, window properties collided, and
    # arm64 applets died with no window. Baseline is EMPTY.
    phm = repo / "tools" / "scan_ptr_highword_mask.py"
    specs.append(
        (
            "scan_ptr_highword_mask",
            [py, str(phm)] if phm.is_file() else ["MISSING"],
            "stdout",
        )
    )
    # A `++`/`--` inside a CALL ARGUMENT whose effect a later operand of the same expression
    # observes. Rust has no `++`, so the translation must split it, and where the second
    # statement lands is the whole meaning — msvcrt's demangler hoisted one out of an `if`
    # condition and every function signature then parsed its return type from the
    # calling-convention letter. Both forms advance the cursor exactly once, so no value-based
    # check can see it; a differential row found it after it had shipped and booted.
    # Baseline holds the 5 known sites (2 in msvcrt's undname.c, both since verified; 3 in the
    # un-flipped sdk/lib/crt copy of the same file); a NEW one is the thing to look at.
    ase = repo / "tools" / "scan_arg_side_effects.py"
    specs.append(
        (
            "scan_arg_side_effects",
            [py, str(ase)] if ase.is_file() else ["MISSING"],
            "stdout",
        )
    )
    # A committed resource object whose TYPE is a string spelling an RT_* name: llvm-rc's
    # reading of a keyword it lacks (MESSAGETABLE). The cargo-linked kernel and ntdll ship
    # such blobs, generated out of band, and a stale one hid every message table in the OS
    # (FormatMessage failed, notepad's error box read "5") behind green gates. Baseline EMPTY.
    srn = repo / "tools" / "scan_res_named_types.py"
    specs.append(
        (
            "scan_res_named_types",
            [py, str(srn)] if srn.is_file() else ["MISSING"],
            "stdout",
        )
    )
    # An index into a bindgen `__bindgen_padding_N` array to reach a member bindgen could
    # not name. Its element type is the generator's choice: a regeneration turned
    # THRDESKHEAD's [u64; 3] into [u8; 24], user32's `head.__bindgen_padding_0[2]` became one
    # byte of the handle, and every posted message went undispatched. Baseline EMPTY.
    sbp = repo / "tools" / "scan_bindgen_padding_index.py"
    specs.append(
        (
            "scan_bindgen_padding_index",
            [py, str(sbp)] if sbp.is_file() else ["MISSING"],
            "stdout",
        )
    )
    # A Rust `extern static` read wider than the C that defines it through __asm__("NAME").
    # fastfat read the shim's 4-byte MAX_LFN_CHARACTERS as usize; on arm64 the high half was
    # the next datum, every rename buffer pointer went non-canonical, and every rename
    # bugchecked 0x23 while amd64 read a zero and passed. Baseline EMPTY.
    sesw = repo / "tools" / "scan_extern_static_width.py"
    specs.append(
        (
            "scan_extern_static_width",
            [py, str(sesw)] if sesw.is_file() else ["MISSING"],
            "stdout",
        )
    )
    # Two definitions of one exported symbol, live on the same arch in the same
    # binary. Every PE links with --allow-multiple-definition (load-bearing: each
    # Rust staticlib graph carries its own panic runtime), so a duplicate resolves
    # by archive order, silently. Models each binary's crate closure, Cargo
    # features and per-arch cfg, and sees global_asm! labels — the i386 prefetch
    # stub that crashed kmtest (88b04bf7f86) was a Rust fn beside its own crate's
    # asm label. Baseline is EMPTY = a hard 0-gate. See the tool docstring.
    sds = repo / "tools" / "scan_duplicate_symbols.py"
    specs.append(
        (
            "scan_duplicate_symbols",
            [py, str(sds), "--lint"] if sds.is_file() else ["MISSING"],
            "stdout",
        )
    )
    # A kernel-mode driver defining a name ntoskrnl/HAL exports (or the internal name an
    # export forwards to) runs its own copy instead of the kernel's. classpnp's no-op
    # RtlInterlocked*EntrySList stubs dropped every transfer packet this way for four
    # months. The baseline is the reviewed set (header inlines, real algorithms); a NEW
    # definition fails. See the tool docstring.
    ses = repo / "tools" / "scan_export_shadows.py"
    specs.append(
        (
            "scan_export_shadows",
            [py, str(ses), "--lint"] if ses.is_file() else ["MISSING"],
            "stdout",
        )
    )
    # Leaked IRQL raise / spinlock acquire on an early-exit path — an early
    # `return`/`?` between a `KeRaiseIrql`/`KeAcquireSpinLock` and its matching
    # lower/release. We translate C's manual balance verbatim, so a goto-cleanup
    # mistranslated as a bare return silently leaks the raise (found the usbport
    # empty-list spinlock leak, upstream ReactOS bug — 2026-07-06). A leaked
    # raise strands soft-IRQL high → the next acquire raises DOWN → preemption
    # mid-critical-section → corruption (field manual §9.24). Tree is CLEAN
    # today, so the baseline is empty = a hard 0-gate; any new leak blocks.
    sib = repo / "tools" / "scan_irql_balance.py"
    specs.append(
        (
            "scan_irql_balance",
            [py, str(sib), "--lint"] if sib.is_file() else ["MISSING"],
            "stdout",
        )
    )
    # Ob-refcount balance — same held-set engine, next lifecycle class: an
    # early return between ObReferenceObject* and ObDereferenceObject. A ref
    # leak never crashes a boot (slow pool exhaustion), so no gate boot can
    # catch it — only this. The 2026-07-07 sweep found 3 UPSTREAM leaks (2
    # fixed: cmapi deleted-KCB KeyBody, dbgk last-thread; sysldr's is dead
    # code) and 0 Rust-translation leaks; baseline holds the verified
    # branch-merge FP residue (field manual §9.28).
    sob = repo / "tools" / "scan_ob_refcount.py"
    specs.append(
        (
            "scan_ob_refcount",
            [py, str(sob), "--lint"] if sob.is_file() else ["MISSING"],
            "stdout",
        )
    )
    # COM double-release: a C++ `CComPtr` local lowered to a raw pointer plus a
    # `CppComPtrGuard` whose explicit `Release()` calls were ALSO kept. The guard
    # IS the destructor, so the second release frees a live object and the guard
    # then calls through a dangling vtable. CDefView's FillList shipped this on
    # three exit paths: arm64 died at once, while amd64 and i386 ran the same code
    # and passed the 91-test conformance sweep three times running. A defect two of
    # three arches certify as green needs a scanner, not a reviewer. The 2026-09-19
    # sweep found 3 in CDefView (fixed) and 15 in the still-orphan utils.rs (fixed
    # before the utils flip could ship them); baseline is empty.
    scdr = repo / "tools" / "scan_com_double_release.py"
    specs.append(
        (
            "scan_com_double_release",
            [py, str(scdr), "--lint"] if scdr.is_file() else ["MISSING"],
            "stdout",
        )
    )
    # An `extern` DECL that contradicts the `#[no_mangle]` DEF of the same symbol.
    # rustc checks a call against the decl and never against the def — they meet
    # only in the linker, which compares nothing but the name — so this class is
    # invisible to every compiler in the pipeline, and to verify_signatures /
    # verify_c_signatures too (both need a C prototype, which a fully-translated
    # module no longer has). It found fastfat's FatGetNextMcbEntry storing 8 bytes
    # through a 4-byte out-param and freeldr's TuiDisplayMenu being called with 11
    # arguments for 10 parameters.
    # A Rust-source walk written as `rglob("*.rs")` + a `"/target/" in path` filter.
    # `rglob` finishes descending into sdk/lib/rust/target BEFORE the filter sees a
    # path, so once `source-early` moved the audited scanners alongside the builds,
    # cargo deleting a directory mid-walk killed the scanner with a traceback — and
    # the gate reported it as `lint:source-early: NEW lint findings (exit 1)` with no
    # findings in the artifact, sending a lane after a source defect that did not
    # exist. The six EARLY_SOURCE_CHECKS scanners are migrated to
    # tools/rust_sources.py walk_files() (prune during descent + onerror); baseline
    # holds the 65 latent sites so the class cannot grow while they burn down. This
    # one is DEFERRED on purpose: a guard against build-concurrency races has no
    # business racing the builds.
    suw = repo / "tools" / "scan_unpruned_walk.py"
    specs.append(
        (
            "scan_unpruned_walk",
            [py, str(suw), "--lint"] if suw.is_file() else ["MISSING"],
            "stdout",
        )
    )
    sdd = repo / "tools" / "scan_extern_decl_drift.py"
    specs.append(
        (
            "scan_extern_decl_drift",
            [py, str(sdd), "--lint"] if sdd.is_file() else ["MISSING"],
            "stdout",
        )
    )
    # A C ASSERT that VANISHED when its FORCEINLINE was hand-translated. Not
    # covered by scan_assert_to_ub (that catches an assert LOWERED to
    # unreachable_unchecked, i.e. present and wrong) and NOT clonable from
    # scan_dropped_prints, whose oracle is rustconv's trailing C comments —
    # the bridge modules that carried this bug are hand-written and carry none,
    # so a same-file diff reports clean on the exact function that cost the boot.
    # The oracle is cross-file: the C FORCEINLINE paired by name against its Rust
    # twin. It found MI_MAKE_HARDWARE_PTE_USER shipping 0 of the C's 4 asserts,
    # which on arm64 turns a no-access protection mask into Valid=1 on a
    # descriptor the MMU rejects (task #39).
    sda = repo / "tools" / "scan_dropped_asserts.py"
    specs.append(
        (
            "scan_dropped_asserts",
            [py, str(sda), "--lint"] if sda.is_file() else ["MISSING"],
            "stdout",
        )
    )
    # NOTE: scan_bind_all_matches (bind-all match-arm detector) is intentionally
    # NOT wired here yet. It runs `cargo check` in a SEPARATE cold target dir
    # (to avoid force-warn polluting the shared cache), which cold-triggers
    # reactos-link's build.rs — and that build.rs hard-panics when it can't find
    # a matching compile_commands.json (`no compile_commands.json entry for
    # …/bitmap.c in target blrtl`). So a whole-workspace scan from a fresh dir
    # fails to compile before it can report findings. It works standalone with a
    # WARM target dir (`SCAN_BINDALL_TARGET_DIR=sdk/lib/rust/target`) or when
    # scoped to non-boot crates (`-p usbhub-rust …`). OWED (task #5): make the
    # scanner robust — either reuse the warm dir by default (accepting a
    # cargo-cache invalidation) or teach reactos-link's build.rs to degrade to a
    # no-op under a check-only scan — then wire it here as a strict 0-baseline.
    # The bind-all *fixes* themselves are already in the tree and compile-verified.

    return [
        (name, [rustlints.SENTINEL, name], "stdout")
        if name in rustlints.PORTED
        else (name, cmd, src)
        for name, cmd, src in specs
    ]


# Audited CLI paths read Rust sources only and prune Cargo output before traversal.
# Build independence alone is insufficient: excluded directories must never be
# entered while builds mutate them. test_gate_source_partitions audits scandir.
# Everything not explicitly audited stays deferred, including new scanners.
# See docs/engineering/fleet-verification-migration-plan.md for input contracts.
EARLY_SOURCE_CHECKS = frozenset(
    {
        "scan_invented_constants",
        "scan_extern_decl_drift",
        "scan_octal_literals",
        "scan_type_alias_contradictions",
        "scan_irql_balance",
        "scan_ob_refcount",
        "scan_rawdylib_linkname",
    }
)


def source_partition_specs(repo: Path, phase: str) -> list[tuple]:
    """Disjoint, exhaustive views of the original commands; unknown work waits."""
    if phase not in ("source-early", "source-deferred"):
        raise ValueError(f"unknown source partition: {phase}")
    specs = source_lint_specs(repo)
    names = [name for name, _, _ in specs]
    if len(names) != len(set(names)) or EARLY_SOURCE_CHECKS - set(names):
        raise ValueError("source inventory has duplicate names or missing audited checks")
    early = phase == "source-early"
    return [spec for spec in specs if (spec[0] in EARLY_SOURCE_CHECKS) == early]


def lint_jobs() -> int:
    """Use available host CPUs; an explicit job budget overrides detection."""
    available = len(os.sched_getaffinity(0)) if hasattr(os, "sched_getaffinity") else os.cpu_count()
    raw = os.environ.get("VIBEOS_LINT_JOBS")
    n = int(raw) if raw else (available or 1)
    if n < 1:
        raise ValueError("VIBEOS_LINT_JOBS must be positive")
    return n


def run_source_lints(
    run: Run,
    repo: Path,
    *,
    strict: bool,
    only: set[str] | None = None,
    new_fail: bool | None = None,
) -> None:
    """Execute the scanners CONCURRENTLY, then classify them in spec order.

    The scanners are independent read-only passes over the source tree, each
    graded against its own baseline file, so running them at the same time cannot
    change any verdict — only the wall clock. It was 48% of a build: 54 stages,
    851.6s summed against an 852.2s span (summed == span is what proves it was
    serial), single-threaded on a 24-core host.

    ONLY the subprocesses overlap. Every mutation of `run` — stage order, and the
    SCANNER_ERROR classification — stays serial and in `specs` order, so the
    artifact is byte-identical to the serial run and a scanner cannot win a race
    to set the verdict."""
    specs = source_lint_specs(repo)
    if only is not None:
        unknown = only - {name for name, _, _ in specs}
        if not only or unknown:
            raise ValueError(f"empty or unknown scanner selection: {sorted(unknown)}")
    todo = [(n, c, s) for n, c, s in specs if only is None or n in only]
    # NOCHECK/MISSING never spawn anything; they are classified in phase 2 alone.
    ported = [n for n, c, _ in todo if c[0] == rustlints.SENTINEL]
    runnable = [
        (n, c) for n, c, _ in todo if c[0] not in ("NOCHECK", "MISSING", rustlints.SENTINEL)
    ]
    done: dict[str, subprocess.CompletedProcess] = {}
    workers = min(lint_jobs(), len(runnable) + bool(ported))
    if workers > 1:
        from concurrent.futures import ThreadPoolExecutor

        # Threads, not processes: each worker blocks in subprocess.run, which holds
        # no GIL, and `Run.run` already serialises its own log writes under a lock.
        with ThreadPoolExecutor(max_workers=workers) as pool:
            shared = (
                pool.submit(
                    rustlints.run_many,
                    repo,
                    ported,
                    lambda cmd: run.run(cmd, cwd=repo, stage="lint:vibeos-lint"),
                )
                if ported
                else None
            )
            futures = {
                name: pool.submit(run.run, cmd, cwd=repo, stage=f"lint:{name}")
                for name, cmd in runnable
            }
            for name, fut in futures.items():
                done[name] = fut.result()
            if shared is not None:
                done.update(shared.result())
    else:
        if ported:
            done.update(
                rustlints.run_many(
                    repo, ported, lambda cmd: run.run(cmd, cwd=repo, stage="lint:vibeos-lint")
                )
            )
        for name, cmd in runnable:
            done[name] = run.run(cmd, cwd=repo, stage=f"lint:{name}")
    for name, cmd, src in todo:
        _lint_stage(
            run,
            repo,
            name,
            cmd,
            strict=strict,
            findings_from=src,
            cp=done.get(name),
            new_fail=new_fail,
        )


def lint_tool_names(repo: Path) -> set[str]:
    """Every source-lint tool name (for --update-baseline validation)."""
    return {name for name, _, _ in source_lint_specs(repo)}


def write_baselines(repo: Path, only: set[str] | None = None) -> dict:
    """Capture current findings into tools/lint-baseline/<name>.txt so the gate
    becomes 'zero NEW vs baseline' (plan §20.3).

    `only` restricts the rewrite to those tool names — the per-tool update
    (T2 item 1b). The blanket rewrite (only=None) is a footgun once the gate
    BLOCKS on lints: re-blessing every baseline to accept ONE intended change
    silently accepts any *unrelated* regression that drifted in at the same
    time. Scoping to the tool you actually changed keeps the others enforcing.

    Returns {name: {"count", "added", "removed"}} so the caller can print the
    exact delta being blessed — blessing a NEW finding is never silent."""
    out: dict = {}
    bdir = repo / BASELINE_DIR
    bdir.mkdir(parents=True, exist_ok=True)
    specs = source_lint_specs(repo)
    ported = [n for n, c, _ in specs if c[0] == rustlints.SENTINEL and (only is None or n in only)]
    shared = (
        rustlints.run_many(
            repo,
            ported,
            lambda cmd: subprocess.run(cmd, cwd=repo, capture_output=True, text=True, timeout=600),
        )
        if ported
        else {}
    )
    # Refuse the whole update before touching any baseline if the shared scan failed.
    for name, cp in shared.items():
        if _scanner_error(cp, "stdout"):
            raise RuntimeError(f"SCANNER_ERROR: {name} did not complete; refusing baseline update")
    for name, cmd, src in specs:
        if only is not None and name not in only:
            continue
        if cmd[0] == "MISSING":
            continue
        try:
            cp = (
                shared[name]
                if name in shared
                else subprocess.run(cmd, cwd=repo, capture_output=True, text=True, timeout=600)
            )
        except (OSError, subprocess.SubprocessError):
            continue
        if _scanner_error(cp, src):
            raise RuntimeError(f"SCANNER_ERROR: {name} did not complete; refusing baseline update")
        text = _rel(cp.stdout if src == "stdout" else (cp.stdout + "\n" + cp.stderr), repo)
        cur = _finding_lines(text)
        prev = _baseline(repo, name)
        findings = sorted(cur)
        (bdir / f"{name}.txt").write_text("\n".join(findings) + ("\n" if findings else ""))
        out[name] = {
            "count": len(findings),
            "added": len(cur - prev),
            "removed": len(prev - cur),
        }
    return out


def run_committed_drift_guard(run: Run, repo: Path, arch: str | None = None) -> None:
    """Guard the cmake-off committed fallbacks (ks*.inc / cflags / .res / widl
    `.rs`) against silent staleness.

    Those artifacts are committed so a cargo build needs no cmake, but they are
    NOT regenerated on a normal build — so an upstream change (a `.template.h`,
    `.rc`, `.spec`/`.idl`, or cmake flags) silently staleens the committed copy
    while the configured cmake build stays green (it uses the fresh copy). Only a
    cmake-off build would then diverge. `tools/check_committed_drift.py`
    regenerates each from the configured `build-<arch>/` and byte-compares; this
    stage surfaces a DRIFT as a gate failure. Self-skips per artifact when the
    build dir / inputs are absent (so it never blocks a tree without builds)."""
    checker = repo / "tools" / "check_committed_drift.py"
    if not checker.is_file():
        run.add(Stage("committed_drift", status="skip", detail="checker absent"))
        return
    cmd = [sys.executable, str(checker)]
    if arch:
        cmd += ["--arch", arch]
    cp = subprocess.run(cmd, cwd=repo, capture_output=True, text=True)
    tail = (cp.stdout or "").strip().splitlines()
    summary = next((ln.strip() for ln in reversed(tail) if " OK," in ln), "")
    if cp.returncode == 0:
        if "0 OK," in summary and "skipped" in summary:
            run.add(Stage("committed_drift", status="skip", detail="no build dir"))
            return
        # A per-artifact SKIP is NOT a pass — it means that fallback was never
        # verified, and folding it into a ✓ is exactly how one goes stale unseen.
        # Real case (2026-07-15): svcctl_s.arm64.rs sat stale vs the generator
        # because NOTHING ever builds services.exe on arm64, so this checker had
        # always self-skipped it and the stage stayed green. The moment the build
        # dir had the inputs, it caught the drift instantly. Report unverified
        # artifacts as WARN (visible, non-blocking) rather than silence.
        m = re.search(r"(\d+) skipped", summary)
        if m and int(m.group(1)) > 0:
            skipped = [ln.strip() for ln in tail if "SKIP" in ln.upper()]
            run.add(
                Stage(
                    "committed_drift",
                    exit=0,
                    status="warn",
                    detail=f"{summary} — skipped artifacts are UNVERIFIED, not OK",
                    salient=skipped[:5],
                )
            )
            return
        run.add(Stage("committed_drift", exit=0, status="ok", detail=summary or "no drift"))
        return
    run.add(Stage("committed_drift", exit=cp.returncode, status="fail", detail=summary or "DRIFT"))


def _sacred_analysis(run: Run, repo: Path, arch: str, script: Path):
    """Reuse successful primitive analysis of the same kernel and tool inputs."""
    from tools.binary_facts import BinaryFacts

    build = repo / f"build-{arch}"
    cmd = ["bash", str(script), str(build)]
    triple = {"arm64": "aarch64", "amd64": "x86_64", "i386": "i686"}[arch]
    names = [f"{triple}-w64-mingw32-objdump", "llvm-objdump"]
    candidates = names + [str(repo / ".build-tools/llvm-mingw/bin" / n) for n in names]
    tools = ["bash", "grep", "dirname", *[p for p in candidates if shutil.which(p)]]
    computed = False

    def extract():
        nonlocal computed
        computed = True
        cp = run.run(cmd, cwd=repo, stage="lint:sacred_primitives")
        if cp.returncode:
            raise subprocess.CalledProcessError(cp.returncode, cmd, cp.stdout, cp.stderr)
        return {"stdout": cp.stdout, "stderr": cp.stderr}

    try:
        facts = BinaryFacts(
            repo,
            namespace="sacred-primitives",
            implementation=[script, Path(__file__)],
            tools=tools,
            options={"arch": arch},
        )
        result = facts.get(build / "ntoskrnl/ntoskrnl.exe", extract)
        if not computed:
            run.log(result["stdout"], header="sacred_primitives: matching binary analysis reused")
        return subprocess.CompletedProcess(cmd, 0, result["stdout"], result["stderr"])
    except subprocess.CalledProcessError as error:
        return subprocess.CompletedProcess(cmd, error.returncode, error.stdout, error.stderr)
    except (OSError, RuntimeError) as error:
        return subprocess.CompletedProcess(cmd, 1, "", f"SCANNER_ERROR: {error}")


def run_binary_lints(run: Run, repo: Path, arch: str, *, strict: bool) -> None:
    """Need a built binary (run after `build`).

    scan_ud2 + sacred primitives + scan_self_recursive + scan_flex_array_panic +
    scan_export_thunks + scan_xchg_ebx_deref + scan_unresolved_imports. All are
    POST-LINK detectors:
    they read the emitted code, because the defects they catch are invisible in
    the source (an ABI mismatch that LLVM turned into a `ud2`; a Rust `memcpy`
    that LLVM lowered into a call to itself; a bindgen `[T; 1]` whose bounds
    check panics on element 2 of a C flexible array; an export that became a
    jump into the DLL that forwards it back; inline asm whose pointer operand
    LLVM put in the very register the asm swaps out). In each case the Rust or C
    reads as correct and only the emitted code says otherwise.

    Two of the six are graded on ONE arch each, for reasons of encoding rather
    than of scope -- the defect class is arch-independent, but only one arch
    emits the evidence the reader needs. Those say so as a SKIP WITH A REASON on
    the others; a reader that cannot see is never silently a pass. Before you add
    a third, check that the other arches really cannot answer:
    scan_flex_array_panic was i386-only on exactly that reasoning, and the 64-bit
    arches turned out to load the same length as an immediate into a register --
    and to carry code the i386 binary does not contain at all.
    """
    py = sys.executable
    build_dir = repo / f"build-{arch}"
    ntos = build_dir / "ntoskrnl" / "ntoskrnl.exe"
    if not ntos.is_file():
        run.add(Stage("scan_ud2", status="skip", detail="no binary"))
        run.add(Stage("sacred_primitives", status="skip", detail="no binary"))
        run.add(Stage("scan_self_recursive", status="skip", detail="no binary"))
        run.add(Stage("scan_flex_array_panic", status="skip", detail="no binary"))
        run.add(Stage("scan_export_thunks", status="skip", detail="no binary"))
        run.add(Stage("scan_xchg_ebx_deref", status="skip", detail="no binary"))
        run.add(Stage("scan_unresolved_imports", status="skip", detail="no binary"))
        run.add(Stage("freeldr_size", status="skip", detail="no binary"))
        return
    # The BIOS loader runs in real-mode memory and bug-checks itself at boot when its PE
    # outgrows MAX_FREELDR_PE_SIZE. Restoring freeldr's debug prints did exactly that and
    # the first symptom was every amd64-bios boot hanging in the gate; read it post-link.
    # Only the arches that build freeldr_pe say anything; the others print SKIP and pass.
    fls = repo / "tools" / "check_freeldr_size.py"
    _lint_stage(
        run,
        repo,
        "freeldr_size",
        [py, str(fls), "--check", str(build_dir)] if fls.is_file() else ["MISSING"],
        strict=strict,
        by_exit_code=True,
    )
    # Binary lints are judged by exit code (by_exit_code=True), not baseline-diff:
    # their output is arch-specific so a single baseline can't match across arches
    # (scan_ud2 emits ud2 on x86 vs branch-to-self spins on arm64). The tools'
    # --check exit already encodes the correct per-arch verdict.
    ud2 = repo / "tools" / "scan_ud2.py"
    _lint_stage(
        run,
        repo,
        "scan_ud2",
        [py, str(ud2), "--check", str(build_dir)] if ud2.is_file() else ["MISSING"],
        strict=strict,
        by_exit_code=True,
    )
    sp = repo / "tools" / "verify_sacred_primitives.sh"
    _lint_stage(
        run,
        repo,
        "sacred_primitives",
        ["bash", str(sp), str(build_dir)] if sp.is_file() else ["MISSING"],
        strict=strict,
        by_exit_code=True,
        cp=_sacred_analysis(run, repo, arch, sp) if sp.is_file() else None,
    )
    # A function whose first instruction jumps to itself = a silent infinite hang.
    # Caught bootvid's self-recursive Rust memcpy/memset (stable-toolchain Axis B);
    # see tools/scan_self_recursive.py for the full story.
    selfrec = repo / "tools" / "scan_self_recursive.py"
    _lint_stage(
        run,
        repo,
        "scan_self_recursive",
        [py, str(selfrec), "--check", str(build_dir)] if selfrec.is_file() else ["MISSING"],
        strict=strict,
        by_exit_code=True,
    )
    # A bounds check against a length of 1 on a bindgen flexible array: the C
    # over-allocates past a trailing `T x[1]`, bindgen believes the 1, and every
    # element after the first panics. Graded on EVERY arch: the length is a push on
    # i386 and an immediate into a register elsewhere, and the 64-bit corpus is not the
    # i386 one (other crates, `#[cfg(target_arch)]` bodies) -- reading i386 alone is how
    # a whole crate's worth of sites went unseen. Sites graded as genuine 1-element
    # arrays live in tools/flex_array_allowlist.txt, scoped per arch; a stale entry
    # fails this stage too, because it means the site was fixed or the reader went blind.
    flexarr = repo / "tools" / "scan_flex_array_panic.py"
    _lint_stage(
        run,
        repo,
        "scan_flex_array_panic",
        [py, str(flexarr), "--lint", "--arch", arch] if flexarr.is_file() else ["MISSING"],
        strict=strict,
        by_exit_code=True,
    )
    # An export that is a jump to another DLL's same-named import: a local body lost the
    # link to an import library. ole32's GetErrorInfo/SetErrorInfo/CreateErrorInfo went
    # to oleaut32, which forwards them back: an endless loop for every caller. amd64 only
    # (the scan reads llvm-objdump's IAT-name annotation); the defect is link-order, not
    # arch, so one graded arch catches the class.
    if arch == "amd64":
        sxt = repo / "tools" / "scan_export_thunks.py"
        _lint_stage(
            run,
            repo,
            "scan_export_thunks",
            [py, str(sxt), "--check", str(build_dir)] if sxt.is_file() else ["MISSING"],
            strict=strict,
            by_exit_code=True,
        )
    else:
        run.add(Stage("scan_export_thunks", status="skip", detail="amd64-only scan"))
    # i386 inline asm that swaps ebx out for cmpxchg8b and then dereferences ebx: LLVM
    # put the pointer operand IN ebx. slist's try_store shipped that in ntdll.dll and
    # ks.sys, and InterlockedPushEntrySList spun forever. i386 only -- ebx is allocatable
    # there and nowhere else; see tools/scan_xchg_ebx_deref.py.
    if arch == "i386":
        sxe = repo / "tools" / "scan_xchg_ebx_deref.py"
        _lint_stage(
            run,
            repo,
            "scan_xchg_ebx_deref",
            [py, str(sxe), "--check", str(build_dir)] if sxe.is_file() else ["MISSING"],
            strict=strict,
            by_exit_code=True,
        )
    else:
        run.add(Stage("scan_xchg_ebx_deref", status="skip", detail="i386-only scan"))
    # A shipped image importing a name its exporter does not export. The loader drops a
    # boot driver that does this and keeps booting, so smoke stays green unless the boot
    # needed that driver: pcix.sys's raw-dylib RtlAssert imported `RtlAssert@16` on i386
    # and was never loaded. Graded on every arch -- decoration is i386's, but a missing
    # export is not. See tools/scan_unresolved_imports.py.
    sui = repo / "tools" / "scan_unresolved_imports.py"
    _lint_stage(
        run,
        repo,
        "scan_unresolved_imports",
        [py, str(sui), "--check", str(build_dir)] if sui.is_file() else ["MISSING"],
        strict=strict,
        by_exit_code=True,
    )
