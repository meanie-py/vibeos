"""Assemble system + per-file prompts for C-to-Rust translation."""

import json
import re
from functools import lru_cache
from pathlib import Path

from .config import NTOSKRNL_DIR, TARGETS, TEMPLATES_DIR, Target
from .resolver import ResolveResult
from .scanner import ScanResult
from .spec_parser import rust_calling_convention

# --- win32k pretranslation sidecar consumption ------------------------------
# Pass 0/2/3 wrote machine-generated maps next to each keep-set .c. They are
# inert until the prompt builder feeds them to the translator. This is that
# consumer (see docs/engineering/win32k-pretranslation-audit.md).


@lru_cache(maxsize=1)
def load_win32k_rules() -> str:
    return (TEMPLATES_DIR / "win32k_translation_prompt.md").read_text()


def load_win32k_sidecars(c_path: Path) -> dict:
    """Read whichever of the Pass-0/2/2b/3 sidecars exist next to the
    C file. Empty dict ⇒ not a pretranslated win32k file ⇒ caller emits
    nothing (ntoskrnl/hal prompts are unaffected)."""
    out: dict[str, dict] = {}
    for suffix, key in (
        (".win32k_meta.json", "seh"),  # Pass 0 SEH triage
        (".win32k_gdilocks.json", "gdi"),  # Pass 2 GDI lock brackets
        (".win32k_userlocks.json", "userlocks"),  # Pass 2b USER critsec
        (".win32k_callouts.json", "callouts"),  # Pass 3 USER callouts
    ):
        p = Path(str(c_path) + suffix)
        if p.exists():
            try:
                out[key] = json.loads(p.read_text())
            except (ValueError, OSError):
                pass
    return out


def render_win32k_context(sidecars: dict) -> str:
    """Compact, high-signal per-function map. Probes are already lowered,
    so the SEH section reports only the *residual* (non-`a`) categories
    the translator still has to reason about."""
    L: list[str] = ["### Win32k Pretranslation Map (ground truth — trust this)\n"]

    gdi = sidecars.get("gdi", {}).get("sites", [])
    if gdi:
        L.append("**GDI lock handles** (var · lock→unlock · shape — §2):")
        for s in gdi:
            L.append(
                f"- `{s['function']}`: `{s['var']}` = "
                f"`{s['lock_fn']}`→`{s['unlock_fn']}` "
                f"[{s['shape']}]" + (f" ⚠ {','.join(s['signals'])}" if s.get("signals") else "")
            )
        L.append("")

    ul = sidecars.get("userlocks", {}).get("sites", [])
    if ul:
        L.append("**USER critsec brackets** (UserEnter*/UserLeave · mode · shape — §2.5):")
        for s in ul:
            top = "@top" if s.get("enter_at_top") else "@cond"
            sig = f" ⚠ {','.join(s['signals'])}" if s.get("signals") else ""
            L.append(
                f"- `{s['function']}`: {s['mode']} "
                f"enter L{s['enter_line']}{top} → "
                f"leave L{s.get('leave_lines')} "
                f"[{s['shape']}]{sig}"
            )
        L.append("")

    co = sidecars.get("callouts", {}).get("sites", [])
    if co:
        L.append("**USER callouts** (preserve the Leave/Enter straddle — §3):")
        for s in co:
            warn = [x for x in s.get("signals", []) if x != "co_prefixed"]
            sig = f" ⚠ {','.join(warn)}" if warn else ""
            L.append(
                f"- `{s['function']}`: {s['callback_index']} "
                f"@L{s['kumc_line']} "
                f"[Leave L{s['leave_co_line']} / Enter "
                f"L{s['enter_co_line']} / probe L{s['result_probe_line']}]"
                f"{sig}"
            )
        L.append("")

    seh = sidecars.get("seh", {}).get("sites", [])
    resid = [s for s in seh if s.get("category") != "a"]
    if resid:
        names = {
            "c": "FAULT_RECOVER (keep SEH — §4)",
            "d": "BELT_SUSPENDERS (drop SEH, translate body — §4)",
            "e": "NEEDS_REVIEW (faithful + TODO — §4)",
            "b": "LOCK_CLEANUP",
        }
        L.append("**Residual SEH** (probes already lowered; these remain):")
        for s in resid:
            cat = s.get("category", "?")
            L.append(
                f"- `{s.get('function', '?')}` "
                f"L{s.get('line_start', '?')}: ({cat}) "
                f"{names.get(cat, cat)}"
            )
        L.append("")
    elif seh:
        L.append(
            "**Residual SEH**: none — every SEH site in this file was "
            "an `(a)` probe and is already lowered to `SafeProbeFor*`.\n"
        )

    return "\n".join(L)


