#!/usr/bin/env python3
"""The boot media as data: `boot/media/<arch>.json`, and mastering an image from it.

cmake already masters each CD from a path list: every `add_cd_file` appends a
`<dest>=<source>` line to `build-<arch>/boot/<cd>.<config>.lst` at configure time, and the
`<cd>` target runs `strip_cd_media.py` over the list, mkisofs over the result and (on x86)
isohybrid. This tool records those lists and commands with each source named by what
makes it, so the media no longer needs cmake's 1,400 configure-time appends to exist:

  - `cell:<out>`   a cargo cell's output (sdk/lib/rust/cells/<arch>.jsonl), build-relative
  - `build:<rel>`  anything else the build makes
  - `repo:<rel>`   a source-tree file

    python3 tools/media_manifest.py --write            # from each configured build-<arch>
    python3 tools/media_manifest.py --check            # committed vs those builds
    python3 tools/media_manifest.py --assemble --arch amd64 --cd livecd --out DIR \\
        [--cells DIR]    # take the cells' outputs from a cmake-off build (cargo_orchestrate.py
                         # --manifest --out-dir DIR) instead of build-<arch>

`--assemble` writes the list and masters the image with the build's own host tools
(build-<arch>/host-tools/bin): self-hosting mkisofs/isohybrid/cabman is owed, and their
absence is a hard failure, not a fallback. bootcd's reactos.cab is a build product here;
re-cabbing cells from `--cells` is owed too, so `--cells` is refused for a CD that carries
one.
"""

from __future__ import annotations

import argparse
import json
import re
import subprocess
import sys
from pathlib import Path

REPO = Path(__file__).resolve().parent.parent
sys.path.insert(0, str(REPO / "tools"))
import cell_manifest  # noqa: E402

ARCHES = ("i386", "amd64", "arm64")
CDS = ("livecd", "bootcd", "bootcdregtest")
CANON = "/tmp/vibeos"
CONFIG = "Debug"


class MediaError(Exception):
    pass


def manifest_path(arch: str, repo: Path = REPO) -> Path:
    return repo / f"boot/media/{arch}.json"


def _enc(path: str, arch: str, cells: set[str]) -> str:
    b = f"{CANON}/build-{arch}/"
    if path.startswith(b):
        rel = path[len(b) :]
        return f"cell:{rel}" if rel in cells else f"build:{rel}"
    if path.startswith(CANON + "/"):
        return f"repo:{path[len(CANON) + 1 :]}"
    raise MediaError(f"media source outside the tree: {path}")


def _enc_cmd(cmd: str, arch: str) -> str:
    return cmd.replace(f"{CANON}/build-{arch}", "{BUILD}").replace(CANON, "{REPO}")


def _command(ninja: str, cd: str) -> str:
    m = re.search(rf"\nbuild boot/CMakeFiles/{cd} [^\n]*\n  COMMAND = ([^\n]*)\n", ninja)
    return m.group(1) if m else ""


def derive(arch: str, repo: Path = REPO) -> dict:
    build = repo / f"build-{arch}"
    ninja = (build / "build.ninja").read_text()
    cells = {o["out"] for c in cell_manifest.current(arch, repo) for o in c["outputs"]}
    out: dict = {"config": CONFIG, "cds": {}}
    for cd in CDS:
        lst = build / f"boot/{cd}.{CONFIG}.lst"
        if not lst.exists():
            raise MediaError(f"{lst} missing: configure with ./vibeos-cli build --arch {arch}")
        entries = []
        for line in lst.read_text().splitlines():
            dest, sep, src = line.rpartition("=")
            entries.append([dest if sep else None, _enc(src, arch, cells)])
        out["cds"][cd] = {"entries": entries, "command": _enc_cmd(_command(ninja, cd), arch)}
    return out


def render(m: dict) -> str:
    """One entry per line, so a module's placement changes one line of the diff."""
    out = ['{"config": ' + json.dumps(m["config"]) + ', "cds": {']
    for i, (cd, spec) in enumerate(sorted(m["cds"].items())):
        out.append(f" {json.dumps(cd)}: {{")
        out.append(f'  "command": {json.dumps(spec["command"])},')
        out.append('  "entries": [')
        rows = [f"   {json.dumps(e)}" for e in spec["entries"]]
        out.append(",\n".join(rows))
        out.append("  ]")
        out.append(" }" + ("," if i < len(m["cds"]) - 1 else ""))
    out.append("}}")
    return "\n".join(out) + "\n"


def load(arch: str, repo: Path = REPO) -> dict:
    path = manifest_path(arch, repo)
    if not path.exists():
        raise MediaError(f"{path} missing: python3 tools/media_manifest.py --write")
    return json.loads(path.read_text())


def current(arch: str, repo: Path = REPO) -> dict:
    """The media as the build lists them: derived from build-<arch> when it is configured
    (cmake still owns placement), else the committed manifest."""
    if (repo / f"build-{arch}/build.ninja").exists():
        return derive(arch, repo)
    return load(arch, repo)


