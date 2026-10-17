#!/usr/bin/env python3
"""The cargo cells a build links, as data: `sdk/lib/rust/cells/<arch>.jsonl`.

`add_cargo_module` (sdk/cmake/gcc.cmake) turns each cell into one ninja custom command:
an env, `cargo build -p <crate> --bin ...`, a copy of each bin's `.exe` to its output,
and the module kind's post-link. This tool reads those commands back out of
`build-<arch>/build.ninja` into one JSON line per cell, so a build with no cmake
(tools/cargo_orchestrate.py --manifest) knows what to build, where it goes and how it is
fixed up, and the media assembly (tools/media_manifest.py) knows which files cells make.

    python3 tools/cell_manifest.py --write      # regenerate from each build-<arch>
    python3 tools/cell_manifest.py --check      # committed vs the configured graphs

What is recorded is what the command does, never the machine it ran on: the artifact
store's env and the canonical mount stay out.
"""

from __future__ import annotations

import argparse
import json
import re
import shlex
import sys
from pathlib import Path

REPO = Path(__file__).resolve().parent.parent
ARCHES = ("i386", "amd64", "arm64")
CANON = "/tmp/vibeos"
# The env add_cargo_module sets that belongs to the cell, not to the host.
CELL_ENV = ("RUSTFLAGS", "RUSTUP_TOOLCHAIN", "VIBEOS_PIC_HAL")

STMT = re.compile(
    r"\nbuild ([^\n]*?): CUSTOM_COMMAND[^\n]*\n  COMMAND = ([^\n]*)\n"
    r"  DESC = Cargo-linking ([^\n]*)"
)


class ManifestError(Exception):
    pass


def manifest_path(arch: str, repo: Path = REPO) -> Path:
    return repo / f"sdk/lib/rust/cells/{arch}.jsonl"


def _build_rel(path: str, arch: str) -> str:
    b = f"{CANON}/build-{arch}/"
    if not path.startswith(b):
        raise ManifestError(f"cell output outside build-{arch}: {path}")
    return path[len(b) :]


def _post(cmd: list[str], out: str, arch: str) -> str:
    """One post-link step, named by what it does to `out`."""
    if cmd[-1] != out:
        raise ManifestError(f"post-link step not on its output: {' '.join(cmd)}")
    if cmd[0].endswith("/pefixup"):
        return f"pefixup {cmd[1]}"
    if cmd[:4] == ["/usr/bin/cmake", "-E", "env", "python3"] and cmd[4].endswith(
        "sdk/tools/fix_lld_forwarding.py"
    ):
        return "fix_lld_forwarding"
    if cmd[0].endswith("/llvm-strip"):
        return "strip " + " ".join(cmd[1:-1])
    raise ManifestError(f"unmodelled post-link step: {' '.join(cmd)}")


def parse_cell(outs: str, command: str, arch: str) -> dict:
    steps = [shlex.split(s) for s in command.split(" && ")]
    if steps[0][:1] != ["cd"]:
        raise ManifestError(f"cargo command without its cd: {command[:120]}")
    run = steps[1]
    if run[:3] != ["/usr/bin/cmake", "-E", "env"]:
        raise ManifestError(f"cargo command without its env: {command[:120]}")
    at = run.index("cargo")
    env = dict(kv.split("=", 1) for kv in run[3:at])
    args = run[at:]
    cell: dict = {
        "crate": args[args.index("-p") + 1],
        "target": args[args.index("--target") + 1].replace(CANON + "/", ""),
        "build_std": "build-std=core,alloc,compiler_builtins" in args,
        "env": {k: env[k] for k in CELL_ENV if k in env},
        "target_dir": Path(env["CARGO_TARGET_DIR"]).name,
        "bins": "all"
        if "--bins" in args
        else [args[i + 1] for i, a in enumerate(args) if a == "--bin"],
        "outputs": [],
    }
    i = 2
    while i < len(steps):
        s = steps[i]
        if s[:3] == ["/usr/bin/cmake", "-E", "copy"]:
            src, dst = s[3], s[4]
            post = []
            i += 1
            while (
                i < len(steps)
                and steps[i][-1] == dst
                and steps[i][:3] != ["/usr/bin/cmake", "-E", "copy"]
            ):
                post.append(_post(steps[i], dst, arch))
                i += 1
            cell["outputs"].append(
                {"bin": Path(src).stem, "out": _build_rel(dst, arch), "post": post}
            )
            continue
        i += 1  # the depfile bookkeeping
    declared = sorted(
        _build_rel(f"{CANON}/build-{arch}/{o}", arch) for o in outs.split("|")[0].split()
    )
    if sorted(o["out"] for o in cell["outputs"]) != declared:
        raise ManifestError(f"{cell['crate']}: copies do not match the declared outputs")
    return cell


def derive(arch: str, repo: Path = REPO) -> list[dict]:
    ninja = repo / f"build-{arch}/build.ninja"
    if not ninja.exists():
        raise ManifestError(f"{ninja} missing: configure with ./vibeos-cli build --arch {arch}")
    cells = [parse_cell(o, c, arch) for o, c, _d in STMT.findall(ninja.read_text())]
    crates = [c["crate"] for c in cells]
    if len(set(crates)) != len(crates):
        raise ManifestError(f"build-{arch}: a crate links twice")
    return sorted(cells, key=lambda c: c["crate"])


def render(cells: list[dict]) -> str:
    return "".join(json.dumps(c, sort_keys=True) + "\n" for c in cells)


def load(arch: str, repo: Path = REPO) -> list[dict]:
    path = manifest_path(arch, repo)
    if not path.exists():
        raise ManifestError(f"{path} missing: python3 tools/cell_manifest.py --write")
    return [json.loads(line) for line in path.read_text().splitlines() if line.strip()]


def main(argv: list[str] | None = None) -> int:
    ap = argparse.ArgumentParser(
        description=__doc__, formatter_class=argparse.RawDescriptionHelpFormatter
    )
    g = ap.add_mutually_exclusive_group(required=True)
    g.add_argument("--write", action="store_true")
    g.add_argument("--check", action="store_true")
    ap.add_argument("--arch", action="append", choices=ARCHES)
    args = ap.parse_args(argv)
    rc = 0
    for arch in args.arch or ARCHES:
        text = render(derive(arch))
        path = manifest_path(arch)
        if args.write:
            path.parent.mkdir(parents=True, exist_ok=True)
            path.write_text(text)
            print(f"wrote {path.relative_to(REPO)} ({text.count(chr(10))} cells)")
        elif not path.exists() or path.read_text() != text:
            print(
                f"{path.relative_to(REPO)}: stale against build-{arch}/build.ninja "
                "(python3 tools/cell_manifest.py --write)",
                file=sys.stderr,
            )
            rc = 1
    return rc


if __name__ == "__main__":
    raise SystemExit(main())