def load_system_prompt(target: Target | None = None) -> str:
    """Load the system prompt template for the given target."""
    if target is None:
        target = TARGETS["ntoskrnl"]
    prompt = target.system_prompt.read_text()
    addendum = getattr(target, "system_prompt_addendum", None)
    if addendum and addendum.exists():
        prompt += "\n\n" + addendum.read_text()
    return prompt


def load_shim_signatures(subsystem: str) -> dict[str, str]:
    """Parse existing shim file Phase 5 to extract gap function signatures.

    Returns dict of function_name → C signature string.
    """
    shim_path = NTOSKRNL_DIR / subsystem / f"{subsystem}_rust_shims.c"
    if not shim_path.exists():
        return {}

    sigs: dict[str, str] = {}
    text = shim_path.read_text()

    # Find Phase 5 section
    phase5_marker = "Phase 5:"
    phase5_start = text.find(phase5_marker)
    if phase5_start < 0:
        return sigs

    phase5_text = text[phase5_start:]

    # Match function definitions: ReturnType [NTAPI] FuncName(params)
    for m in re.finditer(
        r"^(\w[\w\s\*]*?)\s+(?:NTAPI\s+)?(\w+)\s*\(([^)]*)\)\s*\{",
        phase5_text,
        re.MULTILINE,
    ):
        ret_type = m.group(1).strip()
        name = m.group(2)
        params = m.group(3).strip()
        if name.startswith("_tramp_") or name.startswith("_fi_"):
            continue
        sigs[name] = f"{ret_type} {name}({params})"

    return sigs


