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

bootcd's `reactos.cab` is recorded the same way: cmake writes its file list
(`reactos.dff.<config>.dyn`, `"<source>" <dir>` per line) at configure time and cabman
packs `reactos.dff.in` + that list. With `--cells`, `--assemble` re-cabs from the cells'
outputs; otherwise the cab is the build's.

`--assemble` masters with the host tools under build-<arch>/host-tools/bin, which a
cmake-off build's generated SDK provides (tools/sdk_recipe.py records mkisofs, isohybrid
and cabman with the generators). It writes `<out>/<cd>.provenance.json` beside the image:
the source tree's content hash, the cells' (cargo_orchestrate.py --manifest stamps its
out dir) and the image's sha256. `vibeos-cli smoke --media <image>` boots only media
whose stamp matches the tree it is run in.
"""

from __future__ import annotations

import argparse
import hashlib
import json
import re
import shutil
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
CAB = "build:reactos.cab"
CAB_INF = "build:boot/bootdata/packages/reactos.inf"
PACKAGES = "{BUILD}/boot/bootdata/packages"
CELLS_STAMP = ".cells.json"


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


def _output_command(ninja: str, out: str) -> str:
    m = re.search(rf"\nbuild {re.escape(out)} [^\n]*\n  COMMAND = ([^\n]*)\n", ninja)
    if not m:
        raise MediaError(f"build.ninja: no command makes {out}")
    return m.group(1)


def derive_cab(arch: str, build: Path, ninja: str, cells: set[str]) -> dict:
    """reactos.cab: cabman over `reactos.dff.in` + cmake's configure-time file list, first
    for the inf (`-I`) and then for the cab (`-N`, the inf in its reserved area)."""
    dyn = build / f"boot/bootdata/packages/reactos.dff.{CONFIG}.dyn"
    files = []
    for line in dyn.read_text().splitlines():
        m = re.fullmatch(r'"([^"]+)" (.+)', line)
        if not m:
            raise MediaError(f"{dyn}: unmodelled line {line!r}")
        files.append([_enc(m.group(1), arch, cells), m.group(2)])
    return {
        "dff": "repo:boot/bootdata/packages/reactos.dff.in",
        "inf_command": _enc_cmd(_output_command(ninja, "boot/bootdata/packages/reactos.inf"), arch),
        "cab_command": _enc_cmd(_output_command(ninja, "reactos.cab"), arch),
        "files": files,
    }


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
    if any(s == CAB for c in out["cds"].values() for _d, s in c["entries"]):
        out["cab"] = derive_cab(arch, build, ninja, cells)
    return out


#: The rostests a committed manifest carries: the kmtest corpus (`conformance` sweeps it,
#: so every build turns it on) and the shared testdata. apitests and winetests are payload
#: grading -- ENABLE_ROSTESTS[_APITESTS] is sticky in a build dir once a payload run sets
#: it, so a lane that graded anything wrote ~100 test binaries onto the media lists (the
#: 2026-10-18 portwinetest run did, through cargo_cell_gen --convert, silently).
COMMITTED_ROSTESTS = ("kmtests", "testdata")
_ROSTEST_SRC = re.compile(r"(?:^|/)modules/rostests/([^/]+)/")


def committed(m: dict, arch: str = "") -> dict:
    """`m` as it may be committed, or MediaError when the build dir it came from carries
    payload rostests. Not filtered: their import closures come along too (amd64's SysWOW64
    gained i386's hid.dll beside user32_winetest.exe), and nothing in a list says which
    DLL came for a test, so a filtered manifest would be quietly wrong."""
    hits = sorted(
        {
            h.group(1)
            for spec in m["cds"].values()
            for e in spec["entries"]
            if (h := _ROSTEST_SRC.search(e[1].partition(":")[2]))
            and h.group(1) not in COMMITTED_ROSTESTS
        }
    )
    if hits:
        raise MediaError(
            f"build-{arch or '<arch>'} carries payload rostests ({', '.join(hits)}): the "
            "committed lists carry kmtests and testdata only, and these options stay on once a "
            "payload build sets them. Reset them (at the canonical mount, as every build runs), "
            f"rebuild, then write: (cd build-{arch or '<arch>'} && bash ../tools/canon-run.sh "
            "cmake -DENABLE_ROSTESTS=OFF -DENABLE_ROSTESTS_APITESTS=OFF .) && "
            f"./vibeos-cli build --arch {arch or '<arch>'}"
            + (
                ". The winetest A/B control (boot/bootdata/CMakeLists.txt) also stages "
                "build-i386's user32_winetest.exe onto i386 livecd and amd64 syswow64 whenever "
                "that file exists, so remove build-i386/modules/rostests/winetests and rebuild "
                "i386 and amd64 too"
                if "winetests" in hits
                else ""
            )
        )
    return m


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
    if "cab" not in m:
        out.append("}}")
        return "\n".join(out) + "\n"
    cab = m["cab"]
    out.append('}, "cab": {')
    for k in ("dff", "inf_command", "cab_command"):
        out.append(f"  {json.dumps(k)}: {json.dumps(cab[k])},")
    out.append('  "files": [')
    out.append(",\n".join(f"   {json.dumps(e)}" for e in cab["files"]))
    out.append("  ]")
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


def write_list(entries: list, where: dict[str, Path]) -> str:
    lines = []
    for dest, src in entries:
        p = where[src]
        lines.append(str(p) if dest is None else f"{dest}={p}")
    return "\n".join(lines) + "\n"


def source_hash(repo: Path = REPO) -> str:
    """The tree's build-relevant content, as a build stamp records it (docs excluded)."""
    sys.path.insert(0, str(repo / "tests/qemu"))
    from cli import provenance

    return provenance.tree_content_hash(repo, [".", *provenance.NON_BUILD_PATHSPEC])


def cells_source(cells: Path, arch: str) -> str:
    """The source hash a cmake-off cell build was made from (cargo_orchestrate.py stamps
    its out dir after building every cell)."""
    stamp = cells / CELLS_STAMP
    if not stamp.exists():
        raise MediaError(
            f"{stamp} missing: build every cell with cargo_orchestrate.py --arch {arch} --manifest"
        )
    st = json.loads(stamp.read_text())
    if st.get("arch") != arch:
        raise MediaError(f"{cells} holds {st.get('arch')} cells, not {arch}")
    return st["source"]


def _check_tools(cmd: str) -> None:
    for tool in re.findall(r"(\S+/host-tools/bin/\w+)", cmd):
        if not Path(tool).exists():
            raise MediaError(
                f"{tool} missing: a cmake-off build's generated SDK makes it "
                "(cargo_orchestrate.py --manifest --cmake-off), a cmake build makes it too"
            )


def _run(cmd: str) -> None:
    _check_tools(cmd)
    subprocess.run(["bash", "-c", cmd], check=True)


def recab(m: dict, out: Path, build: Path, repo: Path, cells: Path) -> dict[str, Path]:
    """reactos.cab and its inf, packed from the cells' outputs with cmake's two cabman
    commands, into out/cab."""
    cab = m["cab"]
    d = out / "cab"
    if d.exists():
        shutil.rmtree(d)
    d.mkdir(parents=True)
    rows, missing = [], []
    for src, rest in cab["files"]:
        p = resolve(src, build, repo, cells)
        if not p.exists():
            missing.append(str(p))
        rows.append(f'"{p}" {rest}\n')
    if missing:
        raise MediaError(f"reactos.cab: {len(missing)} sources missing, first {missing[:3]}")
    head = resolve(cab["dff"], build, repo, cells).read_text()
    (d / "reactos.dff").write_text(head + "".join(rows))
    sub = {
        PACKAGES: str(d),
        "{BUILD}/reactos.inf": str(d / "reactos.inf"),
        "cd {BUILD} ": f"cd {d} ",
    }
    for key in ("inf_command", "cab_command"):
        cmd = cab[key]
        for k, v in sub.items():
            cmd = cmd.replace(k, v)
        _run(cmd.replace("{BUILD}", str(build)).replace("{REPO}", str(repo)))
    made = {CAB: d / "reactos.cab", CAB_INF: d / "reactos.inf"}
    for p in made.values():
        if not p.exists():
            raise MediaError(f"cabman made no {p}")
    return made


def assemble(arch: str, cd: str, out: Path, cells: Path | None, repo: Path = REPO) -> Path:
    source = source_hash(repo)
    cells_from = None
    if cells is not None:
        cells_from = cells_source(cells, arch)
        if cells_from != source:
            raise MediaError(
                f"{cells} was built from another source tree: rebuild it with "
                f"cargo_orchestrate.py --arch {arch} --manifest"
            )
    m = current(arch, repo)
    spec = m["cds"][cd]
    build = repo / f"build-{arch}"
    out.mkdir(parents=True, exist_ok=True)
    where = {s: resolve(s, build, repo, cells) for _d, s in spec["entries"]}
    if cells is not None and CAB in where:
        where.update(recab(m, out, build, repo, cells))
    missing = [str(p) for p in where.values() if not p.exists()]
    if missing:
        raise MediaError(f"{cd}: {len(missing)} sources missing, first {missing[:3]}")
    lst = out / f"{cd}.lst"
    lst.write_text(write_list(spec["entries"], where))
    cmd = spec["command"]
    if not cmd:
        # arm64's livecd is a list only: qemu/mkramdisk.sh lays it out into ramdisk.img.
        return lst
    cfg = m["config"]
    image = out / f"{cd}.iso"
    cmd = (
        cmd.replace(f"{{BUILD}}/boot/{cd}.{cfg}.lst", str(lst))
        .replace(f"{{BUILD}}/boot/{cd}.stripped.lst", str(out / f"{cd}.stripped.lst"))
        .replace("{BUILD}/boot/stripped ", f"{out / 'stripped'} ")
        .replace(f"{{BUILD}}/{cd}.iso", str(image))
        .replace("{BUILD}", str(build))
        .replace("{REPO}", str(repo))
    )
    image.unlink(missing_ok=True)
    _run(cmd)
    if source_hash(repo) != source:
        raise MediaError("the source tree changed while the image was mastered")
    stamp = {
        "arch": arch,
        "cd": cd,
        "image": image.name,
        "sha256": hashlib.sha256(image.read_bytes()).hexdigest(),
        "source": source,
        "cells": str(cells) if cells else None,
    }
    (out / f"{cd}.provenance.json").write_text(json.dumps(stamp, indent=1) + "\n")
    return image


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
        want = committed(derive(arch), arch)
        text = render(want)
        path = manifest_path(arch)
        if args.write:
            path.parent.mkdir(parents=True, exist_ok=True)
            path.write_text(text)
            print(f"wrote {path.relative_to(REPO)}")
        elif not path.exists() or path.read_text() != text:
            have = load(arch) if path.exists() else {"cds": {}}
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
    try:
        raise SystemExit(main())
    except MediaError as e:
        print(f"media_manifest: {e}", file=sys.stderr)
        raise SystemExit(2) from None
