"""body_audit.py — did a translated body keep what its C body DOES?

Call sealing (`--seal-calls`) makes a dropped call fail loudly at translation time. Crates
translated BEFORE sealing never had that check, and they lose statements silently. The case
that made this a tool: fastfat's `FatCreateNewDirent` lost the tail of its C body, including
`RtlSetBits` on the free-dirent bitmap. It compiled, mounted and passed smoke on three arches
for months while every new directory entry stayed FREE in memory. A later create that reused
deleted slots could be handed a live entry and overwrite another file's dirent (2026-09-26).

For every C function the module defines that has a Rust definition of the same name in the
module's crates, compare the C body's callees with the Rust body. A callee counts as KEPT when
its name appears in the Rust body, in a crate-local helper that body reaches (wrappers such as
cdrom's `DeviceEvtFileClose` -> `CdromDeviceEvtFileClose`, depth 3), or through a known
lowering (`RtlZeroMemory` -> `write_bytes`, `KeAcquireSpinLock` ->
`KeAcquireSpinLockRaiseToDpc`, ...). Callees in code the build compiles but never runs (a
disabled `#if` arm, a DUMMY_PSEH except handler) and in `#if DBG` code are OPTIONAL and never
reported. `_SEH2_FINALLY` blocks DO run under DUMMY_PSEH, so their calls stay required.

This is a lead generator for a human reading the C, not a gate: a macro whose expansion the
Rust open-codes under another name still reads as a drop. Findings are ranked so the ones that
lose STATE come first:

  stub-silent  the Rust body is a small fraction of the C and says nothing about it
  high         a dropped callee whose name mutates state (Set/Release/Unpin/Complete/...)
  medium       any other dropped function call
  low          a dropped memory op / ALL-CAPS macro with no recognised lowering
  stub-loud    a small body that bugchecks or returns NOT_IMPLEMENTED (known, not silent)

A lead someone has read against the C is recorded in `data/body_audit_reviewed.toml` with its
verdict, so the next run does not hand it out again: `false` and `benign` leads are hidden
(counted in the header), `owed` leads stay listed with where the work is tracked. The record
covers the callees it names: a drop the reviewer never saw brings the lead back. A false lead
that a RULE could have recognised belongs in the rules, not the ledger.
"""

from __future__ import annotations

import argparse
import json
import re
import subprocess
import sys
from dataclasses import asdict, dataclass, field
from pathlib import Path

import tree_sitter_rust
from tools.source_walk import source_files
from tree_sitter import Language, Parser

from .c_defs import c_functions, compiled_tus
from .dead_regions import dummy_pseh_except_spans, never_compiled_lines
from .export_audit import _repo_root_of, module_rust_crates
from .lex_mask import directive_spans, mask_code
from .profile_derive import ProfileDeriveError

try:
    import tomllib  # type: ignore[import-not-found]  # stdlib on 3.11+; tomli backport on 3.10
except ModuleNotFoundError:
    import tomli as tomllib  # type: ignore[no-redef]

