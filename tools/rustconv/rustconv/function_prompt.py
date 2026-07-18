"""Per-function prompt builder for function-by-function translation."""

from __future__ import annotations

import re
from typing import TYPE_CHECKING

from .callgraph import FunctionDep
from .config import NTOSKRNL_DIR, OUTPUT_CRATE, TARGETS, TEMPLATES_DIR, Target
from .cpp_com import worker_signature
from .resolver import GAP_FUNCTIONS, ResolveResult
from .spec_parser import rust_calling_convention

if TYPE_CHECKING:
    from .scanner import CGlobal, ScanResult


_FUNCTION_MODE_HAL_CODA = """

---

## Per-Function Pipeline Mode

You are translating ONE FUNCTION at a time. The caller stitches functions
together and emits the file-level `use` statements separately.

**CRITICAL — Output ONLY the function body. Do NOT emit:**
- `use` statements (the caller assembles them from the resolved import list)
- Global `static mut` / `pub static` declarations (the caller collects them once)
- `#[link]` / `extern "C" { ... }` blocks for module-level globals (the caller dedups them)
- Trailing module-level definitions of HELPER functions (the caller adds them once)

You MAY emit:
- The function definition itself, including its `#[no_mangle]` / `#[cfg(...)]` /
  `pub unsafe extern "..."` attributes
- Forward `extern "system" { fn X(...); }` blocks ONLY if absolutely required
  to call a function not yet translated (the caller dedups these)
- Local helper `unsafe fn` definitions that the function needs (the caller
  dedups these by name)

Refer to types by their final names (e.g., `_ADAPTER_OBJECT`, `PADAPTER_OBJECT`)
as if they were already in scope. Don't put `ntkrnl_sys::ddk::` or
`ntkrnl_sys::hal_ddk::` path prefixes inline.
"""


def load_function_system_prompt(target: Target | None = None, mode: str = "idiomatic") -> str:
    """Load the system prompt for per-function translation.

    mode="mechanical" loads function_prompt_mechanical.md instead — a faithful-
    transliteration contract (compiles, preserves C control flow/pointers, no
    idiom, no stubs). It's pass 1 of the two-pass cheap-model pipeline; pass 2
    idiomizes the compiling output separately.

    For HAL targets, we use the full system_prompt_hal.md plus a coda telling
    the LLM not to emit imports/globals (function-by-function pipeline mode).
    """
    if target is None:
        target = TARGETS["ntoskrnl"]
    if target.name == "shell32":
        # C++ COM mode is body-only in BOTH styles. The deterministic cpp_com
        # layer owns every ABI artifact; using the generic mechanical/kernel
        # prompt here would tell the model to regenerate those artifacts.
        return target.system_prompt.read_text()
    if mode == "mechanical":
        mech = TEMPLATES_DIR / "function_prompt_mechanical.md"
        base = mech.read_text()
        if target.name == "hal":
            base += _FUNCTION_MODE_HAL_CODA
        return base
    if target.name == "hal":
        return target.system_prompt.read_text() + _FUNCTION_MODE_HAL_CODA
    path = TEMPLATES_DIR / "function_prompt.md"
    return path.read_text()


