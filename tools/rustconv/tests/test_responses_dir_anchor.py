"""The paid-response bank must follow the PROJECT, never the installed module.

These lock the fix for the divergence that extraction would have armed: ResponseCache
resolved `.responses` module-relative while everything else resolved it project-relative.
They agreed only because the package happens to sit at `tools/rustconv/rustconv/`.
Import rustconv from anywhere else and the bank silently relocates -> every function is a
cache miss and is re-billed, with no error. See response_cache._resolve_responses_dir.

NB: these deliberately never `importlib.reload(config)`. Reloading rebuilds the `Target`
class, so every other module still holding the OLD class fails isinstance/identity checks
— it broke four unrelated tests in test_target_resolve before this was rewritten. Poke the
resolver's inputs directly instead.
"""

from __future__ import annotations

from pathlib import Path

import pytest

import rustconv.response_cache as rc


def _bank_with_work(root: Path) -> Path:
    """A .responses dir holding one banked (paid) function."""
    bank = root / ".responses"
    entry = bank / "shell32__CSomeClass"
    entry.mkdir(parents=True)
    (entry / "manifest.json").write_text("{}")
    (entry / "SomeMethod.rs").write_text("pub fn some_method() {}")
    return bank


def test_module_location_never_wins(monkeypatch, tmp_path):
    """No env override: the bank is config's project-anchored path, NOT the module's."""
    active = tmp_path / "project" / ".responses"
    active.mkdir(parents=True)
    fake_install = tmp_path / "install"
    (fake_install / "pkg").mkdir(parents=True)

    monkeypatch.delenv("RUSTCONV_RESPONSES_DIR", raising=False)
    monkeypatch.setattr(rc, "RUSTCONV_RESPONSES_DIR", active)
    monkeypatch.setattr(rc, "__file__", str(fake_install / "pkg" / "response_cache.py"))

    assert rc._resolve_responses_dir() == active


def test_explicit_env_is_honoured_even_with_a_stranded_bank(monkeypatch, tmp_path):
    """An explicit RUSTCONV_RESPONSES_DIR is deliberate — never second-guessed.

    This is how you RECOVER from a strand, so it must work even when a legacy bank
    holding paid work exists.
    """
    fake_install = tmp_path / "install"
    _bank_with_work(fake_install)
    chosen = tmp_path / "chosen"
    chosen.mkdir()

    monkeypatch.setenv("RUSTCONV_RESPONSES_DIR", str(chosen))
    monkeypatch.setattr(rc, "RUSTCONV_RESPONSES_DIR", chosen)
    monkeypatch.setattr(rc, "__file__", str(fake_install / "pkg" / "response_cache.py"))

    assert rc._resolve_responses_dir() == chosen


def test_stranded_bank_hard_fails(monkeypatch, tmp_path):
    """A legacy module-relative bank holding paid work must ABORT, not be ignored.

    The money case: rustconv imported from outside the project tree with real banked
    responses at the old location. Silently re-billing is the failure being prevented.
    """
    fake_install = tmp_path / "install"
    legacy = _bank_with_work(fake_install)
    active = tmp_path / "project" / ".responses"
    active.mkdir(parents=True)

    monkeypatch.delenv("RUSTCONV_RESPONSES_DIR", raising=False)
    monkeypatch.setattr(rc, "RUSTCONV_RESPONSES_DIR", active)
    monkeypatch.setattr(rc, "__file__", str(fake_install / "pkg" / "response_cache.py"))

    with pytest.raises(RuntimeError) as excinfo:
        rc._resolve_responses_dir()
    msg = str(excinfo.value)
    assert "stranded" in msg
    assert str(legacy) in msg  # the message must name the bank so recovery is copy-pasteable
    assert "RUSTCONV_RESPONSES_DIR" in msg


def test_empty_legacy_dir_is_not_a_strand(monkeypatch, tmp_path):
    """An empty leftover directory carries no paid work — must not block startup."""
    fake_install = tmp_path / "install"
    (fake_install / ".responses").mkdir(parents=True)
    active = tmp_path / "project" / ".responses"
    active.mkdir(parents=True)

    monkeypatch.delenv("RUSTCONV_RESPONSES_DIR", raising=False)
    monkeypatch.setattr(rc, "RUSTCONV_RESPONSES_DIR", active)
    monkeypatch.setattr(rc, "__file__", str(fake_install / "pkg" / "response_cache.py"))

    assert rc._resolve_responses_dir() == active


def test_live_tree_agrees(monkeypatch):
    """In the real vibeos checkout the two resolutions must actually coincide."""
    import rustconv.config as config

    assert rc.RESPONSES_DIR == config.RUSTCONV_RESPONSES_DIR
