"""Compatibility entry point for the native read-only bench TFTP server."""

import os
import sys
from pathlib import Path

ROOT = Path(__file__).resolve().parents[2] if len(Path(__file__).resolve().parents) > 2 else None


def _call(op, kwargs):
    if str(ROOT) not in sys.path:
        sys.path.insert(0, str(ROOT))
    from tools.bench_infra_compat import call

    return call(op, kwargs, ROOT)


def parse_request(payload):
    filename, mode, options = _call("bench_tftp_parse", {"payload": list(payload)})
    return filename, mode, options


def resolve(root, filename):
    return _call("bench_tftp_resolve", {"root": str(root), "filename": filename})


def main():
    root = sys.argv[1] if len(sys.argv) > 1 else "."
    bind = sys.argv[2] if len(sys.argv) > 2 else "0.0.0.0"
    port = sys.argv[3] if len(sys.argv) > 3 else "69"
    adjacent = Path(__file__).resolve().with_name("bench-infra.exe")
    executable = adjacent if os.name == "nt" else ROOT / "tools" / "bench-infra"
    os.execv(str(executable), [str(executable), "tftp", root, bind, port])


if __name__ == "__main__":
    main()
