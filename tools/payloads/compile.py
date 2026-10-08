#!/usr/bin/env python3
"""Payload compiler — resolve tools/payloads/<name>/payload.yaml into build inputs.

Design doc: docs/engineering/payload-harness-design.md. A payload is a
declarative spec for a guest test/demo app: which binaries to graft onto the
boot media, how it autostarts, and how the harness drives + judges it. This
script is the BUILD-TIME consumer: it turns specs into a flat staging manifest
for the two media paths (qemu/mkramdisk.sh for the arm64 ramdisk,
tools/strip_cd_media.py for the x86 livecd). The TEST-TIME consumer is
tests/qemu/test_payload.py, which interprets the same YAML directly.

Usage:
    compile.py manifest --arch arm64 --payloads doom[,httpd,...]
        -> stdout, one line per file:  reactos/system32/doom.exe=/abs/src/path
    compile.py list
        -> available payload names

Spec keys beyond `binaries: [{src, dest, root}]` and `test:`:
  * a binaries entry's `build_arch`: the arch its `{arch}` src is built for, when it is
    not the media's (appstorm64 stages the AMD64 appstorm onto arm64 media);
  * `guest_sets: [{build_dir, dest, roots}]`: programs out of ANOTHER arch's build tree
    plus their complete import closure (tools/fex/wow64-closure.py), staged under
    reactos/<dest> (the x86-64 guest's sysamd64);
  * `test.vm_memory` (read by cli/smoke.py, not here): the guest RAM the payload needs.

Fail-loud rules (the house rule — no silent degradation):
  * unknown payload name, missing payload.yaml, malformed spec  -> exit 1
  * a spec'd src file missing on disk                           -> exit 1
    (with the payload's build_hint so the fix is one paste away)
  * two payloads grafting the same dest                         -> exit 1

Env resolution (shared by both media paths so aliasing lives in ONE place):
    compile.py manifest --arch X --payloads-from-env
reads VIBEOS_PAYLOAD (comma-separated names) and maps the legacy VIBEOS_DOOM=1
flag to the `doom` payload. No payloads selected -> empty output, exit 0
(media stays byte-identical to an unstaged build).

Python: runs under system python3 (>=3.8) — used from cmake custom commands
and shell; keep stdlib + PyYAML only.
"""

from __future__ import annotations

import argparse
import json
import os
import subprocess
import sys
from pathlib import Path

import yaml

PAYLOADS_DIR = Path(__file__).resolve().parent
REPO = PAYLOADS_DIR.parent.parent
ARCHES = ("i386", "amd64", "arm64")


def die(msg: str) -> NoReturn:  # noqa: F821 - py3.8 compat, no typing import needed
    print(f"payloads/compile.py: ERROR: {msg}", file=sys.stderr)
    raise SystemExit(1)


def available() -> list[str]:
    return sorted(p.parent.name for p in PAYLOADS_DIR.glob("*/payload.yaml"))


def load_spec(name: str) -> dict:
    spec_path = PAYLOADS_DIR / name / "payload.yaml"
    if not spec_path.is_file():
        die(
            f"unknown payload {name!r} (no {spec_path}). Available: {', '.join(available()) or '<none>'}"
        )
    try:
        spec = yaml.safe_load(spec_path.read_text())
    except yaml.YAMLError as e:
        die(f"{spec_path}: YAML parse error: {e}")
    if not isinstance(spec, dict):
        die(f"{spec_path}: top level must be a mapping")
    if spec.get("name") != name:
        die(f"{spec_path}: name: {spec.get('name')!r} != directory name {name!r}")
    binaries = spec.get("binaries")
    if not isinstance(binaries, list) or not binaries:
        die(f"{spec_path}: `binaries` must be a non-empty list")
    for b in binaries:
        if not isinstance(b, dict) or "src" not in b or "dest" not in b:
            die(f"{spec_path}: each binaries entry needs `src` and `dest`: {b!r}")
        dest = str(b["dest"])
        if dest.startswith(("/", "\\")) or ".." in dest:
            die(f"{spec_path}: dest must be a relative guest path: {dest!r}")
        if b.get("build_arch", ARCHES[0]) not in ARCHES:
            die(f"{spec_path}: build_arch must be one of {ARCHES}: {b!r}")
        if b.get("root", "reactos") not in ("reactos", "image"):
            die(f"{spec_path}: root must be `reactos` (default) or `image`: {b!r}")
    sets = spec.get("guest_sets", [])
    if not isinstance(sets, list):
        die(f"{spec_path}: `guest_sets` must be a list")
    for g in sets:
        if (
            not isinstance(g, dict)
            or not isinstance(g.get("build_dir"), str)
            or not isinstance(g.get("dest"), str)
            or not isinstance(g.get("roots"), list)
            or not g["roots"]
        ):
            die(
                f"{spec_path}: each guest_sets entry needs `build_dir`, `dest` and a non-empty `roots`: {g!r}"
            )
        if g["dest"].startswith(("/", "\\")) or ".." in g["dest"]:
            die(f"{spec_path}: guest_sets dest must be a relative guest path: {g['dest']!r}")
    return spec


