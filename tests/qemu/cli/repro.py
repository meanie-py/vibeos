"""`vibeos-cli repro` — reproducibility / determinism measurement harness (rung 0).

The determinism lane's gauge (docs/engineering/deterministic-builds-plan.md). Answers two
questions without any external dependency:

  1. **Are two builds of the same commit byte-identical?** (`compare A B`) — and if not,
     *which PE sections differ* and *does the PE TimeDateStamp diverge*, so a leak is
     localized instead of just "the bytes differ".
  2. **What determinism-relevant metadata does an artifact carry today?** (`report F...`) —
     the COFF TimeDateStamp (the measured live leak on all 3 arches) + per-section digests.

The PE parser is pure + self-contained (no pefile/diffoscope needed) so it runs anywhere and
is unit-tested in test_cli_units.py. `diffoscope`, if installed, is offered as the deep
localizer on a mismatch (`--diffoscope`); its absence degrades gracefully.

This is the foundation for the ladder's rung 5 (a build-twice byte-identical gauge wired into
the harness): `diff-dirs` compares the staged artifact set across two build trees, which is
exactly the build-twice-in-two-paths verification methodology the plan calls for.
"""

from __future__ import annotations

import hashlib
import shutil
import subprocess
from dataclasses import dataclass
from pathlib import Path

# Default artifacts to diff across two build trees (relative to a build-<arch>/ root). The
# kernel + the two cargo-driven non-standard links (the #2 footgun) + a representative DLL.
DEFAULT_ARTIFACTS = (
    "ntoskrnl/ntoskrnl.exe",
    "win32ss/win32k.sys",
    "dll/win32/kernel32/kernel32.dll",
)


@dataclass(frozen=True)
class Section:
    name: str
    raw_off: int
    raw_size: int
    digest: str  # sha256 of the section's raw bytes


@dataclass(frozen=True)
class PeImage:
    machine: int
    timestamp: int  # COFF TimeDateStamp (the wall-clock leak)
    sections: tuple[Section, ...]
    size: int
    digest: str  # sha256 of the whole file


def _u16(data: bytes, off: int) -> int:
    return int.from_bytes(data[off : off + 2], "little")


def _u32(data: bytes, off: int) -> int:
    return int.from_bytes(data[off : off + 4], "little")


def parse_pe(data: bytes) -> PeImage:
    """Parse the COFF header + section table from a PE image. Raises ValueError on a
    malformed image. Only reads the fields determinism cares about (timestamp + sections);
    PE32 vs PE32+ is irrelevant here since SizeOfOptionalHeader is in the COFF header."""
    if len(data) < 0x40 or data[:2] != b"MZ":
        raise ValueError("not a PE image (no MZ header)")
    pe = _u32(data, 0x3C)
    if pe + 24 > len(data) or data[pe : pe + 4] != b"PE\x00\x00":
        raise ValueError("not a PE image (no PE signature)")
    coff = pe + 4
    machine = _u16(data, coff)
    num_sections = _u16(data, coff + 2)
    timestamp = _u32(data, coff + 4)
    size_opt = _u16(data, coff + 16)
    sec_table = coff + 20 + size_opt

    sections: list[Section] = []
    for i in range(num_sections):
        base = sec_table + i * 40
        if base + 40 > len(data):
            raise ValueError("truncated section table")
        name = data[base : base + 8].rstrip(b"\x00").decode("latin-1")
        raw_size = _u32(data, base + 16)
        raw_off = _u32(data, base + 20)
        body = data[raw_off : raw_off + raw_size] if raw_size else b""
        sections.append(Section(name, raw_off, raw_size, hashlib.sha256(body).hexdigest()))

    return PeImage(
        machine=machine,
        timestamp=timestamp,
        sections=tuple(sections),
        size=len(data),
        digest=hashlib.sha256(data).hexdigest(),
    )


@dataclass(frozen=True)
class CompareReport:
    identical: bool
    digest_a: str
    digest_b: str
    timestamp_a: int
    timestamp_b: int
    section_diffs: tuple[str, ...]  # sections present in both but differing
    only_in_a: tuple[str, ...]
    only_in_b: tuple[str, ...]

    @property
    def timestamp_differs(self) -> bool:
        return self.timestamp_a != self.timestamp_b


def compare(data_a: bytes, data_b: bytes) -> CompareReport:
    """Byte-identity + localization of where two PE images diverge. Pure → unit-tested."""
    a, b = parse_pe(data_a), parse_pe(data_b)
    sa = {s.name: s.digest for s in a.sections}
    sb = {s.name: s.digest for s in b.sections}
    diffs = tuple(sorted(n for n in sa.keys() & sb.keys() if sa[n] != sb[n]))
    return CompareReport(
        identical=a.digest == b.digest,
        digest_a=a.digest,
        digest_b=b.digest,
        timestamp_a=a.timestamp,
        timestamp_b=b.timestamp,
        section_diffs=diffs,
        only_in_a=tuple(sorted(sa.keys() - sb.keys())),
        only_in_b=tuple(sorted(sb.keys() - sa.keys())),
    )


