"""Depfile retargeting must not reinterpret or discard emitted prerequisites."""

import sys
from pathlib import Path

import pytest

sys.path.insert(0, str(Path(__file__).resolve().parent))

from retarget_cargo_depfile import retarget


def test_preserves_dependency_body_and_phony_rules_verbatim():
    body = " a.rs space\\ name.bin \\\n+ b.rs\na.rs:\nb.rs:\n"
    assert retarget("/cargo/my\\ app:" + body, "/cargo/my app", "module/my app") == (
        "module/my\\ app:" + body
    )


@pytest.mark.parametrize("text", ["", "/wrong: a.rs\n", "/cargo/app2: a.rs\n", "/cargo/app:a.rs\n"])
def test_wrong_or_malformed_rule_is_rejected(text):
    with pytest.raises(ValueError):
        retarget(text, "/cargo/app", "app")


@pytest.mark.parametrize(
    "target", ["line\nbreak", "hash#name", "dollar$name", "colon:name", "back\\slash"]
)
def test_unsupported_target_spelling_fails_loudly(target):
    with pytest.raises(ValueError):
        retarget("/cargo/app: a.rs\n", "/cargo/app", target)
