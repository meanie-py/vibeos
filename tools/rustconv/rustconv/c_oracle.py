"""The pre-flip C of a cohab-flipped module, as an opt-in sibling target: the runtime oracle.

A flip comments every translated TU out of the module's CMakeLists
(`# partlist.c  # replaced by Rust (...)`) and links the Rust archive instead, so the C
stays on disk but is never built again. That C is the best oracle the Rust will ever have:
run both on the same input and diff. This writes `c_oracle.cmake` beside the CMakeLists
and includes it from there:

    add_executable(diskpart_c EXCLUDE_FROM_ALL <the C sources> diskpart.rc)
    set_module_type(diskpart_c win32cui UNICODE)  ...

Every command that configures a flipped target is replayed against `<target>_c`, with the
Rust pieces (`rust_*`, `${x_rust_LIB}`) dropped and the source variables swapped for the
pre-flip lists. A target that LINKS a flipped target gets an oracle too (setuplib.dll
links setuplib_static), so the whole chain is C. `EXCLUDE_FROM_ALL`: nothing builds or
ships it unless asked (`ninja diskpart_c`), and it is a separate file so the source-list
parsers (cohab census, profile_derive) still see only what the module ships.

    python -m rustconv.c_oracle base/system/diskpart [--check]

A module cargo links (`add_cargo_module`, tools/cargo_cell_gen.py) keeps its flipped-TU
comments but no longer says how cmake built it: the importlibs, module type and defines moved
into the cargo cell. Its oracle is generated from the CMakeLists as it stood just before the
commit that introduced `add_cargo_module` (`git log -S`), the last text that did, with the
current text's flipped TUs commented out in it (a flip in that same commit left none).
"""

from __future__ import annotations

import argparse
import re
import subprocess
import sys
from dataclasses import dataclass
from pathlib import Path

ORACLE_FILE = "c_oracle.cmake"
INCLUDE_LINE = (
    f"include({ORACLE_FILE})  # the pre-flip C as <target>_c: python -m rustconv.c_oracle"
)
# Both spellings the tree uses: `# x.c  # replaced by Rust` and `# x.c — replaced by Rust`
# (advapi32, setupapi and ~450 other lines; the em-dash form read as "nothing flipped").
# A Rust piece names "rust" as a whole component (`rust_x`, `${X_RUST_LIB}`, `x-rust`). A
# bare substring test also dropped setupapi's `wintrust` delay import.
_RUST_WORD = re.compile(r"(?i)(?:^|[_${}-])rust(?:[_${}-]|$)")
_FLIPPED = re.compile(r"^\s*#\s*(\S+\.(?:c|cc|cpp|cxx|S|s))\s+(?:#|—)\s*replaced by Rust\b")
_CONTROL = ("if", "elseif", "else", "endif")
# Commands that only ship, package or precompile; an oracle needs none of them.
_SKIP = {"add_cd_file", "add_rostests_file", "add_pch", "add_registry_inf", "set_cpp"}


class OracleError(RuntimeError):
    pass


@dataclass
class Command:
    name: str
    args: str  # the raw text between the parentheses
    text: str  # the whole command, as written


def commands(text: str) -> list[Command]:
    """The top-level commands of a CMakeLists, comments dropped, parentheses balanced."""
    out: list[Command] = []
    i, n = 0, len(text)
    while i < n:
        c = text[i]
        if c == "#":
            i = text.find("\n", i)
            i = n if i < 0 else i
            continue
        m = re.compile(r"([A-Za-z_][A-Za-z0-9_]*)\s*\(").match(text, i)
        if not m:
            i += 1
            continue
        depth, j = 1, m.end()
        quoted = False
        while j < n and depth:
            ch = text[j]
            if ch == '"' and text[j - 1] != "\\":
                quoted = not quoted
            elif not quoted and ch == "#":
                k = text.find("\n", j)
                j = n if k < 0 else k
                continue
            elif not quoted and ch == "(":
                depth += 1
            elif not quoted and ch == ")":
                depth -= 1
            j += 1
        if depth:
            raise OracleError(f"unbalanced parentheses in {m.group(1)}(")
        out.append(Command(m.group(1).lower(), text[m.end() : j - 1], text[m.start() : j]))
        i = j
    return out