def guest_set(name: str, g: dict) -> dict[str, Path]:
    """A guest_sets entry -> {file name: host path}: its root programs and their
    COMPLETE import closure out of another arch's build tree (a guest process's
    DLLs, e.g. the amd64 set an x86-64 guest on arm64 loads from sysamd64).
    tools/fex/wow64-closure.py computes it and refuses a partial closure, for the
    reason it spells out: a half-bound module crashes at first call."""
    build_dir = REPO / g["build_dir"]
    if not build_dir.is_dir():
        die(f"payload {name!r}: guest set needs {build_dir} (./vibeos-cli build --arch ...)")
    cp = subprocess.run(
        [
            sys.executable,
            str(REPO / "tools/fex/wow64-closure.py"),
            "--include-prestaged",
            "--emit-roots",
            "--build-dir",
            str(build_dir),
            *[str(r) for r in g["roots"]],
        ],
        capture_output=True,
        text=True,
    )
    if cp.returncode != 0:
        die(f"payload {name!r}: guest set closure failed:\n{cp.stderr.strip()}")
    files: dict[str, Path] = {}
    for line in cp.stdout.splitlines():
        if line.strip():
            fname, path = line.split("\t", 1)
            files[fname] = Path(path)
    return files


def manifest(names: list[str], arch: str) -> dict[str, Path]:
    """dest (graft path, e.g. reactos/system32/doom.exe) -> absolute host src."""
    if arch not in ARCHES:
        die(f"unknown arch {arch!r} (expected one of {ARCHES})")
    entries: dict[str, Path] = {}
    owner: dict[str, str] = {}
    for name in names:
        spec = load_spec(name)
        arches = spec.get("arches")
        if arches is not None and arch not in arches:
            die(f"payload {name!r} does not support arch {arch!r} (spec says arches: {arches})")
        hint = spec.get("build_hint", "").format(arch=arch)
        for b in spec["binaries"]:
            src = (REPO / str(b["src"]).format(arch=b.get("build_arch", arch))).resolve()
            # root: reactos (default) grafts under reactos/; root: image grafts
            # at the media root (e.g. the Profiles/.../StartUp autostart folder).
            prefix = "reactos/" if b.get("root", "reactos") == "reactos" else ""
            dest = prefix + str(b["dest"]).replace("\\", "/").lstrip("/")
            if dest in entries:
                die(
                    f"dest collision on {dest!r}: payloads "
                    f"{owner[dest]!r} and {name!r} both graft it"
                )
            if not src.is_file():
                die(
                    f"payload {name!r}: src missing: {src}"
                    + (f"\n  build it first: {hint}" if hint else "")
                )
            entries[dest] = src
            owner[dest] = name
        for g in spec.get("guest_sets", []):
            for fname, src in guest_set(name, g).items():
                dest = "reactos/" + g["dest"].replace("\\", "/").strip("/") + "/" + fname
                if dest in entries and entries[dest] != src:
                    die(
                        f"dest collision on {dest!r}: payloads "
                        f"{owner[dest]!r} and {name!r} both graft it"
                    )
                entries[dest] = src
                owner[dest] = name
    return entries


