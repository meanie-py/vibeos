"""Controls for verify_ucrtbase_cohab.py — the gate that keeps msvcrt's CRT out of ucrtbase.

The defect this tool exists for leaves NO trace anywhere: ucrtbase.dll links with
`--allow-multiple-definition`, so an archive carrying msvcrt's whole CRT floor silently
replaces 856 genuine Microsoft UCRT bodies and still builds, boots, and looks right. A gate
against an invisible defect is worth exactly as much as the evidence that it fires, which is
what this file is.

The positive control is the real accident, not a synthetic one: `crt-shared-cargo` growing a
dependency on `msvcrt-crt-rust`. That is one line in a Cargo.toml, it is the obvious way to
"just reuse the msvcrt cohab for ucrtbase", and it is what the whole crate split exists to
prevent.
"""

from __future__ import annotations

import importlib.util
import sys
from pathlib import Path

import pytest

REPO = Path(__file__).resolve().parent.parent
_spec = importlib.util.spec_from_file_location(
    "verify_ucrtbase_cohab", REPO / "tools" / "verify_ucrtbase_cohab.py"
)
assert _spec and _spec.loader
vuc = importlib.util.module_from_spec(_spec)
sys.modules["verify_ucrtbase_cohab"] = vuc
_spec.loader.exec_module(vuc)


def _crate(ws: Path, name: str, deps: str = "", extra: str = "") -> None:
    """Write a minimal manifest, which is all the structural check reads."""
    d = ws / name
    d.mkdir(parents=True, exist_ok=True)
    (d / "Cargo.toml").write_text(
        f'[package]\nname = "{name}"\nversion = "0.1.0"\nedition = "2021"\n'
        f"{extra}\n[dependencies]\n{deps}\n"
    )


# ── positive controls: the gate MUST fire ────────────────────────────────────────────────


def test_direct_dependency_on_msvcrt_is_caught(tmp_path: Path):
    """The one-line accident: ucrtbase's archive depends on msvcrt's cohab crate."""
    _crate(tmp_path, "crt-shared-cargo", 'msvcrt-crt-rust = { path = "../msvcrt-crt-rust" }\n')
    _crate(tmp_path, "msvcrt-crt-rust")
    findings = vuc.check_structure(tmp_path)
    assert findings, "the forbidden edge was not reported"
    assert "msvcrt-crt-rust" in findings[0]


def test_transitive_dependency_is_caught(tmp_path: Path):
    """Depth is the point of walking the graph rather than reading one manifest.

    A future crate that innocently wants `crt-shared-rust` AND some msvcrt helper would
    reintroduce the whole floor without naming it in ucrtbase's own manifest.
    """
    _crate(tmp_path, "crt-shared-cargo", 'helper = { path = "../helper" }\n')
    _crate(tmp_path, "helper", 'msvcrt-crt-rust = { path = "../msvcrt-crt-rust" }\n')
    _crate(tmp_path, "msvcrt-crt-rust")
    findings = vuc.check_structure(tmp_path)
    assert findings, "a transitive edge slipped through"
    # the chain is reported, not just the endpoint, so the fix is obvious from the message
    assert "crt-shared-cargo -> helper -> msvcrt-crt-rust" in findings[0]


def test_missing_archive_crate_is_caught(tmp_path: Path):
    """Deleting the crate must not read as 'nothing to check, all clear'."""
    findings = vuc.check_structure(tmp_path)
    assert findings and "MISSING" in findings[0]


# ── negative controls: the gate must STAY QUIET ──────────────────────────────────────────


def test_the_real_layout_is_clean(tmp_path: Path):
    """The shipping workspace. If this ever fires, the split has been undone."""
    assert vuc.check_structure() == []


def test_shared_crate_dependency_is_allowed(tmp_path: Path):
    """The CORRECT wiring must not be flagged, or the gate is just noise."""
    _crate(tmp_path, "crt-shared-cargo", 'crt-shared-rust = { path = "../crt-shared-rust" }\n')
    _crate(tmp_path, "crt-shared-rust", 'msvcrt-sys = { path = "../msvcrt-sys" }\n')
    _crate(tmp_path, "msvcrt-sys")
    assert vuc.check_structure(tmp_path) == []


