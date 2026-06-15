"""Classify _SEH2_TRY blocks into pattern categories for Rust idiomation.

Uses regex-based extraction for SEH block boundaries (reliable across all
macro patterns) and brace-counting for function boundaries. Tree-sitter
can't handle the SEH macros in complex files (tokencls.c, query.c).

The classification of try-body and handler content uses regex patterns that
map to specific Rust API patterns (probe, safe_copy, RAII guard, ? operator).
"""

import json
import re
from dataclasses import dataclass, field
from enum import Enum, auto
from pathlib import Path


class TryBodyPattern(Enum):
    """What's inside the _SEH2_TRY body."""

    PROBE = auto()  # ProbeForRead/Write (-> probe_read/write_user)
    PROBE_TYPED = auto()  # ProbeForWriteHandle/Ulong/etc (-> probe_write_user::<T>)
    PROBE_UNICODE = auto()  # ProbeForReadUnicodeString (-> probe_capture_unicode_string_header)
    WRITE_BACK = auto()  # *ptr = value (-> probe_write_user)
    BULK_COPY = auto()  # RtlCopyMemory et al (-> safe_copy_to/from_user)
    CAPTURE = auto()  # SepCapture*/ObpCapture* (-> capture helpers)
    POOL_ALLOC = auto()  # ExAllocatePool inside try (incidental)
    OTHER = auto()  # Unclassified code


class HandlerPattern(Enum):
    """What's in the _SEH2_EXCEPT / _SEH2_FINALLY handler."""

    SET_STATUS = auto()  # Status = _SEH2_GetExceptionCode() (-> ? operator)
    YIELD_RETURN = auto()  # _SEH2_YIELD(return ...) (-> ? operator)
    YIELD_GOTO = auto()  # _SEH2_YIELD(goto Cleanup) (-> ? with Drop guard)
    CLEANUP = auto()  # Calls Free/Release/Dereference in handler (-> Drop guard)
    SET_FLAG = auto()  # Sets a flag/break variable
    FINALLY = auto()  # _SEH2_FINALLY block (-> Drop guard)
    SYSTEM_FILTER = auto()  # ExSystemExceptionFilter() (-> special handling)


@dataclass
class SEHBlock:
    """A single _SEH2_TRY..._SEH2_EXCEPT/_SEH2_FINALLY..._SEH2_END block."""

    function_name: str
    line_start: int
    line_end: int
    try_body: str
    handler_body: str
    filter_expr: str
    try_patterns: list[TryBodyPattern] = field(default_factory=list)
    handler_patterns: list[HandlerPattern] = field(default_factory=list)
    is_finally: bool = False
    probe_calls: list[str] = field(default_factory=list)
    typed_probes: list[str] = field(default_factory=list)
    write_backs: list[str] = field(default_factory=list)
    copy_calls: list[str] = field(default_factory=list)
    cleanup_calls: list[str] = field(default_factory=list)


@dataclass
class FunctionSEHInfo:
    """SEH classification for a single C function."""

    name: str
    file: str
    subsystem: str
    line_start: int
    line_end: int
    return_type: str
    body_text: str = ""
    seh_blocks: list[SEHBlock] = field(default_factory=list)

    @property
    def all_try_patterns(self) -> set[TryBodyPattern]:
        result = set()
        for b in self.seh_blocks:
            result.update(b.try_patterns)
        return result

    @property
    def all_handler_patterns(self) -> set[HandlerPattern]:
        result = set()
        for b in self.seh_blocks:
            result.update(b.handler_patterns)
        return result

    @property
    def needs_safe_copy(self) -> bool:
        return TryBodyPattern.BULK_COPY in self.all_try_patterns

    @property
    def needs_raii_guard(self) -> bool:
        hp = self.all_handler_patterns
        return (
            HandlerPattern.CLEANUP in hp
            or HandlerPattern.YIELD_GOTO in hp
            or HandlerPattern.FINALLY in hp
        )

    @property
    def rust_api_surface(self) -> list[str]:
        """List of Rust API functions this function will need."""
        apis = []
        tp = self.all_try_patterns
        hp = self.all_handler_patterns
        if TryBodyPattern.PROBE in tp:
            apis.append("probe_user_buffer")
        if TryBodyPattern.PROBE_TYPED in tp or TryBodyPattern.WRITE_BACK in tp:
            apis.append("probe_write_user::<T>")
            apis.append("probe_read_user::<T>")
        if TryBodyPattern.PROBE_UNICODE in tp:
            apis.append("probe_capture_unicode_string_header")
        if TryBodyPattern.BULK_COPY in tp:
            apis.append("safe_copy_to_user / safe_copy_from_user")
        if HandlerPattern.CLEANUP in hp or HandlerPattern.FINALLY in hp:
            apis.append("RAII Drop guard")
        if HandlerPattern.YIELD_GOTO in hp:
            apis.append("RAII Drop guard (goto Cleanup)")
        return apis