_CALL_RE = re.compile(r"\b([A-Za-z_]\w*)\s*\(")
_IDENT_RE = re.compile(r"\b[A-Za-z_]\w*\b")
_C_KEYWORDS = frozenset(
    {
        *("if", "else", "for", "while", "switch", "return", "sizeof", "do", "case"),
        *("default", "goto", "typedef", "struct", "union", "enum", "defined"),
        *("__typeof__", "__alignof__", "_Alignof"),
    }
)
# Never a finding: tracing, assertions, SAL, and the SEH scaffolding itself. None of these
# changes what the function does in a release build.
_IGNORED = re.compile(
    r"^(?:DebugTrace|DebugDump|DebugUnwind|DbgPrint\w*|KdPrint\w*|TracePrint\w*|TraceLog\w*|"
    r"DPRINT\w*|TRACE\w*|WARN|ERR|FIXME|ASSERT\w*|NT_ASSERT\w*|NT_VERIFY\w*|PAGED_CODE|"
    r"UNREFERENCED_PARAMETER|UNREFERENCED_LOCAL_VARIABLE|DBG_UNREFERENCED\w*|_Analysis_\w+|"
    r"_SEH2_\w+|__try|__except|__finally|__leave|try_return|try_leave|TimerStart|TimerStop|"
    r"C_ASSERT|FIELD_OFFSET|RTL_FIELD_SIZE|ARRAYSIZE|RTL_NUMBER_OF\w*|_countof|warning|"
    r"__analysis_assume|__assume|UNIMPLEMENTED|DBGCHK\w*|WPP_\w+|DoTraceMessage|_Pragma|"
    r"except|GetExceptionCode|GetExceptionInformation|_exception_code|_exception_info|"
    r"ULongToPtr|PtrToUlong|PtrToUshort|UlongToPtr|LongToPtr|PtrToLong|HandleToUlong|"
    r"UlongToHandle|IntToPtr|PtrToInt|NT_SUCCESS|NT_ERROR|NT_WARNING|NT_INFORMATION|debugstr_\w+|wine_dbgstr_\w+|wine_dbg_sprintf|"
    r"pdump|pcheck|FIXME_\w*|TRACE_\w*|WARN_\w*|ERR_\w*|MAKEINTRESOURCE[AW]?|SUCCEEDED|FAILED|"
    r"RTL_CONSTANT_STRING|DECLARE_CONST_UNICODE_STRING|HRESULT_FROM_WIN32|"
    # Pure or lowered by construction: string-literal and layout macros, varargs, VARIANT
    # accessors, ACPICA's trace-and-return, Wine list iteration and TRACE-level dumps.
    r"_T|TEXT|offsetof|VOID|assert|va_start|va_end|va_arg|va_copy|V_[A-Z]\w*|"
    r"Int32x32To64|UInt32x32To64|Int64ShllMod32|return_\w+|LIST_FOR_EACH\w*|dump_\w+|MAKELONG|MAKEWORD|LOWORD|HIWORD|LOBYTE|HIBYTE|MAKEWPARAM|MAKELPARAM)$"
)
# Names whose Rust spelling is legitimately different. Any listed pattern satisfies the callee.
_LOWERINGS: dict[str, tuple[str, ...]] = {
    "ZeroMemory": (r"write_bytes", r"zeroed", r"memset", r"RtlZeroMemory", r"fill\(\s*0"),
    "CopyMemory": (r"copy_nonoverlapping", r"ptr::copy", r"\bcopy\(", r"memcpy", r"RtlCopyMemory"),
    "MoveMemory": (r"ptr::copy", r"\bcopy\(", r"memmove", r"RtlMoveMemory"),
    "FillMemory": (r"write_bytes", r"memset", r"\.fill\("),
    "strcpyW": (r"wcscpy", r"lstrcpyW", r"copy", r"strcpyW"),
    "wcscpy": (r"wcscpy", r"lstrcpyW", r"copy"),
    "strlenW": (r"wcslen", r"lstrlenW", r"\.len\(\)", r"strlenW"),
    "lstrlenW": (r"wcslen", r"lstrlenW", r"\.len\(\)"),
    "wcslen": (r"wcslen", r"lstrlenW", r"strlenW", r"\.len\(\)"),
    "lstrcpyW": (r"wcscpy", r"lstrcpyW", r"strcpyW", r"copy"),
    "strcpy": (r"strcpy", r"lstrcpyA", r"copy"),
    "RtlZeroMemory": (r"write_bytes", r"zeroed", r"memset", r"fill\(\s*0"),
    "RtlSecureZeroMemory": (r"write_bytes", r"write_volatile", r"zeroed"),
    "RtlFillMemory": (r"write_bytes", r"memset", r"\.fill\("),
    "RtlFillMemoryUlong": (r"write_bytes", r"\.fill\(", r"write_unaligned", r"\bwrite\("),
    "memset": (r"write_bytes", r"zeroed", r"\.fill\("),
    "RtlCopyMemory": (r"copy_nonoverlapping", r"ptr::copy", r"\bcopy\(", r"copy_from", r"memcpy"),
    "RtlMoveMemory": (
        r"ptr::copy",
        r"\bcopy\(",
        r"copy_nonoverlapping",
        r"memmove",
        r"copy_within",
    ),
    "memcpy": (r"copy_nonoverlapping", r"ptr::copy", r"\bcopy\(", r"copy_from"),
    "memmove": (r"ptr::copy", r"\bcopy\(", r"copy_within"),
    "RtlEqualMemory": (r"==", r"memcmp", r"RtlCompareMemory"),
    "RtlCompareMemory": (r"memcmp", r"==", r"iter\(\)"),
    "IsEqualGUID": (r"==", r"memcmp", r"RtlCompareMemory", r"IsEqualGUID"),
    "InlineIsEqualGUID": (r"==", r"IsEqualGUID"),
    # wdm.h FORCEINLINE: three stores into the next stack location (fastfat's FatSingleAsync).
    "IoSetCompletionRoutine": (r"\.CompletionRoutine\s*=",),
    "KeAcquireSpinLock": (r"KeAcquireSpinLock\w*", r"KfAcquireSpinLock"),
    "KeReleaseSpinLock": (r"KeReleaseSpinLock\w*", r"KfReleaseSpinLock"),
    "KeRaiseIrql": (r"KfRaiseIrql", r"KeRaiseIrql\w*"),
    "KeLowerIrql": (r"KfLowerIrql", r"KeLowerIrql"),
    "ExInitializeWorkItem": (r"WorkerRoutine",),
    "InitializeListHead": (r"InitializeListHead", r"Flink"),
    "InsertTailList": (r"InsertTailList", r"Blink"),
    "InsertHeadList": (r"InsertHeadList", r"Flink"),
    "RemoveEntryList": (r"RemoveEntryList", r"Flink"),
    "RemoveHeadList": (r"RemoveHeadList", r"Flink"),
    "IsListEmpty": (r"IsListEmpty", r"Flink\s*==", r"==\s*\(?\*?\w*\)?\.?Flink"),
    "NT_SUCCESS": (r">=\s*0", r"NT_SUCCESS", r"< 0", r"<\s*0"),
    "min": (r"\.min\(", r"\bmin\b", r"cmp::min"),
    "max": (r"\.max\(", r"\bmax\b", r"cmp::max"),
    "CONTAINING_RECORD": (r"offset_of", r"CONTAINING_RECORD", r"\.sub\(", r"byte_sub"),
    "ObDereferenceObject": (r"ObDereferenceObject", r"ObfDereferenceObject"),
    "ObReferenceObject": (r"ObReferenceObject", r"ObfReferenceObject"),
    "IoCallDriver": (r"IoCallDriver", r"IofCallDriver"),
    "IoCompleteRequest": (r"IoCompleteRequest", r"IofCompleteRequest"),
    "IoGetCurrentIrpStackLocation": (r"CurrentStackLocation", r"IoGetCurrentIrpStackLocation"),
    "IoGetNextIrpStackLocation": (r"CurrentStackLocation", r"IoGetNextIrpStackLocation"),
    "IoSkipCurrentIrpStackLocation": (r"CurrentLocation", r"IoSkipCurrentIrpStackLocation"),
    "IoCopyCurrentIrpStackLocationToNext": (
        r"IoCopyCurrentIrpStackLocationToNext",
        r"copy_nonoverlapping",
        r"CompletionRoutine",
    ),
    "InterlockedIncrement": (r"Interlocked\w*", r"fetch_add", r"atomic"),
    "InterlockedDecrement": (r"Interlocked\w*", r"fetch_sub", r"atomic"),
    "InterlockedExchange": (r"Interlocked\w*", r"swap", r"atomic"),
    "InterlockedCompareExchange": (r"Interlocked\w*", r"compare_exchange", r"atomic"),
    "InterlockedExchangePointer": (r"Interlocked\w*", r"swap", r"atomic"),
    "InterlockedCompareExchangePointer": (r"Interlocked\w*", r"compare_exchange", r"atomic"),
    "InterlockedOr": (r"Interlocked\w*", r"fetch_or", r"atomic"),
    # winuser.h wrappers over the Ex / Param entry points.
    "CreateWindowW": (r"CreateWindowExW",),
    "CreateWindowA": (r"CreateWindowExA",),
    "CreateDialogW": (r"CreateDialogParamW",),
    "CreateDialogA": (r"CreateDialogParamA",),
    "DialogBoxW": (r"DialogBoxParamW",),
    "DialogBoxA": (r"DialogBoxParamA",),
    # Wine CRT allocation in a port that went to the process heap.
    "malloc": (r"HeapAlloc", r"heap_alloc", r"malloc"),
    "calloc": (r"HeapAlloc", r"heap_alloc_zero", r"calloc"),
    "realloc": (r"HeapReAlloc", r"heap_realloc", r"realloc"),
    "free": (r"HeapFree", r"heap_free", r"\bfree\b"),
    "InterlockedExchangeAdd": (r"Interlocked\w*", r"fetch_add", r"atomic"),
    # Inline initialisers the Rust open-codes as stores (acpi's AcpiOsCreateLock).
    "KeInitializeSpinLock": (r"KeInitializeSpinLock", r"=\s*0\b", r"write\(\s*\w*,?\s*0\s*\)"),
    "ExAllocatePoolZero": (r"ExAllocatePool\w*",),
    "_disable": (r"\bcli\b", r"_disable"),
    "_enable": (r"\bsti\b", r"_enable"),
    "InterlockedAnd": (r"Interlocked\w*", r"fetch_and", r"atomic"),
}
_MEMORY_OPS = frozenset(k for k in _LOWERINGS if "Memory" in k or k.startswith("mem"))
# A dropped callee whose NAME says it changes state is the class that corrupts things.
_STATE_VERB = re.compile(
    r"(Set|Clear|Delete|Free|Release|Acquire|Unpin|Pin|Repin|Complete|Insert|Remove|"
    r"Initialize|Uninitialize|Close|Dereference|Reference|Truncate|Flush|Purge|Unlock|Lock|"
    r"Raise|Add|Write|Mark|Queue|Signal|Cancel|Destroy|Deallocate|Allocate|Create|Open|"
    r"Unmap|Map|Commit|Update|Enter|Leave|Wait|Post|Send|Start|Stop|Enable|Disable|Put|"
    r"Push|Pop|Register|Unregister|Attach|Detach|Invalidate|Notify|Reset|Store|Save|"
    r"Restore|Move|Rename|Grow|Shrink|Extend|Zero)"
)
_LOUD_STUB = re.compile(
    r"(?i:bug_?check)|STATUS_NOT_IMPLEMENTED|STATUS_NOT_SUPPORTED|STATUS_INVALID_DEVICE_REQUEST|"
    r"unimplemented!|todo!|panic!|UNIMPLEMENTED|unreachable_unchecked"
)
# A body this far below its C is a stub unless it delegates, which the helper walk measures.
_STUB_RATIO = 0.15
_STUB_MIN_C_LINES = 30
_HELPER_DEPTH = 3