def test_feature_naming_a_crate_is_not_a_dependency(tmp_path: Path):
    """`[features]` may NAME a crate without depending on it, and the parser must know.

    msvcrt-crt-rust really does write `cpp = ["crt-shared-rust/cpp"]`, so a parser that
    scanned the whole file for crate names would report the reverse edge as a violation and
    the gate would be red on a correct tree from day one.
    """
    _crate(
        tmp_path,
        "crt-shared-cargo",
        'crt-shared-rust = { path = "../crt-shared-rust" }\n',
        extra='[features]\nwith-msvcrt = ["msvcrt-crt-rust/cpp"]\n',
    )
    _crate(tmp_path, "crt-shared-rust")
    assert vuc.check_structure(tmp_path) == []


@pytest.mark.parametrize("nm_name", ["llvm-nm", "nm"])
def test_unreadable_nm_does_not_silently_pass(tmp_path: Path, nm_name: str):
    """A tool that cannot run its own check must SAY so, not return clean.

    This is the bug the tool shipped with: a present-but-non-executable `llvm-nm` raised
    PermissionError straight through a `except FileNotFoundError`. Fixed to `except OSError`,
    but the property worth pinning is the one after that -- when no nm works at all,
    `_nm_defined` returns None so the caller can emit a finding.
    """
    missing = tmp_path / "not-an-archive.a"
    missing.write_text("this is not an archive")
    assert vuc._nm_defined(missing) in (None, set())


# ── the ARCHIVE check's model of the link line ───────────────────────────────────────────


def _ucrtbase_link_libraries() -> list[str]:
    """The names inside ucrtbase's `target_link_libraries(ucrtbase ...)` block."""
    import re

    text = (REPO / "dll" / "win32" / "ucrtbase" / "CMakeLists.txt").read_text()
    m = re.search(r"target_link_libraries\(ucrtbase\b(.*?)\)", text, re.S)
    assert m, "ucrtbase's link line moved or changed shape"
    names = []
    for tok in m.group(1).split():
        # skip linker flags and the ${...} holding the Rust archive path
        if tok.startswith(("-", "$")):
            continue
        names.append(tok)
    return names


def test_the_authority_set_is_the_whole_link_line_except_ucrt():
    """The archive check is only correct if its model IS the link line.

    This is the assertion that actually rotted once: the tool modelled the authority as
    `msvcrt_shared` alone, and the ucrtbase math flip -- a correct change, every symbol of it
    a crtmath body we own and had graded -- came back as 60 findings. A gate that is red on a
    correct change is one somebody switches off, so the set has to track the CMake rather than
    be remembered. `ucrt` is the deliberate exclusion: it is the vendored Microsoft library,
    and displacing one of ITS bodies is the entire defect this file exists to prevent.
    """
    linked = _ucrtbase_link_libraries()
    assert "ucrt" in linked, "ucrt vanished from the link line; the exclusion below is stale"
    assert set(linked) - {"ucrt"} == set(vuc.OURS_ON_LINK_LINE), (
        "ucrtbase's link line and OURS_ON_LINK_LINE disagree -- a library added there without "
        "a matching entry makes every symbol it defines look like ours shadowing genuine UCRT"
    )


def test_export_set_honours_the_specs_arch_filter():
    r"""An export exists only on the arches its spec line names, and (b) depends on that.

    This one shipped and bounced a correct lane. The parse skipped `-arch=` with
    `(?:-\S+\s+)*`, so the export set was arch-blind, and the 11 float bodies ucrtbase
    declares `-arch=!i386` -- `tanf`, `sqrtf`, `_logbf` and friends -- were reported as i386
    shadowing. On i386 ucrtbase has no such export, so nothing there can be displaced and
    every finding was noise. Pin both directions: absent where the spec excludes it, present
    where it does not, or the assertion silently stops meaning anything.
    """
    spec = vuc.REPO / "dll" / "win32" / "ucrtbase" / "ucrtbase.spec"
    if not spec.is_file():
        pytest.skip("ucrtbase.spec not present")
    floats = {"acosf", "asinf", "atan2f", "atanf", "coshf", "modff", "sinhf", "sqrtf", "tanf"}
    i386 = vuc._spec_exports(spec, "i386")
    assert not (floats & i386), (
        f"-arch=!i386 exports leaked into the i386 set: {sorted(floats & i386)}"
    )
    for arch in ("amd64", "arm64"):
        assert floats <= vuc._spec_exports(spec, arch), (
            f"{arch} must still see them, or the filter is dropping real exports"
        )
    # And the filter must not gut the set wholesale -- a parser that returns nothing also
    # passes the assertion above, and would make the whole shadowing check vacuous.
    assert len(i386) > 1000, f"i386 export set implausibly small ({len(i386)})"