def _words(args: str) -> list[str]:
    return re.findall(r'"[^"]*"|[^\s"]+', re.sub(r"#[^\n]*", "", args))


def source_lists(text: str, *, all_lists: bool = False) -> dict[str, list[str]]:
    """Each `list(APPEND V ..)` / `set(V ..)` holding a flipped TU -> its pre-flip entries
    (every such list with `all_lists`)."""
    lists: dict[str, list[str]] = {}
    flipped: set[str] = set()
    for cmd in commands(text):
        m = re.match(r"\s*(?:APPEND\s+)?([A-Za-z_]\w*)", cmd.args)
        if cmd.name not in ("list", "set") or m is None:
            continue
        if cmd.name == "list" and not cmd.args.lstrip().upper().startswith("APPEND"):
            continue
        var = m.group(1)
        entries: list[str] = []
        for line in cmd.args[m.end() :].splitlines():
            f = _FLIPPED.match(line)
            if f:
                entries.append(f.group(1))
                flipped.add(var)
                continue
            # All of the flip's rust_*.c scaffolding: netsh's oracle compiled
            # rust_varargs_shim.c, whose trampolines redefine netsh.c's own variadics.
            entries += [w for w in _words(line) if not Path(w).name.startswith("rust_")]
        lists.setdefault(var, []).extend(entries)
    return {v: lists[v] for v in lists if v in flipped or all_lists}


def _target_of(cmd: Command) -> str | None:
    w = _words(cmd.args)
    return w[0] if w else None


def inline_sources(text: str) -> dict[str, list[str]]:
    """Targets whose flipped TUs sit in the `add_executable(..)` itself -> those TUs.

    `port flip` writes a single-list module that way (find, sort, and every other batch C
    and D1 tool): `add_executable(find\n    # find.c  # replaced by Rust (..)\n ..)`. With
    only `list(APPEND ..)` understood, none of them could be oracled."""
    out: dict[str, list[str]] = {}
    for cmd in commands(text):
        if cmd.name not in ("add_executable", "add_library"):
            continue
        entries = [f.group(1) for line in cmd.args.splitlines() if (f := _FLIPPED.match(line))]
        target = _target_of(cmd)
        if entries and target:
            out[target] = entries
    return out


