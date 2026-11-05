#!/usr/bin/env python3
"""Compatibility facade for native payload PE checks."""

import sys
from pathlib import Path

REPO = Path(__file__).resolve().parents[2]
if str(REPO) not in sys.path:
    sys.path.insert(0, str(REPO))
from tools.build_infra_compat import call  # noqa: E402


class PEError(ValueError):
    pass


def _call(op, **kwargs):
    try:
        return call("payload_pe_" + op, kwargs, REPO)
    except ValueError as e:
        raise PEError(str(e)) from e


def _guard_value(expr, off_refs):
    return _call("guard_value", expr=expr, off_refs=off_refs)


def compiled_in(src, off_refs):
    return _call("compiled_in", src=src, off_refs=off_refs)


def parse(path):
    subsystem, imports = _call("parse", path=path)
    return subsystem, imports


def machine_arch(path):
    return _call("machine_arch", path=path)


def export_version(repo=None):
    return _call("export_version", repo=repo or REPO)


def spec_index(repo=None):
    return {k: [Path(p) for p in v] for k, v in _call("spec_index", repo=repo or REPO).items()}


def spec_exports(spec, arch, version):
    return _call("spec_exports", spec=spec, arch=arch, version=version)


def check_imports(exe, imports, repo=None):
    return _call("check_imports", exe=exe, imports=imports, repo=repo or REPO)


def main(argv):
    result = _call("main", argv=argv)
    print(result["stdout"], end="")
    return result["code"]


if __name__ == "__main__":
    sys.exit(main(sys.argv[1:]))
