"""Compatibility facade for native canonical-path/toolchain mechanisms."""

from __future__ import annotations

import os
from functools import lru_cache
from pathlib import Path

from tools import build_infra_compat as native

CANON = "/tmp/vibeos"
REPO = Path(__file__).resolve().parents[3]


def _call(operation, **arguments):
    return native.call("cli_canon_" + operation, {"env": dict(os.environ), **arguments})


def canon_path():
    return _call("path")


@lru_cache(maxsize=1)
def available():
    return _call("available", repo=REPO)


def wrap(repo, cmd):
    return _call("wrap", repo=repo, argv=cmd)


def cmake_home(build_dir):
    return _call("home", build=build_dir)


def is_canonical(build_dir):
    return _call("is_canonical", build=build_dir)


def carried_options(build_dir):
    return _call("carried", build=build_dir)


def ensure_toolchain(repo):
    return _call("toolchain", repo=repo)


def configure_cmd(arch, extra=None):
    return _call("configure", arch=arch, argv=extra or [])