def _format_import_context(resolve: ResolveResult, target: Target | None = None) -> str:
    """Format pre-resolved imports as context for the LLM.

    For the HAL target, kernel-API symbols (hand-written modules ke/ex/mm/rtl/
    ob/ps/interlocked) are funneled into `ntkrnl_sys::ddk` because:
      a) the same symbols are re-exported from ddk, and
      b) HAL convention (dma.rs, usage.rs, kdpci.rs) imports everything from
         ddk + hal_ddk only — never per-subsystem hand-written modules.
    """
    parts = []
    parts.append("## Available Imports (pre-resolved for this function)")
    if target is not None and target.name == "shell32":
        parts.append(
            "These exact symbols are available from crate::types / crate::shellcom. "
            "Use them directly; keep unresolved constants symbolic:\n"
        )
    else:
        parts.append("These types/functions are available from ntkrnl_sys. Use them directly:\n")

    is_hal = target is not None and target.name == "hal"
    # Hand-written module names that should be folded into `ddk` for HAL.
    _kernel_api_modules = {
        "ke",
        "ex",
        "mm",
        "rtl",
        "ob",
        "ps",
        "interlocked",
        "base",
        "macros",
        "constants",
    }

    # Build a merged view: module → set of names
    merged: dict[str, set[str]] = {}
    for module, group in resolve.imports.items():
        bucket = module
        if is_hal and module in _kernel_api_modules:
            bucket = "ddk"
        merged.setdefault(bucket, set()).update(s.name for s in group.symbols)

    for module in sorted(merged):
        names = sorted(merged[module])
        if target is not None and target.name == "shell32":
            parts.append(f"From `crate::{module}`: {', '.join(f'`{n}`' for n in names)}")
            continue
        if module in ("ddk", "hal_ddk"):
            parts.append(f"From `ntkrnl_sys::{module}`: {', '.join(f'`{n}`' for n in names[:60])}")
            if len(names) > 60:
                parts.append(f"  ... and {len(names) - 60} more")
        else:
            parts.append(f"From `ntkrnl_sys::{module}`: {', '.join(f'`{n}`' for n in names)}")

    return "\n".join(parts)


def _format_gap_functions(gap_fns: set[str], subsystem: str) -> str:
    """Format gap function info for the LLM."""
    if not gap_fns:
        return ""

    parts = []
    parts.append("\n## Gap Functions (macros/FORCEINLINE — use extern or expand)")

    # Try to load shim signatures
    shim_sigs = _load_shim_signatures(subsystem)

    for name in sorted(gap_fns):
        if name in shim_sigs:
            parts.append(f"- `{shim_sigs[name]}` — available via extern")
        else:
            parts.append(f"- `{name}` — expand inline per rules")

    return "\n".join(parts)


def _load_shim_signatures(subsystem: str) -> dict[str, str]:
    """Load gap function C signatures from the shim file."""
    shim_path = NTOSKRNL_DIR / subsystem / f"{subsystem}_rust_shims.c"
    if not shim_path.exists():
        return {}

    sigs: dict[str, str] = {}
    text = shim_path.read_text()

    phase5_marker = "Phase 5:"
    phase5_start = text.find(phase5_marker)
    if phase5_start < 0:
        return sigs

    phase5_text = text[phase5_start:]

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


def _rust_signature(rust_src: str) -> str:
    """Extract just the attributes + `fn` signature (no body) from generated Rust.

    The caller of a function needs its *signature* (name, params, return type,
    calling convention) to emit a correct call — never its body. The body open
    brace is the first `{` in generated translations (attributes use parens, and
    signature return types here don't contain braces), so everything before it
    is the attribute stack + signature. We append `;` so it reads as a decl.
    """
    src = rust_src.strip()
    brace = src.find("{")
    if brace < 0:
        # No body brace (already a forward decl / fn-pointer type) — first lines.
        return "\n".join(src.splitlines()[:6])
    return src[:brace].rstrip() + ";"


