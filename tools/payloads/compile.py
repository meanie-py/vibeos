#!/usr/bin/env python3
"""Transport for the native payload compiler."""

from __future__ import annotations

import argparse
import os
import sys
from pathlib import Path

PAYLOADS_DIR = Path(__file__).resolve().parent
REPO = PAYLOADS_DIR.parent.parent
ARCHES = ("i386", "amd64", "arm64")
STAMP_NAME = "payload-selection.json"


def die(msg):
    print(f"payloads/compile.py: ERROR: {msg}", file=sys.stderr)
    raise SystemExit(1)


sys.path.insert(0, str(REPO))
from tools.build_infra_compat import NativeError, call  # noqa: E402


def _call(op, **kwargs):
    try:
        return call("payload_build_" + op, kwargs, root=REPO)
    except NativeError as exc:
        die(str(exc))


def available():
    return _call("available")


def load_spec(name):
    return _call("load_spec", name=name)


def binaries(spec, arch):
    return _call("binaries", spec=spec, arch=arch)


def unreferenced_by_build(build_dir):
    return lambda path: _call("unreferenced_by_build", build_dir=build_dir, path=path)


def guest_build_current(name, build_dir):
    return _call("guest_build_current", name=name, build_dir=build_dir)


def guest_set(name, g):
    return {k: Path(v) for k, v in _call("guest_set", name=name, guest=g).items()}


def stale_generator(src):
    result = _call("stale_generator", src=src)
    return Path(result) if result is not None else None


def ninja_targets(names, arch):
    return _call("ninja_targets", names=names, arch=arch)


def stale_binaries(name, arch):
    return [Path(p) for p in _call("stale_binaries", name=name, arch=arch)]


def borrowed_builds(name, arch):
    return [tuple(p) for p in _call("borrowed_builds", name=name, arch=arch)]


def borrowed_owners(name, arch):
    return _call("borrowed_owners", name=name, arch=arch)


def ensure_built(names, arch):
    return _call("ensure_built", names=names, arch=arch)


def manifest(names, arch):
    return {k: Path(v) for k, v in _call("manifest", names=names, arch=arch).items()}


def _env(environ):
    return {k: environ[k] for k in ("VIBEOS_PAYLOAD", "VIBEOS_DOOM") if k in environ}


def resolve_env_payloads(environ=os.environ):
    return _call("resolve_env_payloads", environ=_env(environ))


def stamp_path(build_dir):
    return Path(build_dir) / STAMP_NAME


def read_stamp(build_dir):
    return _call("read_stamp", build_dir=build_dir)


def write_stamp(build_dir, environ=os.environ):
    return _call("write_stamp", build_dir=build_dir, environ=_env(environ))


def resolve_payloads(build_dir, environ=os.environ):
    return _call("resolve_payloads", build_dir=build_dir, environ=_env(environ))


def main(argv=None):
    ap = argparse.ArgumentParser(description=__doc__)
    sub = ap.add_subparsers(dest="cmd", required=True)
    m = sub.add_parser("manifest")
    m.add_argument("--arch", required=True, choices=ARCHES)
    sel = m.add_mutually_exclusive_group(required=True)
    sel.add_argument("--payloads")
    sel.add_argument("--payloads-auto")
    media = sub.add_parser("media-manifest")
    media.add_argument("--output", required=True, type=Path)
    st = sub.add_parser("stamp")
    st.add_argument("--build-dir", required=True)
    sub.add_parser("list")
    args = ap.parse_args(argv)
    if args.cmd == "list":
        for name in available():
            print(name)
        return 0
    if args.cmd == "stamp":
        names = write_stamp(args.build_dir)
        print(f"payloads: {','.join(names) if names else '<none>'}")
        return 0
    if args.cmd == "media-manifest":
        entries = _call("media_manifest", output=args.output, environ=_env(os.environ))
        for dest, src in entries.items():
            print(f"{dest}={src}")
        return 0
    names = (
        resolve_payloads(Path(args.payloads_auto))
        if args.payloads_auto
        else [n.strip() for n in args.payloads.split(",") if n.strip()]
    )
    arch = args.arch
    if names:
        for dest, src in manifest(names, arch).items():
            print(f"{dest}={src}")
    return 0


if __name__ == "__main__":
    sys.exit(main())