SEVERITY_ORDER = ("stub-silent", "high", "medium", "low", "stub-loud")
REVIEWED_PATH = Path(__file__).resolve().parent.parent / "data" / "body_audit_reviewed.toml"
VERDICTS = ("false", "benign", "owed")


@dataclass
class Finding:
    function: str
    c_file: str
    rust_file: str
    severity: str
    c_lines: int
    rust_lines: int
    dropped: list[str] = field(default_factory=list)
    # From the reviewed ledger: (verdict, note) when every dropped callee was reviewed.
    reviewed: tuple[str, str] | None = None
    # Dropped callees the C file did not contain when the Rust file was first committed:
    # upstream work that landed on C already flipped, not something the translator lost.
    upstream: list[str] = field(default_factory=list)


def _git(repo: Path, *args: str) -> str | None:
    r = subprocess.run(["git", "-C", str(repo), *args], capture_output=True)
    return r.stdout.decode("utf-8", errors="replace") if r.returncode == 0 else None


def mark_upstream(findings: list[Finding], repo: Path) -> None:
    """Fill `upstream` from git: the C as it stood at the Rust file's first commit."""
    born: dict[str, str | None] = {}
    then: dict[tuple[str, str], str | None] = {}
    for f in findings:
        if f.rust_file not in born:
            revs = _git(repo, "log", "--diff-filter=A", "--format=%H", "--", f.rust_file)
            born[f.rust_file] = revs.split()[-1] if revs and revs.split() else None
        rev = born[f.rust_file]
        if rev is None:
            continue
        if (rev, f.c_file) not in then:
            then[(rev, f.c_file)] = _git(repo, "show", f"{rev}:{f.c_file}")
        old = then[(rev, f.c_file)]
        if old is None:
            continue
        # Judge against the function's own old body: dhcpcsvc's state_panic gained a whole
        # alternate-configuration arm full of RegSetValueExA calls the FILE already had.
        # A function the old file never named is new outright; one it named but whose body
        # the regex could not frame falls back to the whole file rather than calling all
        # of it new.
        body = _old_body(old, f.function)
        if body is None and re.search(rf"\b{re.escape(f.function)}\b", old):
            body = old
        idents = set(_IDENT_RE.findall(body or ""))
        f.upstream = [c for c in f.dropped if c.split("(", 1)[0] not in idents]


def _old_body(text: str, name: str) -> str | None:
    masked = mask_code(text)
    # Only a definition: from the line start, nothing but a return type before the name, so
    # cdrom's `if (BootEnvironmentIsWinPE()) {` is not taken for the function's body.
    pat = rf"(?m)^[ \t]*(?:[\w*]+[ \t*]+)*{re.escape(name)}\s*\([^;{{]*?\)\s*\{{"
    for m in re.finditer(pat, masked):
        end = _matching(masked, m.end() - 1, "{", "}")
        if end is not None:
            return masked[m.end() - 1 : end + 1]
    return None


def load_reviewed(path: Path = REVIEWED_PATH) -> dict[tuple[str, str], dict]:
    """`(module, function)` -> its ledger entry. A malformed entry is an error, not a skip."""
    if not path.exists():
        return {}
    out: dict[tuple[str, str], dict] = {}
    for e in tomllib.loads(path.read_text(encoding="utf-8")).get("reviewed", []):
        key = (e["module"], e["function"])
        if e.get("verdict") not in VERDICTS or not e.get("note") or not e.get("callees"):
            raise ValueError(f"{path}: {key}: needs verdict in {VERDICTS}, callees and a note")
        if key in out:
            raise ValueError(f"{path}: {key} is listed twice")
        out[key] = e
    return out


def apply_reviewed(module: str, findings: list[Finding], ledger: dict) -> None:
    for f in findings:
        e = ledger.get((module, f.function))
        if e and set(f.dropped) <= set(e["callees"]):
            f.reviewed = (e["verdict"], e["note"])


def _blank_ignored_calls(text: str) -> str:
    """Blank the argument lists of ignored calls, preserving offsets.

    `NT_ASSERT(KeAreApcsDisabled())` asks nothing of a release build, so the calls inside an
    assertion's (or a trace's) arguments are not owed by the translation either.
    """
    out = list(text)
    for m in _CALL_RE.finditer(text):
        if not _IGNORED.match(m.group(1)):
            continue
        close = _matching(text, m.end() - 1, "(", ")")
        if close is None:
            continue
        for pos in range(m.end(), close):
            if out[pos] != "\n":
                out[pos] = " "
    return "".join(out)


def _line_of(text: str, pos: int) -> int:
    return text.count("\n", 0, pos) + 1


def _matching(text: str, open_idx: int, opener: str, closer: str) -> int | None:
    depth = 0
    for pos in range(open_idx, len(text)):
        ch = text[pos]
        if ch == opener:
            depth += 1
        elif ch == closer:
            depth -= 1
            if depth == 0:
                return pos
    return None


# `if (_SEH2_AbnormalTermination()) { ... }` with NOTHING else in the condition. Under
# DUMMY_PSEH (pseh2.h's `_USE_DUMMY_PSEH` arm) the macro is `(0)`, so the block is compiled and
# never entered -- fastfat's create.c failure-unwind blocks (FatDeleteFcb, FatDeleteCcb) are all
# this shape. `_SEH2_AbnormalTermination() || !NT_SUCCESS(Status)` stays live and owed.
_ABNORMAL_ONLY_RE = re.compile(r"\bif\s*\(\s*(?:_SEH2_)?AbnormalTermination\s*\(\s*\)\s*\)\s*\{")


def _abnormal_only_spans(masked: str) -> list[tuple[int, int]]:
    spans = []
    for m in _ABNORMAL_ONLY_RE.finditer(masked):
        end = _matching(masked, m.end() - 1, "{", "}")
        if end is not None:
            spans.append((m.start(), end))
    return spans


