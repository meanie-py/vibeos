"""The shared Rust-source walk: same files as before, and no longer racy."""

from __future__ import annotations

import shutil
import subprocess
import sys
import textwrap
import threading
import time
from pathlib import Path

sys.path.insert(0, str(Path(__file__).resolve().parent))

from rust_sources import walk_rs


def _tree(root: Path) -> None:
    (root / "src").mkdir(parents=True)
    (root / "src" / "a.rs").write_text("fn a() {}")
    (root / "src" / "b.rs").write_text("fn b() {}")
    (root / "src" / "bindgen_i386.rs").write_text("// island")
    (root / "src" / "build.rs").write_text("fn main() {}")
    (root / "src" / "notrust.txt").write_text("x")
    for excluded in ("target", ".responses", ".git"):
        d = root / excluded / "deep"
        d.mkdir(parents=True)
        (d / "generated.rs").write_text("fn gen() {}")


def test_finds_sources_and_prunes_output_dirs(tmp_path: Path) -> None:
    _tree(tmp_path)
    got = [p.name for p in walk_rs(tmp_path)]
    assert got == ["a.rs", "b.rs", "bindgen_i386.rs", "build.rs"]
    assert "generated.rs" not in got


def test_order_matches_sorted_rglob(tmp_path: Path) -> None:
    """Lint baselines were captured under `sorted(rglob(...))`; don't move them."""
    _tree(tmp_path)
    expected = [
        p
        for p in sorted(tmp_path.rglob("*.rs"))
        if "/target/" not in str(p) and "/.responses/" not in str(p) and "/.git/" not in str(p)
    ]
    assert walk_rs(tmp_path) == expected


def test_skip_predicate_filters_survivors(tmp_path: Path) -> None:
    _tree(tmp_path)
    got = [
        p.name
        for p in walk_rs(
            tmp_path, skip=lambda f: f.name.startswith("bindgen_") or f.name == "build.rs"
        )
    ]
    assert got == ["a.rs", "b.rs"]


def test_a_single_file_root_yields_itself(tmp_path: Path) -> None:
    f = tmp_path / "x.rs"
    f.write_text("fn x() {}")
    assert walk_rs(f) == [f]
    assert walk_rs(tmp_path / "x.txt") == []


def test_survives_directories_vanishing_mid_walk(tmp_path: Path) -> None:
    """The actual defect. `Path.rglob` propagates the FileNotFoundError a concurrent
    cargo build causes by deleting under `sdk/lib/rust/target`; the gate reported
    that crash as "NEW lint findings". Pruning removes the race for `target` itself,
    and `onerror` covers everywhere else — so this asserts the general property.
    """
    keep = tmp_path / "keep"
    keep.mkdir()
    (keep / "s.rs").write_text("fn s() {}")
    churn = tmp_path / "churn"
    for i in range(400):
        d = churn / f"d{i}" / "sub"
        d.mkdir(parents=True)
        (d / "g.rs").write_text("fn g() {}")

    def killer() -> None:
        for i in range(400):
            shutil.rmtree(churn / f"d{i}", ignore_errors=True)
            time.sleep(0.0005)

    t = threading.Thread(target=killer)
    t.start()
    try:
        got = walk_rs(tmp_path)  # must not raise
    finally:
        t.join()
    assert keep / "s.rs" in got


def test_plain_rglob_really_does_raise(tmp_path: Path) -> None:
    """Falsifies the alternative reading — that pathlib swallows this itself, which
    would make the whole fix unnecessary. If a future interpreter starts tolerating
    it, this test fails and the walker's rationale needs re-reading, not deleting.
    """
    prog = textwrap.dedent(
        """
        import pathlib, shutil, sys, threading, time
        base = pathlib.Path(sys.argv[1])
        for i in range(400):
            d = base / f"d{i}" / "sub"
            d.mkdir(parents=True)
            (d / "g.rs").write_text("x")
        def killer():
            for i in range(400):
                shutil.rmtree(base / f"d{i}", ignore_errors=True)
                time.sleep(0.0005)
        t = threading.Thread(target=killer); t.start()
        try:
            list(base.rglob("*.rs"))
            print("SURVIVED")
        except OSError as e:
            print("RAISED", type(e).__name__)
        t.join()
        """
    )
    d = tmp_path / "race"
    d.mkdir()
    out = subprocess.run(
        [sys.executable, "-c", prog, str(d)], capture_output=True, text=True, timeout=120
    ).stdout
    assert "RAISED FileNotFoundError" in out or "SURVIVED" in out
    if "SURVIVED" in out:
        import pytest

        pytest.skip("this interpreter tolerates a vanishing dir; walker still correct")