def build_file_prompt(
    scan: ScanResult, resolve: ResolveResult, target: Target | None = None
) -> str:
    """Build the per-file prompt with C source, imports, exports, and gap functions."""
    parts = []

    # Section 1: File metadata
    rel_path = scan.path.name
    parts.append(f"## File: `{rel_path}`\n")

    # Determine subsystem for shim lookup
    subsystem = scan.path.parent.name

    # Section 2: Exports from the module's .spec. Name the REAL module: this said
    # "ntoskrnl.spec" / "exported from ntoskrnl.exe" unconditionally, so a model
    # translating ws2_32 was told its exports come from the kernel. A false premise in the
    # prompt is not cosmetic — it is the one part of the input the model cannot check.
    mod = target.name if target is not None else "ntoskrnl"
    dll = "ntoskrnl.exe" if mod == "ntoskrnl" else f"{mod}.dll"
    if resolve.exports:
        parts.append(f"### Exported Functions (from {mod}.spec)")
        parts.append(
            f"These functions are exported from {dll} and MUST be `#[no_mangle] pub unsafe`:\n"
        )
        for name, entry in sorted(resolve.exports.items()):
            # Dotted target = forwarded at another DLL; bare target = the symbol WE define.
            fwd = (
                f" (forwards to {entry.forward_target})"
                if entry.forward_target
                else f" (define as `{entry.impl_symbol}`)"
                if entry.impl_symbol
                else ""
            )
            arch = f" [arch: {','.join(entry.arch_filter)}]" if entry.arch_filter else ""
            conv = rust_calling_convention(entry)
            if entry.convention == "fastcall":
                parts.append(
                    f"- `#[no_mangle] pub unsafe {conv} fn {name}(...)` — "
                    f'Use `extern "fastcall"` on `#[cfg(target_arch = "x86")]` '
                    f'and `extern "C"` on other arches{fwd}{arch}'
                )
            else:
                parts.append(f"- `#[no_mangle] pub unsafe {conv} fn {name}(...)`{fwd}{arch}")
        parts.append("")

    # Section 3: Available imports.
    # The catalog module name "win32k" is unique to win32k-sys symbols
    # (the win32k-internal surface: _WND/_DESKTOP/Int*/Gre*/co_*/PBYTE/
    # SORTCOMP/…). Those live in the `win32k_sys` crate, NOT `ntkrnl_sys`
    # (which has no `win32k` module). Render the correct crate path per
    # module so the LLM emits a `use` that actually resolves. postprocess
    # also rewrites any stray `ntkrnl_sys::win32k` → `win32k_sys::win32k`
    # as a deterministic safety net (it is universally wrong).
    def _crate_path(module: str) -> str:
        if module == "win32k":
            return "win32k_sys::win32k"
        if module == "winsrv":
            return "winsrv_sys::winsrv"
        return f"ntkrnl_sys::{module}"

    if resolve.imports:
        parts.append("### Available Imports")
        parts.append(
            "These types and functions are available. Use them in your `use` declarations:\n"
        )
        for module, group in sorted(resolve.imports.items()):
            names = sorted(s.name for s in group.symbols)
            path = _crate_path(module)
            if module in ("ddk", "hal_ddk", "win32k", "winsrv"):
                parts.append(f"From `{path}::*`: {', '.join(f'`{n}`' for n in names[:50])}")
                if len(names) > 50:
                    parts.append(f"  ... and {len(names) - 50} more")
            else:
                parts.append(f"From `{path}::*`: {', '.join(f'`{n}`' for n in names)}")
        parts.append("")

    # Section 4: Gap functions
    if resolve.gap_functions:
        parts.append("### Gap Functions (macros/FORCEINLINE — not importable)")
        parts.append(
            "These C macros/inline functions must be expanded or replaced "
            "per the system prompt rules."
        )
        parts.append(
            "They are provided as real symbols via the shim file. Declare them in "
            '`extern "system" { ... }` blocks:\n'
        )

        shim_sigs = load_shim_signatures(subsystem)

        for name in sorted(resolve.gap_functions):
            if name in shim_sigs:
                parts.append(f"- `{shim_sigs[name]}` (shimmed)")
            else:
                parts.append(f"- `{name}`")
        parts.append("")

    # Section 5: Architecture-specific code
    if scan.arch_ifdefs:
        parts.append("### Architecture Conditionals")
        parts.append(f"This file uses: {', '.join(f'`{m}`' for m in scan.arch_ifdefs)}")
        parts.append("Map these to `#[cfg(target_arch = ...)]` per the system prompt rules.\n")

    # Section 5.5: win32k pretranslation context (only for keep-set files
    # that have Pass-0/2/3 sidecars — no-op for ntoskrnl/hal).
    sidecars = load_win32k_sidecars(scan.path)
    if sidecars:
        parts.append("## Win32k Pretranslation\n")
        parts.append(load_win32k_rules())
        parts.append("")
        parts.append(render_win32k_context(sidecars))
        parts.append("")

    # Section 6: The C source code
    parts.append("### C Source Code\n")
    parts.append("Translate the following C code to Rust:\n")
    parts.append("```c")
    parts.append(scan.raw_source)
    parts.append("```")

    return "\n".join(parts)


def build_full_prompt(
    scan: ScanResult, resolve: ResolveResult, target: Target | None = None
) -> str:
    """Build the complete prompt (system + file)."""
    system = load_system_prompt(target)
    file_prompt = build_file_prompt(scan, resolve, target)
    return system + "\n\n---\n\n" + file_prompt


def estimate_tokens(text: str) -> int:
    """Rough token estimate (1 token ≈ 4 chars for code)."""
    return len(text) // 4