# A build dir is only an ORACLE once its build finished. `vibeos-cli build` writes this
# at the very end (cli/provenance.py STAMP_NAME), so its presence is the difference
# between "cmake configured a directory" and "ninja produced the objects in it".
_BUILD_STAMP = ".vibeos-build-stamp.json"


def test_every_modelled_target_has_a_real_object_directory():
    """A path typo silently contributes ZERO symbols, which reads as mass shadowing.

    The check treats an authority target with no objects as a hard failure rather than as
    'nothing to add', but only a real build proves the *path* is right -- so assert it against
    whichever build dirs this worktree happens to have, and skip if it has none.

    "Has a build dir" is NOT "has a built build dir". `vibeos-cli gate` runs the tooling
    stage CONCURRENTLY with its three builds, so on the first build of an arch in a fresh
    worktree this raced: cmake creates `build-i386/` in the first seconds, ninja creates
    `.../CMakeFiles/msvcrt_shared.dir` about forty later, and a test that only checked for
    the build dir judged the gap. It failed a green gate on 2026-09-13 that way -- every
    build, lint, smoke and conformance lane passed and the object dir it called missing was
    on disk, timestamped inside that same gate's run. Gate the skip on the completion stamp
    so the oracle is a FINISHED build, not a directory that exists.
    """
    builds = [REPO / f"build-{a}" for a in ("i386", "amd64", "arm64")]
    builds = [b for b in builds if (b / _BUILD_STAMP).is_file()]
    if not builds:
        pytest.skip("no finished build dir in this worktree")
    # Report EVERY missing pair, not the first. A bare `assert` inside the nested loop
    # fails on one (arch, target) at a time, so a model change that predates all three
    # build dirs is discovered three times: research rebuilt i386, hit amd64, rebuilt
    # that, hit arm64 -- three warm rebuilds and three queue round trips for one cause.
    # `wine` entering OURS_ON_LINK_LINE in c474d932eb7 did exactly that.
    missing = [
        (build.name, target, build / obj_dir)
        for build in builds
        for target, obj_dir in vuc.OURS_ON_LINK_LINE.items()
        if not (build / obj_dir).is_dir()
    ]
    if missing:
        arches = sorted({name.removeprefix("build-") for name, _, _ in missing})
        raise AssertionError(
            "modelled target(s) have no object directory:\n  "
            + "\n  ".join(f"{name}: {target} -> {path}" for name, target, path in missing)
            + "\n\nA COMPLETION STAMP PROVES A BUILD FINISHED, NOT THAT IT BUILT "
            "EVERYTHING: a build dir predating a new entry in OURS_ON_LINK_LINE is stamped "
            "and still lacks it. If this names a target that was recently added to the "
            "model, the tree is stale rather than the model wrong.\n"
            + "fix: "
            + " && ".join(f"./vibeos-cli build --arch {a}" for a in arches)
        )


# ── the REACHABILITY check ──────────────────────────────────────────────────────────────


def _cargo_crate(ws, name: str, deps: str, lib_rs: str) -> None:
    _crate(ws, name, deps)
    src = ws / name / "src"
    src.mkdir(parents=True, exist_ok=True)
    (src / "lib.rs").write_text(lib_rs)


