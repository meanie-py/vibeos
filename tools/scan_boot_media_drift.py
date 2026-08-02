#!/usr/bin/env python3
"""Registry hives staged onto the livecd but NOT into the arm64 ramdisk.

The two boot media are built by completely separate machinery: x86 boots the
`livecd.iso` that cmake's add_cd_file() assembles, arm64 boots a ramdisk that
`qemu/mkramdisk.sh` assembles by hand. Nothing makes the two agree, and the
same five hives are spelled out in SIX independent hand-maintained lists
(mkhive's -h: argument, add_custom_command OUTPUT, add_custom_target DEPENDS,
add_cd_file FILE, and mkramdisk.sh's copy list + REQUIRED_FILES).

Drifting apart is not a build error and does not announce itself at boot. A
hive missing from the ramdisk is FABRICATED by the kernel: CmpLoadHiveThread
fails to open the file, sets Allocate, and CmpLinkHiveToMaster conjures an
empty volatile hive whose root key takes CmpHiveRootSecurityDescriptor(). So
\\REGISTRY\\MACHINE\\SAM exists, opens fine, and holds no accounts.

That is exactly what happened: `sam` and `security` were absent from the
ramdisk for a long time, so arm64 booted with an empty SAM on every boot. It
surfaced only because arm64 then SCORED BETTER than x86 on the CmSecurity
kmtest -- the fabricated descriptor happens to be the one the test expects --
which pointed the investigation at the wrong arch entirely. See the field
manual, "A file MISSING from the arm64 ramdisk is fabricated by the kernel".

Exit 1 on any finding, so this gates.
"""

import argparse
import os
import re
import sys

CMAKE = "sdk/cmake/CMakeMacros.cmake"
MKRAMDISK = "qemu/mkramdisk.sh"
# Where CmpInitializeHiveList looks. A hive absent from HERE is the one that gets
# fabricated; hives staged elsewhere (efi/boot) are firmware artifacts, not CM hives.
CONFIG_DIR = "reactos/system32/config"


def repo_root():
    return os.path.dirname(os.path.dirname(os.path.abspath(__file__)))


def line_of(text, idx):
    return text.count("\n", 0, idx) + 1


def balanced(text, open_idx):
    """Return (body, end_idx) for the parenthesised block starting at open_idx."""
    depth, i = 0, open_idx
    while i < len(text):
        if text[i] == "(":
            depth += 1
        elif text[i] == ")":
            depth -= 1
            if depth == 0:
                return text[open_idx + 1 : i], i
        i += 1
    return text[open_idx + 1 :], len(text)


def blocks(text, command):
    """Yield (body, line) for each `command(...)` invocation."""
    for m in re.finditer(rf"\b{command}\s*\(", text):
        body, _ = balanced(text, m.end() - 1)
        yield body, line_of(text, m.start())


# Anything cmake would treat as the start of the next argument group. Without this
# the OUTPUT list runs on into COMMAND/DEPENDS and swallows mkhive's own .inf INPUTS,
# which then read as undeclared outputs.
_NEXT_KEYWORD = re.compile(
    r"^\s*(COMMAND|DEPENDS|WORKING_DIRECTORY|COMMENT|MAIN_DEPENDENCY|IMPLICIT_DEPENDS"
    r"|BYPRODUCTS|VERBATIM|APPEND|USES_TERMINAL|COMMAND_EXPAND_LISTS|TARGET|DESTINATION"
    r"|FOR|NAME_ON_CD|NO_CAB|OUTPUT|FILE)\b",
    re.M,
)


def bootdata_files(body, section=None):
    """The boot/bootdata/<name> hive files in a cmake block.

    With `section` ("OUTPUT"/"FILE"), read only that argument group — a hive is a
    generated artifact, so an .inf under the same directory is an input and must
    not be counted.
    """
    if section is not None:
        m = re.search(rf"\b{section}\b", body)
        if not m:
            return []
        rest = body[m.end() :]
        nxt = _NEXT_KEYWORD.search(rest)
        body = rest[: nxt.start()] if nxt else rest
    return [
        n
        for n in re.findall(r"boot/bootdata/([A-Za-z0-9_.]+)", body)
        if not n.lower().endswith(".inf")
    ]


