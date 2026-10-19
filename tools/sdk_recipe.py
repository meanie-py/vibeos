#!/usr/bin/env python3
"""The generated SDK a cell build reads, as a recipe that needs no cmake: rung 6b.

A cell's C and resources read headers cmake GENERATES into `build-<arch>/sdk/include`
(psdk's widl headers, the xdk headers), and some cells compile or embed generated files
(widl RPC stubs and typelibs, flex scanners, message-compiler output). This tool records
the commands that make them, in dependency order, from ninja itself
(`ninja -t commands`), and replays them into a directory that stands in for the build
tree (`$VIBEOS_GEN_DIR`, which `@build/` names in a cmake-off build):

    python3 tools/sdk_recipe.py --write          # sdk/lib/rust/generated/<arch>.cmds
    python3 tools/sdk_recipe.py --check
    python3 tools/sdk_recipe.py --run --arch amd64 --gen DIR

The host tools those commands run (widl, hpp, utf16le, ...) and the ones that master the
boot media (mkisofs, isohybrid, cabman) are themselves the host-tools project's commands,
recorded the same way (`<arch>.host.cmds`) and replayed into `DIR/host-tools/bin`. Only generator commands are kept: compiles, archives, import
libraries (the engine makes its own) and cmake's ExternalProject bookkeeping are not.
"""

from __future__ import annotations

import argparse
import re
import shlex
import subprocess
import sys
from pathlib import Path

REPO = Path(__file__).resolve().parent.parent
ARCHES = ("i386", "amd64", "arm64")
CANON = "/tmp/vibeos"
# geninc and gendib are not here: the engine self-hosts both (ks*.inc, win32k's DIB tables).
GENERATORS = ("widl", "hpp", "utf16le", "wmc", "bin2c", "txt2nls")
HOST_TOOLS = ("widl", "hpp", "utf16le", "wmc", "bin2c", "txt2nls")
# The tools that master the boot media (tools/media_manifest.py --assemble).
MEDIA_TOOLS = ("cabman", "isohybrid", "mkisofs")
CELL_EDGE = re.compile(
    r"\nbuild ([^\n]*?): CUSTOM_COMMAND([^\n]*)\n  COMMAND = [^\n]*\n  DESC = Cargo-linking"
)


class RecipeError(Exception):
    pass


def recipe_path(arch: str, host: bool = False, repo: Path = REPO) -> Path:
    return repo / f"sdk/lib/rust/generated/{arch}{'.host' if host else ''}.cmds"


def _ninja_commands(build: Path, targets: list[str]) -> list[str]:
    r = subprocess.run(
        [
            "bash",
            str(REPO / "tools/canon-run.sh"),
            "ninja",
            "-C",
            str(build),
            "-t",
            "commands",
            *targets,
        ],
        capture_output=True,
        text=True,
    )
    if r.returncode:
        raise RecipeError(f"ninja -t commands in {build}: {r.stderr.strip()[-400:]}")
    return r.stdout.splitlines()


def _tool(line: str) -> str:
    body = line.split(" && ", 1)[1] if line.startswith("cd ") else line
    first = shlex.split(body)[0] if body.strip() else ""
    if first == "/usr/bin/cmake":
        return "cmake -E " + body.split()[2]
    return Path(first).name


def _keep(line: str) -> bool:
    if "host-jobs" in line or "ExternalProject" in line:
        return False
    tool = _tool(line)
    # mc: binutils' windmc on the host (add_message_headers), not a host-tools build.
    if tool.endswith("-windmc"):
        return True
    # rosload's 16-bit drvmap.S: clang cannot assemble .code16, so cmake runs mingw gcc
    # on it (compile_asm_with_gcc) into an object the cell links from the build tree.
    if tool.endswith("gcc") and " -x assembler-with-cpp " in line:
        return True
    return (
        tool in GENERATORS
        or tool in ("flex", "bison")
        or tool
        in (
            "cmake -E copy",
            "cmake -E copy_if_different",
            "cmake -E touch",
            "cmake -E make_directory",
        )
    )


HOST_JOBS = re.compile(r'"?\S*/\.cache/host-jobs/host-jobs"? --kind \w+ -- ')


def _unwrap(line: str) -> str:
    """The fleet's host-jobs admission wrapper is how this host scheduled the command, not
    part of it (and a fresh checkout has not built it)."""
    return HOST_JOBS.sub("", line)


def _cell_deps(ninja: str) -> list[str]:
    deps: list[str] = []
    for m in CELL_EDGE.finditer(ninja):
        for x in re.split(r"\s+", m.group(2).replace("||", " ").replace("|", " ")):
            if x and not x.startswith("/") and x not in deps and "rust" not in x:
                deps.append(x)
    return deps


def _encode(line: str, arch: str) -> str:
    b = f"{CANON}/build-{arch}"
    return (
        line.replace(f"{b}/host-tools/bin", "{TOOLS}")
        .replace(b, "{BUILD}")
        .replace(CANON, "{REPO}")
    )