def test_unreferenced_dependency_is_caught(tmp_path: Path):
    """The real accident: a manifest edit alone, and the flip silently does not happen.

    Adding `cnt-rust = { features = ["cabi-math"] }` to crt-shared-cargo without an
    `extern crate cnt_rust;` built green, gated green, and put 268 symbols in the archive of
    which not one was `tan`. Nothing in the tree said so, because every other check asks
    whether the archive defines too MUCH.
    """
    _cargo_crate(
        tmp_path,
        "crt-shared-cargo",
        'crt-shared-rust = { path = "../crt-shared-rust" }\ncnt-rust = { path = "../cnt-rust" }\n',
        "extern crate crt_shared_rust;\n",
    )
    findings = vuc.check_extern_crates(tmp_path)
    assert findings and "cnt-rust" in findings[0]
    assert "extern crate cnt_rust;" in findings[0], "the fix must be in the message"


def test_referenced_dependency_is_allowed(tmp_path: Path):
    """The hyphen/underscore spelling difference must not read as a violation."""
    _cargo_crate(
        tmp_path,
        "crt-shared-cargo",
        'crt-shared-rust = { path = "../crt-shared-rust" }\ncnt-rust = { path = "../cnt-rust" }\n',
        "extern crate crt_shared_rust;\nextern crate cnt_rust;\n",
    )
    assert vuc.check_extern_crates(tmp_path) == []


def test_panic_common_needs_no_extern_crate(tmp_path: Path):
    """It is reached through `#[panic_handler]`, which no grep for `extern crate` can see."""
    _cargo_crate(
        tmp_path,
        "crt-shared-cargo",
        'panic-common = { path = "../panic-common" }\n',
        "#[panic_handler]\nfn panic(_: &core::panic::PanicInfo) -> ! { loop {} }\n",
    )
    assert vuc.check_extern_crates(tmp_path) == []


def test_the_real_crate_references_everything_it_depends_on(tmp_path: Path):
    """The shipping crate. Red here means a cohab is wired up but not actually linked."""
    assert vuc.check_extern_crates() == []


# ---------------------------------------------------------------------------
# A stale archive is not a weaker verdict — it is a verdict about another tree
# ---------------------------------------------------------------------------
#
# `vibeos-cli build` is warm by default and will leave a cargo target untouched
# (its own `warm-build` warning says exactly that), so an archive can sit on
# disk older than the Rust that produced it. This check is usually run right
# after a fix, which makes a stale read report the symbol you just removed --
# indistinguishable from the fix not working. That happened twice before this
# guard existed, and on two of three arches the archive was silently stale.


def test_stale_archive_is_detected(tmp_path: Path, monkeypatch):
    crate = tmp_path / "sdk" / "lib" / "rust" / "cnt-rust" / "src"
    crate.mkdir(parents=True)
    archive = tmp_path / "libcrt_shared_cargo.a"
    archive.write_bytes(b"")
    src = crate / "lib.rs"
    src.write_text("// newer than the archive\n")
    import os

    os.utime(archive, (1_000_000, 1_000_000))
    os.utime(src, (2_000_000, 2_000_000))

    monkeypatch.setattr(vuc, "REPO", tmp_path)
    assert vuc._stale_against_sources(archive) == "sdk/lib/rust/cnt-rust/src/lib.rs"


def test_fresh_archive_is_not_flagged(tmp_path: Path, monkeypatch):
    crate = tmp_path / "sdk" / "lib" / "rust" / "cnt-rust" / "src"
    crate.mkdir(parents=True)
    archive = tmp_path / "libcrt_shared_cargo.a"
    archive.write_bytes(b"")
    src = crate / "lib.rs"
    src.write_text("// older than the archive\n")
    import os

    os.utime(src, (1_000_000, 1_000_000))
    os.utime(archive, (2_000_000, 2_000_000))

    monkeypatch.setattr(vuc, "REPO", tmp_path)
    assert vuc._stale_against_sources(archive) is None


def test_unrelated_crate_does_not_trigger_staleness(tmp_path: Path, monkeypatch):
    """The watch list is the archive's own dependency closure, deliberately.

    A staleness warning that fires whenever any lane touches any .rs anywhere in
    sdk/lib/rust would be noise, and noise is what gets a check ignored.
    """
    other = tmp_path / "sdk" / "lib" / "rust" / "shell32-rust" / "src"
    other.mkdir(parents=True)
    archive = tmp_path / "libcrt_shared_cargo.a"
    archive.write_bytes(b"")
    src = other / "lib.rs"
    src.write_text("// a different lane's crate\n")
    import os

    os.utime(archive, (1_000_000, 1_000_000))
    os.utime(src, (2_000_000, 2_000_000))

    monkeypatch.setattr(vuc, "REPO", tmp_path)
    assert "shell32-rust" not in vuc._archive_source_crates()
    assert vuc._stale_against_sources(archive) is None