def generate(cml: str) -> str:
    """The `c_oracle.cmake` text for a module's CMakeLists (refuses an unflipped one)."""
    lists = source_lists(cml)
    inline = inline_sources(cml)
    if not lists and not inline:
        raise OracleError(
            "no `# x.c  # replaced by Rust` lines: nothing flipped, nothing to oracle"
        )
    cmds = commands(cml)
    oracle: dict[str, str] = {t: f"{t}_c" for t in inline}
    for cmd in cmds:  # targets built from a flipped list
        if cmd.name in ("add_executable", "add_library"):
            words = _words(cmd.args)
            if any(f"${{{v}}}" in w for w in words[1:] for v in lists):
                oracle[words[0]] = f"{words[0]}_c"
    if not oracle:
        raise OracleError(f"no add_executable/add_library uses the flipped list(s) {sorted(lists)}")
    grew = True
    while grew:  # ... and targets linking one of them
        grew = False
        for cmd in cmds:
            if cmd.name != "target_link_libraries":
                continue
            words = _words(cmd.args)
            if words and words[0] not in oracle and any(w in oracle for w in words[1:]):
                oracle[words[0]] = f"{words[0]}_c"
                grew = True

    # A list the oracle names that the flip did NOT touch is copied too: the cargo flip
    # deletes `list(APPEND PCH_SKIP_SOURCE .._stubs.c)` along with the sources, so imagehlp_c
    # read an empty ${PCH_SKIP_SOURCE}, lost spec2def's stubs, and failed to link.
    every = source_lists(cml, all_lists=True)
    used = {
        v
        for cmd in cmds
        if _target_of(cmd) in oracle
        for v in re.findall(r"\$\{(\w+)\}", cmd.args)
        if v in every and v not in lists
    }
    lists = {**lists, **{v: every[v] for v in used}}

    def rewrite(word: str) -> str | None:
        if _RUST_WORD.search(word):
            return None
        for v in lists:
            word = word.replace(f"${{{v}}}", f"${{{v}_C_ORACLE}}")
        return oracle.get(word, word)

    body: list[str] = []
    for cmd in cmds:
        if cmd.name in _CONTROL:
            body.append(f"{cmd.name}({cmd.args.strip()})")
            continue
        target = _target_of(cmd)
        if cmd.name in _SKIP or cmd.name.startswith("rust_") or target not in oracle:
            continue
        raw = _words(cmd.args)
        words = [r for w in raw if (r := rewrite(w)) is not None]
        if cmd.name in ("add_executable", "add_library"):
            at = 2 if len(words) > 1 and words[1] in ("STATIC", "SHARED", "MODULE", "OBJECT") else 1
            # add_executable's own keywords precede EXCLUDE_FROM_ALL and the sources: testgui's
            # `add_executable(testgui WIN32 ..)` became `testgui_c EXCLUDE_FROM_ALL testgui.c
            # WIN32`, and CMake looked for a source file named WIN32.
            while at < len(words) and words[at] in ("WIN32", "MACOSX_BUNDLE"):
                at += 1
            if "EXCLUDE_FROM_ALL" not in words:
                words.insert(at, "EXCLUDE_FROM_ALL")
            if target in inline:
                words[at + 1 : at + 1] = inline[target]
        elif len(words) < len(raw) and not [
            w for w in words[1:] if w not in ("PUBLIC", "PRIVATE", "INTERFACE")
        ]:
            # Only Rust pieces were configured (add_dependencies(x rust_x)). A command that
            # takes the target ALONE is real configuration: comctl32's oracle dropped
            # set_wine_module_FIXME(comctl32) this way and lost Wine's ARRAY_SIZE.
            continue
        if cmd.name == "set_target_properties" and "OUTPUT_NAME" in words:
            # cmd_help's `OUTPUT_NAME "help"` copied as-is: two rules generated help.exe and
            # ninja refused the whole tree. The oracle's file is suffixed like its target.
            at = words.index("OUTPUT_NAME") + 1
            if at < len(words):
                q = words[at].endswith('"')
                words[at] = words[at][:-1] + '_c"' if q else words[at] + "_c"
        body.append(f"{cmd.name}({' '.join(words)})")
    body = _prune_empty_ifs(body)
    head = [
        "# GENERATED by `python -m rustconv.c_oracle` -- do not edit; re-run it after the",
        "# CMakeLists changes. The module's pre-flip C as "
        + ", ".join(sorted(oracle.values()))
        + ":",
        "# the runtime oracle the Rust is diffed against (tools/payloads/modrun). Built only",
        "# on request (`ninja <target>_c`), never shipped.",
    ]
    for v, entries in sorted(lists.items()):
        head.append(f"set({v}_C_ORACLE")
        head += [f"    {e}" for e in entries]
        head.append(")")
    return "\n".join(head + body) + "\n"


def _prune_empty_ifs(lines: list[str]) -> list[str]:
    changed = True
    while changed:
        changed = False
        for i in range(len(lines) - 1):
            a, b = lines[i].split("(")[0], lines[i + 1].split("(")[0]
            if a in ("if", "elseif", "else") and b in ("elseif", "else", "endif"):
                if a == "if" and b == "endif":
                    del lines[i : i + 2]
                elif a != "if":
                    del lines[i]
                else:
                    continue
                changed = True
                break
    return lines