def _format_callee_context(
    func: FunctionDep,
    translated: dict[str, str],
    all_deps: list[FunctionDep],
    catalog_fns: set[str],
) -> str:
    """Build callee context section.

    For each callee of the function being translated we emit only its SIGNATURE
    (the calling contract) — never its body. Bodies are uncached per-function
    bulk that adds no translation value and tempts the model to copy callee
    internals into the caller.
    - If already translated to Rust: show the Rust signature
    - If extern (in catalog): show extern declaration
    - If local but not yet translated: show C signature
    - If gap function: skip (handled by gap section)
    """
    if not func.callees:
        return ""

    local_names = {d.name for d in all_deps}
    by_name = {d.name: d for d in all_deps}

    parts = []
    parts.append("\n## Callee Context")
    parts.append("Functions called by this function and their signatures:\n")

    for callee_name in sorted(func.callees):
        if callee_name in GAP_FUNCTIONS:
            continue  # Handled by gap functions section
        if callee_name == func.name:
            continue  # Self-recursion, skip

        if callee_name in translated:
            # Already translated — show only the Rust signature (the contract).
            sig = _rust_signature(translated[callee_name])
            parts.append(f"### `{callee_name}` (already translated to Rust)")
            parts.append(f"```rust\n{sig}\n```\n")

        elif callee_name.endswith("_body") and callee_name[:-5] in by_name:
            # shell32 deterministically rewrites same-file helpers and sibling
            # methods to their collision-free Rust worker names before sealing.
            from .cpp_com import worker_signature

            dep = by_name[callee_name[:-5]]
            sig = worker_signature(dep.name, dep.cpp_class, dep.params, dep.return_type) + ";"
            parts.append(f"### `{callee_name}` (generated local Rust worker)")
            parts.append(f"```rust\n{sig}\n```\n")

        elif callee_name in local_names:
            # Local function not yet translated — show only the C signature.
            dep = by_name[callee_name]
            params_str = ", ".join(f"{t} {n}" for t, n in dep.params)
            sig = (
                f"{dep.return_type} {dep.calling_convention or ''} {dep.name}({params_str})".strip()
            )
            parts.append(f"### `{callee_name}` (local C function, not yet translated)")
            parts.append(f"```c\n{sig};\n```\n")

        elif callee_name in catalog_fns:
            # Extern function from catalog
            parts.append(f"- `{callee_name}` — extern function (available from ntkrnl_sys)")

        elif getattr(func, "cpp_class", None) and callee_name.startswith(
            f"{func.cpp_class}_cpp_com_"
        ):
            parts.append(
                f"- `{callee_name}` — deterministic COM scaffold helper "
                "(already available; call it directly and do not declare it)"
            )

        else:
            # Unknown — also look in existing translated Rust files
            rust_sig = _find_rust_function_sig(callee_name)
            if rust_sig:
                parts.append(f"### `{callee_name}` (from another Rust module)")
                parts.append(f"```rust\n{rust_sig}\n```\n")
            else:
                parts.append(
                    f"- `{callee_name}` — unresolved external "
                    "(keep the symbolic call; deterministic integration owns its declaration)"
                )

    if len(parts) <= 2:
        return ""  # No meaningful callee context

    return "\n".join(parts)


def _find_rust_function_sig(name: str) -> str | None:
    """Search existing ntoskrnl-rust .rs files for a function signature."""
    src_dir = OUTPUT_CRATE / "src"
    if not src_dir.exists():
        return None

    # Search for #[no_mangle] pub unsafe ... fn <name>(
    pattern = re.compile(
        rf'((?:#\[no_mangle\]\s*)?pub\s+unsafe\s+(?:extern\s+"[^"]+"\s+)?fn\s+{re.escape(name)}\s*\([^)]*\)\s*(?:->\s*\S+)?)',
        re.MULTILINE,
    )

    for rs_file in src_dir.rglob("*.rs"):
        if rs_file.name == "mod.rs" or rs_file.name == "lib.rs":
            continue
        try:
            text = rs_file.read_text()
            m = pattern.search(text)
            if m:
                return m.group(1)
        except (OSError, UnicodeDecodeError):
            continue

    return None


def _format_globals_context(globals_list: list[CGlobal], globals_text: str) -> str:
    """Format file-level globals context."""
    if not globals_list and not globals_text.strip():
        return ""

    parts = []
    parts.append("\n## File-Level Globals")
    parts.append("These globals are declared at file scope. Reference them directly:\n")

    if globals_text.strip():
        parts.append("```c")
        parts.append(globals_text.strip())
        parts.append("```")
    elif globals_list:
        for g in globals_list:
            init = f" = {g.initializer}" if g.initializer else ""
            parts.append(f"- `{g.type_str} {g.name}{init};`")

    return "\n".join(parts)


def _format_export_info(func: FunctionDep, spec_exports: dict) -> str:
    """Format export info for this specific function."""
    if not func.is_exported or func.name not in spec_exports:
        return ""

    entry = spec_exports[func.name]
    conv = rust_calling_convention(entry)
    fwd = f" (forwards to {entry.forward_target})" if entry.forward_target else ""
    arch = f" [arch: {','.join(entry.arch_filter)}]" if entry.arch_filter else ""

    parts = []
    parts.append("\n## Export Info")
    if entry.convention == "fastcall":
        parts.append(f"This function is EXPORTED as `{func.name}`. Use `#[no_mangle] pub unsafe`.")
        parts.append(
            f'Use `extern "fastcall"` on `#[cfg(target_arch = "x86")]` '
            f'and `extern "C"` on other arches.{fwd}{arch}'
        )
    else:
        parts.append(
            f"This function is EXPORTED as `{func.name}`. "
            f"Use `#[no_mangle] pub unsafe {conv} fn {func.name}(...)`{fwd}{arch}"
        )

    return "\n".join(parts)