def main():
    ap = argparse.ArgumentParser(description=__doc__)
    ap.add_argument("--lint", action="store_true", help="(accepted for uniformity)")
    ap.add_argument("--cmake", help="override the CMakeMacros.cmake path (tests)")
    ap.add_argument("--ramdisk", help="override the mkramdisk.sh path (tests)")
    args = ap.parse_args()

    repo = repo_root()
    cmake_path = args.cmake or os.path.join(repo, CMAKE)
    ramdisk_path = args.ramdisk or os.path.join(repo, MKRAMDISK)
    for p, rel in ((cmake_path, CMAKE), (ramdisk_path, MKRAMDISK)):
        if not os.path.isfile(p):
            print(f"{rel}:1: [MISSING] cannot audit boot media without this file")
            return 1

    with open(cmake_path, encoding="utf-8", errors="replace") as fh:
        cmake = fh.read()
    with open(ramdisk_path, encoding="utf-8", errors="replace") as fh:
        ramdisk = fh.read()

    findings = 0

    # --- what mkhive is TOLD to emit vs what cmake DECLARES it emits ----------
    # A hive added to -h: but not to OUTPUT is built and then never tracked as a
    # dependency, so it goes stale silently.
    generated = {}  # basename -> line it was declared on
    for body, line in blocks(cmake, "add_custom_command"):
        m = re.search(r"native-mkhive\s+-h:([A-Za-z0-9_,]+)", body)
        if not m:
            continue
        asked = [h for h in m.group(1).split(",") if h]
        outputs = bootdata_files(body, "OUTPUT")
        if len(asked) != len(outputs):
            print(
                f"{CMAKE}:{line}: [HIVE_LIST_DRIFT] native-mkhive is told to emit "
                f"{len(asked)} hive(s) ({','.join(asked)}) but the command declares "
                f"{len(outputs)} OUTPUT file(s) ({','.join(outputs) or 'none'}) — "
                f"an undeclared output is never a dependency, so it goes stale silently"
            )
            findings += 1
        for name in outputs:
            generated[name] = line

    # --- hives the LIVECD ships into the CM config directory (the reference set) ---
    # Scoped to CONFIG_DIR on purpose. The fabricate-on-absence behaviour is specific
    # to hives CmpInitializeHiveList loads out of system32\config; the BCD hive is
    # staged to efi/boot for the EFI boot manager and is not a \REGISTRY\MACHINE hive
    # at all, so demanding it on the ramdisk would be a false positive.
    livecd = {}  # basename -> line
    for body, line in blocks(cmake, "add_cd_file"):
        if not re.search(r"\bFOR\b[^)]*\blivecd\b", body):
            continue
        dest = re.search(r"\bDESTINATION\s+(\S+)", body)
        if not dest or dest.group(1).strip().lower() != CONFIG_DIR:
            continue
        for name in bootdata_files(body, "FILE"):
            if name in generated:
                livecd[name] = line

    if not livecd:
        print(
            f"{CMAKE}:1: [NO_REFERENCE] found no add_cd_file(... FOR livecd) block "
            f"staging generated hives — this audit cannot see the reference set, so it "
            f"would pass vacuously; the parser or the cmake layout has changed"
        )
        return 1

    # --- and what the arm64 ramdisk stages ------------------------------------
    staged = set(re.findall(r"^\s*RMC\s+\"[^\"]*/boot/bootdata/([A-Za-z0-9_.]+)\"", ramdisk, re.M))

    req_m = re.search(r"^REQUIRED_FILES=\((.*?)^\)", ramdisk, re.M | re.S)
    required = set()
    if req_m:
        required = set(re.findall(r"/boot/bootdata/([A-Za-z0-9_.]+)", req_m.group(1)))
    else:
        print(
            f"{MKRAMDISK}:1: [NO_REQUIRED_FILES] no REQUIRED_FILES=( ... ) array — "
            f"without it a missing hive is not caught before boot"
        )
        findings += 1

    for name, cd_line in sorted(livecd.items()):
        if name not in staged:
            print(
                f"{MKRAMDISK}:1: [RAMDISK_MISSING] hive '{name}' is staged onto the livecd "
                f"({CMAKE}:{cd_line}) but never copied into the arm64 ramdisk — the kernel "
                f"will FABRICATE an empty volatile hive for it and boot without complaint"
            )
            findings += 1
        elif name not in required:
            print(
                f"{MKRAMDISK}:1: [RAMDISK_UNGUARDED] hive '{name}' is copied into the ramdisk "
                f"but is not in REQUIRED_FILES — if the build ever stops producing it the copy "
                f"silently stages nothing and the kernel fabricates it instead of failing"
            )
            findings += 1

    print(
        f"\nSUMMARY: generated={len(generated)}  livecd={len(livecd)}  "
        f"ramdisk={len(staged)}  guarded={len(required)}  findings={findings}"
    )
    return 1 if findings else 0


if __name__ == "__main__":
    sys.exit(main())
