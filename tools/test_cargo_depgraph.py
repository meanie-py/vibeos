"""Resolved graph controls: renamed, build, target, cyclic and missing roots."""

import sys
from pathlib import Path

import pytest

sys.path.insert(0, str(Path(__file__).resolve().parent))

from cargo_depgraph import local_closure


def graph():
    edges = {
        "app": ["shared", "builder", "registry"],
        "shared": ["target_only"],
        "builder": [],
        "registry": ["patched"],
        "patched": [],
        "target_only": ["shared"],
        "body": ["shared"],
        "unrelated": [],
    }
    return {
        "packages": [
            {
                "id": name,
                "name": name,
                "source": "registry" if name == "registry" else None,
                "manifest_path": f"/fixture/{name}/Cargo.toml",
            }
            for name in edges
        ],
        "resolve": {"nodes": [{"id": name, "dependencies": deps} for name, deps in edges.items()]},
    }


def test_resolved_graph_keeps_build_target_and_patched_dependencies():
    assert [p["name"] for p in local_closure(graph(), ["app", "body"])] == [
        "app",
        "body",
        "builder",
        "patched",
        "shared",
        "target_only",
    ]


def test_nested_build_body_must_be_explicit():
    assert "body" not in {p["name"] for p in local_closure(graph(), ["app"])}


@pytest.mark.parametrize("roots", [[], ["missing"], ["registry"]])
def test_missing_or_nonlocal_root_is_an_error(roots):
    with pytest.raises(ValueError):
        local_closure(graph(), roots)


def test_incomplete_resolve_is_not_silently_accepted():
    data = graph()
    data["resolve"] = None
    with pytest.raises(ValueError):
        local_closure(data, ["app"])
    data = graph()
    data["resolve"]["nodes"].pop(1)
    with pytest.raises(ValueError):
        local_closure(data, ["app"])