def _git(cwd: Path, *args: str) -> str:
    cp = subprocess.run(["git", *args], cwd=cwd, capture_output=True, text=True)
    if cp.returncode:
        raise OracleError(f"git {' '.join(args)}: {cp.stderr.strip()}")
    return cp.stdout


def pre_cargo(cml_path: Path) -> str:
    """The CMakeLists text from before the module went to `add_cargo_module`: the parent of
    the oldest commit that changed how often that string occurs in the file."""
    where = cml_path.resolve().parent
    rel = _git(where, "ls-files", "--full-name", cml_path.name).strip()
    shas = _git(
        where, "log", "--reverse", "--format=%H", "-S", "add_cargo_module", "--", cml_path.name
    )
    if not rel or not shas.split():
        raise OracleError(f"{cml_path}: no commit introduced add_cargo_module (born cargo?)")
    try:
        return _git(where, "show", f"{shas.split()[0]}^:{rel}")
    except OracleError:
        raise OracleError(f"{cml_path}: born in cargo form, no C build to oracle") from None


def reflip(old: str, cml: str) -> str:
    """`old` with the TUs `cml` marks flipped commented out the same way. A module flipped and
    moved to cargo in ONE commit (cscript) has no flipped line before that commit: its text is
    the plain C build, and this is the flip the cargo commit made."""
    flipped = {f.group(1) for line in cml.splitlines() if (f := _FLIPPED.match(line))}
    out = []
    for line in old.splitlines():
        entry = line.strip().rstrip(")").strip()
        if entry in flipped and not line.lstrip().startswith("#"):
            indent = line[: len(line) - len(line.lstrip())]
            closes = line.rstrip().endswith(")")
            line = f"{indent}# {entry}  # replaced by Rust"
            if closes:
                # The list's `)` on a line of its own: after the comment it would be part of
                # it, and the list never closes (imagehlp's `modify.c)`, its last entry).
                out.append(line)
                line = f"{indent})"
        out.append(line)
    return "\n".join(out) + "\n"


def write(module_dir: Path, *, check: bool = False) -> bool:
    """Write (or with check, compare) the module's oracle. True when already current."""
    cml_path = module_dir / "CMakeLists.txt"
    cml = cml_path.read_text()
    want = generate(reflip(pre_cargo(cml_path), cml) if "add_cargo_module(" in cml else cml)
    path = module_dir / ORACLE_FILE
    current = path.is_file() and path.read_text() == want and f"include({ORACLE_FILE})" in cml
    if check or current:
        return current
    path.write_text(want)
    if f"include({ORACLE_FILE})" not in cml:
        cml_path.write_text(cml.rstrip("\n") + "\n\n" + INCLUDE_LINE + "\n")
    return False


def main(argv: list[str] | None = None) -> int:
    ap = argparse.ArgumentParser(description=(__doc__ or "").splitlines()[0])
    ap.add_argument(
        "module", nargs="+", type=Path, help="module directory (holding CMakeLists.txt)"
    )
    ap.add_argument("--check", action="store_true", help="exit 1 if an oracle is missing or stale")
    args = ap.parse_args(argv)
    stale = 0
    for module in args.module:
        try:
            ok = write(module, check=args.check)
        except OracleError as e:
            print(f"c_oracle: {module}: {e}", file=sys.stderr)
            return 1
        if args.check and not ok:
            print(f"c_oracle: {module}/{ORACLE_FILE} is missing or stale", file=sys.stderr)
            stale += 1
        elif not args.check:
            print(f"c_oracle: {module}/{ORACLE_FILE} {'current' if ok else 'written'}")
    return 1 if stale else 0


if __name__ == "__main__":
    raise SystemExit(main())
