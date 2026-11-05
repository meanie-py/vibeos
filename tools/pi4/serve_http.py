"""Compatibility entry point for the native read-only bench HTTP server."""

import os
import sys
from pathlib import Path

ROOT = Path(__file__).resolve().parents[2] if len(Path(__file__).resolve().parents) > 2 else None


def _call(op, kwargs):
    if str(ROOT) not in sys.path:
        sys.path.insert(0, str(ROOT))
    from tools.bench_infra_compat import call

    return call(op, kwargs, ROOT)


def resolve(root, path):
    return _call("bench_http_resolve", {"root": str(root), "path": path})


def span(size, range_header=None):
    result = _call("bench_http_span", {"size": size, "range": range_header})
    return tuple(result) if result is not None else None


def main():
    root = sys.argv[1] if len(sys.argv) > 1 else "."
    bind = sys.argv[2] if len(sys.argv) > 2 else "0.0.0.0"
    port = sys.argv[3] if len(sys.argv) > 3 else "8069"
    adjacent = Path(__file__).resolve().with_name("bench-infra.exe")
    executable = adjacent if os.name == "nt" else ROOT / "tools" / "bench-infra"
    os.execv(str(executable), [str(executable), "http", root, bind, port])


if __name__ == "__main__":
    main()
