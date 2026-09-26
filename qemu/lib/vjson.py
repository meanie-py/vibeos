#!/usr/bin/env python3
"""JSON for the qemu/ shell toolkit -- stdlib only, in place of `jq`.

Why not jq: it is one more host package, and when it was missing the toolkit did not stop.
`jq ... > file` created the file EMPTY and carried on; `jq ... > tmp && mv` truncated a live
manifest; reads came back as empty strings that looked exactly like "field absent". The
reaper then judged live VMs stale and deleted their locks. Here a write is atomic (tmp +
rename, so a reader never sees half a file) and every failure is a nonzero exit.

  vjson.py obj   [-c] ARGS...          print a JSON object
  vjson.py write PATH ARGS...          atomically write that object to PATH
  vjson.py get   PATH FIELD            print FIELD (empty if absent/null); exit 2 if unreadable
  vjson.py set   PATH FIELD VALUE [--json]   atomically update one field

ARGS: --arg K V (string)  --argjson K V (JSON literal)  --argnum K V (int; "" / "null" -> null)
      --argopt K V (string; "" / "null" -> null)  --arglines K V (list of lines)
"""

from __future__ import annotations

import json
import os
import sys
import tempfile


def _die(msg: str, code: int = 1) -> None:
    print(f"vjson: {msg}", file=sys.stderr)
    sys.exit(code)


def _build(args: list[str]) -> dict:
    out: dict = {}
    i = 0
    while i < len(args):
        if i + 2 >= len(args):
            _die(f"{args[i]} needs a key and a value")
        flag, key, val = args[i], args[i + 1], args[i + 2]
        if flag == "--arg":
            out[key] = val
        elif flag == "--argjson":
            try:
                out[key] = json.loads(val)
            except ValueError:
                _die(f"--argjson {key}: not JSON: {val!r}")
        elif flag == "--argnum":
            out[key] = None if val in ("", "null") else int(val)
        elif flag == "--argopt":
            out[key] = val if val not in ("", "null") else None
        elif flag == "--arglines":
            out[key] = val.split("\n")
        else:
            _die(f"unknown flag {flag!r}")
        i += 3
    return out


def _atomic_write(path: str, obj: object) -> None:
    d = os.path.dirname(os.path.abspath(path))
    fd, tmp = tempfile.mkstemp(dir=d, prefix=".vjson-", suffix=".tmp")
    try:
        with os.fdopen(fd, "w") as f:
            json.dump(obj, f, indent=2)
            f.write("\n")
        os.replace(tmp, path)
    except BaseException:
        try:
            os.unlink(tmp)
        except OSError:
            pass
        raise


def _read(path: str) -> dict:
    try:
        with open(path) as f:
            obj = json.load(f)
    except (OSError, ValueError) as e:
        _die(f"cannot read {path}: {e}", 2)
    if not isinstance(obj, dict):
        _die(f"{path} is not a JSON object", 2)
    return obj  # type: ignore[return-value]


def main(argv: list[str]) -> None:
    if not argv:
        _die("usage: see the module docstring")
    cmd, rest = argv[0], argv[1:]
    if cmd == "obj":
        compact = bool(rest) and rest[0] == "-c"
        obj = _build(rest[1:] if compact else rest)
        print(json.dumps(obj, separators=(",", ":")) if compact else json.dumps(obj, indent=2))
    elif cmd == "write":
        _atomic_write(rest[0], _build(rest[1:]))
    elif cmd == "get":
        v = _read(rest[0]).get(rest[1])
        if v is not None:
            print(json.dumps(v) if isinstance(v, (dict, list, bool)) else v)
    elif cmd == "set":
        path, field, value = rest[0], rest[1], rest[2]
        obj = _read(path)
        if len(rest) > 3 and rest[3] == "--json":
            try:
                obj[field] = json.loads(value)
            except ValueError:
                _die(f"set {field} --json: not JSON: {value!r}")
        else:
            obj[field] = value
        _atomic_write(path, obj)
    else:
        _die(f"unknown command {cmd!r}")


if __name__ == "__main__":
    main(sys.argv[1:])
