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


def test_every_modelled_target_has_a_real_object_directory():
    """A path typo silently contributes ZERO symbols, which reads as mass shadowing.

    The check treats an authority target with no objects as a hard failure rather than as
    'nothing to add', but only a real build proves the *path* is right -- so assert it against
    whichever build dirs this worktree happens to have, and skip if it has none.
    """
    builds = [REPO / f"build-{a}" for a in ("i386", "amd64", "arm64")]
    builds = [b for b in builds if b.is_dir()]
    if not builds:
        pytest.skip("no build dir in this worktree")
    for build in builds:
        for target, obj_dir in vuc.OURS_ON_LINK_LINE.items():
            d = build / obj_dir
            assert d.is_dir(), f"{build.name}: {target} object dir does not exist at {d}"


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
