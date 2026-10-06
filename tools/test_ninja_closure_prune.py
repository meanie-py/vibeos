"""tools/ninja_closure_prune.py: what a warm build dir may keep."""

from __future__ import annotations

import sys
from pathlib import Path

sys.path.insert(0, str(Path(__file__).resolve().parent))

import ninja_closure_prune as ncp

MANIFEST = """\
include rules.ninja
build build.ninja cmake.verify_globs: RERUN_CMAKE | CMakeLists.txt
build app.exe | app.map: LINK app.obj libref.a || gen.h
build app.obj: CC app.c
build libref.a: AR ref.obj
build ref.obj: CC ref.c
build gen.h: GEN
build orphan.a: AR orphan.obj
build orphan.obj: CC orphan.c
build sp$ ace.obj: CC sp$ ace.c
build /tmp/vibeos/build-x/abs.obj: CC abs.c
build livecd: phony app.exe
"""


def _tree(tmp: Path, logged: list[str]) -> Path:
    b = tmp / "build-x"
    b.mkdir()
    (b / "build.ninja").write_text(MANIFEST)
    (b / "rules.ninja").write_text("rule CC\n  command = cc\nbuild included.obj: CC i.c\n")
    lines = ["# ninja log v5"] + [f"1\t2\t3\t{p}\tdeadbeef" for p in logged]
    (b / ".ninja_log").write_text("\n".join(lines) + "\n")
    for p in logged:
        f = b / p.removeprefix("/tmp/vibeos/build-x/")
        if p == "build.ninja":
            continue  # the manifest is real; don't clobber it with a stub
        if not p.startswith("/") or p.startswith("/tmp/vibeos/build-x/"):
            f.parent.mkdir(parents=True, exist_ok=True)
            f.write_text("x")
    return b


def test_keeps_the_closure_and_its_side_outputs(tmp_path):
    logged = ["app.exe", "app.map", "app.obj", "libref.a", "ref.obj", "gen.h", "build.ninja"]
    b = _tree(tmp_path, [*logged, "orphan.a", "orphan.obj"])
    victims, unknown = ncp.plan(b, ["build.ninja", "livecd"], "/tmp/vibeos/build-x")
    assert unknown == []
    # app.map is a side output of the link: deleting it would re-run the link forever.
    # gen.h is order-only: ninja builds it first, so it is part of the closure.
    assert victims == ["orphan.a", "orphan.obj"]


def test_an_unrooted_regeneration_edge_would_be_pruned(tmp_path):
    # Why build.py always roots build.ninja: without it the manifest itself is a victim.
    b = _tree(tmp_path, ["build.ninja", "app.exe"])
    victims, _ = ncp.plan(b, ["livecd"], None)
    assert "build.ninja" in victims


def test_escaped_spaces_and_absolute_canonical_paths(tmp_path):
    b = _tree(tmp_path, ["sp ace.obj", "/tmp/vibeos/build-x/abs.obj", "abs.obj"])
    victims, _ = ncp.plan(b, ["livecd"], "/tmp/vibeos/build-x")
    # One file under two spellings is one victim.
    assert victims == ["abs.obj", "sp ace.obj"]
    victims, _ = ncp.plan(b, ["livecd", "sp ace.obj", "abs.obj"], "/tmp/vibeos/build-x")
    assert victims == []


def test_never_leaves_the_build_dir_and_reports_unknown_roots(tmp_path):
    b = _tree(tmp_path, ["/etc/hostname", "../elsewhere.obj", "included.obj"])
    victims, unknown = ncp.plan(b, ["livecd", "no-such-target"], None)
    assert unknown == ["no-such-target"]
    assert victims == ["included.obj"]  # parsed through `include`, outside the closure