# --- Regex patterns for classification ---

# Try body patterns
RE_PROBE_READ = re.compile(r"\bProbeForRead\s*\(")
RE_PROBE_WRITE = re.compile(r"\bProbeForWrite\s*\(")
RE_PROBE_TYPED = re.compile(
    r"\bProbeFor(Write|Read)(Handle|Ulong|UnicodeString|LargeInteger|Pointer|Boolean|UlongPtr)\s*\("
)
RE_PROBE_READ_UNICODE = re.compile(r"\bProbeForReadUnicodeString\s*\(")
RE_WRITE_BACK = re.compile(r"\*\s*(\w+)\s*=\s*([^=].*?);")
RE_BULK_COPY = re.compile(
    r"\b(RtlCopyMemory|RtlMoveMemory|RtlCopySidAndAttributesArray|"
    r"RtlCopyLuidAndAttributesArray|RtlCopyUnicodeString|"
    r"memcpy|memmove)\s*\("
)
RE_CAPTURE = re.compile(r"\b(SepCapture|ObpCapture)\w+\s*\(")
RE_POOL_ALLOC = re.compile(r"\bExAllocatePool\w*\s*\(")

# Handler patterns
RE_SET_STATUS = re.compile(r"\bStatus\s*=\s*_SEH2_GetExceptionCode\s*\(\s*\)")
RE_YIELD_RETURN = re.compile(r"_SEH2_YIELD\s*\(\s*return\s")
RE_YIELD_GOTO = re.compile(r"_SEH2_YIELD\s*\(\s*goto\s")
RE_CLEANUP_CALL = re.compile(
    r"\b(ExFreePool\w*|ObDereferenceObject|ObpRelease\w*|"
    r"SepRelease\w*|KeRelease\w*|ExRelease\w*|"
    r"IoFreeIrp|IopCleanup\w*|CcUnpinData|FsRtlFree\w*)\s*\("
)
RE_SET_FLAG = re.compile(r"\b(Break|BadStack|Ret|Result)\s*=\s*")
RE_SYSTEM_FILTER = re.compile(r"ExSystemExceptionFilter\s*\(")

# Function signature (handles NTAPI, FASTCALL, static, etc.)
RE_FUNC_SIG = re.compile(
    r"^(?:static\s+)?"  # optional static
    r'(?:CODE_SEG\("[^"]*"\)\s*)?'  # optional CODE_SEG
    r"(\w[\w\s\*]*?)\s+"  # return type
    r"(?:NTAPI|FASTCALL|WINAPI)?\s*"  # optional calling convention
    r"(\w+)\s*\(",  # function name
    re.MULTILINE,
)