def test_a_stale_archive_is_a_NOTE_not_a_finding(tmp_path: Path, monkeypatch, capsys):
    """Staleness must never reject a lane — it is not the lane's fault.

    mtime is the only staleness signal available here and it is NOT sound: `git
    merge` rewrites a source file's mtime without changing its content, while
    cargo correctly declines to relink an archive whose inputs are unchanged. The
    conductor merges every queued lane at the start of a drain, so in the
    integration tree those two facts coincide on EVERY batch.

    Emitted as a finding (which is how this first shipped) it rejected a lane for
    a merge that lane did not make, with `libcrt_shared_cargo.a is OLDER than
    cabi/math.rs` on all three arches — and had it landed it would have done that
    to every lane, because this check runs in the SOURCE-lint phase where build
    state is whatever the previous batch happened to leave behind.
    """
    import os

    build = tmp_path / "build-amd64" / "dll" / "win32" / "ucrtbase"
    build.mkdir(parents=True)
    archive = build / "libcrt_shared_cargo.a"
    archive.write_bytes(b"")

    src = tmp_path / "sdk" / "lib" / "rust" / "cnt-rust" / "src"
    src.mkdir(parents=True)
    (src / "lib.rs").write_text("// merged: newer mtime, identical content\n")

    os.utime(archive, (1_000_000, 1_000_000))
    os.utime(src / "lib.rs", (2_000_000, 2_000_000))

    monkeypatch.setattr(vuc, "REPO", tmp_path)
    findings = vuc.check_archive()

    # The staleness branch must short-circuit BEFORE the missing-objects branch,
    # or this arch would be blamed for absent .obj files instead.
    assert findings == [], f"staleness must not produce findings, got: {findings}"
    assert "older than" in capsys.readouterr().err, "but it must still SAY so"


def _git(repo: Path, *args: str, when: str | None = None) -> None:
    import subprocess

    env = dict(os.environ, GIT_CONFIG_GLOBAL="/dev/null", GIT_CONFIG_SYSTEM="/dev/null")
    if when:
        env |= {"GIT_AUTHOR_DATE": when, "GIT_COMMITTER_DATE": when}
    subprocess.run(["git", "-C", str(repo), *args], check=True, capture_output=True, env=env)


import os  # noqa: E402  (used by _git above and the tests below)


def test_a_merge_touching_mtime_is_NOT_stale(tmp_path: Path, monkeypatch):
    """The case that rejected a lane: mtime moved, content did not.

    `git merge` and `git checkout` rewrite mtimes without changing bytes, and the
    conductor merges every queued lane at the start of every drain. A pure-mtime
    check therefore calls a perfectly current archive stale on EVERY batch — which
    is exactly what happened, three arches at once. Git knows the file's content is
    unchanged since its last commit, so this must come back clean.
    """
    _git(tmp_path, "init", "-q", "-b", "main")
    _git(tmp_path, "config", "user.email", "t@t")
    _git(tmp_path, "config", "user.name", "t")
    src = tmp_path / "sdk" / "lib" / "rust" / "cnt-rust" / "src"
    src.mkdir(parents=True)
    (src / "lib.rs").write_text("// committed long ago\n")
    _git(tmp_path, "add", "-A")
    _git(tmp_path, "commit", "-qm", "seed", when="2001-01-01T00:00:00 +0000")

    archive = tmp_path / "libcrt_shared_cargo.a"
    archive.write_bytes(b"")
    os.utime(archive, (1_500_000_000, 1_500_000_000))  # built after that commit
    os.utime(src / "lib.rs", (2_000_000_000, 2_000_000_000))  # "merged" just now

    monkeypatch.setattr(vuc, "REPO", tmp_path)
    assert vuc._stale_against_sources(archive) is None