def build_preamble_prompt(
    scan: ScanResult,
    resolve: ResolveResult,
    target: Target | None = None,
) -> str:
    """Build the prompt for translating a file's FILE-SCOPE PREAMBLE.

    `translate` is function-granular and the stitcher never emits the C
    file's typedefs / structs / enums / #defines / static lookup tables —
    so a translated file with a non-trivial preamble does not compile.
    This prompt translates exactly that preamble as ONE synthetic unit;
    cmd_translate prepends the result via stitch_file(preamble=...).

    Uses the same resolved import context as the function prompts. The C
    text is scan.preamble_text (AST-derived, marker-independent), falling
    back to globals_text for marker-split files.
    """
    preamble_c = (scan.preamble_text or scan.globals_text or "").strip()
    parts: list[str] = []
    parts.append(f"## Translate the FILE-SCOPE PREAMBLE of `{scan.path.name}`\n")
    parts.append(_format_import_context(resolve, target=target))
    parts.append(
        "\n## Task\n"
        "Translate ONLY the file-scope C declarations below to Rust. These\n"
        "are the types/constants/tables the translated functions reference.\n\n"
        "Rules:\n"
        "- `typedef struct {..} X, *PX;` → `#[repr(C)] pub struct X {..}`\n"
        "  plus `pub type PX = *mut X;`. Keep field order/types exact.\n"
        "- `#define NAME value` (object-like, constant) →\n"
        "  `pub const NAME: <inferred-ty> = value;`. Skip function-like or\n"
        "  token-paste macros (the functions inline those).\n"
        "- `enum {..}` → `pub const`s (C enum int values) unless a real\n"
        "  `#[repr(C)] enum` is clearly intended.\n"
        "- `static const T tbl[] = {..}` → `pub static tbl: [T; N] = [..];`\n"
        "  (compute N; preserve every element verbatim).\n"
        "  **Transcribe every element EXACTLY — do not summarise, elide, or\n"
        "  'continue' a table with a comment.** A single wrong byte in a lookup\n"
        "  table compiles, links and boots, and is wrong forever. If the C\n"
        "  declares a size larger than the elements it writes (`BYTE t[256]` with\n"
        "  103 written), C zero-fills the tail — you must spell all 256.\n"
        "- **SKIP `static const <X>Vtbl <Name> = {..};` COM vtable tables.** Those\n"
        "  are generated deterministically from the bindgen struct (vtbl_gen.py);\n"
        "  emitting them here would only duplicate the definitions.\n"
        "- Emit EVERY other declaration. This preamble is the ONLY pass that sees\n"
        "  file-scope C — anything you drop here is simply absent from the crate.\n"
        "- Reference resolved types by their final names (e.g. `_LIST_ENTRY`,\n"
        "  `PVOID`) as if already in scope. Do NOT add `ntkrnl_sys::` path\n"
        "  prefixes inline.\n\n"
        "**Output ONLY the Rust declarations.** Do NOT emit: `use`\n"
        'statements, `extern ".." {}` blocks, ANY `fn` definitions, or\n'
        "prose. The caller assembles imports and function bodies separately."
    )
    parts.append("\n## C File-Scope Preamble\n")
    parts.append("```c")
    parts.append(preamble_c)
    parts.append("```")
    return "\n".join(parts)


