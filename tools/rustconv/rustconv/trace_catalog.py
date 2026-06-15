"""Generate debugcon trace catalog from Rust kernel source.

Scans Rust files for `out 0xe9` (bochs debug port) trace points,
extracts the trace character, enclosing function, and source location,
then emits an execution-ordered JSON catalog for use by the debugcon
decoder in the smoke test infrastructure.
"""

from __future__ import annotations

import json
import re
from dataclasses import asdict, dataclass, field
from pathlib import Path

# Project paths
REACTOS_DIR = Path(__file__).resolve().parent.parent.parent.parent
NTOSKRNL_RUST_SRC = REACTOS_DIR / "sdk" / "lib" / "rust" / "ntoskrnl-rust" / "src"
DATA_DIR = Path(__file__).resolve().parent.parent / "data"

# Regex for: core::arch::asm!("out dx, al", in("dx") 0xe9u16, in("al") b'X', ...)
# Matches literal byte chars like b'@', b'1', b'\\' (escaped backslash)
RE_TRACE_LITERAL = re.compile(
    r"""core::arch::asm!\(\s*"out dx, al"\s*,\s*in\("dx"\)\s*0xe9u16\s*,\s*"""
    r"""in\("al"\)\s*b'(\\?.)'.*\)"""
)

# Regex for: core::arch::asm!("out dx, al", in("dx") 0xe9u16, in("al") <expr>, ...)
# Matches computed expressions (hex dump data bytes)
RE_TRACE_COMPUTED = re.compile(
    r"""core::arch::asm!\(\s*"out dx, al"\s*,\s*in\("dx"\)\s*0xe9u16\s*,\s*"""
    r"""in\("al"\)\s*(?!b')(.*?)\s*,\s*options"""
)

# Regex for dbg(b'X') calls (used in KiVerifyCpuFeatures)
RE_DBG_CALL = re.compile(r"""dbg\(b'(\\?.)'\)""")

# Regex for function declarations
RE_FN = re.compile(r'(?:pub\s+)?(?:unsafe\s+)?(?:extern\s+"[^"]+"\s+)?fn\s+(\w+)\s*\(')


@dataclass
class TracePoint:
    char: str
    function: str
    file: str  # relative to ntoskrnl-rust/src/
    line: int
    description: str


@dataclass
class HexDumpDelimiter:
    open: str
    close: str
    function: str
    description: str


@dataclass
class TraceCatalog:
    version: int = 1
    arch: str = "i386"
    trace_points: list[TracePoint] = field(default_factory=list)
    hex_dump_delimiters: list[HexDumpDelimiter] = field(default_factory=list)


# i386 boot execution segments: (function, min_line, max_line).
# Handles nested function calls (e.g., KiVerifyCpuFeatures is called
# IN THE MIDDLE of KiSystemStartup between traces '4' and '5').
# Each segment is a contiguous range of lines within a single function.
EXECUTION_SEGMENTS_I386 = [
    # KiSystemStartup entry through KiVerifyCpuFeatures call (line 1125)
    ("KiSystemStartup", 990, 1124),
    # KiVerifyCpuFeatures: A-D (up to KiGetFeatureBits call at line 579)
    ("KiVerifyCpuFeatures", 540, 578),
    # KiGetFeatureBits: [vendor]{CpuFeatures}<FeatureBits>
    ("KiGetFeatureBits", 0, 9999),
    # KiVerifyCpuFeatures: E-H (after KiGetFeatureBits returns)
    ("KiVerifyCpuFeatures", 579, 9999),
    # KiSystemStartup: '5' through 'j' (after KiVerifyCpuFeatures returns)
    ("KiSystemStartup", 1125, 9999),
    # KiSystemStartupBootStack (via KiSwitchToBootStack)
    ("KiSystemStartupBootStack", 0, 9999),
    # KiInitializeKernel (called from KiSystemStartupBootStack)
    ("KiInitializeKernel", 0, 9999),
    # ExpInitializeExecutive (called from KiInitializeKernel)
    ("ExpInitializeExecutive", 0, 9999),
    # MmArmInitSystem (called from ExpInitializeExecutive)
    ("MmArmInitSystem", 0, 9999),
    # MiInitMachineDependent (called from MmArmInitSystem)
    ("MiInitMachineDependent", 0, 9999),
]


def scan_file(path: Path, rel_prefix: str = "") -> list[dict]:
    """Scan a single .rs file for trace points.

    Returns list of dicts with keys: char, function, file, line, is_literal, is_dbg.
    """
    try:
        lines = path.read_text().splitlines()
    except (OSError, UnicodeDecodeError):
        return []

    rel_path = rel_prefix or str(path.relative_to(NTOSKRNL_RUST_SRC))
    results = []
    current_fn = "<unknown>"

    for i, line in enumerate(lines, 1):
        # Track enclosing function
        fn_match = RE_FN.search(line)
        if fn_match:
            name = fn_match.group(1)
            # Skip the inner dbg helper function definition
            if name != "dbg" and name != "hex":
                current_fn = name

        # Check for literal trace: b'X'
        m = RE_TRACE_LITERAL.search(line)
        if m:
            raw = m.group(1)
            ch = raw.replace("\\\\", "\\")  # unescape Rust \\
            results.append(
                {
                    "char": ch,
                    "function": current_fn,
                    "file": rel_path,
                    "line": i,
                    "is_literal": True,
                    "is_dbg": False,
                }
            )
            continue

        # Check for computed trace (hex dump data)
        m = RE_TRACE_COMPUTED.search(line)
        if m:
            results.append(
                {
                    "char": None,  # computed — not a milestone
                    "function": current_fn,
                    "file": rel_path,
                    "line": i,
                    "is_literal": False,
                    "is_dbg": False,
                }
            )
            continue

        # Check for dbg(b'X') calls
        m = RE_DBG_CALL.search(line)
        if m:
            raw = m.group(1)
            ch = raw.replace("\\\\", "\\")
            results.append(
                {
                    "char": ch,
                    "function": current_fn,
                    "file": rel_path,
                    "line": i,
                    "is_literal": True,
                    "is_dbg": True,
                }
            )

    return results