def test_a_real_local_edit_IS_stale(tmp_path: Path, monkeypatch):
    """The other half: a DIRTY file's mtime is the content clock, and must win.

    Without this the git-based check would wave through the case the guard exists
    for — you edited cabi/math.rs, did not rebuild, and the archive predates it.
    """
    _git(tmp_path, "init", "-q", "-b", "main")
    _git(tmp_path, "config", "user.email", "t@t")
    _git(tmp_path, "config", "user.name", "t")
    src = tmp_path / "sdk" / "lib" / "rust" / "cnt-rust" / "src"
    src.mkdir(parents=True)
    (src / "lib.rs").write_text("// committed long ago\n")
    _git(tmp_path, "add", "-A")
    _git(tmp_path, "commit", "-qm", "seed", when="2001-01-01T00:00:00 +0000")

    archive = tmp_path / "libcrt_shared_cargo.a"
    archive.write_bytes(b"")
    os.utime(archive, (1_500_000_000, 1_500_000_000))

    (src / "lib.rs").write_text("// edited, NOT rebuilt\n")  # now dirty
    os.utime(src / "lib.rs", (2_000_000_000, 2_000_000_000))

    monkeypatch.setattr(vuc, "REPO", tmp_path)
    assert vuc._stale_against_sources(archive) == "sdk/lib/rust/cnt-rust/src/lib.rs"


def test_the_authority_includes_ucrtbases_own_C_not_just_its_libraries():
    """The subset assertion is against OUR C — and ucrtbase's own TUs are ours.

    OURS_ON_LINK_LINE must stay exactly the link LIBRARIES (the test above pins it
    against the CMake). But ucrtbase.dll also gets symbols from its own 132 lines —
    printf.c and stubs.c — and six of those are EXPORTED: the intrinsic placeholders
    spec2def cannot stub. Left out of the authority, a Rust twin of any of them reads
    as displacing a genuine Microsoft UCRT body when it would be displacing ReactOS's
    own. That is the defect the link-line test's docstring records (a correct ucrtbase
    math flip coming back as 60 findings), one target over.
    """
    assert set(vuc.OURS_C_AUTHORITY) == set(vuc.OURS_ON_LINK_LINE) | set(vuc.OURS_OWN_TUS)
    assert set(vuc.OURS_ON_LINK_LINE) & set(vuc.OURS_OWN_TUS) == set(), "the two must be disjoint"
    # The symbols that motivated this. If stubs.c stops defining them the entry is
    # stale, and a stale authority entry is exactly what this file must not carry.
    stubs = REPO / "dll" / "win32" / "ucrtbase" / "stubs.c"
    text = stubs.read_text(errors="replace")
    for sym in ("fma", "fmaf", "log2", "log2f", "lrint", "lrintf"):
        assert f"{sym}(" in text, f"{sym} no longer in ucrtbase/stubs.c — re-derive OURS_OWN_TUS"


# ── the burn-down ratchet ────────────────────────────────────────────────────


def test_the_recorded_ceiling_matches_what_the_tree_actually_measures():
    """The ratchet is only meaningful if its baseline is real.

    A ceiling recorded once and never re-measured decays into a number nobody
    trusts, which is worse than no number: it still passes, so nobody looks.
    This asserts the recorded value is the value a build actually produces —
    and it is allowed to be HIGHER (work landed since it was recorded), never
    lower, which is the ratchet's own rule applied to itself.
    """
    import yaml

    doc = yaml.safe_load(vuc.TAKEOVER.read_text(encoding="utf-8"))
    want = doc.get("burndown") or {}
    assert want, "tools/ucrt-takeover.yaml lost its burndown ceilings"

    measured_any = False
    for arch, ceiling in want.items():
        if not (vuc.REPO / f"build-{arch}").is_dir():
            continue
        # A build older than its sources measures the PREVIOUS commit. The gate runs this
        # concurrently with its own builds, so that is the normal case there.
        if vuc.burndown_stale(arch):
            continue
        from ucrt_burndown import classify

        buckets, _ = classify(arch)
        have = len(buckets["MICROSOFT"])
        measured_any = True
        assert have <= ceiling, (
            f"{arch}: {have} Microsoft-backed exports exceeds the recorded ceiling "
            f"{ceiling} — the ratchet has been breached"
        )
    if not measured_any:
        import pytest

        pytest.skip("no build tree in this worktree; the ratchet needs objects")