def build_file_context(
    file_res: ResolveResult,
    scan: ScanResult,
    subsystem: str,
    target: Target | None = None,
) -> str:
    """Per-file shared context — byte-stable across every function in the file.

    Contains the import SUPERSET (file-level resolve), gap functions, and the
    file globals/preamble. It names no specific function, so it is identical
    for all N function calls of a file and belongs in a *cached* system block:
    written once on function 1 (~1.25x), read by functions 2..N (~0.1x), versus
    the old path that re-sent this whole block uncached (1x) on every call.
    The function-specific bit (export info, translated callees, the C body)
    stays in the per-call user message — build_function_prompt(include_shared=False).
    """
    parts = [
        "## File Translation Context",
        "(Shared by every function in this file. The specific function to "
        "translate is in the user message that follows.)\n",
    ]
    parts.append(_format_import_context(file_res, target=target))
    gap_info = _format_gap_functions(file_res.gap_functions, subsystem)
    if gap_info:
        parts.append(gap_info)
    globals_info = (
        ""
        if target is not None and target.name == "shell32"
        else _format_globals_context(scan.globals, scan.globals_text or scan.preamble_text)
    )
    if globals_info:
        parts.append(globals_info)
    return "\n".join(parts)


def _format_method_context(
    func: FunctionDep, scan: ScanResult, target: Target | None = None
) -> str:
    """C++-method translation contract + the owning class's definition
    (bodies stripped). The flat `Cls_method(this: *mut Cls, …)` convention is
    the cohab ABI — see docs/engineering/explorer-rust-translation-plan.md."""
    from .scanner import extract_class_context

    cls = func.cpp_class
    m = func.cpp_method
    if not cls or not m:  # not a C++ method (or no method name) — no contract to emit
        return ""
    if target is not None and target.name == "shell32":
        signature = worker_signature(func.name, cls, func.params, func.return_type)
        lines = [
            "\n## C++ Method Body Worker\n",
            f"This is `{cls}::{m}`. The deterministic COM generator owns its ABI wrapper.\n",
            f"- Emit exactly `{signature}` with the C++ body.\n",
            "- Do not emit `#[no_mangle]`, `extern`, vtables, QueryInterface, refcounting, "
            "or factories.\n",
            f"- Implicit fields become `(*this).field`; sibling bodies are "
            f"`{cls}_<method>_body`.\n",
            "- A COM call is `((*(*ptr).lpVtbl).Method)(ptr, ...)`; vtable slots are "
            "mandatory function pointers, never `Option`, `as_ref`, or `unwrap`.\n",
            "- `CPP_COM_WSTR_*` values are deterministic UTF-16 statics; preserve each exact "
            "`.as_ptr()` expression. Never define a symbolic C constant locally.\n",
        ]
        ctx = extract_class_context(scan, cls)
        if ctx:
            lines.append(f"\n### Class definition (method bodies stripped)\n```cpp\n{ctx}\n```\n")
        return "".join(lines)
    lines = [
        "\n## C++ Method → Free Function\n",
        f"This C body is the C++ method `{cls}::{m}`. Translate it as a FREE function:\n",
        '- Rust signature: `#[no_mangle] pub unsafe extern "C" fn '
        f"{func.name}(this: *mut {cls}, …)`\n",
        f"- `this: *mut {cls}` is the FIRST parameter; implicit member "
        "access `m_x` (or `this->m_x`) becomes `(*this).m_x`.\n",
        f"- Calls to sibling `{cls}` methods become calls to their "
        f"`{cls}_<method>` free functions, passing `this`.\n",
        "- Do NOT emit a Rust `impl` block or trait — keep the flat "
        "#[no_mangle] convention (cohab ABI).\n",
        f"- If the `{cls}` struct is not yet defined in the crate, assume "
        f"`#[repr(C)] pub struct {cls} {{ … }}` matching the layout below.\n",
    ]
    ctx = extract_class_context(scan, cls)
    if ctx:
        lines.append(f"\n### Class definition (method bodies stripped)\n```cpp\n{ctx}\n```\n")
    return "".join(lines)


def _format_shell32_helper_context(func: FunctionDep) -> str:
    signature = worker_signature(func.name, None, func.params, func.return_type)
    return (
        "\n## Same-file Helper Body Worker\n\n"
        "This helper belongs to the deterministic closure of the selected COM class.\n"
        f"- Emit exactly `{signature}` with the C++ body.\n"
        "- Its name and signature were derived from the source; do not add `this`.\n"
        "- Calls already ending in `_body` were overload-bound deterministically; preserve "
        "those exact names.\n"
        "- `CPP_COM_WSTR_*` values are deterministic UTF-16 statics; preserve each exact "
        "`.as_ptr()` expression.\n"
        "- Never define a symbolic C constant locally. Use its supplied name directly.\n"
        "- Do not emit `#[no_mangle]`, `extern`, vtables, factories, imports, or globals.\n"
    )