def _classify_try_body(body: str, block: SEHBlock) -> None:
    """Classify what's inside a _SEH2_TRY block."""
    if RE_PROBE_READ.search(body):
        block.try_patterns.append(TryBodyPattern.PROBE)
        block.probe_calls.extend(m.group(0).rstrip("( ") for m in RE_PROBE_READ.finditer(body))
    if RE_PROBE_WRITE.search(body):
        block.try_patterns.append(TryBodyPattern.PROBE)
        block.probe_calls.extend(m.group(0).rstrip("( ") for m in RE_PROBE_WRITE.finditer(body))

    for m in RE_PROBE_TYPED.finditer(body):
        block.try_patterns.append(TryBodyPattern.PROBE_TYPED)
        block.typed_probes.append(m.group(0).rstrip("( "))

    if RE_PROBE_READ_UNICODE.search(body):
        block.try_patterns.append(TryBodyPattern.PROBE_UNICODE)

    for m in RE_WRITE_BACK.finditer(body):
        full = m.group(0)
        if not any(p in full for p in ("ProbeFor", "RtlCopy", "memcpy")):
            block.try_patterns.append(TryBodyPattern.WRITE_BACK)
            block.write_backs.append(full.strip())

    if RE_BULK_COPY.search(body):
        block.try_patterns.append(TryBodyPattern.BULK_COPY)
        block.copy_calls.extend(m.group(0).rstrip("( ") for m in RE_BULK_COPY.finditer(body))

    if RE_CAPTURE.search(body):
        block.try_patterns.append(TryBodyPattern.CAPTURE)

    if RE_POOL_ALLOC.search(body):
        block.try_patterns.append(TryBodyPattern.POOL_ALLOC)

    if not block.try_patterns:
        block.try_patterns.append(TryBodyPattern.OTHER)

    block.try_patterns = list(dict.fromkeys(block.try_patterns))


def _classify_handler(handler: str, filter_expr: str, block: SEHBlock) -> None:
    """Classify what's in the EXCEPT/FINALLY handler."""
    if block.is_finally:
        block.handler_patterns.append(HandlerPattern.FINALLY)
        return

    if RE_SYSTEM_FILTER.search(filter_expr):
        block.handler_patterns.append(HandlerPattern.SYSTEM_FILTER)

    if RE_SET_STATUS.search(handler):
        block.handler_patterns.append(HandlerPattern.SET_STATUS)

    if RE_YIELD_RETURN.search(handler):
        block.handler_patterns.append(HandlerPattern.YIELD_RETURN)

    if RE_YIELD_GOTO.search(handler):
        block.handler_patterns.append(HandlerPattern.YIELD_GOTO)

    if RE_CLEANUP_CALL.search(handler):
        block.handler_patterns.append(HandlerPattern.CLEANUP)
        block.cleanup_calls.extend(
            m.group(0).rstrip("( ") for m in RE_CLEANUP_CALL.finditer(handler)
        )

    if RE_SET_FLAG.search(handler):
        block.handler_patterns.append(HandlerPattern.SET_FLAG)

    if not block.handler_patterns:
        block.handler_patterns.append(HandlerPattern.SET_STATUS)

    block.handler_patterns = list(dict.fromkeys(block.handler_patterns))


def _find_brace_matched_block(text: str, start: int) -> tuple[int, int] | None:
    """Find the { ... } block starting at or after `start`. Returns (open, close) indices."""
    open_pos = text.find("{", start)
    if open_pos == -1:
        return None
    depth = 1
    i = open_pos + 1
    while i < len(text) and depth > 0:
        if text[i] == "{":
            depth += 1
        elif text[i] == "}":
            depth -= 1
        # Skip string/char literals
        elif text[i] in ('"', "'"):
            quote = text[i]
            i += 1
            while i < len(text) and text[i] != quote:
                if text[i] == "\\":
                    i += 1
                i += 1
        # Skip // comments
        elif text[i] == "/" and i + 1 < len(text) and text[i + 1] == "/":
            while i < len(text) and text[i] != "\n":
                i += 1
        # Skip /* */ comments
        elif text[i] == "/" and i + 1 < len(text) and text[i + 1] == "*":
            i += 2
            while i + 1 < len(text) and not (text[i] == "*" and text[i + 1] == "/"):
                i += 1
            i += 1  # skip the /
        i += 1
    if depth == 0:
        return (open_pos, i - 1)
    return None


