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
from pathlib import Path

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
    r"|^\s*Found \d+ unprotected"  # lint_kernel header tally
    r"|^\s*Total: \d+ violations)"  # lint_kernel footer tally (mixed-case)
)


def _finding_lines(out: str) -> set[str]:
    return {ln.rstrip() for ln in out.splitlines() if ln.strip() and not _SUMMARY_LINE.search(ln)}


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
) -> Stage:
    """Run one lint; classify NEW-vs-baseline. Advisory unless strict.

    by_exit_code: judge purely on the tool's --check exit status instead of
    line-baseline-diff. Required for BINARY lints (scan_ud2, sacred_primitives):
    their output is ARCH-SPECIFIC (i386 ud2 lines ≠ arm64 branch-to-self spins ≠
    amd64), so a single arch-agnostic baseline file can never match across arches
    — every line reads as "new". These tools already encode the correct per-arch
    pass/fail in their exit code (their own EXPECTED-site classification), so the
    exit code is the right signal."""
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
    cp = run.run(cmd, cwd=cwd or repo, stage=f"lint:{name}")
    if by_exit_code:
        text = _rel((cp.stdout or "") + "\n" + (cp.stderr or ""), repo)
        if cp.returncode == 0:
            return run.add(Stage(name, exit=0, status="ok"))
        st = "fail" if strict else "warn"
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
    new = cur - base
    # The gate is NEW findings vs baseline — NOT the exit code. Tools like
    # shellcheck/verify_* exit nonzero whenever ANY finding exists, but those
    # are baselined/accepted; only new ones matter.
    if new:
        st = "fail" if strict else "warn"
        return run.add(
            Stage(
                name,
                exit=cp.returncode,
                status=st,
                detail=f"{len(new)} new",
                salient=sorted(new)[:3],
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
    sbc = repo / "tools" / "scan_bugcheck_codes.py"
    specs.append(
        (
            "scan_bugcheck_codes",
            [py, str(sbc), "--lint"] if sbc.is_file() else ["MISSING"],
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
    # Duplicate #[no_mangle] symbol ratchet: symbols defined in >=2 crates that
    # link into the kernel PE are masked by --allow-multiple-definition (which is
    # load-bearing — the multi-staticlib runtime dups are irreducible). We can't
    # drop the flag, so we FREEZE its masked set and gate NEW cross-crate
    # collisions — the IsEqualGUID-class near-miss where a translation re-defines
    # a symbol another crate already owns. Baseline = accepted deliberate cohab
    # (today: arbiter's amd64 KeGetCurrentIrql). See the tool docstring +
    # docs/engineering/allow-multiple-definition-ratchet.md.
    sds = repo / "tools" / "scan_duplicate_symbols.py"
    specs.append(
        (
            "scan_duplicate_symbols",
            [py, str(sds), "--lint"] if sds.is_file() else ["MISSING"],
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

    return specs


def run_source_lints(run: Run, repo: Path, *, strict: bool) -> None:
    for name, cmd, src in source_lint_specs(repo):
        _lint_stage(run, repo, name, cmd, strict=strict, findings_from=src)


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
    for name, cmd, src in source_lint_specs(repo):
        if only is not None and name not in only:
            continue
        if cmd[0] == "MISSING":
            continue
        try:
            cp = subprocess.run(cmd, cwd=repo, capture_output=True, text=True, timeout=600)
        except (OSError, subprocess.SubprocessError):
            continue
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


# --- cargo-driven link PoCs (off-default-path → bit-rot risk) ---------------
# Each: (label, crate dir under sdk/lib/rust, cargo --bin, supported arches,
#        plausible-output floor in bytes). The cargo-driven links replace the
#        cmake link long-term; they live OFF the default build path, so nothing
#        else notices when their machinery (header-free C shims, pre-expanded
#        .s, self-hosted spec2def, the static-lib list, the reactos-link engine
#        itself, or the checked-in .res) drifts out of sync with the tree.
_CARGO_LINK_POCS = [
    ("ntoskrnl", "ntoskrnl-cargo", "ntoskrnl", ("i386", "amd64", "arm64"), 1_000_000),
    ("ntdll", "ntdll-cargo", "ntdll", ("i386", "amd64", "arm64"), 1_000_000),
    ("ntdll_vista", "ntdll_vista-cargo", "ntdll_vista", ("i386", "amd64", "arm64"), 200_000),
]
# amd64/arm64 build the BUILTIN triple on stable (no -Z build-std); i386 keeps the
# custom softfloat kernel json. gcc.cmake:add_cargo_module is the source of truth.
_ARCH_TARGET = {
    "i386": "i686-kernel",
    "amd64": "x86_64-pc-windows-gnu",
    "arm64": "aarch64-pc-windows-gnullvm",
}


def _one_cargo_link(
    run: Run, repo: Path, rust_ws: Path, label: str, crate: str, binname: str, arch: str, floor: int
) -> None:
    """Re-run one cargo-driven link and assert it still produces a valid PE,
    WITHOUT booting. Skips cleanly when prerequisites are absent."""
    stage = f"cargo_link:{label}:{arch}"
    target = _ARCH_TARGET[arch]
    if not (rust_ws / crate / f"{target}.json").is_file():
        run.add(Stage(stage, status="skip", detail="PoC absent"))
        return
    # The engine reuses cmake-built sibling .a (librtl is universal); a missing
    # build-<arch> means the inputs the link consumes were never produced.
    if not (repo / f"build-{arch}" / "sdk" / "lib" / "rtl" / "librtl.a").is_file():
        run.add(Stage(stage, status="skip", detail=f"build-{arch} not built"))
        return
    if shutil.which("rustup") is None:
        run.add(Stage(stage, status="skip", detail="no rustup (need nightly+rust-src)"))
        return

    # Mirror gcc.cmake:add_cargo_module EXACTLY — this stage exists to prove the
    # cmake→cargo link hasn't bit-rotted, so a different invocation would prove
    # nothing. i386: pinned nightly + `-Z build-std` + the softfloat kernel json.
    # amd64/arm64: pinned stable + the builtin triple, with the two `-C` flags that
    # restore what the retired json carried (see gcc.cmake for the why).
    env = None
    if target.endswith("-kernel"):  # i386 — custom json, needs nightly build-std
        nightly = (repo / ".rust-nightly").read_text().strip()
        argv = [
            "cargo",
            f"+{nightly}",
            "build",
            "--release",
            "--target",
            f"{crate}/{target}.json",
            "-Z",
            "build-std=core,alloc,compiler_builtins",
            "-Z",
            "build-std-features=compiler-builtins-mem",
        ]
    else:  # amd64 / arm64 — builtin triple on stable
        argv = ["cargo", "build", "--release", "--target", target]
        # link-self-contained is amd64-only: gnullvm rejects the flag outright.
        rustflags = "-C force-frame-pointers=yes"
        if target.startswith("x86_64"):
            rustflags += " -C link-self-contained=no"
        env = dict(os.environ, RUSTFLAGS=rustflags)

    cp = run.run(
        [*argv, "-p", crate, "--bin", binname],
        cwd=rust_ws,
        stage=stage,
        timeout=900,
        env=env,
    )
    out = rust_ws / "target" / target / "release" / f"{binname}.exe"
    if cp.returncode != 0:
        tail = [ln for ln in (cp.stderr or "").splitlines() if ln.strip()][-3:]
        run.add(
            Stage(
                stage,
                exit=cp.returncode,
                status="fail",
                detail="link FAILED — cmake→cargo PoC bit-rotted",
                salient=tail,
            )
        )
        return
    if not out.is_file():
        run.add(Stage(stage, status="fail", detail="reported success but no output PE"))
        return
    size = out.stat().st_size
    if size < floor:
        run.add(
            Stage(stage, status="fail", detail=f"{binname}.exe implausibly small ({size} bytes)")
        )
        return
    with open(out, "rb") as f:
        if f.read(2) != b"MZ":
            run.add(Stage(stage, status="fail", detail="output not a PE (no MZ header)"))
            return
    run.add(Stage(stage, exit=0, status="ok", detail=f"{binname}.exe {size // 1024}K"))


def run_cargo_link_guard(run: Run, repo: Path, arch: str | None = None) -> None:
    """Guard every cargo-driven link PoC against silent bit-rot.

    Re-runs the cargo-driven links (`ntoskrnl-cargo`, `ntdll-cargo`,
    `ntdll_vista-cargo` via the shared `reactos-link` engine) and asserts each
    still produces a valid PE. These are the long-term replacement for the
    cmake-driven links; because they live OFF the default build path, the
    pre-commit/merge gate (`check --strict`) is exactly where to catch them
    rotting. Defaults to i386 (canonical, cheapest, the only arch all three
    PoCs share); pass an arch to guard ntdll/ntdll_vista on amd64/arm64.

    This is the LINK half of the gate (PE-valid, fast, no boot). Its runtime
    complement is `vibeos-cli cargo-swap` (tests/qemu/cli/cargoswap.py): that
    boot-swaps the cargo-built ntdll DLLs into the livecd and boots them via
    `vibeos-cli smoke`, catching a link that relinks clean but boots broken.
    Kept separate because cargo-swap boots QEMU (minutes) — too slow for the
    hot `check` loop; run it before trusting a cargo-drive handoff.

    Each link self-skips when its prerequisites (the crate's target json, a
    built `build-<arch>`, or nightly+rust-src) are absent, so this never blocks
    a tree without the PoCs. Gated to `--strict` (not every `build`) because
    each relink re-compiles a Rust staticlib into its own target dir (~20s) and
    would tax the hot loop."""
    rust_ws = repo / "sdk" / "lib" / "rust"
    arch = arch or "i386"
    for label, crate, binname, arches, floor in _CARGO_LINK_POCS:
        if arch in arches:
            _one_cargo_link(run, repo, rust_ws, label, crate, binname, arch, floor)


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


def run_binary_lints(run: Run, repo: Path, arch: str, *, strict: bool) -> None:
    """Need a built binary (run after `build`).

    scan_ud2 + sacred primitives + scan_self_recursive. All three are POST-LINK
    miscompile detectors: they read the emitted code, because the defects they
    catch are invisible in the source (an ABI mismatch that LLVM turned into a
    `ud2`; a Rust `memcpy` that LLVM lowered into a call to itself).
    """
    py = sys.executable
    build_dir = repo / f"build-{arch}"
    ntos = build_dir / "ntoskrnl" / "ntoskrnl.exe"
    if not ntos.is_file():
        run.add(Stage("scan_ud2", status="skip", detail="no binary"))
        run.add(Stage("sacred_primitives", status="skip", detail="no binary"))
        run.add(Stage("scan_self_recursive", status="skip", detail="no binary"))
        return
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