# ---- version guards ---------------------------------------------------------------------
# `#if (NTDDI_VERSION > NTDDI_WIN8)` has a macro on BOTH sides, which dead_regions' evaluator
# (NAME op LITERAL) cannot decide, so Win8+ blocks read as live and their calls as owed:
# fastfat's FatMountVolume "dropped" IoVolumeDeviceToGuid, compiled out at our profile. The
# tree pins `-D_WIN32_WINNT=0x502` (top-level CMakeLists), which sdkddkver.h turns into
# NTDDI_WS03SP1; a module's own `-D_WIN32_WINNT=`/`-DNTDDI_VERSION=` overrides it.
_VERSION_ALIAS_RE = re.compile(
    r"^[ \t]*#[ \t]*define[ \t]+((?:NTDDI|_WIN32_WINNT|_WIN32_IE)_\w+)[ \t]+"
    r"((?:NTDDI|_WIN32_WINNT|_WIN32_IE)_\w+)[ \t]*(?://.*)?$",
    re.M,
)
_VERSION_DEFINE_RE = re.compile(
    r"^[ \t]*#[ \t]*define[ \t]+((?:NTDDI|_WIN32_WINNT|_WIN32_IE)_\w+)[ \t]+(0[xX][0-9a-fA-F]+)",
    re.M,
)
_COND_LINE_RE = re.compile(r"^([ \t]*#[ \t]*(?:if|elif))\b(.*)$")


def version_pins(module_dir: Path, repo: Path) -> dict[str, int]:
    pins: dict[str, int] = {}
    ddk = repo / "sdk" / "include" / "psdk" / "sdkddkver.h"
    if ddk.is_file():
        text = ddk.read_text(errors="replace")
        for name, value in _VERSION_DEFINE_RE.findall(text):
            pins.setdefault(name, int(value, 16))
        # `#define NTDDI_VISTA NTDDI_WIN6`: an alias guards fastfat's FatOpenVolume Vista block.
        for name, target in _VERSION_ALIAS_RE.findall(text):
            if target in pins:
                pins.setdefault(name, pins[target])
    cml = (module_dir / "CMakeLists.txt").read_text(errors="replace")
    winnt = re.search(r"-D_WIN32_WINNT=(0[xX][0-9a-fA-F]+)", cml)
    ntddi = re.search(r"-DNTDDI_VERSION=(0[xX][0-9a-fA-F]+)", cml)
    pins["_WIN32_WINNT"] = int(winnt.group(1), 16) if winnt else 0x502
    pins["WINVER"] = pins["_WIN32_WINNT"]
    if ntddi:
        pins["NTDDI_VERSION"] = int(ntddi.group(1), 16)
    elif pins["_WIN32_WINNT"] == 0x502:
        pins["NTDDI_VERSION"] = pins.get("NTDDI_WS03SP1", 0x05020100)  # sdkddkver.h:153
    else:
        pins["NTDDI_VERSION"] = pins["_WIN32_WINNT"] << 16
    pins.update(disabled_switches(module_dir, cml))
    return pins


_COMMENTED_DEFINE_RE = re.compile(r"^[ \t]*//[ \t]*#[ \t]*define[ \t]+(\w+)", re.M)


def disabled_switches(module_dir: Path, cml: str) -> dict[str, int]:
    """Feature switches the module turns OFF by commenting out their only `#define`.

    wlansvc's rpcserver.c has `//#define GET_IF_ENTRY2_IMPLEMENTED 1` and guards the real
    _RpcEnumInterfaces body with `#if GET_IF_ENTRY2_IMPLEMENTED`: an undefined name in `#if`
    is 0, so the build compiles the UNIMPLEMENTED arm and a stub Rust body is faithful.
    Pinned to 0 only when no live `#define` of it exists in the module and no `-D` sets it.
    """
    texts = [
        p.read_text(encoding="utf-8", errors="replace")
        for pat in ("*.c", "*.h")
        for p in source_files(module_dir, pat)
    ]
    off = {name for t in texts for name in _COMMENTED_DEFINE_RE.findall(t)}
    out: dict[str, int] = {}
    for name in sorted(off):
        live = re.compile(rf"^[ \t]*#[ \t]*define[ \t]+{re.escape(name)}\b", re.M)
        if re.search(rf"-D{re.escape(name)}\b", cml) or any(live.search(t) for t in texts):
            continue
        out[name] = 0
    return out


def _fold(expr: str) -> bool | None:
    """Evaluate an all-numeric `#if` expression, or None when anything symbolic is left."""
    import ast

    expr = expr.split("//")[0].replace("&&", " and ").replace("||", " or ")
    expr = re.sub(r"!(?!=)", " not ", expr)
    expr = re.sub(r"(0[xX][0-9a-fA-F]+|\d+)[uUlL]+", r"\1", expr)
    try:
        tree = ast.parse(expr.strip(), mode="eval")
    except SyntaxError:
        return None
    allowed = (
        ast.Expression,
        ast.BoolOp,
        ast.And,
        ast.Or,
        ast.UnaryOp,
        ast.Not,
        ast.Compare,
        ast.Constant,
        ast.Eq,
        ast.NotEq,
        ast.Lt,
        ast.LtE,
        ast.Gt,
        ast.GtE,
    )
    if not all(isinstance(n, allowed) for n in ast.walk(tree)):
        return None
    return bool(eval(compile(tree, "<if>", "eval")))  # literal-only AST, checked above


_DEFINED_RE = re.compile(r"\bdefined\s*(?:\(\s*(\w+)\s*\)|(\w+))")
_IFDEF_LINE_RE = re.compile(r"^([ \t]*#[ \t]*)(ifdef|ifndef)[ \t]+(\w+)(.*)$")
# Defined for every TU this tree compiles: llvm-mingw forces DUMMY_PSEH on all three arches
# (sdk/cmake/gcc.cmake), and the root CMakeLists defines __REACTOS__.
TREE_DEFINES = frozenset({"__REACTOS__", "_USE_DUMMY_PSEH"})
# Never defined for a Windows target; Wine code guards host-only arms on them (advapi32
# CredReadW's `#ifdef __APPLE__`). In a `defines` set a `!` prefix means KNOWN undefined.
TREE_UNDEFINED = frozenset({"!__APPLE__", "!__MACH__", "!__linux__", "!__unix__", "!__FreeBSD__"})


def _fold_defined(expr: str, pins: dict[str, int], defines: frozenset[str]) -> bool | None:
    """Fold an `#if` whose only unknowns are `defined(X)` terms: try each of them both ways,
    and answer only when every assignment agrees (`defined(_WIN32K_) || defined(FOO)`)."""
    expr = re.sub(r"/\*.*?\*/", " ", expr).split("//")[0]
    unknown: list[str] = []

    def sub_defined(m: re.Match[str]) -> str:
        name = m.group(1) or m.group(2)
        if "!" + name in defines:
            return "0"
        if name in defines or (name in pins and pins[name] != 0):
            return "1"
        unknown.append(name)
        return f"__U{len(unknown) - 1}"

    expr = _DEFINED_RE.sub(sub_defined, expr)
    if pins:
        names = re.compile(
            r"\b(" + "|".join(map(re.escape, sorted(pins, key=len, reverse=True))) + r")\b"
        )
        expr = names.sub(lambda n: hex(pins[n.group(1)]), expr)
    if len(unknown) > 4:
        return None
    seen: set[bool] = set()
    for bits in range(1 << len(unknown)):
        trial = re.sub(r"__U(\d+)", lambda u, b=bits: str((b >> int(u.group(1))) & 1), expr)
        v = _fold(trial)
        if v is None:
            return None
        seen.add(v)
    return seen.pop() if len(seen) == 1 else None