def _extract_seh_blocks(source: str, func_name: str) -> list[SEHBlock]:
    """Extract SEH blocks from source text using regex + brace counting."""
    blocks = []
    pos = 0
    while True:
        # Find next _SEH2_TRY
        try_match = re.search(r"\b_SEH2_TRY\b", source[pos:])
        if try_match is None:
            break
        try_pos = pos + try_match.start()
        try_line = source[:try_pos].count("\n") + 1

        # Find the try body (next { ... } block)
        try_block = _find_brace_matched_block(source, try_pos + len("_SEH2_TRY"))
        if try_block is None:
            pos = try_pos + 1
            continue
        try_body = source[try_block[0] + 1 : try_block[1]]

        # After try body, look for _SEH2_EXCEPT or _SEH2_FINALLY
        after_try = try_block[1] + 1
        handler_body = ""
        filter_expr = ""
        is_finally = False
        end_line = try_line

        # Search for EXCEPT or FINALLY within reasonable range (500 chars)
        search_zone = source[after_try : after_try + 500]

        except_match = re.search(r"_SEH2_EXCEPT\s*\(", search_zone)
        finally_match = re.search(r"_SEH2_FINALLY\b", search_zone)

        if except_match:
            # Extract filter expression (balance parens)
            filter_start = after_try + except_match.end()
            paren_depth = 1
            fi = filter_start
            while fi < len(source) and paren_depth > 0:
                if source[fi] == "(":
                    paren_depth += 1
                elif source[fi] == ")":
                    paren_depth -= 1
                fi += 1
            filter_expr = source[filter_start : fi - 1].strip()

            # Handler body
            handler_block = _find_brace_matched_block(source, fi)
            if handler_block:
                handler_body = source[handler_block[0] + 1 : handler_block[1]]
                end_line = source[: handler_block[1]].count("\n") + 1

        elif finally_match:
            is_finally = True
            finally_pos = after_try + finally_match.end()
            handler_block = _find_brace_matched_block(source, finally_pos)
            if handler_block:
                handler_body = source[handler_block[0] + 1 : handler_block[1]]
                end_line = source[: handler_block[1]].count("\n") + 1

        block = SEHBlock(
            function_name=func_name,
            line_start=try_line,
            line_end=end_line,
            try_body=try_body,
            handler_body=handler_body,
            filter_expr=filter_expr,
            is_finally=is_finally,
        )
        _classify_try_body(try_body, block)
        _classify_handler(handler_body, filter_expr, block)
        blocks.append(block)

        pos = try_block[1] + 1

    return blocks


def _extract_functions_with_seh(source: str) -> list[tuple[str, str, int, int, str]]:
    """Extract (name, return_type, line_start, line_end, body) for functions containing SEH.

    Uses a hybrid approach: regex to find function signatures, then brace-counting
    to find the function body boundaries.
    """
    results = []

    # Find all function-like definitions by looking for pattern: type name(
    # We need to handle multi-line signatures
    for m in RE_FUNC_SIG.finditer(source):
        return_type = m.group(1).strip()
        func_name = m.group(2)
        sig_start = m.start()
        line_start = source[:sig_start].count("\n") + 1

        # Find the function body (brace-matched block starting from the signature)
        body_match = _find_brace_matched_block(source, m.end())
        if body_match is None:
            continue

        body = source[body_match[0] : body_match[1] + 1]
        line_end = source[: body_match[1]].count("\n") + 1

        # Only include functions with SEH
        if "_SEH2_TRY" in body:
            results.append((func_name, return_type, line_start, line_end, body))

    return results


def classify_file(path: Path) -> list[FunctionSEHInfo]:
    """Parse a C file and classify all SEH blocks by function."""
    source = path.read_text(errors="replace")

    # Determine subsystem from path
    parts = path.parts
    subsys = "unknown"
    for i, p in enumerate(parts):
        if p == "ntoskrnl" and i + 1 < len(parts):
            subsys = parts[i + 1]
            break

    results = []
    funcs = _extract_functions_with_seh(source)

    for func_name, return_type, line_start, line_end, body in funcs:
        blocks = _extract_seh_blocks(body, func_name)
        if blocks:
            results.append(
                FunctionSEHInfo(
                    name=func_name,
                    file=str(path),
                    subsystem=subsys,
                    line_start=line_start,
                    line_end=line_end,
                    return_type=return_type,
                    body_text=body,
                    seh_blocks=blocks,
                )
            )

    return results


def classify_directory(dir_path: Path) -> list[FunctionSEHInfo]:
    """Classify all C files in a directory tree."""
    results = []
    for c_file in sorted(dir_path.rglob("*.c")):
        results.extend(classify_file(c_file))
    return results


