import json
import sys
from pathlib import Path

import pytest

sys.path.insert(0, str(Path(__file__).resolve().parent))

import media_manifest as m


def test_a_source_is_named_by_what_makes_it():
    cells = {"base/m/m.exe"}
    assert m._enc("/tmp/vibeos/build-amd64/base/m/m.exe", "amd64", cells) == "cell:base/m/m.exe"
    assert m._enc("/tmp/vibeos/build-amd64/reactos.cab", "amd64", cells) == "build:reactos.cab"
    assert m._enc("/tmp/vibeos/media/fonts/x.ttf", "amd64", cells) == "repo:media/fonts/x.ttf"
    with pytest.raises(m.MediaError):
        m._enc("/usr/share/x", "amd64", cells)


def test_the_list_rebuilds_with_cells_taken_from_elsewhere(tmp_path):
    entries = [
        [None, "build:boot/empty"],
        ["reactos/system32/m.exe", "cell:base/m/m.exe"],
        ["reactos/fonts/x.ttf", "repo:media/fonts/x.ttf"],
    ]
    b, r, c = Path("/b"), Path("/r"), Path("/c")

    def where(cells):
        return {s: m.resolve(s, b, r, cells) for _d, s in entries}

    assert m.write_list(entries, where(None)) == (
        "/b/boot/empty\nreactos/system32/m.exe=/b/base/m/m.exe\nreactos/fonts/x.ttf=/r/media/fonts/x.ttf\n"
    )
    assert "reactos/system32/m.exe=/c/base/m/m.exe" in m.write_list(entries, where(c))


def test_the_cab_section_renders_after_the_cds():
    man = {
        "config": "Debug",
        "cds": {"bootcd": {"command": "x", "entries": [["reactos/reactos.cab", m.CAB]]}},
        "cab": {
            "dff": "repo:boot/bootdata/packages/reactos.dff.in",
            "inf_command": "i",
            "cab_command": "c",
            "files": [["cell:base/m/m.exe", "2"], ["repo:boot/x.txt", "1"]],
        },
    }
    text = m.render(man)
    assert json.loads(text) == man
    assert '   ["cell:base/m/m.exe", "2"],' in text.splitlines()


def test_the_cab_is_packed_from_the_cells_with_cmakes_commands(tmp_path, monkeypatch):
    build, repo, cells, out = (tmp_path / d for d in ("b", "r", "c", "o"))
    (repo / "boot").mkdir(parents=True)
    (repo / "boot/head.in").write_text(".Set X=1\n")
    (cells / "base/m").mkdir(parents=True)
    (cells / "base/m/m.exe").write_text("m")
    pk, cab = m.PACKAGES, "{BUILD}/host-tools/bin/cabman"
    man = {
        "cab": {
            "dff": "repo:boot/head.in",
            "inf_command": f"cd {pk} && {cab} -C {pk}/reactos.dff -L {pk} -I -P {{REPO}}",
            "cab_command": (
                f"cd {{BUILD}} && {cab} -C {pk}/reactos.dff "
                "-RC {BUILD}/reactos.inf -N -P {REPO}"
            ),
            "files": [["cell:base/m/m.exe", "2"]],
        }
    }
    ran = []

    def fake(cmd):
        ran.append(cmd)
        for name in ("reactos.inf", "reactos.cab"):
            (out / "cab" / name).write_text("")

    monkeypatch.setattr(m, "_run", fake)
    made = m.recab(man, out, build, repo, cells)
    d = out / "cab"
    assert (d / "reactos.dff").read_text() == f'.Set X=1\n"{cells}/base/m/m.exe" 2\n'
    tool = f"{build}/host-tools/bin/cabman"
    assert ran[0] == f"cd {d} && {tool} -C {d}/reactos.dff -L {d} -I -P {repo}"
    assert ran[1] == f"cd {d} && {tool} -C {d}/reactos.dff -RC {d}/reactos.inf -N -P {repo}"
    assert made == {m.CAB: d / "reactos.cab", m.CAB_INF: d / "reactos.inf"}


def test_cells_from_another_source_tree_are_refused(tmp_path, monkeypatch):
    cells = tmp_path / "cells"
    cells.mkdir()
    with pytest.raises(m.MediaError, match="cargo_orchestrate"):
        m.cells_source(cells, "amd64")
    (cells / m.CELLS_STAMP).write_text(json.dumps({"arch": "amd64", "source": "old"}))
    monkeypatch.setattr(m, "source_hash", lambda repo=None: "new")
    with pytest.raises(m.MediaError, match="another source tree"):
        m.assemble("amd64", "livecd", tmp_path / "out", cells, tmp_path)
    (cells / m.CELLS_STAMP).write_text(json.dumps({"arch": "i386", "source": "new"}))
    with pytest.raises(m.MediaError, match="i386 cells"):
        m.assemble("amd64", "livecd", tmp_path / "out", cells, tmp_path)


def test_the_rendered_manifest_is_json_one_entry_per_line():
    man = {
        "config": "Debug",
        "cds": {"livecd": {"command": "x", "entries": [[None, "build:e"], ["a", "repo:b"]]}},
    }
    text = m.render(man)
    assert json.loads(text) == man
    assert '   ["a", "repo:b"]' in text.splitlines()


def test_a_build_carrying_payload_rostests_is_refused_not_filtered():
    """A payload build leaves ENABLE_ROSTESTS[_APITESTS] on; its lists carry test exes AND
    their import closures (amd64's SysWOW64 got i386's hid.dll), so they cannot be committed."""
    clean = {
        "config": "Debug",
        "cds": {
            "livecd": {
                "command": "",
                "entries": [
                    ["reactos/bin/kmtest.exe", "build:modules/rostests/kmtests/kmtest/kmtest.exe"],
                    ["reactos/bin/testdata/x", "repo:modules/rostests/testdata/x"],
                    ["reactos/system32/pdh.dll", "cell:dll/win32/pdh/pdh.dll"],
                ],
            }
        },
    }
    assert m.committed(clean, "amd64") is clean
    dirty = json.loads(json.dumps(clean))
    dirty["cds"]["livecd"]["entries"].append(
        [
            "reactos/syswow64/user32_winetest.exe",
            "repo:build-i386/modules/rostests/winetests/user32/user32_winetest.exe",
        ]
    )
    with pytest.raises(m.MediaError, match=r"winetests.*ENABLE_ROSTESTS_APITESTS=OFF"):
        m.committed(dirty, "amd64")
