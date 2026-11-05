"""Argument routing and compatibility imports for native build benchmarking."""

from __future__ import annotations

import subprocess
from pathlib import Path

from tools.build_infra_compat import _inherited_descriptors, invoke

ARCHES = ("i386", "amd64", "arm64")
REPO = Path(__file__).resolve().parents[3]


def _read_meminfo(path: Path = Path("/proc/meminfo")) -> dict[str, int]:
    return invoke("build_bench_meminfo", {"path": path})


def _read_vmstat(path: Path = Path("/proc/vmstat")) -> dict[str, int]:
    return invoke("build_bench_vmstat", {"path": path})


def _read_pressure(path: Path = Path("/proc/pressure")) -> dict:
    return invoke("build_bench_pressure", {"path": path})


def _git_state(repo: Path) -> dict:
    return invoke("build_bench_git", {}, root=repo)


def _media_path(repo: Path, arch: str) -> Path:
    return Path(invoke("build_bench_media_path", {"arch": arch}, root=repo))


def _run_child(name: str, command: list[str], repo: Path, log: Path) -> dict:
    return invoke(
        "build_bench_run_child", {"name": name, "command": command, "log": log}, root=repo
    )


def add_parser(sub) -> None:
    parser = sub.add_parser("build-bench", help="measure concurrent build and lint resource use")
    parser.add_argument("--arches", nargs="+", choices=ARCHES, required=True)
    parser.add_argument(
        "--clean",
        action="store_true",
        help="clean build outputs; shared compiler caches remain warm",
    )
    parser.add_argument("--repeat", type=int, default=1)
    parser.add_argument(
        "--output", type=Path, help="summary JSON path (default: unique .cache/build-bench file)"
    )
    parser.set_defaults(fn=run)


def run(args, repo: Path) -> int:
    argv = [
        str(REPO / "tools/build-infra"),
        "--root",
        str(repo),
        "build-bench",
        "--arches",
        *args.arches,
    ]
    if args.clean:
        argv.append("--clean")
    argv.extend(("--repeat", str(args.repeat)))
    if args.output is not None:
        argv.extend(("--output", str(args.output)))
    return subprocess.call(argv, pass_fds=_inherited_descriptors())