def print_classification(infos: list[FunctionSEHInfo], verbose: bool = False) -> None:
    """Print classification results in a human-readable table."""
    if not infos:
        print("No SEH blocks found.")
        return

    try_counts: dict[TryBodyPattern, int] = {}
    handler_counts: dict[HandlerPattern, int] = {}
    total_blocks = 0

    for info in infos:
        for block in info.seh_blocks:
            total_blocks += 1
            for p in block.try_patterns:
                try_counts[p] = try_counts.get(p, 0) + 1
            for p in block.handler_patterns:
                handler_counts[p] = handler_counts.get(p, 0) + 1

    print(f"\n{'=' * 70}")
    print(f"SEH Classification: {total_blocks} blocks in {len(infos)} functions")
    print(f"{'=' * 70}\n")

    print("TRY body patterns:")
    for p in sorted(try_counts.keys(), key=lambda x: try_counts[x], reverse=True):
        pct = try_counts[p] * 100 / total_blocks
        print(f"  {try_counts[p]:>4} ({pct:4.1f}%)  {p.name}")

    print("\nHandler patterns:")
    for p in sorted(handler_counts.keys(), key=lambda x: handler_counts[x], reverse=True):
        pct = handler_counts[p] * 100 / total_blocks
        print(f"  {handler_counts[p]:>4} ({pct:4.1f}%)  {p.name}")

    need_safe_copy = sum(1 for i in infos if i.needs_safe_copy)
    need_raii = sum(1 for i in infos if i.needs_raii_guard)
    print(f"\nFunctions needing safe_copy:  {need_safe_copy}")
    print(f"Functions needing RAII guard: {need_raii}")

    if verbose:
        print(f"\n{'=' * 70}")
        # Group by subsystem
        by_subsys: dict[str, list[FunctionSEHInfo]] = {}
        for info in infos:
            by_subsys.setdefault(info.subsystem, []).append(info)

        for subsys in sorted(by_subsys.keys()):
            sub_infos = by_subsys[subsys]
            sub_blocks = sum(len(i.seh_blocks) for i in sub_infos)
            print(f"\n--- {subsys}/ ({sub_blocks} blocks in {len(sub_infos)} functions) ---")
            for info in sub_infos:
                apis = info.rust_api_surface
                api_str = ", ".join(apis) if apis else "? only"
                print(
                    f"\n  {info.name}()  L{info.line_start}-{info.line_end}"
                    f"  [{len(info.seh_blocks)} SEH]"
                )
                print(f"    Needs: {api_str}")
                for block in info.seh_blocks:
                    tp = ", ".join(p.name for p in block.try_patterns)
                    hp = ", ".join(p.name for p in block.handler_patterns)
                    print(f"      L{block.line_start}: try=[{tp}] handler=[{hp}]")
                    if block.write_backs:
                        for wb in block.write_backs[:3]:
                            print(f"        write-back: {wb[:70]}")
                    if block.copy_calls:
                        for cc in block.copy_calls:
                            print(f"        copy: {cc}")
                    if block.cleanup_calls:
                        for cl in block.cleanup_calls:
                            print(f"        cleanup: {cl}")


def to_json(infos: list[FunctionSEHInfo]) -> str:
    """Serialize classification to JSON for pipeline consumption."""
    data = []
    for info in infos:
        blocks = []
        for b in info.seh_blocks:
            blocks.append(
                {
                    "line_start": b.line_start,
                    "line_end": b.line_end,
                    "filter": b.filter_expr,
                    "is_finally": b.is_finally,
                    "try_patterns": [p.name for p in b.try_patterns],
                    "handler_patterns": [p.name for p in b.handler_patterns],
                    "probe_calls": b.probe_calls,
                    "typed_probes": b.typed_probes,
                    "write_backs": b.write_backs[:5],
                    "copy_calls": b.copy_calls,
                    "cleanup_calls": b.cleanup_calls,
                }
            )
        data.append(
            {
                "name": info.name,
                "file": info.file,
                "subsystem": info.subsystem,
                "line_start": info.line_start,
                "line_end": info.line_end,
                "return_type": info.return_type,
                "seh_blocks": blocks,
                "needs_safe_copy": info.needs_safe_copy,
                "needs_raii_guard": info.needs_raii_guard,
                "rust_api_surface": info.rust_api_surface,
            }
        )
    return json.dumps(data, indent=2)