def build_function_prompt(
    func: FunctionDep,
    resolve: ResolveResult,
    translated: dict[str, str],
    all_deps: list[FunctionDep],
    scan: ScanResult,
    catalog_fn_names: set[str],
    spec_exports: dict | None = None,
    target: Target | None = None,
    include_shared: bool = True,
    c2rust_anchor: str | None = None,
    deterministic_contract: str = "",
) -> str:
    """Build a complete prompt for translating a single C function.

    c2rust_anchor: optional c2rust transpilation of THIS function (raw/unsafe but
    behavior-correct by construction). When given it is appended as a "Behavior
    Anchor" section — the mechanical-mode system prompt instructs the model to
    preserve every operation it contains, defeating the silent-stub/drop-logic
    failure mode of pure LLM translation.

    When ``include_shared`` is False, the file-shared context (import list, gap
    functions, globals) is OMITTED — the caller is sending it once in a cached
    system block via build_file_context() instead of re-paying for it on every
    function. Only the function-specific parts (export info, already-translated
    callees, the C body) are emitted. Default True keeps the self-contained
    prompt for the dry-run/preview and any non-cached caller.

    Args:
        func: The function to translate
        resolve: Per-function ResolveResult (from function_resolver)
        translated: dict of function_name → Rust code (already translated in this batch)
        all_deps: All FunctionDeps from this file (for local callee context)
        scan: ScanResult for the file (for globals)
        catalog_fn_names: Set of function names in the catalog
        spec_exports: dict of function_name → SpecEntry

    Returns:
        User-message prompt string (system prompt is separate for caching).
    """
    if spec_exports is None:
        spec_exports = {}

    subsystem = scan.path.parent.name
    parts = []

    # Header
    parts.append(f"## Translate: `{func.name}` from `{scan.path.name}`\n")

    # C++ method → free-function convention + class layout context
    if getattr(func, "cpp_class", None):
        parts.append(_format_method_context(func, scan, target=target))
    elif target is not None and target.name == "shell32":
        parts.append(_format_shell32_helper_context(func))

    # Export info
    export_info = _format_export_info(func, spec_exports)
    if export_info:
        parts.append(export_info)

    # File-shared context (imports / gap / globals). Omitted when the caller
    # supplies it once via build_file_context() in a cached system block.
    if include_shared:
        parts.append(_format_import_context(resolve, target=target))
        gap_info = _format_gap_functions(resolve.gap_functions, subsystem)
        if gap_info:
            parts.append(gap_info)
        globals_info = (
            ""
            if target is not None and target.name == "shell32"
            else _format_globals_context(scan.globals, scan.globals_text or scan.preamble_text)
        )
        if globals_info:
            parts.append(globals_info)

    # Callee context
    callee_info = _format_callee_context(func, translated, all_deps, catalog_fn_names)
    if callee_info:
        parts.append(callee_info)

    if deterministic_contract:
        parts.append(deterministic_contract)

    # The C function source
    parts.append("\n## C Function to Translate\n")
    parts.append("```c")
    parts.append(func.body_text)
    parts.append("```")

    # Behavior anchor: c2rust's mechanical (behavior-correct-by-construction)
    # transpilation of THIS function. The mechanical-mode system prompt tells the
    # model to preserve every operation it contains — the structural defense
    # against silently stubbing/truncating hard functions.
    if c2rust_anchor:
        parts.append(
            "\n## Behavior Anchor (c2rust transpilation)\n"
            "This is c2rust's exact transpilation of the function above — raw/unsafe/ugly but it\n"
            "contains ALL the logic (c2rust is a compiler, it cannot drop or stub anything). Your\n"
            "Rust MUST preserve every statement/loop/branch/call it has; clean up its surface\n"
            "(types, imports, names) but NEVER omit logic it contains.\n"
        )
        parts.append("```rust")
        parts.append(c2rust_anchor.strip())
        parts.append("```")

    return "\n".join(parts)