def pin_version_guards(
    text: str, pins: dict[str, int], defines: frozenset[str] = TREE_DEFINES
) -> str:
    """Rewrite decidable `#if`s to `#if 0`/`#if 1`, keeping every line in place.

    Decidable means: over integer pins (versions, switches) and names the build always
    `-D`s (`defines`). A `defined(X)` of anything else is tried both ways."""
    out = []
    for line in text.split("\n"):
        d = _IFDEF_LINE_RE.match(line)
        if d and d.group(3) in defines:
            line = f"{d.group(1)}if {int(d.group(2) == 'ifdef')}{d.group(4)}"
        elif d and "!" + d.group(3) in defines:
            line = f"{d.group(1)}if {int(d.group(2) == 'ifndef')}{d.group(4)}"
        m = _COND_LINE_RE.match(line)
        if m and not re.fullmatch(r"\s*\d+\s*", m.group(2).split("//")[0]):
            folded = _fold_defined(m.group(2), pins, defines)
            if folded is not None:
                line = f"{m.group(1)} {int(folded)}"
        out.append(line)
    return "\n".join(out)


_CMAKE_D_RE = re.compile(r"-D([A-Za-z_]\w*)(?:=(\S+?))?(?=[\s)\"]|$)")


def module_defines(module_dir: Path) -> frozenset[str]:
    """Names the module's CMakeLists `-D`s unconditionally (outside every `if()` block),
    plus `!NAME` for names known undefined: host-OS macros and the module's commented-out
    feature switches (desk.c's `// #define CHECK_SCR_FULL_PATH` above `#ifdef` blocks)."""
    cml = module_dir / "CMakeLists.txt"
    names = set(TREE_DEFINES) | set(TREE_UNDEFINED)
    if not cml.is_file():
        return frozenset(names)
    text = cml.read_text(encoding="utf-8", errors="replace")
    names |= {"!" + n for n in disabled_switches(module_dir, text)}
    depth = 0
    for raw in text.splitlines():
        line = raw.split("#")[0]
        if re.match(r"\s*if\s*\(", line, re.I):
            depth += 1
        elif re.match(r"\s*endif\s*\(", line, re.I):
            depth = max(0, depth - 1)
        elif depth == 0:
            names.update(m.group(1) for m in _CMAKE_D_RE.finditer(line))
    return frozenset(names)


def _dbg_lines(text: str) -> set[int]:
    """Lines inside `#if DBG` / `#ifdef DBG` / `#if <module>DBG` arms: optional in release."""
    out: set[int] = set()
    stack: list[bool] = []
    for n, line in enumerate(text.splitlines(), 1):
        s = line.strip()
        m = re.match(r"#\s*(if|ifdef|ifndef|elif|else|endif)\b(.*)", s)
        if m:
            d, rest = m.group(1), m.group(2)
            if d in ("if", "ifdef"):
                stack.append(bool(re.search(r"\b\w*DBG\w*\b|\bDEBUG\b", rest)) and "!" not in rest)
            elif d == "ifndef":
                stack.append(False)
            elif d in ("else", "elif") and stack:
                stack[-1] = False
            elif d == "endif" and stack:
                stack.pop()
            continue
        if any(stack):
            out.add(n)
    return out


@dataclass
class CBody:
    name: str
    file: Path
    lines: int
    required: set[str]


def c_bodies(
    path: Path,
    pins: dict[str, int] | None = None,
    real_seh: bool = False,
    defines: frozenset[str] = TREE_DEFINES,
) -> dict[str, CBody]:
    """Each function this .c DEFINES, with the callees its LIVE body must perform."""
    text = path.read_text(encoding="utf-8", errors="replace")
    # acpi main.c's own `#define NDEBUG` decides its `#ifndef NDEBUG` blocks.
    if re.search(r"^[ \t]*#[ \t]*define[ \t]+NDEBUG\b", text, re.M):
        defines = defines | {"NDEBUG"}
    text = pin_version_guards(text, pins or {}, defines)
    masked = _blank_ignored_calls(mask_code(text))
    dead = never_compiled_lines(text)
    masked_lines = masked.splitlines()
    optional_lines = dead | _dbg_lines(text)
    # Under DUMMY_PSEH an except handler and an AbnormalTermination-only unwind never run, so
    # a port that omits them matches the shipped C. A crate on REAL SEH (fastfat, cdfs:
    # ntkrnl_sys::seh trampolines) runs both, and then they are owed like any other code.
    except_spans = (
        [] if real_seh else dummy_pseh_except_spans(text, masked) + _abnormal_only_spans(masked)
    )
    directive = directive_spans(text)
    out: dict[str, CBody] = {}
    for name in c_functions(text):
        for m in re.finditer(rf"\b{re.escape(name)}\s*\(", masked):
            close = _matching(masked, m.end() - 1, "(", ")")
            if close is None:
                continue
            brace = close + 1
            while brace < len(masked) and masked[brace] in " \t\r\n":
                brace += 1
            if brace >= len(masked) or masked[brace] != "{":
                continue
            end = _matching(masked, brace, "{", "}")
            if end is None:
                continue
            required: set[str] = set()
            for call in _CALL_RE.finditer(masked, brace, end):
                callee = call.group(1)
                if callee in _C_KEYWORDS or callee == name or _IGNORED.match(callee):
                    continue
                pos = call.start(1)
                if directive[pos]:  # `#pragma prefast(...)`, a nested `#define X(`
                    continue
                if _line_of(masked, pos) in optional_lines:
                    continue
                if any(a <= pos < b for a, b in except_spans):
                    continue
                required.add(callee)
            first, last = _line_of(masked, brace), _line_of(masked, end)
            # Only lines the build compiles: a dead `#else` arm is not body the Rust owes.
            live_lines = sum(
                1 for n in range(first, last + 1) if n not in dead and masked_lines[n - 1].strip()
            )
            out[name] = CBody(name, path, live_lines, required)
            break
    return out