def scan_all(arch: str = "i386") -> list[dict]:
    """Scan all relevant Rust source files for the given architecture.

    Returns all trace points found, unsorted.
    """
    # Only scan files that have i386/x86 traces
    search_dirs = [
        NTOSKRNL_RUST_SRC / "ke" / "i386",
        NTOSKRNL_RUST_SRC / "i386",
        NTOSKRNL_RUST_SRC / "ex",
        NTOSKRNL_RUST_SRC / "ARM3",
    ]

    all_traces = []
    seen_files = set()

    for d in search_dirs:
        if not d.is_dir():
            continue
        for f in sorted(d.glob("*.rs")):
            if f in seen_files:
                continue
            seen_files.add(f)
            traces = scan_file(f)
            # Only include traces that are in x86-gated code
            # (the scan finds them regardless — they all use 0xe9u16 which is x86)
            all_traces.extend(traces)

    return all_traces


def build_catalog(arch: str = "i386") -> TraceCatalog:
    """Build execution-ordered trace catalog for the given architecture."""
    all_traces = scan_all(arch)

    # Separate milestones (literal chars) from computed (hex dump data)
    milestones = [t for t in all_traces if t["is_literal"] and t["char"] is not None]

    # Sort milestones by execution segments: walk segments in order,
    # collecting matching milestones from each segment's (function, line_range).
    ordered = []
    used = set()  # track (function, line) to avoid duplicates

    for fn_name, min_line, max_line in EXECUTION_SEGMENTS_I386:
        segment_traces = [
            t
            for t in milestones
            if t["function"] == fn_name
            and min_line <= t["line"] <= max_line
            and (t["function"], t["line"]) not in used
        ]
        # Sort by line within segment
        segment_traces.sort(key=lambda t: t["line"])
        for t in segment_traces:
            used.add((t["function"], t["line"]))
            ordered.append(t)

    # Any milestones not in any segment go at the end (shouldn't happen)
    for t in milestones:
        if (t["function"], t["line"]) not in used:
            ordered.append(t)

    # Build trace points with descriptions
    catalog = TraceCatalog(version=1, arch=arch)

    for t in ordered:
        desc = _describe_trace(t)
        catalog.trace_points.append(
            TracePoint(
                char=t["char"],
                function=t["function"],
                file=t["file"],
                line=t["line"],
                description=desc,
            )
        )

    # Identify hex dump delimiters by looking for paired open/close chars
    _find_hex_dump_delimiters(catalog, all_traces)

    return catalog


def _describe_trace(t: dict) -> str:
    """Generate a human-readable description for a trace point."""
    fn = t["function"]
    ch = t["char"]

    # Error indicators
    if ch == "!":
        return "error/bugcheck"

    return f"trace '{ch}' in {fn}"


def _find_hex_dump_delimiters(catalog: TraceCatalog, all_traces: list[dict]) -> None:
    """Find hex dump delimiter pairs in trace data.

    Hex dumps are sequences like: '{' <hex bytes> '}' where the inner bytes
    are computed (not literal).
    """
    delimiter_pairs = {
        "[": ("]", "CPU vendor"),
        "{": ("}", "CpuFeatures"),
        "<": (">", "FeatureBits"),
        "(": (")", "MmHighSectionBase setup"),
    }

    for open_ch, (close_ch, desc) in delimiter_pairs.items():
        # Check if both delimiters appear in traces
        has_open = any(t["char"] == open_ch and t["is_literal"] for t in all_traces)
        has_close = any(t["char"] == close_ch and t["is_literal"] for t in all_traces)
        if has_open and has_close:
            # Find which function they're in
            fn = next(
                (t["function"] for t in all_traces if t["char"] == open_ch and t["is_literal"]),
                "<unknown>",
            )
            catalog.hex_dump_delimiters.append(
                HexDumpDelimiter(
                    open=open_ch,
                    close=close_ch,
                    function=fn,
                    description=desc,
                )
            )


def generate_catalog(arch: str = "i386") -> Path:
    """Generate and write the trace catalog JSON file."""
    catalog = build_catalog(arch)

    output = DATA_DIR / f"trace_catalog_{arch}.json"
    DATA_DIR.mkdir(parents=True, exist_ok=True)

    data = {
        "version": catalog.version,
        "arch": catalog.arch,
        "trace_points": [asdict(tp) for tp in catalog.trace_points],
        "hex_dump_delimiters": [asdict(d) for d in catalog.hex_dump_delimiters],
    }

    output.write_text(json.dumps(data, indent=2) + "\n")
    return output


def print_catalog(arch: str = "i386") -> None:
    """Print catalog summary to stdout."""
    catalog = build_catalog(arch)

    print(f"Trace catalog for {arch}:")
    print(f"  {len(catalog.trace_points)} milestone trace points")
    print(f"  {len(catalog.hex_dump_delimiters)} hex dump delimiters")
    print()

    current_fn = None
    for tp in catalog.trace_points:
        if tp.function != current_fn:
            current_fn = tp.function
            print(f"  {current_fn}():")
        print(f"    '{tp.char}' @ {tp.file}:{tp.line} — {tp.description}")

    if catalog.hex_dump_delimiters:
        print()
        print("  Hex dump delimiters:")
        for d in catalog.hex_dump_delimiters:
            print(f"    '{d.open}'...'{d.close}' = {d.description} ({d.function})")