def test_the_ratchet_fails_on_a_regression_and_only_notes_an_improvement(tmp_path: Path):
    """Both directions, because the asymmetry is the design.

    Failing on an improvement would bounce lanes for making the tree better —
    and improvements arrive from OTHER lanes routinely, since any Rust body
    that happens to answer a ucrtbase export moves this count. That is how a
    gate gets switched off, so it must only ever fail upward.
    """
    import pytest
    import yaml

    original = vuc.TAKEOVER.read_text(encoding="utf-8")
    doc = yaml.safe_load(original)
    fresh = [
        a
        for a in doc["burndown"]
        if (vuc.REPO / f"build-{a}").is_dir() and not vuc.burndown_stale(a)
    ]
    if not fresh:
        pytest.skip("no fresh build tree in this worktree; the ratchet needs current objects")
    arch = fresh[0]
    recorded = doc["burndown"][arch]
    # Anchor on what the tree MEASURES, not on the recorded ceiling. The two differ
    # whenever the build dir holds a newer (or older) Rust archive than the commit
    # recorded -- routinely, in the conductor's integration worktree, which the
    # previous gate built. A +/-50 around the RECORDED number stopped being "below
    # reality" once a batch's archive answered 53 more exports, the check truthfully
    # found no regression, and the test read that as the ratchet being broken: it
    # halted the merge train as "master is red" (2026-09-26).
    import ucrt_burndown

    buckets, byfile = ucrt_burndown.classify(arch)
    measured = len(buckets["MICROSOFT"])
    recorded_line = f"  {arch}: {recorded}"

    # PIN the measurement for the checks below. check_burndown() measures again, and
    # the conductor runs this test while its own builds rewrite the integration
    # worktree's build dirs: the two readings differed by 91 (634, then 725) once a
    # batch moved the count that far, and the test failed a ratchet whose logic was
    # fine (halting the train as "master red", 2026-09-27). What is under test is
    # the RATCHET -- ceiling vs one measurement -- so it gets exactly one.
    original_classify = ucrt_burndown.classify
    original_stale = vuc.burndown_stale
    # Only THIS arch is measured (once); the other arches' ceilings are not what
    # this test perturbs, so they read as stale and are skipped with a note.
    ucrt_burndown.classify = lambda a: (buckets, byfile) if a == arch else ({}, {})
    vuc.burndown_stale = lambda a: None if a == arch else "pinned out by the test"

    # Perturb a COPY, never the tracked file. Writing tools/ucrt-takeover.yaml in place and
    # restoring it leaves the content clean but moves the mtime, and `vibeos-cli build`'s
    # artifact-check compares source mtimes against the build stamp -- so a gate running this
    # test concurrently with its own builds fails one arch at random with
    # "build source changed since build (C/asm/cmake/spec/rc): tools/ucrt-takeover.yaml".
    # Cost one arm64 BUILD_INCOMPLETE plus the cascade into every artifact-dependent scanner.
    original_takeover = vuc.TAKEOVER
    scratch = tmp_path / "ucrt-takeover.yaml"
    vuc.TAKEOVER = scratch
    try:
        # Ceiling below reality -> a regression, and it must FAIL.
        scratch.write_text(original.replace(recorded_line, f"  {arch}: {measured - 1}"))
        fails, _ = vuc.check_burndown()
        assert any("REGRESSED" in f and arch in f for f in fails), fails

        # Ceiling above reality -> an improvement, and it must only NOTE.
        scratch.write_text(original.replace(recorded_line, f"  {arch}: {measured + 1}"))
        fails, notes = vuc.check_burndown()
        assert not fails, fails
        assert any("fewer Microsoft-backed exports" in n for n in notes), notes
    finally:
        vuc.TAKEOVER = original_takeover
        ucrt_burndown.classify = original_classify
        vuc.burndown_stale = original_stale


def test_archive_source_crates_are_walked_not_listed() -> None:
    """ucrt-rust reaches the archive through crt-shared-cargo; a hand-kept list
    missed it, so the closure must be derived from the manifests."""
    crates = vuc._archive_source_crates()
    assert "ucrt-rust" in crates
    assert "crt-shared-rust" in crates
    assert "msvcrt-crt-rust" not in crates