def stale_binaries(name: str, arch: str) -> list[Path]:
    """The payload's binaries for `arch` that are missing or older than its sources.

    Sources are the payload directory's own files (build.sh, *.c, *.h, ...) plus the
    headers shared by every payload (tools/payloads/*.h). The binaries are gitignored
    build outputs, so without this a worktree that built one once kept staging it across
    every later edit to its source: appstorm's new fastfat raise check (2026-09-26) booted
    as the OLD binary and printed the old SKIP line. The conformance suite had the same
    bug in the gate (gate.py `_ensure_suite_binaries`).
    """
    spec = load_spec(name)
    pdir = PAYLOADS_DIR / name
    srcs = [
        p
        for p in pdir.iterdir()
        if p.is_file()
        and not p.name.startswith(".")
        and p.suffix.lower() not in (".exe", ".dll", ".png", ".log")
    ]
    srcs += list(PAYLOADS_DIR.glob("*.h"))
    newest = max((p.stat().st_mtime for p in srcs), default=0.0)
    resolved_srcs = {p.resolve() for p in srcs}
    out = []
    for b in spec["binaries"]:
        src = (REPO / str(b["src"]).format(arch=b.get("build_arch", arch))).resolve()
        # Only binaries built from this payload's own directory (never a shared artifact
        # like a build-dir DLL, which ninja owns).
        # A `binaries` entry that is itself a committed source (kmtest's
        # kmtest-tests-<arch>.txt) is data, not a build output: build.sh never writes it.
        if src in resolved_srcs:
            continue
        if pdir.resolve() in src.parents and (not src.is_file() or src.stat().st_mtime < newest):
            out.append(src)
    return out


def borrowed_builds(name: str, arch: str) -> list[tuple[str, str]]:
    """(owner, arch) for every other payload whose build outputs `name` stages for `arch`
    media (e.g. every Wine-suite payload borrows tools/payloads/apitest/apitest-sweep-<arch>.exe).
    The arch is the entry's `build_arch` when it has one: appstorm64 stages the AMD64
    appstorm onto arm64 media, so judging the owner's arm64 output would pass a stale
    x86-64 binary. stale_binaries only judges a payload's OWN outputs, so without this a
    borrowed binary was staged however stale: olewinetest booted a sweep runner built before
    sweep.c learned apitest-cwd.txt, and the suites' scratch files went to the read-only CD
    (2026-10-04)."""
    builds: list[tuple[str, str]] = []
    for b in load_spec(name)["binaries"]:
        src = (REPO / str(b["src"]).format(arch=b.get("build_arch", arch))).resolve()
        try:
            owner = src.relative_to(PAYLOADS_DIR.resolve()).parts[0]
        except ValueError:
            continue  # not under tools/payloads (a build-dir artifact ninja owns)
        build = (owner, str(b.get("build_arch", arch)))
        if (
            owner != name
            and build not in builds
            and (PAYLOADS_DIR / owner / "payload.yaml").is_file()
        ):
            builds.append(build)
    return builds


def borrowed_owners(name: str, arch: str) -> list[str]:
    """The owners borrowed_builds names, once each."""
    return list(dict.fromkeys(owner for owner, _ in borrowed_builds(name, arch)))


def ensure_built(names: list[str], arch: str) -> list[str]:
    """Rebuild every selected payload whose binaries are stale (see stale_binaries) with
    its own build.sh, and first every payload whose outputs they borrow
    (borrowed_owners). Returns the names rebuilt; dies loudly if a build fails or a
    stale payload has no build.sh to run."""
    expanded: list[tuple[str, str]] = []
    for name in names:
        for build in borrowed_builds(name, arch) + [(name, arch)]:
            if build not in expanded:
                expanded.append(build)
    rebuilt = []
    for name, arch in expanded:
        stale = stale_binaries(name, arch)
        if not stale:
            continue
        script = PAYLOADS_DIR / name / "build.sh"
        if not script.is_file():
            die(f"payload {name!r}: {stale[0]} is stale or missing and there is no {script}")
        cp = subprocess.run(
            ["bash", str(script), arch], cwd=str(REPO), capture_output=True, text=True
        )
        if cp.returncode != 0:
            tail = "\n".join((cp.stderr or cp.stdout or "(no output)").strip().splitlines()[-5:])
            die(f"payload {name!r}: build.sh {arch} failed:\n{tail}")
        still = stale_binaries(name, arch)
        if still:
            die(f"payload {name!r}: build.sh {arch} succeeded but {still[0]} is still stale")
        rebuilt.append(name)
    return rebuilt


