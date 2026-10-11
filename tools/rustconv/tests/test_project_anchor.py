"""A project declares itself with rustconv.toml; nothing is inferred from the install.

The anchor exists so that project-scoped paths — above all the durable bank of PAID API
responses — follow the tree you invoke rustconv in, never the directory the package was
installed to. Discovery walks up from CWD, so each worktree anchors to itself.
"""

from __future__ import annotations

from pathlib import Path

import pytest

from rustconv import project


def _anchor(root: Path, body: str) -> Path:
    root.mkdir(parents=True, exist_ok=True)
    anchor = root / project.ANCHOR
    anchor.write_text(body)
    return anchor


def test_no_anchor_is_not_an_error(tmp_path):
    """An anchorless checkout keeps working on the legacy inference."""
    assert project.load(tmp_path) is None


def test_found_by_walking_up(tmp_path):
    """Invoking from deep inside the tree still finds the root's anchor."""
    _anchor(tmp_path, '[project]\nname = "demo"\n')
    deep = tmp_path / "a" / "b" / "c"
    deep.mkdir(parents=True)
    loaded = project.load(deep)
    assert loaded is not None
    assert loaded.name == "demo"
    assert loaded.root == tmp_path


def test_nearest_anchor_wins(tmp_path):
    """A nested project anchors to ITSELF, not to an outer tree that also has one.

    This is the worktree case: siblings under one parent must not resolve to each other.
    """
    _anchor(tmp_path, '[project]\nname = "outer"\n')
    inner = tmp_path / "inner"
    _anchor(inner, '[project]\nname = "inner"\n')
    loaded = project.load(inner / "src")
    assert loaded is not None and loaded.name == "inner"


def test_responses_resolves_against_the_anchor(tmp_path):
    """The bank is relative to the anchor's directory, so one file works in every clone."""
    _anchor(tmp_path, '[project]\nname = "demo"\nresponses = "tools/rustconv/.responses"\n')
    loaded = project.load(tmp_path)
    assert loaded.responses == (tmp_path / "tools" / "rustconv" / ".responses").resolve()


def test_absolute_responses_is_left_alone(tmp_path):
    _anchor(tmp_path, f'[project]\nname = "demo"\nresponses = "{tmp_path}/bank"\n')
    assert project.load(tmp_path).responses == tmp_path / "bank"


def test_responses_defaults_next_to_the_anchor(tmp_path):
    _anchor(tmp_path, '[project]\nname = "demo"\n')
    assert project.load(tmp_path).responses == (tmp_path / ".responses").resolve()


def test_malformed_anchor_raises_rather_than_falling_back(tmp_path):
    """Silently ignoring a declaration and inferring something else is the bug being fixed."""
    _anchor(tmp_path, "this is not toml {{{")
    with pytest.raises(project.ProjectError, match="malformed"):
        project.load(tmp_path)


def test_anchor_without_project_table_raises(tmp_path):
    _anchor(tmp_path, "[other]\nkey = 1\n")
    with pytest.raises(project.ProjectError, match=r"\[project\]"):
        project.load(tmp_path)


def test_anchor_without_name_raises(tmp_path):
    _anchor(tmp_path, '[project]\nresponses = ".responses"\n')
    with pytest.raises(project.ProjectError, match="name"):
        project.load(tmp_path)


def test_plugin_is_optional(tmp_path):
    _anchor(tmp_path, '[project]\nname = "demo"\n')
    assert project.load(tmp_path).load_plugin() is None


def test_unimportable_plugin_raises_naming_the_anchor(tmp_path):
    _anchor(tmp_path, '[project]\nname = "demo"\nplugin = "no.such.module"\n')
    loaded = project.load(tmp_path)
    with pytest.raises(project.ProjectError) as excinfo:
        loaded.load_plugin()
    assert str(loaded.anchor) in str(excinfo.value)


def test_live_vibeos_anchor_is_consistent():
    """The real anchor must agree with what config and the response bank actually use."""
    import rustconv.config as config
    import rustconv.response_cache as rc

    assert config.PROJECT is not None, "vibeos ships a rustconv.toml"
    assert config.PROJECT.name == "vibeos"
    assert config.PROJECT.root == config.REACTOS_ROOT
    assert config.PROJECT.responses == config.RUSTCONV_RESPONSES_DIR
    assert config.PROJECT.responses == rc.RESPONSES_DIR
    plugin = config.PROJECT.load_plugin()
    assert plugin is not None and len(plugin.TARGETS) == len(config.TARGETS)