def test_rebuild_lists_match_the_crate_closure() -> None:
    assert vuc.check_rebuild_lists() == []


def test_a_crate_missing_from_a_rebuild_list_is_named(tmp_path: Path) -> None:
    """Control: a list that drops a closure crate must be reported -- a gate nobody
    has watched fail is a gate nobody knows works."""
    for cmake, _var, _root in vuc.REBUILD_LISTS:
        src = (vuc.REPO / cmake).read_text()
        dst = tmp_path / cmake
        dst.parent.mkdir(parents=True, exist_ok=True)
        dst.write_text(src.replace(" crt-portable-rust", "", 1))
    found = vuc.check_rebuild_lists(repo=tmp_path)
    assert any("omits `crt-portable-rust`" in f for f in found), found


def test_a_stale_build_is_not_a_ratchet_regression(tmp_path: Path, monkeypatch) -> None:
    """The gate measures the ratchet while its own builds are still running, so a
    commit that lowers the ceiling is first measured against the PREVIOUS archive.
    That must be a note, never a failure -- it bounced the lane that earned the drop."""
    scratch = tmp_path / "ucrt-takeover.yaml"
    scratch.write_text("burndown:\n  i386: 1\n  amd64: 1\n  arm64: 1\n")
    monkeypatch.setattr(vuc, "TAKEOVER", scratch)
    monkeypatch.setattr(vuc, "burndown_stale", lambda arch: "archive older than sources")
    fails, notes = vuc.check_burndown()
    assert fails == []
    if any((vuc.REPO / f"build-{a}").is_dir() for a in ("i386", "amd64", "arm64")):
        assert any("not measured" in n for n in notes), notes


def test_an_archive_from_another_tree_is_stale_whatever_its_mtime(tmp_path, monkeypatch) -> None:
    """The land queue reuses one integration build dir across batches, so an archive
    can be NEWER than this tree's commits and still be another tree's. Content decides:
    a declared takeover the archive does not define means it was not built from here."""
    monkeypatch.setattr(vuc, "_ucrt_archive", lambda build, triple: tmp_path / "lib.a")
    monkeypatch.setattr(vuc, "_stale_against_sources", lambda a: None)  # clock says fresh
    monkeypatch.setattr(vuc, "declared_takeovers", lambda: {"strlen": "string", "isalpha": "ctype"})
    monkeypatch.setattr(vuc, "_nm_defined", lambda a: {"strlen", "memcpy"})
    why = vuc.burndown_stale("arm64")
    assert why and "isalpha" in why and "different tree" in why
    monkeypatch.setattr(vuc, "_nm_defined", lambda a: {"strlen", "isalpha"})
    assert vuc.burndown_stale("arm64") is None


def _initializer_tree(tmp_path: Path, replaced_by_rust: bool) -> Path:
    init = tmp_path / "sdk" / "lib" / "ucrt" / "initializers"
    init.mkdir(parents=True)
    (init / "timeset_initializer.cpp").write_text(
        'extern "C" _CRTALLOC(".CRT$XIC")  _PIFV const __acrt_timeset_initializer'
        " = __acrt_initialize_timeset;\n"
    )
    src = tmp_path / "sdk" / "lib" / "rust" / "ucrt-rust" / "src"
    src.mkdir(parents=True)
    (src / "timeset.rs").write_text(
        "crate::force_include!(__acrt_timeset_initializer);\n"
        if replaced_by_rust
        else "// nothing\n"
    )
    return tmp_path


def test_a_hook_whose_force_include_was_deleted_is_caught(tmp_path: Path):
    # The positive control is the real loss: timeset.cpp deleted, its
    # `_CRT_LINKER_FORCE_INCLUDE(__acrt_timeset_initializer)` gone with it.
    findings = vuc.check_force_includes(_initializer_tree(tmp_path, replaced_by_rust=False))
    assert len(findings) == 1 and "__acrt_timeset_initializer" in findings[0]


def test_a_rust_force_include_satisfies_it(tmp_path: Path):
    assert vuc.check_force_includes(_initializer_tree(tmp_path, replaced_by_rust=True)) == []


def test_the_real_tree_force_includes_every_hook():
    assert vuc.check_force_includes() == []