def resolve(src: str, build: Path, repo: Path, cells: Path | None) -> Path:
    kind, _, rel = src.partition(":")
    if kind == "cell":
        return (cells or build) / rel
    if kind == "build":
        return build / rel
    if kind == "repo":
        return repo / rel
    raise MediaError(f"unknown media source {src!r}")


def write_list(entries: list, build: Path, repo: Path, cells: Path | None) -> str:
    lines = []
    for dest, src in entries:
        p = resolve(src, build, repo, cells)
        lines.append(str(p) if dest is None else f"{dest}={p}")
    return "\n".join(lines) + "\n"


def assemble(arch: str, cd: str, out: Path, cells: Path | None, repo: Path = REPO) -> Path:
    m = current(arch, repo)
    spec = m["cds"][cd]
    build = repo / f"build-{arch}"
    if cells is not None and any(s == "build:reactos.cab" for _d, s in spec["entries"]):
        raise MediaError(f"{cd} carries reactos.cab, which is not re-cabbed from --cells yet")
    missing = [
        str(p) for _d, s in spec["entries"] if not (p := resolve(s, build, repo, cells)).exists()
    ]
    if missing:
        raise MediaError(f"{cd}: {len(missing)} sources missing, first {missing[:3]}")
    out.mkdir(parents=True, exist_ok=True)
    lst = out / f"{cd}.lst"
    lst.write_text(write_list(spec["entries"], build, repo, cells))
    cmd = spec["command"]
    if not cmd:
        # arm64's livecd is a list only: qemu/mkramdisk.sh lays it out into ramdisk.img.
        return lst
    cfg = m["config"]
    cmd = (
        cmd.replace(f"{{BUILD}}/boot/{cd}.{cfg}.lst", str(lst))
        .replace(f"{{BUILD}}/boot/{cd}.stripped.lst", str(out / f"{cd}.stripped.lst"))
        .replace("{BUILD}/boot/stripped ", f"{out / 'stripped'} ")
        .replace(f"{{BUILD}}/{cd}.iso", str(out / f"{cd}.iso"))
        .replace("{BUILD}", str(build))
        .replace("{REPO}", str(repo))
    )
    for tool in re.findall(r"(\S+/host-tools/bin/\w+)", cmd):
        if not Path(tool).exists():
            raise MediaError(f"{tool} missing: self-hosting it is owed, build it with cmake")
    subprocess.run(["bash", "-c", cmd], check=True)
    return out / f"{cd}.iso"


def main(argv: list[str] | None = None) -> int:
    ap = argparse.ArgumentParser(
        description=__doc__, formatter_class=argparse.RawDescriptionHelpFormatter
    )
    g = ap.add_mutually_exclusive_group(required=True)
    g.add_argument("--write", action="store_true")
    g.add_argument("--check", action="store_true")
    g.add_argument("--assemble", action="store_true")
    ap.add_argument("--arch", action="append", choices=ARCHES)
    ap.add_argument("--cd", choices=CDS, default="livecd")
    ap.add_argument("--out", type=Path)
    ap.add_argument("--cells", type=Path)
    args = ap.parse_args(argv)
    if args.assemble:
        if not args.out or not args.arch or len(args.arch) != 1:
            ap.error("--assemble needs one --arch and --out")
        if str(REPO) != CANON:
            # The lists and commands name the canonical mount, as every build does.
            import os

            argv = list(sys.argv[1:] if argv is None else argv)
            argv = [str(Path(a).resolve()) if Path(a).exists() else a for a in argv]
            os.chdir(REPO)
            os.execvp(
                "bash",
                [
                    "bash",
                    str(REPO / "tools/canon-run.sh"),
                    sys.executable,
                    f"{CANON}/tools/{Path(__file__).name}",
                    *argv,
                ],
            )
        print(assemble(args.arch[0], args.cd, args.out.resolve(), args.cells))
        return 0
    rc = 0
    for arch in args.arch or ARCHES:
        text = render(derive(arch))
        path = manifest_path(arch)
        if args.write:
            path.parent.mkdir(parents=True, exist_ok=True)
            path.write_text(text)
            print(f"wrote {path.relative_to(REPO)}")
        elif not path.exists() or path.read_text() != text:
            have = load(arch) if path.exists() else {"cds": {}}
            want = derive(arch)
            parts = []
            for cd in CDS:
                h = {tuple(e) for e in have["cds"].get(cd, {}).get("entries", [])}
                w = {tuple(e) for e in want["cds"][cd]["entries"]}
                if h != w:
                    parts.append(f"{cd} +{len(w - h)} -{len(h - w)}")
            print(
                f"{path.relative_to(REPO)}: stale: {', '.join(parts) or 'order or command'} "
                "(python3 tools/media_manifest.py --write)"
            )
            rc = 1
    return rc


if __name__ == "__main__":
    raise SystemExit(main())