# `#define KeWaitForMutexObject KeWaitForSingleObject`: an object-like ALIAS of a function.
_ALIAS_RE = re.compile(r"^[ \t]*#[ \t]*define[ \t]+([A-Za-z_]\w*)[ \t]+([A-Za-z_]\w*)[ \t]*$")
_MEMBER_ALIAS_RE = re.compile(
    r"^[ \t]*#[ \t]*define[ \t]+([A-Za-z_]\w*)[ \t]+\(?[A-Za-z_]\w*(?:\.|->)"
    r"([A-Za-z_]\w*)\)?[ \t]*$"
)
_DEFINE_RE = re.compile(r"^[ \t]*#[ \t]*define[ \t]+([A-Za-z_]\w*)\(([^)]*)\)(.*)$")
# Where a module's function-like macros come from: its own headers, and the DDK/NDK/XDK the
# drivers include. Read, not preprocessed: a macro is only ever EXPANDED here to ask which
# real calls it makes, so a slightly wrong arm costs a false lead, never a hidden drop.
_MACRO_INCLUDE_DIRS = (
    "sdk/include/xdk",
    "sdk/include/ddk",
    "sdk/include/ndk",
    "sdk/include/reactos",
)


def macro_table(module_dir: Path, repo: Path) -> dict[str, str]:
    """name -> body of every function-like `#define` visible to the module (first wins).

    The module's own `.c` files count: fastfat defines FatReserveClusters, FatFreeClusters and
    FatLockFreeClusterBitMap as macros at the top of allocsup.c, and without them every
    open-coded `RtlSetBits` / `ExAcquireFastMutexUnsafe` read as a dropped call.
    """
    table: dict[str, str] = {}
    headers = sorted(source_files(module_dir, "*.h")) + sorted(source_files(module_dir, "*.c"))
    for d in _MACRO_INCLUDE_DIRS:
        if (repo / d).is_dir():
            headers += sorted(source_files(repo / d, "*.h"))
    for h in headers:
        lines = h.read_text(encoding="utf-8", errors="replace").splitlines()
        i = 0
        while i < len(lines):
            m = _DEFINE_RE.match(lines[i])
            if not m:
                alias = _ALIAS_RE.match(lines[i]) or _MEMBER_ALIAS_RE.match(lines[i])
                if alias:
                    # Modelled as a macro whose body calls the target: for a member alias
                    # (`#define AccFreeIndexArray NtMartaStatic.FreeIndexArray`) the field.
                    table.setdefault(alias.group(1), f"{alias.group(2)}()")
                i += 1
                continue
            body = m.group(3)
            while body.rstrip().endswith("\\") and i + 1 < len(lines):
                i += 1
                body = body.rstrip()[:-1] + " " + lines[i]
            table.setdefault(m.group(1), _blank_ignored_calls(mask_code(body)))
            i += 1
    return table


def _macro_lost(name: str, macros: dict[str, str], kept, depth: int = 3) -> set[str] | None:
    """The real calls a macro expands to that `kept` rejects (None: not a macro).

    A nested macro the Rust calls by name counts as kept as a whole: fastfat's Rust
    expands `FatReadDirent` inline and calls `FatUnpinBcb`, a cohab FUNCTION standing in
    for the `FatUnpinBcb` -> `CcUnpinData` macro, so its CcUnpinData was never lost.
    """
    if name not in macros:
        return None
    out: set[str] = set()
    for callee in set(_CALL_RE.findall(_blank_ignored_calls(macros[name]))):
        if callee in _C_KEYWORDS or callee == name or _IGNORED.match(callee) or kept(callee):
            continue
        inner = _macro_lost(callee, macros, kept, depth - 1) if depth else None
        if inner is None:
            out.add(callee)
        else:
            out |= inner
    return out


def rust_definitions_all(text: str) -> list[tuple[str, str]]:
    """Every `fn` with a body, INCLUDING cfg-split twins of one name.

    `function_coverage.rust_definition_bodies` keys a dict, so a second definition of a name
    in the same file replaces the first. hal's `HalpQueryPciRegistryInfo` has a full body under
    `cfg(not(feature = "minihal"))` and a `return NULL` face under `cfg(feature = "minihal")`,
    mirroring the C's `#ifndef _MINIHAL_`; the audit saw only the three-line face and called a
    complete translation a stub. The union of the variants is what the C's arms are judged
    against.
    """
    source = text.encode()
    tree = Parser(Language(tree_sitter_rust.language())).parse(source)
    out: list[tuple[str, str]] = []
    stack = [tree.root_node]
    while stack:
        node = stack.pop()
        if node.type == "function_item":
            name = node.child_by_field_name("name")
            if name is not None and node.child_by_field_name("body") is not None:
                out.append(
                    (
                        source[name.start_byte : name.end_byte].decode(),
                        source[node.start_byte : node.end_byte].decode(),
                    )
                )
        if node.type in ("macro_definition", "macro_invocation"):
            continue
        stack.extend(node.named_children)
    return out


_REAL_SEH_RE = re.compile(r"ntkrnl_sys::seh\b|\bfat_try_except\b|\bseh::try_(?:except|finally)\b")


def uses_real_seh(rust_srcs: list[Path]) -> bool:
    """Does this crate run its handlers (the ntkrnl_sys::seh trampolines)?"""
    return any(
        _REAL_SEH_RE.search(f.read_text(encoding="utf-8", errors="replace")) for f in rust_srcs
    )


@dataclass
class RustIndex:
    bodies: dict[str, str]  # masked (comment/literal-free) body text per fn name
    files: dict[str, Path]
    macros: dict[str, str]
    # `#[link_name = "ClassAcquireRemoveLock"] fn class_acquire_remove_lock(..)`: the Rust
    # name -> the symbol it really calls.
    aliases: dict[str, str] = field(default_factory=dict)


_LINK_NAME_RE = re.compile(
    r'#\[link_name\s*=\s*"(\w+)"\]\s*(?:#\[[^\]]*\]\s*)*(?:pub(?:\([^)]*\))?\s+)?'
    r"(?:unsafe\s+)?(?:extern\s+\"[^\"]*\"\s+)?fn\s+(\w+)"
)


def rust_index(srcs: list[Path]) -> RustIndex:
    bodies: dict[str, str] = {}
    files: dict[str, Path] = {}
    macros: dict[str, str] = {}
    aliases: dict[str, str] = {}
    for f in srcs:
        text = f.read_text(encoding="utf-8", errors="replace")
        # Generated bindgen islands hold declarations, never translated bodies.
        if "bindgen" in f.name or f.name.startswith("types_"):
            continue
        for name, body in rust_definitions_all(text):
            # Masked, because translated Rust carries the original C in `//` comments: a
            # dropped call quoted in a comment must not count as kept.
            bodies[name] = bodies.get(name, "") + "\n" + mask_code(body)
            files.setdefault(name, f)
        for m in _LINK_NAME_RE.finditer(text):
            aliases[m.group(2)] = m.group(1)
        for m in re.finditer(r"macro_rules!\s*(\w+)\s*\{", text):
            end = _matching(text, m.end() - 1, "{", "}")
            if end is not None:
                macros[m.group(1)] = mask_code(text[m.end() : end])
    return RustIndex(bodies, files, macros, aliases)


