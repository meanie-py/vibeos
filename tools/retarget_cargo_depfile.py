"""Retarget Cargo's emitted dependency rule to CMake's copied output.

Preserve the compiler-emitted dependency body verbatim: do not guess which Rust,
embedded, or build-script inputs matter. Linux canonical paths and spaces are
supported; ambiguous output spellings fail rather than emit a broken Ninja rule.
"""

import argparse
from pathlib import Path


def escaped_output(path):
    text = str(path)
    if any(char in text for char in "\r\n\t\\:#$"):
        raise ValueError(f"unsupported depfile output spelling: {text!r}")
    return text.replace(" ", "\\ ")


def retarget(text, cargo_output, copied_output):
    prefix = escaped_output(cargo_output) + ":"
    if not text.startswith(prefix) or not text[len(prefix) :].startswith((" ", "\n", "\r")):
        raise ValueError("Cargo depfile does not start with the expected output rule")
    return escaped_output(copied_output) + ":" + text[len(prefix) :]


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--source", type=Path, required=True)
    parser.add_argument("--cargo-output", type=Path, required=True)
    parser.add_argument("--target", type=Path, required=True)
    parser.add_argument("--output", type=Path, required=True)
    args = parser.parse_args()
    try:
        text = retarget(args.source.read_text(), args.cargo_output, args.target)
        # Per-edge output, never a shared worktree or package-cache file.
        args.output.write_text(text)
    except (OSError, ValueError) as error:
        parser.exit(2, f"cargo-depfile: {error}\n")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
