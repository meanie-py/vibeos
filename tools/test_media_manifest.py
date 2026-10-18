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
    assert m.write_list(entries, b, r, None) == (
        "/b/boot/empty\nreactos/system32/m.exe=/b/base/m/m.exe\nreactos/fonts/x.ttf=/r/media/fonts/x.ttf\n"
    )
    assert "reactos/system32/m.exe=/c/base/m/m.exe" in m.write_list(entries, b, r, c)


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