def _effective(name: str, idx: RustIndex, c_names: set[str]) -> str:
    """The Rust body plus the crate-local helpers and macros it reaches (not other ports)."""
    seen = {name}
    parts = [idx.bodies.get(name, "")]
    frontier = [parts[0]]
    for _ in range(_HELPER_DEPTH):
        nxt = []
        for text in frontier:
            for ident in set(_IDENT_RE.findall(text)):
                if ident in seen:
                    continue
                if ident in idx.macros and re.search(rf"\b{ident}!", text):
                    seen.add(ident)
                    parts.append(idx.macros[ident])
                    nxt.append(idx.macros[ident])
                elif ident in idx.bodies and ident not in c_names:
                    seen.add(ident)
                    parts.append(idx.bodies[ident])
                    nxt.append(idx.bodies[ident])
        frontier = nxt
    return "\n".join(parts)


def _snake(name: str) -> str:
    return re.sub(r"(?<=[a-z0-9])(?=[A-Z])", "_", name).lower()


_COM_MACRO = re.compile(r"^I[A-Z]\w*?_(\w+)$")


def _kept(callee: str, rust: str, idents: set[str]) -> bool:
    if callee in idents or _snake(callee) in idents:
        return True
    # dbgbitmap.h's RtlSetBitsDbg & co. wrap the plain call with release-invisible checks.
    if callee.endswith("Dbg") and _kept(callee[:-3], rust, idents):
        return True
    # Case alone: Wine's `midl_user_free` is MIDL's `MIDL_user_free`.
    low = callee.lower()
    if any(i.lower() == low for i in idents):
        return True
    # A/W generic names resolve to one of their variants; the IoXxxRemoveLock macros are
    # their `Ex` exports (serial calls IoReleaseRemoveLockEx directly).
    if any(callee + sfx in idents for sfx in ("W", "A", "Ex")):
        return True
    # `_InterlockedExchangeAdd` is the intrinsic spelling, `_swprintf` the msvcrt one.
    if callee.startswith("_") and callee.lstrip("_") and _kept(callee.lstrip("_"), rust, idents):
        return True
    # Compiler intrinsics (`__readmsr`, `_ReadWriteBarrier`, `__inbyte`) are inline asm or
    # fences in Rust, under whatever name the port chose.
    if (callee.startswith("__") or callee.endswith("Barrier")) and re.search(
        r"asm!|compiler_fence|\bfence\(|_mm_|intrinsics", rust
    ):
        return True
    # A COM vtable macro `IShellFolder_Release(p)` is `((*(*p).lpVtbl).Release)(p)` in Rust.
    com = _COM_MACRO.match(callee)
    if com and com.group(1) in idents:
        return True
    # Wine's own implementation of the method: `ICreateTypeInfo2_fnAddRefTypeInfo`.
    if com and any(i.endswith("_fn" + com.group(1)) for i in idents):
        return True
    return any(re.search(p, rust) for p in _LOWERINGS.get(callee, ()))


def _severity(callee: str) -> str:
    callee = callee.split("(", 1)[0] if "(->" not in callee else callee.split("(->", 1)[1]
    if callee in _MEMORY_OPS or (callee.isupper() and len(callee) > 2):
        return "low"
    return "high" if _STATE_VERB.search(callee) else "medium"


def audit(
    c_files: list[Path],
    rust_srcs: list[Path],
    repo: Path,
    *,
    glue_c: list[Path] | None = None,
    macros: dict[str, str] | None = None,
    pins: dict[str, int] | None = None,
    real_seh: bool | None = None,
    defines: frozenset[str] = TREE_DEFINES,
) -> list[Finding]:
    """`glue_c`: C the build still compiles (cohab shims). A Rust body that delegates to one
    of its functions has not dropped that work, so those bodies join the helper pool."""
    macros = macros or {}
    c_all: dict[str, CBody] = {}
    if real_seh is None:
        real_seh = uses_real_seh(rust_srcs)
    for f in c_files:
        c_all.update(c_bodies(f, pins, real_seh, defines))
    idx = rust_index(rust_srcs)
    for f in glue_c or []:
        text = f.read_text(encoding="utf-8", errors="replace")
        masked = mask_code(text)
        for name in c_functions(text):
            if name in idx.bodies:
                continue
            m = re.search(rf"\b{re.escape(name)}\s*\([^;{{]*?\)\s*\{{", masked)
            if not m:
                continue
            end = _matching(masked, m.end() - 1, "{", "}")
            if end is not None:
                idx.bodies[name] = masked[m.end() - 1 : end + 1]
                idx.files.setdefault(name, f)
    c_names = set(c_all)
    live_c = {name for name, f in idx.files.items() if f.suffix == ".c"}
    findings: list[Finding] = []
    for name in sorted(c_all):
        # A name whose only definition is live C was superseded by C, not translated: gdi32
        # keeps Wine's old enhmfdrv/*.c on disk out of every CMake list, and the live emfdrv.c
        # EMFDRV_* bodies that replaced them are FIXME stubs by design (recording moved to
        # emfdc.c). Glue bodies stay in the index as helpers a Rust body may delegate to.
        if name not in idx.bodies or name in live_c:
            continue
        cb = c_all[name]
        rust = _effective(name, idx, c_names)
        rust_lines = max(1, rust.strip().count("\n") + 1) if rust.strip() else 0
        idents = set(_IDENT_RE.findall(rust))
        idents |= {idx.aliases[i] for i in idents if i in idx.aliases}
        dropped = []
        for c in sorted(cb.required):
            if _kept(c, rust, idents):
                continue
            lost_set = _macro_lost(c, macros, lambda x, r=rust, i=idents: _kept(x, r, i))
            if lost_set is not None:
                # A macro is kept when every real call it expands to is; a pure one
                # (SetFlag, FlagOn, ARGUMENT_PRESENT) makes none and cannot be dropped.
                lost = sorted(lost_set)
                if not lost:
                    continue
                c = f"{c}(->{','.join(lost)})"
            dropped.append(c)

        def rel(p: Path) -> str:
            try:
                return str(p.resolve().relative_to(repo))
            except ValueError:
                return str(p)

        common = (name, rel(cb.file), rel(idx.files[name]))
        if cb.lines >= _STUB_MIN_C_LINES and rust_lines < cb.lines * _STUB_RATIO:
            sev = "stub-loud" if _LOUD_STUB.search(rust) else "stub-silent"
            findings.append(Finding(*common, sev, cb.lines, rust_lines, dropped))
            continue
        if not dropped:
            continue
        sev = min((_severity(c) for c in dropped), key=SEVERITY_ORDER.index)
        findings.append(Finding(*common, sev, cb.lines, rust_lines, dropped))
    findings.sort(key=lambda f: (SEVERITY_ORDER.index(f.severity), f.c_file, f.function))
    return findings