def resolve_env_payloads(environ=os.environ) -> list[str]:
    """VIBEOS_PAYLOAD=a,b,c plus the legacy VIBEOS_DOOM=1 alias -> ordered names."""
    names = [n.strip() for n in environ.get("VIBEOS_PAYLOAD", "").split(",") if n.strip()]
    if environ.get("VIBEOS_DOOM", "0") == "1" and "doom" not in names:
        names.append("doom")
    return names


STAMP_NAME = "payload-selection.json"


def stamp_path(build_dir: Path) -> Path:
    return Path(build_dir) / STAMP_NAME


def write_stamp(build_dir: Path, environ=os.environ) -> list[str]:
    """Persist the env's payload selection into the build dir (ALWAYS — set or
    cleared). Media content must be a declared property of the build tree, not
    of whatever env a later rebuild happens to run under: the 2026-07-07 doom
    clobber (smoke's net stage re-ran `ninja livecd` without VIBEOS_PAYLOAD and
    silently produced doom-less media between two payload-gated boots) is the
    incident this prevents."""
    names = resolve_env_payloads(environ)
    stamp_path(build_dir).write_text(json.dumps({"payloads": names}) + "\n")
    return names


def read_stamp(build_dir: Path) -> list[str]:
    try:
        data = json.loads(stamp_path(build_dir).read_text())
    except OSError:
        return []
    except ValueError as e:
        die(f"corrupt {stamp_path(build_dir)}: {e} (delete it or re-run vibeos-cli build)")
    names = data.get("payloads", [])
    if not isinstance(names, list):
        die(f"corrupt {stamp_path(build_dir)}: `payloads` must be a list")
    return [str(n) for n in names]


def resolve_payloads(build_dir: Path | None, environ=os.environ) -> list[str]:
    """Env wins when the operator expressed ANY selection (VIBEOS_PAYLOAD set —
    even set-empty — or the VIBEOS_DOOM alias); otherwise fall back to the
    build-dir stamp so env-less rebuilds (conftest, raw ninja, the smoke net
    stage) reproduce the same media instead of silently reverting it."""
    if "VIBEOS_PAYLOAD" in environ or environ.get("VIBEOS_DOOM", "0") == "1":
        return resolve_env_payloads(environ)
    if build_dir is not None:
        return read_stamp(build_dir)
    return []


def main(argv: list[str] | None = None) -> int:
    ap = argparse.ArgumentParser(description=__doc__.splitlines()[0])
    sub = ap.add_subparsers(dest="cmd", required=True)

    m = sub.add_parser("manifest", help="print dest=src staging lines")
    m.add_argument("--arch", required=True, choices=ARCHES)
    sel = m.add_mutually_exclusive_group(required=True)
    sel.add_argument("--payloads", help="comma-separated payload names")
    sel.add_argument(
        "--payloads-auto",
        metavar="BUILD_DIR",
        help="resolve from VIBEOS_PAYLOAD/VIBEOS_DOOM env when set, else from "
        f"BUILD_DIR/{STAMP_NAME} (written by vibeos-cli build)",
    )

    st = sub.add_parser(
        "stamp",
        help=f"persist the env's payload selection to BUILD_DIR/{STAMP_NAME}",
    )
    st.add_argument("--build-dir", required=True)

    sub.add_parser("list", help="print available payload names")

    args = ap.parse_args(argv)

    if args.cmd == "list":
        for n in available():
            print(n)
        return 0

    if args.cmd == "stamp":
        names = write_stamp(Path(args.build_dir))
        print(f"payloads: {','.join(names) if names else '<none>'}")
        return 0

    if args.payloads_auto:
        names = resolve_payloads(Path(args.payloads_auto))
    else:
        names = [n.strip() for n in args.payloads.split(",") if n.strip()]
    if not names:
        return 0  # nothing selected -> nothing staged, media byte-identical

    for dest, src in manifest(names, args.arch).items():
        print(f"{dest}={src}")
    return 0


if __name__ == "__main__":
    sys.exit(main())
