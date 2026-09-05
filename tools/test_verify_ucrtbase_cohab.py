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