def module_inputs(
    module_dir: Path, arch: str = "i386"
) -> tuple[list[Path], list[Path], list[Path]]:
    """The module's REPLACED C (on disk, out of the build) and its crates' Rust sources.

    Only flipped-out TUs are audited: a .c the build still compiles is what ships, and its
    same-named Rust (if any) is parked, so a difference there is not a lost statement.
    Hand-written cohab glue (`rust_cohab_*.c`) is never a translation source.
    """
    repo = _repo_root_of(module_dir)
    cml = (module_dir / "CMakeLists.txt").read_text(encoding="utf-8", errors="replace")
    if "add_cargo_" in cml and not re.search(r"^[ \t]*add_(?:library|executable)\(", cml, re.M):
        # A pure cargo link (lsass, smss, ...): the module compiles no C of its own.
        live: set[Path] = set()
    else:
        live = {p.resolve() for p in compiled_tus(module_dir, repo, arch=arch)}
    c_files = [
        p
        for p in sorted(source_files(module_dir, "*.c"))
        if p.resolve() not in live
        and not p.name.startswith("rust_")
        and not _other_target(p, module_dir)
    ]
    glue = sorted(p for p in live if p.is_relative_to(module_dir.resolve()))
    workspace = repo / "sdk" / "lib" / "rust"
    srcs: list[Path] = []
    for crate in module_rust_crates(module_dir, workspace):
        if (crate / "src").is_dir():
            srcs.extend(sorted(source_files(crate / "src", "*.rs")))
    return c_files, srcs, glue


_TARGET_RE = re.compile(
    r"^[ \t]*(?:set_module_type|add_library|add_executable|add_cargo_\w+)\(", re.M
)


def _other_target(c_file: Path, module_dir: Path) -> bool:
    """True when a CMakeLists between the file and `module_dir` builds its own binary.

    win32k's own C was deleted when ntuser/ntgdi went Rust-only, so `win32ss` paired its
    crate with user32/, gdi32/ and framebuf/ C: same names (IntDrawScrollBar, Eng*), other
    binaries. 21 false leads, each comparing win32k Rust against a user-mode body."""
    d = c_file.parent
    while d != module_dir and module_dir in d.parents:
        cml = d / "CMakeLists.txt"
        if cml.is_file() and _TARGET_RE.search(cml.read_text(encoding="utf-8", errors="replace")):
            return True
        d = d.parent
    return False


_AUDIT_ROOTS = ("base", "dll", "drivers", "hal", "ntoskrnl", "subsystems", "win32ss", "boot")


def all_modules(repo: Path) -> list[Path]:
    """Every dir whose CMakeLists links a Rust crate (cheap text test, resolved later)."""
    out = []
    for root in _AUDIT_ROOTS:
        for cml in sorted(source_files(repo / root, "CMakeLists.txt")):
            text = cml.read_text(encoding="utf-8", errors="replace")
            if "rust_crate_sources(" in text or "add_cargo_" in text:
                out.append(cml.parent)
    return out


def render(module: str, findings: list[Finding], limit: int, show_reviewed: bool = False) -> str:
    hidden = [f for f in findings if f.reviewed and f.reviewed[0] != "owed"]
    owed = [f for f in findings if f.reviewed and f.reviewed[0] == "owed"]
    open_ = [f for f in findings if not f.reviewed]
    counts = {s: sum(1 for f in open_ if f.severity == s) for s in SEVERITY_ORDER}
    head = ", ".join(f"{n} {s}" for s, n in counts.items() if n)
    tail = "".join(
        f"; {n} {what}" for n, what in ((len(owed), "owed"), (len(hidden), "reviewed")) if n
    )
    lines = [f"body-audit {module}: {head or 'clean'}{tail}"]
    shown = open_ + owed + (hidden if show_reviewed else [])
    for f in shown[:limit] if limit else shown:
        dropped = ", ".join(f.dropped[:8]) + (" …" if len(f.dropped) > 8 else "")
        tag = f"{f.severity}, {f.reviewed[0]}" if f.reviewed else f.severity
        lines.append(
            f"  [{tag}] {f.function}  C {f.c_lines}L / Rust {f.rust_lines}L"
            f"  ({f.c_file} -> {f.rust_file})"
        )
        if dropped:
            lines.append(f"      dropped: {dropped}")
        if f.upstream:
            whole = " (all)" if len(f.upstream) == len(f.dropped) else ""
            new = ", ".join(f.upstream[:8])
            lines.append(f"      new in the C since translation{whole}: {new}")
        if f.reviewed:
            lines.append(f"      {f.reviewed[0]}: {f.reviewed[1]}")
    return "\n".join(lines)


def main(argv: list[str] | None = None) -> int:
    ap = argparse.ArgumentParser(
        prog="rustconv body-audit", description=(__doc__ or "").split("\n")[0]
    )
    ap.add_argument("modules", nargs="*", type=Path, help="module source dirs (with CMakeLists)")
    ap.add_argument(
        "--all",
        action="store_true",
        help="every module dir that links a Rust crate and has flipped C",
    )
    ap.add_argument("--json", action="store_true")
    ap.add_argument("--limit", type=int, default=0, help="show at most N findings per module")
    ap.add_argument(
        "--min-severity", choices=SEVERITY_ORDER, default="stub-loud", help="hide lower ranks"
    )
    ap.add_argument(
        "--provenance",
        action="store_true",
        help="mark dropped callees the C gained after the Rust was committed (git, slower)",
    )
    ap.add_argument(
        "--show-reviewed",
        action="store_true",
        help="also list leads the ledger marks false/benign (data/body_audit_reviewed.toml)",
    )
    args = ap.parse_args(argv)
    ledger = load_reviewed()
    cutoff = SEVERITY_ORDER.index(args.min_severity)
    modules = [m.resolve() for m in args.modules]
    if args.all:
        modules += all_modules(_repo_root_of(Path.cwd()))
    if not modules:
        ap.error("name a module dir or pass --all")
    report = {}
    skipped: list[str] = []
    for module in modules:
        try:
            c_files, srcs, glue = module_inputs(module)
        except ProfileDeriveError as exc:
            if not args.all:
                raise
            skipped.append(f"{module}: {exc}")
            continue
        if not srcs:
            if args.all:
                continue
            print(f"body-audit {module}: no Rust crate linked into this module", file=sys.stderr)
            return 2
        if not c_files:
            continue
        repo = _repo_root_of(module)
        macros = macro_table(module, repo)
        found = [
            f
            for f in audit(
                c_files,
                srcs,
                repo,
                glue_c=glue,
                macros=macros,
                pins=version_pins(module, repo),
                defines=module_defines(module),
            )
            if SEVERITY_ORDER.index(f.severity) <= cutoff
        ]
        key = str(module.relative_to(repo))
        apply_reviewed(key, found, ledger)
        if args.provenance:
            mark_upstream(found, repo)
        report[key] = found
        if not args.json:
            print(render(key, found, args.limit, args.show_reviewed))
    for line in skipped:
        print(f"body-audit SKIPPED (could not resolve the build): {line}", file=sys.stderr)
    if args.json:
        print(json.dumps({k: [asdict(f) for f in v] for k, v in report.items()}, indent=2))
    return 0


if __name__ == "__main__":
    sys.exit(main())