def _ts(v: int) -> str:
    """Render a COFF TimeDateStamp: 0 = deterministic; else hex + a wall-clock hint."""
    if v == 0:
        return "0 (deterministic)"
    import datetime

    try:
        utc = datetime.datetime.fromtimestamp(v, datetime.timezone.utc)
        return f"0x{v:08x} ({utc:%Y-%m-%d %H:%M:%S}Z)"
    except (OSError, OverflowError, ValueError):
        return f"0x{v:08x}"


def _read(path: Path) -> bytes | None:
    try:
        return path.read_bytes()
    except OSError as e:
        print(f"  ! cannot read {path}: {e}")
        return None


def _run_diffoscope(a: Path, b: Path) -> None:
    tool = shutil.which("diffoscope")
    if not tool:
        print("  (diffoscope not installed — `pip install diffoscope` for byte-level localization)")
        return
    print(f"\n  diffoscope {a} {b}:")
    try:
        cp = subprocess.run(
            [tool, "--max-report-size", "40000", str(a), str(b)],
            capture_output=True,
            text=True,
            timeout=300,
        )
        print("\n".join(("    " + ln) for ln in cp.stdout.splitlines()[:200]))
    except (OSError, subprocess.SubprocessError) as e:
        print(f"    diffoscope failed: {e}")


def _print_compare(name: str, rep: CompareReport) -> None:
    mark = "✓ identical" if rep.identical else "✗ DIFFERS"
    print(f"  {mark}  {name}")
    if rep.identical:
        return
    if rep.timestamp_differs:
        print(f"      TimeDateStamp: {_ts(rep.timestamp_a)}  vs  {_ts(rep.timestamp_b)}")
    if rep.section_diffs:
        print(f"      sections differ: {', '.join(rep.section_diffs)}")
    if rep.only_in_a or rep.only_in_b:
        print(f"      section set differs: -{list(rep.only_in_a)} +{list(rep.only_in_b)}")
    if not (rep.timestamp_differs or rep.section_diffs or rep.only_in_a or rep.only_in_b):
        print("      headers/overlay differ (no section-body or timestamp delta)")


def _cmd_compare(args) -> int:
    a, b = Path(args.file_a), Path(args.file_b)
    da, db = _read(a), _read(b)
    if da is None or db is None:
        return 1
    try:
        rep = compare(da, db)
    except ValueError as e:
        print(f"  ! parse error: {e}")
        return 1
    _print_compare(f"{a.name}", rep)
    if not rep.identical and getattr(args, "diffoscope", False):
        _run_diffoscope(a, b)
    return 0 if rep.identical else 1


def _cmd_report(args) -> int:
    rc = 0
    for f in args.files:
        p = Path(f)
        data = _read(p)
        if data is None:
            rc = 1
            continue
        try:
            pe = parse_pe(data)
        except ValueError as e:
            print(f"  ! {p}: {e}")
            rc = 1
            continue
        leak = " ← LEAK" if pe.timestamp != 0 else ""
        print(f"  {p}")
        print(f"      sha256        {pe.digest}")
        print(f"      TimeDateStamp {_ts(pe.timestamp)}{leak}")
        print(f"      sections      {len(pe.sections)}")
    return rc


def _cmd_diff_dirs(args) -> int:
    dir_a, dir_b = Path(args.dir_a), Path(args.dir_b)
    artifacts = args.artifacts or list(DEFAULT_ARTIFACTS)
    print(f"vibeos-cli repro — diff {dir_a}  vs  {dir_b}\n")
    any_diff = False
    any_found = False
    for rel in artifacts:
        a, b = dir_a / rel, dir_b / rel
        if not a.is_file() or not b.is_file():
            print(f"  - skip {rel} (missing in one tree)")
            continue
        any_found = True
        da, db = a.read_bytes(), b.read_bytes()
        try:
            rep = compare(da, db)
        except ValueError as e:
            print(f"  ! {rel}: {e}")
            any_diff = True
            continue
        _print_compare(rel, rep)
        if not rep.identical:
            any_diff = True
            if getattr(args, "diffoscope", False):
                _run_diffoscope(a, b)
    if not any_found:
        print("  (no artifacts found in both trees — build both first)")
        return 1
    print()
    print("  → not reproducible (see diffs above)" if any_diff else "  → byte-identical ✓")
    return 1 if any_diff else 0


def repro(args, repo: Path) -> int:
    """Dispatch the `repro` subverb. repo is unused (artifacts are addressed by path)."""
    cmd = getattr(args, "repro_cmd", None)
    if cmd == "compare":
        return _cmd_compare(args)
    if cmd == "report":
        return _cmd_report(args)
    if cmd == "diff-dirs":
        return _cmd_diff_dirs(args)
    print("repro: specify a subcommand (compare | report | diff-dirs)")
    return 2