def _factor(lines: list[str]) -> list[str]:
    """Every widl command repeats the same include/define run: name it once, as the
    recipe's first line `{WIDL} = <tokens>`, and write `{WIDL}` where it was."""
    sys.path.insert(0, str(REPO / "tools"))
    from cargo_cell_gen import _majority_run

    widl = [ln.split(" ") for ln in lines if _tool(ln) == "widl"]
    run = _majority_run(widl, 0.9) if widl else []
    if len(run) < 4:
        return lines
    out = []
    for ln in lines:
        t = ln.split(" ")
        for i in range(len(t) - len(run) + 1):
            if t[i : i + len(run)] == run:
                t = [*t[:i], "{WIDL}", *t[i + len(run) :]]
                break
        out.append(" ".join(t))
    return ["{WIDL} = " + " ".join(run), *out]


def derive(arch: str, repo: Path = REPO) -> tuple[list[str], list[str]]:
    build = repo / f"build-{arch}"
    ninja = (build / "build.ninja").read_text()
    targets = ["sdk/include/psdk/psdk", "sdk/include/xdk/xdk", *_cell_deps(ninja)]
    sdk, seen = [], set()
    for line in _ninja_commands(build, targets):
        if _keep(line) and line not in seen:
            seen.add(line)
            sdk.append(_encode(line, arch))
    sdk = _factor(sdk)
    used = sorted({m for line in sdk for m in re.findall(r"\{TOOLS\}/(\w+)", line)})
    unknown = set(used) - set(HOST_TOOLS)
    if unknown:
        raise RecipeError(
            f"build-{arch}: generator commands run host tools not recorded: {unknown}"
        )
    host = [
        _unwrap(line)
        .replace(f"{CANON}/build-{arch}/host-tools/bin", "{TOOLS}")
        .replace(CANON, "{REPO}")
        for line in _ninja_commands(build / "host-tools/bin", [*used, *MEDIA_TOOLS])
    ]
    return sdk, host


def run(arch: str, gen: Path, repo: Path = REPO) -> None:
    """Replay the host-tools recipe into gen/host-tools/bin, then the SDK recipe into gen."""
    tools = gen / "host-tools/bin"
    subs = {"{TOOLS}": str(tools), "{BUILD}": str(gen), "{REPO}": str(repo)}
    for host in (True, False):
        path = recipe_path(arch, host, repo)
        if not path.exists():
            raise RecipeError(f"{path} missing: python3 tools/sdk_recipe.py --write")
        lines = path.read_text().splitlines()
        widl = ""
        if lines and lines[0].startswith("{WIDL} = "):
            widl, lines = lines[0][len("{WIDL} = ") :], lines[1:]
        for line in lines:
            line = line.replace("{WIDL}", widl)
            for k, v in subs.items():
                line = line.replace(k, v)
            # ninja makes an output's directory; a replay has to. The host-tools project's
            # commands name theirs relative to its build root, where ninja runs them.
            cwd = tools if host else gen
            for d in re.findall(rf"(?:^cd |[ =])({re.escape(str(gen))}/\S*)", line):
                p = Path(d)
                (p if line.startswith(f"cd {d} ") else p.parent).mkdir(parents=True, exist_ok=True)
            for tok in shlex.split(line.replace("&&", " ")):
                rel = tok.split("=", 1)[-1] if tok.startswith("-") else tok
                if "/" in rel and not rel.startswith(("/", "-", "$")):
                    (cwd / rel).parent.mkdir(parents=True, exist_ok=True)
            cwd.mkdir(parents=True, exist_ok=True)
            r = subprocess.run(["bash", "-c", line], capture_output=True, text=True, cwd=cwd)
            if r.returncode:
                raise RecipeError(f"{path.name}: failed: {line[:300]}\n{r.stderr[-800:]}")


def main(argv: list[str] | None = None) -> int:
    ap = argparse.ArgumentParser(
        description=__doc__, formatter_class=argparse.RawDescriptionHelpFormatter
    )
    g = ap.add_mutually_exclusive_group(required=True)
    g.add_argument("--write", action="store_true")
    g.add_argument("--check", action="store_true")
    g.add_argument("--run", action="store_true")
    ap.add_argument("--arch", action="append", choices=ARCHES)
    ap.add_argument("--gen", type=Path)
    args = ap.parse_args(argv)
    if args.run:
        if not args.gen or not args.arch or len(args.arch) != 1:
            ap.error("--run needs one --arch and --gen")
        run(args.arch[0], args.gen.resolve())
        return 0
    rc = 0
    for arch in args.arch or ARCHES:
        sdk, host = derive(arch)
        for is_host, lines in ((False, sdk), (True, host)):
            path = recipe_path(arch, is_host)
            text = "\n".join(lines) + "\n"
            if args.write:
                path.parent.mkdir(parents=True, exist_ok=True)
                path.write_text(text)
                print(f"wrote {path.relative_to(REPO)} ({len(lines)} commands)")
            elif not path.exists() or path.read_text() != text:
                print(f"{path.relative_to(REPO)}: stale (python3 tools/sdk_recipe.py --write)")
                rc = 1
    return rc


if __name__ == "__main__":
    raise SystemExit(main())
