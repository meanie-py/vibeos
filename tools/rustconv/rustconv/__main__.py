"""CLI entry point for rustconv."""

import argparse
import collections
import difflib
import json
import re
import sys
import tempfile
import threading
import time
from collections.abc import Callable, Iterable
from concurrent.futures import ThreadPoolExecutor, as_completed
from dataclasses import replace
from pathlib import Path

from . import (
    callout_annotate,
    candidate_oracle,
    cpp_com,
    cpp_core,
    cpp_tu,
    crate_repair,
    difftest_contract,
    difftest_gen,
    dup_census,
    file_splitter,
    gdi_lock_inventory,
    hybrid_closure,
    profile_derive,
    seh_belt,
    seh_inventory,
    seh_llm_lower,
    seh_rewrite,
    semantic_skeleton,
    semdiff,
    shell32_tail_inventory,
    substrate_closure,
    switch_extract,
    trivial_c,
    typedefs,
    user_lock_inventory,
)
from . import port as port_mod
from .argument_facts import require_argument_facts
from .assembler import AssemblyClobberError, assemble_file, generate_lib_rs, generate_mod_rs
from .attempts import CHECKER_FINGERPRINT, digest, record_validation
from .bank_audit import audit as bank_audit
from .bank_audit import exported_symbols
from .bank_audit import format_report as format_bank_report
from .batch_runner import TranslationResult, submit_batch, submit_single
from .body_audit import main as body_audit_main
from .c2rust_anchor import C2RustError, CompiledOut, build_anchor_map, build_preamble
from .c_body_checks import body_findings, control_flow_findings, parse_findings
from .call_seal import (
    CallSealError,
    call_contract,
    scrub_comment_token_echoes,
    seal_calls,
    unseal_calls,
)
from .callgraph import extract_call_graph, topological_sort
from .catalog import build_catalog, print_catalog_stats
from .checker import check_all_targets, print_check_report
from .config import (
    BATCH_DISCOUNT,
    DEEPSEEK_MODEL_V4_PRO,
    DEFAULT_MODELS,
    HAL_DIR,
    MODEL_PRICING,
    NTOSKRNL_DIR,
    OUTPUT_CRATE,
    REACTOS_ROOT,
    RUST_DIR,
    RUSTCONV_RESPONSES_DIR,
    SHELL32_ISLAND_FILES,
    TARGETS,
    cost_usd,
    detect_target,
    get_backend,
    make_user_dll_target,
)
from .cpp_bridge import validate_reference, write_bridge
from .cpp_com import (
    CppComError,
    analyze_class,
    apply_interface_method_signatures,
    audit_worker_body,
    discover_widl_header,
    find_class_header,
    island_lpvtbl_interfaces,
    lower_trivial_worker,
    normalize_cpp_idioms,
    normalize_worker_body,
    normalize_worker_semantics,
    parse_widl_interface,
    prepare_helper_closure,
    provided_raii_contract,
    raii_container_contract,
    render_class_scaffold,
    render_cpp_support,
    render_interface,
    render_object_map_adapter,
    render_rewrite_support,
    render_wide_literal_statics,
    require_supported_worker_raii,
    rewrite_wide_literals,
    strip_debug_statements,
    worker_signature,
)
from .crate_integrate import resolve_in_crate_imports, strip_top_level_items
from .dead_regions import (
    conditional_arm_notes,
    never_compiled_lines,
    pinned_int_macros,
    sibling_header_int_macros,
)
from .extern_audit import run_extern_audit, run_extern_fix
from .fact_seal import (
    FactSealError,
    canonicalize_hresult_predicates,
    fact_contract,
    prepare_sealed_source,
    unseal_facts,
)
from .function_coverage import require_c_plan, require_rust_definitions
from .function_prompt import (
    build_file_context,
    build_function_prompt,
    build_preamble_prompt,
    load_function_system_prompt,
)
from .function_resolver import resolve_function
from .kernel_com import generate_owner
from .kernel_preamble import derive_kernel_preamble
from .macro_authority import function_like_macros, string_literal_macros
from .objdiff import run_objdiff
from .outcome_ledger import OutcomeLedger
from .placement_audit import placement_findings
from .postprocess import postprocess
from .preamble_audit import audit_preamble
from .probe_migrate import (
    build_probe_file_prompt,
    load_probe_system_prompt,
    run_probe_migration,
    scan_rust_file,
)
from .profile_derive import ProfileDeriveError, _target_names, parse_cmakelists
from .projects.reactos.targets import module_lock_for_crate
from .prompt_builder import (
    build_file_prompt,
    build_full_prompt,
    estimate_tokens,
    load_system_prompt,
)
from .resolver import print_resolve_result, resolve
from .resolver import resolve as file_resolve
from .response_cache import (
    PREAMBLE_KEY,
    RESPONSES_DIR,
    ExactBodyTranslationMemory,
    ResponseCache,
    require_no_namespace_rebuy,
)
from .scanner import print_scan_result, scan_file
from .seh_classifier import classify_directory, classify_file, print_classification, to_json
from .shim_generator import check_shims, generate_shims
from .sink import (
    ResponseSink,
    RunArchive,
    install_kill_guard,
    paid_invocation_banner,
    warn_if_tmp_output,
)
from .spec_parser import parse_spec
from .stitcher import stitch_file
from .table_audit import audit_tables
from .trace_catalog import generate_catalog, print_catalog
from .vtbl_gen import find_vtbl_tables
from .wide_seal import (
    WideSealError,
    render_wide_statics,
    seal_wide_literals,
    unseal_wide_literals,
    wide_contract,
)

_VTBL_STATIC_RE = re.compile(
    r"^\s*(?:pub\s+)?static(?:\s+mut)?\s+[A-Za-z_]\w*\s*:\s*[A-Za-z_]\w*Vtbl\b",
)


def _strip_vtbl_statics(rust: str) -> str:
    """Drop `static X: IFooVtbl = { .. };` items (brace/bracket-balanced).

    Used on the c2rust preamble: c2rust spells each slot `f as unsafe extern "C"
    fn(..)`, hardcoding the amd64 ABI, which does not typecheck against i386's
    `extern "stdcall"` slot. vtbl_gen emits these per-arch instead.
    """
    out: list[str] = []
    depth, dropping = 0, False
    for ln in rust.splitlines():
        if not dropping and _VTBL_STATIC_RE.match(ln):
            dropping, depth = True, 0
        if dropping:
            depth += ln.count("[") - ln.count("]") + ln.count("{") - ln.count("}")
            if depth <= 0 and ln.rstrip().endswith(";"):
                dropping = False
            continue
        out.append(ln)
    return "\n".join(out)


def resolve_target(args, path):
    """Pick the rustconv Target: an explicit --target wins (a known TARGETS key, or a
    standalone dll/win32/<name> crate name), else auto-detect from the path. Auto-detect
    now RAISES for unrecognized paths instead of silently using ntoskrnl."""
    name = getattr(args, "target", None)
    if name:
        if name in TARGETS:
            return TARGETS[name]
        if ":" in name:
            # `setuplib:base/setup/lib` — a module outside dll/ and sdk/lib, named for its
            # crate (setuplib-rust) and located by its directory. `rustconv port` passes it.
            name, rel = name.split(":", 1)
            src = (REACTOS_ROOT / rel).resolve()
            if not (src / "CMakeLists.txt").is_file():
                raise SystemExit(f"--target {name}:{rel}: no CMakeLists.txt in {src}")
            return make_user_dll_target(
                name,
                src,
                output_crate=getattr(args, "output_crate", None),
                sys_crate=getattr(args, "sys_crate", None),
            )
        src = REACTOS_ROOT / "dll" / "win32" / name
        if not src.exists():
            src = REACTOS_ROOT / "dll" / name
        if not src.exists():
            # Standalone sdk/lib/<name> static libs (dnslib, conutils, skiplist,
            # ...) → <name>-rust on the ntdll-sys user substrate. Only on the
            # explicit --target path; detect_target (rtl/cmlib routing) untouched.
            src = REACTOS_ROOT / "sdk" / "lib" / name
        return make_user_dll_target(
            name,
            src,
            output_crate=getattr(args, "output_crate", None),
            sys_crate=getattr(args, "sys_crate", None),
        )
    return detect_target(
        path,
        output_crate=getattr(args, "output_crate", None),
        sys_crate=getattr(args, "sys_crate", None),
    )


# Model shortcuts for CLI convenience
MODEL_ALIASES = {
    "sonnet": "claude-sonnet-4-5-20250929",
    "haiku": "claude-haiku-4-5-20251001",
    "opus": "claude-opus-4-8",
    "opus-4-7": "claude-opus-4-7",
    "gemini-pro": "gemini-2.5-pro",
    "gemini-flash": "gemini-2.5-flash",
    # The provisioned key serves the v4 tier (deepseek-v4-pro / deepseek-v4-flash).
    "deepseek": "deepseek-v4-pro",  # default to pro for translation quality
    "deepseek-pro": "deepseek-v4-pro",
    "deepseek-flash": "deepseek-v4-flash",
    "deepseek-v4-pro": "deepseek-v4-pro",
    "deepseek-v4-flash": "deepseek-v4-flash",
    "deepseek-chat": "deepseek-chat",  # legacy aliases (route server-side)
    "deepseek-r1": "deepseek-reasoner",
    "deepseek-reasoner": "deepseek-reasoner",
}


def resolve_model(model_str: str | None) -> str:
    """Resolve model name, supporting aliases."""
    if not model_str:
        return DEFAULT_MODELS[get_backend()]
    return MODEL_ALIASES.get(model_str, model_str)


def resolve_c_path(path_str: str) -> Path:
    """Resolve a C file path, allowing relative paths within ntoskrnl or hal."""
    p = Path(path_str)
    if p.exists():
        return p.resolve()
    # Try as relative to ntoskrnl/
    candidate = NTOSKRNL_DIR / path_str
    if candidate.exists():
        return candidate.resolve()
    # Try as relative to hal/ (HAL target)

    hal_candidate = HAL_DIR / path_str
    if hal_candidate.exists():
        return hal_candidate.resolve()
    # Try with .c extension
    candidate_c = NTOSKRNL_DIR / (path_str + ".c") if not path_str.endswith(".c") else candidate
    if candidate_c.exists():
        return candidate_c.resolve()
    hal_candidate_c = HAL_DIR / (path_str + ".c") if not path_str.endswith(".c") else hal_candidate
    if hal_candidate_c.exists():
        return hal_candidate_c.resolve()
    print(f"Error: file not found: {path_str}", file=sys.stderr)
    print(f"  Tried: {p}, {NTOSKRNL_DIR / path_str}, {HAL_DIR / path_str}", file=sys.stderr)
    sys.exit(1)


def _exports_lost_to_unextracted(scan) -> set[str]:
    """Names the module's .spec EXPORTS that the scanner failed to extract.

    `scan.unextracted` entries are free text ("* cp @ L69", "Cls::m @ L120") because the
    declarator didn't parse — so the failed fn's real name is precisely what we do NOT
    have. Recover it from the C instead: any spec export whose definition is present in
    the source but absent from `scan.functions` was lost in the parse.
    """
    target = detect_target(scan.path)
    if target is None or not target.spec_file or not target.spec_file.is_file():
        return set()
    try:
        spec = parse_spec(target.spec_file)
    except (OSError, ValueError):
        return set()

    got = {f.name for f in scan.functions}
    text = scan.raw_source
    lost: set[str] = set()
    for name, entry in spec.items():
        if name in got or not getattr(entry, "needs_implementation", True):
            continue
        # A .spec entry may carry the i386 stdcall decoration LITERALLY in the export name
        # — `@ stdcall _PfAddFiltersToInterface@24()` — the CORE-8462/CORE-15665 workaround
        # for keeping decoration that autogenerated stubs would lose. The C definition is of
        # course the undecorated `PfAddFiltersToInterface`, so matching only the raw spec
        # name silently defeats this whole guard. That is exactly what happened on
        # iphlpapi_fakestubs.c: 14 of its 16 Pf* exports were unextractable (the parser
        # cannot see through `PFAPIENTRY`, a macro carrying return type AND convention),
        # the cross-check found no spec hit, and the run exited 0 having translated 2/16.
        # So test the undecorated form too.
        candidates = {name}
        m = re.fullmatch(r"_([A-Za-z_][A-Za-z0-9_]*)@\d+", name)
        if m:
            candidates.add(m.group(1))
        if name in got or (m and m.group(1) in got):
            continue
        # Defined in THIS file? A definition is `name` at the head of a line followed by
        # `(` — the ReactOS/wine house style puts the return type + convention on their own
        # lines above it. Cheap and precise enough to distinguish from a call site.
        if any(re.search(rf"^{re.escape(c)}\s*\(", text, re.M) for c in candidates):
            lost.add(name)
    return lost


def _gate_unextracted(scan, allow_partial: bool):
    """Fail-loud gate: refuse to run a paid per-function translation over a
    file whose bodies the scanner could only PARTIALLY extract (C++ methods —
    explorer tier-B/C files surface 1-6 of dozens). AGENTS.md fail-loud rule:
    surface + stop, let the caller opt into the partial explicitly."""
    if not scan.unextracted:
        return
    is_cpp = scan.path.suffix.lower() in (".cpp", ".cxx", ".cc")
    n = len(scan.unextracted)
    total = len(scan.functions) + n
    print(
        f"\n\u26a0 {scan.path.name}: {n} function bodies NOT extractable "
        f"(C++ methods) \u2014 only {len(scan.functions)}/{total} would be translated:",
        file=sys.stderr,
    )
    for entry in scan.unextracted[:12]:
        print(f"    {entry}", file=sys.stderr)
    if n > 12:
        print(f"    \u2026 and {n - 12} more", file=sys.stderr)
    if not is_cpp:
        # .c files: parse artifacts (PSEH macro blocks etc.) have always been
        # dropped silently by the kernel pipeline — keep that path unblocked,
        # just make the drop visible.
        #
        # …UNLESS the dropped body is a real function. The "it's only a parse artifact"
        # assumption held for PSEH macro blocks and quietly stopped holding for ws2_32:
        # `inet_addr(IN CONST CHAR FAR* cp)` came back as an unextractable body named `cp`
        # (tree-sitter can't parse `CHAR FAR*`), so an EXPORTED fn — one the net smoke test
        # actually calls — would have been dropped from a paid run with a warning nobody
        # reads. A lost export is never an artifact. Cross-check the drop against the
        # module's .spec and HARD FAIL on a hit, per the fail-loud rule.
        lost = sorted(_exports_lost_to_unextracted(scan))
        if lost:
            print(
                f"\n✗ {scan.path.name}: {len(lost)} SPEC-EXPORTED function(s) are in the "
                "unextractable set — this is a scanner parse failure, not a macro artifact:",
                file=sys.stderr,
            )
            for name in lost[:12]:
                print(f"    {name}", file=sys.stderr)
            print(
                "  Refusing to spend a paid translation that silently omits an export.\n"
                "  Fix the scanner (see _blank_memmodel / _normalize_callbacks for the shape),\n"
                "  or pass --allow-partial to translate the extractable subset deliberately.",
                file=sys.stderr,
            )
            if not allow_partial:
                sys.exit(1)
        print("  (.c file: legacy behavior, proceeding without these)", file=sys.stderr)
        return
    if allow_partial:
        print(
            "  --allow-partial: proceeding with the extractable free-function "
            "subset ONLY (deliberate partial).",
            file=sys.stderr,
        )
        return
    print(
        "  HARD FAIL: this would silently under-translate the file. "
        "Pass --allow-partial to translate the free-function subset "
        "deliberately, or wait for the C++ method extraction path "
        "(docs/engineering/explorer-rust-translation-plan.md).",
        file=sys.stderr,
    )
    sys.exit(1)


def cmd_scan(args):

    path = resolve_c_path(args.file)
    result = scan_file(path)
    print_scan_result(result)


def cmd_dup_census(args):
    """Read-only exact/alpha-normalized function duplication census."""
    paths = []
    for raw in args.paths:
        path = Path(raw)
        if not path.is_absolute():
            path = REACTOS_ROOT / path
        if not path.exists():
            raise FileNotFoundError(f"dup-census scope not found: {path}")
        paths.append(path.resolve())
    report = dup_census.run_census(paths, REACTOS_ROOT, top=args.top)
    dup_census.print_census(report, as_json=args.json)


def cmd_hybrid_closure(args):
    """Measure which symbols of a still-compiled TU the rest of the link actually needs."""
    build_dir, link_root, archive = hybrid_closure.resolve_paths(REACTOS_ROOT, args)
    try:
        report = hybrid_closure.measure(
            REACTOS_ROOT, args.tu, build_dir, link_root, archive, args.arch, args.target_dir
        )
    except (FileNotFoundError, RuntimeError) as exc:
        print(f"error: {exc}", file=sys.stderr)
        sys.exit(2)
    hybrid_closure.print_report(report, as_json=args.json)


def cmd_shell32_tail_inventory(args):
    """Ratchet the planned shell32 tail against the literal live CMake source lists."""
    ledger = (
        Path(args.ledger) if args.ledger else REACTOS_ROOT / shell32_tail_inventory.DEFAULT_LEDGER
    )
    if not ledger.is_absolute():
        ledger = (Path.cwd() / ledger).resolve()
    report = shell32_tail_inventory.check_inventory(REACTOS_ROOT, ledger)
    shell32_tail_inventory.print_report(report, as_json=args.json)
    if not report.ok:
        sys.exit(1)


def _trivial_cpp_body(body_text: str) -> bool:
    """True for an empty body with no init-list or an all-zero init-list."""
    match = re.search(r"\{(.*)\}\s*$", body_text, flags=re.S)
    if match is None or match.group(1).strip():
        return False
    opening = body_text.find("(")
    closing = cpp_com._matching_paren(body_text, opening) if opening >= 0 else -1
    tail = body_text[closing + 1 : match.start()].strip() if closing >= 0 else ""
    if ":" not in tail:
        return True
    initializers = tail.split(":", 1)[1]
    return all(
        re.fullmatch(r"[A-Za-z_]\w*\s*\(\s*(?:NULL|nullptr|FALSE|0)\s*\)", item.strip())
        for item in cpp_com._split_args(initializers)
    )


def _cost_cap_reached(cost_cap: float | None, running_cost: float) -> bool:
    """True before a paid call when the configured run budget is exhausted."""
    return cost_cap is not None and running_cost >= cost_cap


def _scan_cpp_class_fragment(header: Path, class_name: str):
    """Scan one class body without a surrounding header's preprocessor noise.

    SDK C++ helpers often live under ``#ifdef __cplusplus``.  Whole-header
    preprocessing correctly removes that branch for a C-oriented scan, but here
    the caller has already selected a concrete C++ base.  Isolating its balanced
    class body makes that explicit and keeps unrelated SDK inline functions out
    of the paid closure.
    """
    body = cpp_com._find_class_text(header, class_name)
    bases = cpp_com._class_bases(header, class_name)
    base_clause = f" : {', '.join(bases)}" if bases else ""
    fragment = f"class {class_name}{base_clause} {{\n{body}\n}};\n"
    with tempfile.NamedTemporaryFile(mode="w", suffix=".cpp", encoding="utf-8") as temp:
        temp.write(fragment)
        temp.flush()
        return scan_file(Path(temp.name), force_cpp=True)


_HEADER_SWITCHES = frozenset(
    {
        "NDEBUG",
        "YDEBUG",
        "WIN32_NO_STATUS",
        "NTOS_MODE_USER",
        "WIN32_LEAN_AND_MEAN",
        "COM_NO_WINDOWS_H",
        "_INC_WINDOWS",
        "NONAMELESSUNION",
        "NONAMELESSSTRUCT",
        "COBJMACROS",
    }
)


def _preamble_has_translatable_content(preamble_src: str) -> bool:
    """True when a file-scope preamble holds anything a translation must carry over.

    False ONLY for a preamble made up entirely of #include / #pragma once / comments —
    i.e. one whose correct Rust rendering is genuinely the empty string, because the crate
    prelude (not the per-file translation) resolves those includes.

    This exists to keep "the model emitted nothing" distinguishable from "there was nothing
    to emit". Those are indistinguishable downstream, so an all-includes preamble would
    otherwise burn a call and then trip the empty-output hard-fail — after every function in
    the file had already been paid for.

    Conservative by construction: anything else at file scope (a typedef, struct, static,
    #define, WINE_DEFAULT_DEBUG_CHANNEL, extern) returns True and takes the normal paid path.
    Preamble content is what every file-scope struct/table rides on, so the bias must be
    toward translating.
    """
    src = re.sub(r"/\*.*?\*/", " ", preamble_src, flags=re.S)  # block comments
    src = re.sub(r"//[^\n]*", " ", src)  # line comments
    for line in src.splitlines():
        stripped = line.strip()
        if not stripped:
            continue
        if re.match(r"^#\s*include\b", stripped):
            continue
        if re.match(r"^#\s*pragma\s+once\b", stripped):
            continue
        # A wine debug-channel declaration is owned by the CRATE PRELUDE in this project, not
        # by the per-file translation (the lsasrv/netapi32 pattern: wine/debug.h is parsed but
        # not allowlisted, and the prelude supplies the no-op TRACE/ERR/WARN/FIXME macros).
        # So its correct per-file Rust rendering is empty too. Left in, it is also actively
        # harmful: the model renders it NONDETERMINISTICALLY — one run emits nothing (→ the
        # empty-output hard-fail), the next invents a __wine_debug_channel static that collides
        # with the prelude's. wkssvc domain.c and rpcserver.c took one branch each, same input.
        if re.match(r"^WINE_(DEFAULT|DECLARE)_DEBUG_CHANNEL\s*\(", stripped):
            continue
        # A value-less switch that only steers how the INCLUDED headers expand (`#define
        # NDEBUG` before `<debug.h>`): no per-file Rust rendering, like the includes it
        # configures. setuplib's genlist.c preamble was exactly that, and the model's correct
        # empty answer hit the hard-fail after 10/10 paid functions. INITGUID is NOT here: it
        # changes which GUIDs this TU emits.
        m = re.match(r"^#\s*(?:define|undef)\s+([A-Za-z_]\w*)\s*$", stripped)
        if m and m.group(1) in _HEADER_SWITCHES:
            continue
        return True  # any surviving token => real content
    return False


def _extend_cpp_class_scan(scan, source: Path, class_name: str, local_bases: tuple = ()):
    """Merge source-owned in-class method definitions from the defining header.

    C++ permits method bodies in the class header.  They are part of the same
    class-atomic translation and vtable contract even when the implementation
    file contains most out-of-line methods.  Force the C++ grammar for `.h` and
    merge only this class's definitions; unrelated header helpers stay outside
    the paid closure.

    ``local_bases`` are concrete same-header base classes being flattened into
    this translation (CppComClass.local_base_classes): their method bodies are
    merged too, then re-owned by the derived class so the vtable binder, worker
    naming, and `this` typing all see one flat class.
    """
    wanted = {class_name, *local_bases}
    header = find_class_header(source, class_name)
    scans = []
    if header != source:
        scans.append(scan_file(header, force_cpp=True))
    for base in local_bases:
        base_header = find_class_header(source, base)
        if base_header == header:
            continue
        scans.append(_scan_cpp_class_fragment(base_header, base))
        # A concrete base commonly keeps declarations/state in Base.h and its
        # inherited vtable bodies in the matching Base.cpp. Flatten both: the
        # derived object publishes those slots and cannot omit their bodies.
        base_impl = base_header.with_suffix(".cpp")
        if base_impl.is_file() and base_impl.resolve() != source.resolve():
            scans.append(scan_file(base_impl, force_cpp=True))
    for header_scan in scans:
        header_lines = header_scan.raw_source.splitlines()

        def is_template_method(item, lines=header_lines) -> bool:
            # A class-local function template is instantiated by whichever derived TU
            # calls it; it is not an owned body of this concrete class TU. Pulling it
            # into a class-atomic worker invents an unbound `T` (CEnumIDListBase's
            # HasItemWithCLSIDImpl was the first tail pilot to expose this).
            prefix = "\n".join(lines[max(0, item.line_start - 4) : item.line_start + 1])
            return bool(re.search(r"\btemplate\s*<", prefix))

        inline = [
            item
            for item in header_scan.functions
            if item.cpp_class in wanted and not is_template_method(item)
        ]
        # A method is uniquely identified by (name, arity): C++ overloads that differ
        # in parameter count are distinct methods, not duplicate bodies (e.g. a static
        # inline `GetClassID()` helper in the header alongside the virtual
        # `GetClassID(CLSID*)` defined out-of-line). Only a genuine same-name AND
        # same-arity body present in BOTH header and impl is a real double-definition
        # conflict; true overloads are resolved downstream by the arity-based binder.
        # A stripped destructor shares (name, arity) with a 0-arg constructor —
        # ctor-vs-dtor is never a double definition, so the kind flags join the key.
        existing = {
            (item.cpp_class, item.name, len(item.params), item.is_ctor, item.is_dtor)
            for item in scan.functions
            if item.cpp_class in wanted
        }
        duplicate = sorted(
            item.name
            for item in inline
            if (item.cpp_class, item.name, len(item.params), item.is_ctor, item.is_dtor) in existing
        )
        if duplicate:
            raise CppComError(
                f"{class_name}: method body defined in both implementation and header: "
                + ", ".join(duplicate)
            )
        scan.functions.extend(inline)
        scan.identifiers.update(header_scan.identifiers)
        scan.local_names.update(header_scan.local_names)
        scan.unextracted.extend(
            item for item in header_scan.unextracted if any(name in item for name in wanted)
        )
    if local_bases:
        # C++ construction order is base-then-derived. One flattened worker per
        # special member: a trivially-empty derived ctor/dtor yields to the base's;
        # two non-trivial bodies would need sequencing we don't generate — fail loud.
        for is_dtor_kind in (False, True):
            kind = "dtor" if is_dtor_kind else "ctor"
            derived_special = [
                item
                for item in scan.functions
                if item.cpp_class == class_name and (item.is_dtor if is_dtor_kind else item.is_ctor)
            ]
            base_special = [
                item
                for item in scan.functions
                if item.cpp_class in local_bases
                and (item.is_dtor if is_dtor_kind else item.is_ctor)
            ]
            if derived_special and base_special:
                trivial = [
                    item
                    for item in derived_special + base_special
                    if _trivial_cpp_body(item.body_text)
                ]
                keep_pool = derived_special + base_special
                survivors = [item for item in keep_pool if item not in trivial]
                if is_dtor_kind:
                    # C++ destruction is derived first, then concrete bases. Keep
                    # the derived dtor as the scaffold's primary dtor worker and
                    # turn each non-trivial base dtor into an explicitly sequenced
                    # ordinary worker. (Zero-only/empty dtors disappear.)
                    for item in trivial:
                        scan.functions.remove(item)
                    base_survivors = [item for item in base_special if item in survivors]
                    for item in base_survivors:
                        item.name = f"{item.cpp_class}_base_dtor"
                        item.return_type = "void"
                        item.is_dtor = False
                    continue
                if len(survivors) > 1:
                    raise CppComError(
                        f"{class_name}: both derived and base define a non-trivial {kind}; "
                        "flattening cannot sequence them — translate by hand or simplify the C++"
                    )
                for item in trivial if survivors else trivial[1:]:
                    scan.functions.remove(item)
        # A derived method body hiding a base body of the same (name, arity, kind)
        # cannot keep that name once both are flattened into one class — but it is
        # an ordinary C++ override, not a shape we have to refuse. Give the BASE
        # body its own qualified worker, exactly as the dtor branch above does with
        # `_base_dtor`, so the shared implementation is translated ONCE and the
        # override can call it instead of the model re-deriving the base by hand.
        # (Silently duplicating a shared concrete base into the derived translation
        # is a known way to fork behaviour — CObjectWithSiteBase::SetSite is six
        # lines of refcount swap that every site-aware class in the tail inherits.)
        derived_keys = {
            (item.name, len(item.params), item.is_ctor, item.is_dtor)
            for item in scan.functions
            if item.cpp_class == class_name
        }
        for item in scan.functions:
            if (
                item.cpp_class in local_bases
                and not (item.is_ctor or item.is_dtor)
                and (item.name, len(item.params), item.is_ctor, item.is_dtor) in derived_keys
            ):
                item.name = f"{item.cpp_class}_base_{item.name}"
        for item in scan.functions:
            if item.cpp_class in local_bases:
                item.cpp_class = class_name
    return scan


def _cache_suffix(
    args, fact_seal_enabled: bool, call_seal_enabled: bool, wide_seal_enabled: bool = False
) -> str:
    """Namespace the resume cache by the seals that were active when it was filled.

    One INDEPENDENT token per seal, appended — never folded into an existing one. That is
    what lets a new protection ship for free: a `--seal-wide` run misses the cache and
    pays, while every `__factseal_v1__callseal_v1` body already banked stays addressable
    under its own name and is never re-billed.
    """
    cache_tag = getattr(args, "cache_tag", None)
    suffix = f"__{cache_tag}" if cache_tag else ""
    if fact_seal_enabled:
        suffix += "__factseal_v1"
    if call_seal_enabled:
        suffix += "__callseal_v1"
    if wide_seal_enabled:
        suffix += "__wideseal_v1"
    return suffix


def _open_response_cache(
    args, subsystem, c_file, fact_seal_enabled, call_seal_enabled, wide_seal_enabled=False
):
    """Durable per-function resume cache → re-run to resume a killed/capped run for $0.

    Skips every function whose C body is unchanged. Coexists with the sink archive
    (per-run, kill-safe streaming); this one is the stable cross-run skip layer.
    `--no-resume` forces fresh.
    """
    suffix = _cache_suffix(args, fact_seal_enabled, call_seal_enabled, wide_seal_enabled)
    base = f"{subsystem}__{c_file.stem}"
    if not getattr(args, "dry_run", False):
        require_no_namespace_rebuy(base, base + suffix)
    return ResponseCache(base + suffix)


# Bodies accepted on a LAST attempt despite ignoring the fact envelope. Drained into
# the run's AUDIT report, so "unverified constants" is a finding a human sees rather
# than a silent relaxation. Never a reason to skip review: `_tzset` is what one of
# these looks like when it is wrong (fact_seal.unseal_facts).
UNVERIFIED_CONSTANT_BODIES: list[str] = []

# Bodies spanning a preprocessor conditional, with which arm is live when that is
# decidable. See the emission site for why this is report-only.
CONDITIONAL_ARM_BODIES: list[str] = []

# Bodies whose UTF-16 tokens all survived but in a different ORDER than the C used them.
# The one damage a closed-set check cannot see (the permutation that got `_tzset` past the
# constant seal), so it is a review queue rather than a rejection — legitimate reordering
# exists, and a human decides. See wide_seal.unseal_wide_literals.
REORDERED_WIDE_LITERAL_BODIES: list[str] = []


def _audit_paid_body(dep, result, **kwargs) -> str:
    """Attach a new validation edge; never mutate the archived raw purchase."""

    checker = CHECKER_FINGERPRINT
    start = time.monotonic()
    try:
        code = _audit_paid_body_impl(dep, result, **kwargs)
    except Exception as error:
        record_validation(
            getattr(result, "attempt_id", None),
            checker=checker,
            verdict="audit_reject",
            unit=dep.name,
            source_sha256=digest(dep.body_text),
            error=str(error),
            elapsed_ms=round((time.monotonic() - start) * 1000),
        )
        raise
    record_validation(
        getattr(result, "attempt_id", None),
        checker=checker,
        verdict="audit_pass",
        unit=dep.name,
        source_sha256=digest(dep.body_text),
        candidate_sha256=digest(code),
        elapsed_ms=round((time.monotonic() - start) * 1000),
        scope="deterministic body audits, not semantic equivalence",
    )
    return code


def _audit_paid_body_impl(
    dep,
    result,
    *,
    cpp_com_mode: bool,
    is_cpp_source: bool,
    fact_seals,
    call_seals,
    wide_seals,
    target_name: str,
    strict: bool = True,
) -> str:
    """Unseal and audit one paid reply; raise `CppComError` if it fails.

    Extracted so the sequential path can re-roll a rejected worker without
    duplicating the acceptance criteria — a retry that checked anything less than
    the first attempt would launder exactly the defects sealing exists to catch.

    Mutates ``result.rust_code`` to the normalized/unsealed text, which is what the
    caller banks: the cache must hold what was ACCEPTED, not the raw reply.
    """
    if not cpp_com_mode:
        # Sealing is language-neutral; unsealing must be too. EXACTLY ONCE: this used to
        # unseal, then unseal the unsealed text again. The second pass sees no tokens, so
        # every one reads as missing, and it only ever passed because the by-name recovery
        # inside `unseal_calls` re-sealed them from the names the FIRST pass had just
        # restored. That accident rejected four netcfgx bodies the first pass accepted and
        # cost two re-samplings of five TUs. See `_arity_disagreements` for the real check
        # that replaced what the second pass was catching by luck.
        result.rust_code = _unseal_worker(
            dep, result.rust_code, fact_seals, call_seals, wide_seals, strict=strict
        )
        if is_cpp_source:
            result.rust_code = normalize_cpp_idioms(dep.body_text, result.rust_code)
            result.rust_code = _cpp_free_half_semantics(dep, result.rust_code)
        return result.rust_code
    result.rust_code = _normalize_unseal_cpp_worker(dep, result.rust_code, fact_seals, call_seals)
    signature = worker_signature(dep.name, dep.cpp_class, dep.params, dep.return_type)
    findings = audit_worker_body(dep.body_text, result.rust_code, f"{dep.name}_body", signature)
    if findings:
        raise CppComError(
            f"{dep.name}: paid body failed deterministic audit:\n  - " + "\n  - ".join(findings)
        )
    return result.rust_code


def _stale_c_body(dep, hit: str, args, reaudit_failed: set[str]) -> bool:
    """Re-judge a banked plain-C body against TODAY's deterministic checks.

    A cache hit used to be accepted as banked, so a check added after the purchase could
    never reach it -- the stricter audit was blind to everything already paid for. The
    finding is always printed; the body is discarded for re-purchase only under an
    explicit --rebuy-rejected (a policy change asks for revalidation, it does not
    authorise payment by itself).
    """
    if getattr(dep, "cpp_class", None):
        return False
    findings = body_findings(dep.body_text, hit)
    if not findings:
        return False
    print(
        f"  {dep.name}: banked body FAILS the current audit: " + "; ".join(findings),
        file=sys.stderr,
    )
    if not getattr(args, "rebuy_rejected", False):
        return False
    reaudit_failed.add(dep.name)
    return True


def _adopt_rejected(
    dep, cache, fact_seals, call_seals, wide_seals, cpp_com_mode: bool, func_idx, contract, sealed
) -> str | None:
    """A stored REJECTED reply for ``dep``, IF it passes today's audit. Else None.

    A reject cost an API call, so it is paid artifact under the same rule as an accepted
    one, and the reasons it stops being a reject are ordinary: a seal defect gets fixed
    (this file's double-unseal alone rejected four netcfgx bodies that were never wrong),
    or a human repairs by hand the one line the model would not — netcfgx's
    `ScanForInfFile` refused three samplings to reproduce the C's `continue` past its own
    `FindNextFileW`, and the rest of that body is correct. Paying twice for a body already
    on disk is waste, not caution.

    The audit is the whole gate and it is the SAME audit, so nothing is laundered: a reply
    that still damages the envelope is still refused, silently, and the caller pays.
    Skipped for cpp_com, whose acceptance also runs the worker-boundary normalizations.
    """
    if cpp_com_mode:
        return None
    rejected = cache.get_rejected(dep.name)
    if rejected is None:
        return None
    try:
        body = _unseal_worker(dep, rejected, fact_seals, call_seals, wide_seals)
    except CppComError:
        return None
    # Banked under this run's contract and seal set, exactly as a paid acceptance is —
    # otherwise the next run misses and buys the same body a third time.
    cache.put(dep.name, dep.body_text, body, contract, sealed)
    print(f"  [{func_idx}] {dep.name} ADOPTED a stored reject — passes the audit now ($0)")
    return body


def _reject_correction(error: "CppComError | None") -> str:
    """What to append to a worker prompt when its previous attempt was REJECTED.

    Re-rolling the IDENTICAL prompt only fixes a reject that was sampling noise, and the
    ones that survive to a second attempt mostly are not. netcfgx's `ScanForInfFile`
    duplicated the same call token in two independent samplings because the C `continue`s
    straight past its own `FindNextFileW` — an infinite loop the model will not write —
    and `CreateInstanceKey` dropped the `if (ret == FALSE)` guard both times by inlining
    the `goto done` tail per path. Nothing about a blind re-roll moves either. The audit
    already knows exactly what broke, so tell the model instead of hoping.

    The faithfulness clause is the load-bearing half: every one of these rejects is the
    model IMPROVING on the C, and without it a correction just invites a different
    improvement.
    """
    if error is None:
        return ""
    return (
        "\n\n## Your previous attempt was REJECTED\n"
        f"{error}\n\n"
        "Translate the function again and fix exactly that. The envelope is not advice: a "
        "token stands for one C occurrence. The C is authoritative even where it looks "
        "wrong — if it loops forever, leaks, or tests a value that cannot change, "
        "reproduce that. Do not repair the C, and do not restructure the body to make the "
        "finding go away."
    )


def _unseal_worker(
    dep, rust: str, fact_seals, call_seals, wide_seals, *, strict: bool = True
) -> str:
    """Undo fact/call sealing on a model response, verifying the closed envelope.

    Target- and language-neutral: sealing is a lexical transform, so ANY translation
    that was sealed must be unsealed, not just a shell32 COM worker. Keeping this
    inside the COM-only helper meant a sealed plain-C run shipped raw
    `__RUSTCONV_FACT_nnnn__` / `__RUSTCONV_CALL_nnnn__` tokens straight into the
    assembled module — latent until now only because sealing for plain C landed
    2026-07-24 and no paid plain-C run had used it yet (all 5,170 ledger entries
    with seal_facts=True are shell32), while AGENTS.md tells every plain-C
    translation to "always pass both".

    THE one unseal for both dialects. A separate plain-C copy existed alongside this
    and every plain-C caller ran both, in sequence, over the same text — see
    `_audit_paid_body`. It is not the shell32 boundary that is shared here
    (`normalize_worker_body` / `normalize_worker_semantics` reshape a COM worker onto a
    fixed signature and mean nothing for a free function); only the unseal itself is,
    and it always was.
    """
    rust = scrub_comment_token_echoes(rust)
    # Sealed tokens the model omitted that live only in code the build never executes
    # (`#if`-disabled arms, DUMMY_PSEH except-blocks). Accepted, but printed: relaxing a
    # guarantee silently is the failure mode sealing exists to prevent (dead_regions.py).
    dead_omissions: list[str] = []
    # A body that ignored the fact envelope wholesale, accepted only because this was
    # the LAST attempt. Reported the same way `dead_omissions` is, and for the same
    # reason. See fact_seal.unseal_facts.
    ignored_envelope: list[str] = []
    # UTF-16 tokens that survived but moved. Reported, never rejected — see
    # REORDERED_WIDE_LITERAL_BODIES.
    order_findings: list[str] = []
    try:
        calls = call_seals.get(dep.name)
        if calls is not None:
            require_argument_facts(rust, calls)
            rust = unseal_calls(rust, calls, dead_omissions=dead_omissions)
        facts = fact_seals.get(dep.name)
        if facts is not None:
            rust = canonicalize_hresult_predicates(rust)
            rust = unseal_facts(
                rust,
                facts,
                dead_omissions=dead_omissions,
                ignored_envelope=ignored_envelope,
                strict=strict,
            )
            # The closed SET check above passes a permutation; this catches constants
            # that came back in each other's places (placement_audit.py).
            for finding in placement_findings(dep.body_text, rust):
                if strict:
                    raise FactSealError(finding.message())
                ignored_envelope.append(finding.message().replace("\n", "; "))
        if not getattr(dep, "cpp_class", None):
            for note in parse_findings(rust) + control_flow_findings(dep.body_text, rust):
                if strict:
                    raise FactSealError(note)
                ignored_envelope.append(note)
        # LAST, which is the exact reverse of the sealing order. It is also the only one
        # that substitutes RUST rather than restoring C: running it earlier would hand the
        # fact seal's by-name recovery a body full of generated `0x0041u16` elements to
        # rummage through.
        wides = wide_seals.get(dep.name)
        if wides is not None:
            rust = unseal_wide_literals(
                rust,
                wides,
                dead_omissions=dead_omissions,
                order_findings=order_findings,
            )
    except (CallSealError, FactSealError, WideSealError) as error:
        raise CppComError(f"{dep.name}: {error}") from error
    if dead_omissions:
        print(f"    [{dep.name}] omitted in non-executed code: {', '.join(dead_omissions)}")
    for note in ignored_envelope:
        UNVERIFIED_CONSTANT_BODIES.append(f"{dep.name}: {note}")
        print(f"    [{dep.name}] ACCEPTED UNVERIFIED: {note}", file=sys.stderr)
    for note in order_findings:
        REORDERED_WIDE_LITERAL_BODIES.append(f"{dep.name}: {note}")
        print(f"    [{dep.name}] WIDE LITERALS REORDERED: {note}", file=sys.stderr)
    return rust


# Findings from the `--free-functions` half, which until now was never audited at all:
# `_audit_paid_body` returns for every non-cpp_com body BEFORE it reaches
# `audit_worker_body`. So the half that owns the `*_Constructor` family — the functions that
# ALLOCATE — had no check that the model hadn't invented a deallocator, reversed a call
# nesting, or dropped a constant.
#
# REPORT-ONLY, and it stays that way until these have been read. The class-half audit does
# not transfer wholesale: a free function legitimately carries `#[no_mangle]` and an
# `extern "C"` definition, both of which the class half forbids because there the scaffold
# owns the export. So a finding here is evidence about the CHECK as much as about the body.
FREE_HALF_AUDIT_FINDINGS: list[str] = []


def _report_free_half_audit(dep, rust: str) -> None:
    """Run the language-neutral worker audit over one free function, report-only."""
    for finding in audit_worker_body(dep.body_text, rust, dep.name, None, scaffold_owned=False):
        FREE_HALF_AUDIT_FINDINGS.append(f"{dep.name}: {finding}")
        print(f"    [{dep.name}] FREE-HALF AUDIT: {finding}", file=sys.stderr)


def _cpp_free_half_semantics(dep, rust: str) -> str:
    """Apply the C++ semantic lowerings to the FREE half of a C++ TU.

    Everything the class path does after a body comes back -- the `delete this` lowering, the
    `new X()`/`delete v` scaffold pairing, the SENDTO_ITEM guard -- was gated on
    `cpp_com_mode`, so the `--free-functions` half of the same `.cpp` got only the lighter
    `normalize_cpp_idioms` and none of it. That is precisely backwards: the free half is where
    the `*_Constructor` family lives, which is to say the functions that ALLOCATE and FREE the
    objects. `RecycleBin5File_Constructor` shipped `RecycleBin5File()` (a call to the struct,
    skipping the vptr store) and a CRT `free` over `HeapAlloc(GetProcessHeap())` memory.

    Scoped to `is_cpp_source` at the call sites, so a plain-C module's cached bodies keep
    replaying byte-identically; these rewrites are all keyed on C++ syntax anyway.

    The report-only audit rides along here rather than at each of the five call sites: a
    check you must remember to invoke in five places is a check that will be missed in one,
    which is how the free half went unaudited to begin with.
    """
    rust = normalize_worker_semantics(dep.body_text, rust)
    _report_free_half_audit(dep, rust)
    return rust


def _normalize_unseal_cpp_worker(dep, rust: str, fact_seals, call_seals) -> str:
    """Apply the deterministic shell32 worker boundary to a model response."""
    signature = worker_signature(dep.name, dep.cpp_class, dep.params, dep.return_type)
    rust = normalize_worker_body(rust, signature)
    # The COM path has its own frozen UTF-16 lowering (cpp_com.rewrite_wide_literals), so
    # there is never a wide envelope to undo here — passing one would be a lie about which
    # mechanism owns those bytes.
    rust = _unseal_worker(dep, rust, fact_seals, call_seals, {})
    rust = normalize_worker_body(rust, signature)
    return normalize_worker_semantics(dep.body_text, rust)


_USE_LINE_RE = re.compile(r"^use ([A-Za-z_][\w:]*)::(?:\{([^{}]*)\}|([A-Za-z_]\w*))\s*;$")


def _dedupe_hoisted_imports(code: str) -> str:
    """Import each NAME once, keeping the first `use` that brings it in.

    The materializer HOISTS a body-level `use` to file scope, and writing the import inside the
    function body is the only way to make it survive materialization at all — so a name several
    bodies need arrives several times and every copy after the first is
    `E0252: the name ... is defined multiple times`.

    Deduping whole LINES is not enough: the copies are rarely identical, they are overlapping
    SETS (`use m::{A, B};` beside `use m::{B, C};`). So this works per name, rewrites a brace
    list down to the names still needed, and drops a `use` that has nothing left.
    """
    seen: set[str] = set()
    out: list[str] = []
    for line in code.split("\n"):
        match = _USE_LINE_RE.match(line.strip())
        if match is None:
            out.append(line)
            continue
        path, braced, single = match.group(1), match.group(2), match.group(3)
        names = [n.strip() for n in (braced or single or "").split(",") if n.strip()]
        keep = [n for n in names if n.split(" as ")[-1].strip() not in seen]
        seen.update(n.split(" as ")[-1].strip() for n in names)
        if not keep:
            continue
        out.append(
            f"use {path}::{{{', '.join(keep)}}};" if len(keep) > 1 else f"use {path}::{keep[0]};"
        )
    return "\n".join(out)


def _crate_of(output: str | None) -> Path | None:
    """The crate directory an `-o` path lands in — the nearest ancestor holding a Cargo.toml.

    `-o` is how a TU with more than one class writes its second and later slices, and nothing
    requires that path to sit inside the crate `--target` resolves to.
    """
    if not output:
        return None
    for parent in Path(output).resolve().parents:
        if (parent / "Cargo.toml").is_file():
            return parent
    return None


def _module_lock_of(args, rustconv_target=None) -> tuple[str, str]:
    """The COM module-lock pair for the crate this run's output lands in.

    Keyed on the CRATE rather than `--target` because the host DLL defines the symbol. A
    `--target shell32` run writing into menuband-rust is the case that matters: menuband's
    staticlib links into rshell.dll too, which has no `Shell32RustObject*`.
    """
    crate = _crate_of(getattr(args, "output", None))
    if crate is None and rustconv_target is not None:
        crate = getattr(rustconv_target, "output_crate", None)
    return module_lock_for_crate(crate.name if crate is not None else None)


def _resolve_activation_source(args) -> Path | None:
    """`--activation-source`, resolved against the repo root like every other source path.

    A class is often constructed by a different TU than the one defining it, and scanning
    only its own file reports the activation as unmapped when it is merely next door.
    """
    raw = getattr(args, "activation_source", None)
    if raw is None:
        return None
    path = Path(raw)
    return path if path.is_absolute() else REACTOS_ROOT / path


def cmd_candidate_check(args):
    """Batch-compile candidate workers against real target substrate."""
    manifest = Path(args.manifest)
    if not manifest.is_absolute():
        manifest = (Path.cwd() / manifest).resolve()
    arches = args.arches or ["i386", "amd64", "arm64"]
    report = candidate_oracle.run_oracle(
        manifest, arches, repair=args.repair, check_only=getattr(args, "check_only", False)
    )
    candidate_oracle.print_report(report, as_json=args.json)
    if any(item["status"] != "ok" for item in report["results"]):
        sys.exit(1)


def cmd_difftest_contract(args):
    """Compile and grade a reviewed stateless host contract without paid calls."""
    report = difftest_contract.grade(Path(args.contract), Path(args.out))
    print(
        json.dumps(report, indent=2)
        if args.json
        else f"{report['status']}: {report.get('cases', 0)} cases; {Path(args.out) / 'report.json'}"
    )
    if report["status"] != "pass":
        sys.exit(1)


def cmd_difftest_gen(args):
    """Generate a differential harness for a whole translated C file, from its signatures.

    The existing `msvcrt-difftest` proved the mechanism — compile the genuine `.c` as the
    reference, rename its symbols to `ref_*`, assert parity in one process — but every
    instrument is hand-written and its symbol list is hardcoded with a "regenerate if the C
    changes" warning. That does not scale to a file-at-a-time translation track.

    Prints the plan to stderr and writes the cases to `--out`. Refusals are part of the
    OUTPUT, not a silent omission: a function this cannot grade still needs a hand-written
    case, and it must be visible in the generated file rather than absent from it.
    """
    c_path = Path(args.c_file)
    scan = scan_file(c_path)
    plan = difftest_gen.plan_file(scan, c_path.stem)
    total = len([f for f in scan.functions if not f.is_static])
    print(plan.summary(), file=sys.stderr)
    for refusal in plan.refusals:
        print(f"  refused: {refusal}", file=sys.stderr)
    text = difftest_gen.emit(plan, str(c_path), total)
    if args.out:
        Path(args.out).write_text(text)
        print(f"  -> {args.out} ({len(text.splitlines())} lines)", file=sys.stderr)
    else:
        print(text)
    return 0


def cmd_crate_repair(args):
    """Close the mechanical rustc roots of a whole crate deterministically, for $0.

    Runs BEFORE any paid repair pass: the same "substrate closure before payment"
    discipline the candidate path already follows. Every applied edit records its
    rule and authority, and a pointer-width-dependent cast is only applied when
    >=2 targets independently propose the identical replacement — a cast inferred
    from one arch bakes that arch's width and silently breaks the other two.
    """
    session = crate_repair.repair_crate(
        args.crate,
        root=RUST_DIR,
        targets=args.arches,
        rules=args.rules,
        crate_dir=Path(args.crate_dir) if args.crate_dir else None,
        types_module=args.types_module,
        max_passes=args.max_passes,
        allow_single_target_width=args.allow_single_target_width,
        dry_run=args.dry_run,
    )
    crate_repair.print_session(session, as_json=args.json)
    if session.status in ("regressed", "max_passes"):
        sys.exit(1)


def cmd_substrate_plan(args):
    """Inspect or regenerate the authority plan for a candidate oracle report."""
    report_path = Path(args.report).resolve()
    report = json.loads(report_path.read_text())
    analysis = report.get("substrate_analysis")
    if analysis is None or args.refresh:
        sources = {
            item["name"]: Path(item["source"]).resolve() if item.get("source") else None
            for item in report.get("results", [])
        }
        analysis = substrate_closure.analyze_report(report, candidate_sources=sources)
        if args.refresh:
            report["substrate_analysis"] = analysis
            report_path.write_text(json.dumps(report, indent=2, sort_keys=True))
    substrate_closure.print_analysis(analysis, as_json=args.json, limit=args.limit)


def cmd_skeleton_check(args):
    """Compare C++ and Rust candidates using an experimental semantic skeleton."""
    manifest = Path(args.manifest)
    if not manifest.is_absolute():
        manifest = (Path.cwd() / manifest).resolve()
    report = semantic_skeleton.run_corpus(manifest)
    semantic_skeleton.print_report(report, as_json=args.json)


def cmd_cpp_tu(args):
    """Fail-loud whole-C++-TU ownership/layout/activation preflight."""
    source = Path(args.path)
    if not source.is_absolute():
        source = REACTOS_ROOT / source
    report = cpp_tu.analyze_translation_unit(source, REACTOS_ROOT)
    cpp_tu.print_report(report, as_json=args.json)
    if not report.ok:
        sys.exit(1)


def cmd_cpp_com(args):
    """Prove and emit the deterministic ABI scaffold for one ATL COM class."""
    source = Path(args.path)
    if not source.is_absolute():
        source = REACTOS_ROOT / source
    if not source.is_file() or source.suffix.lower() not in (".cpp", ".cc", ".cxx"):
        raise CppComError(f"C++ source not found: {source}")

    scan = _extend_cpp_class_scan(scan_file(source), source, args.class_name)
    class_methods = [f for f in scan.functions if f.cpp_class == args.class_name]
    if not class_methods:
        raise CppComError(f"{args.class_name}: no extracted method bodies in {source}")
    if scan.unextracted:
        raise CppComError(
            f"{source.name}: {len(scan.unextracted)} body/bodies remain unextractable; "
            "fix scanner coverage before deterministic class translation:\n  - "
            + "\n  - ".join(scan.unextracted)
        )

    if getattr(args, "kernel_owned", False):
        if args.compiler_owned or not args.kernel_base or not args.kernel_bridge or not args.output:
            raise CppComError(
                "--kernel-owned requires --kernel-base, --kernel-bridge and -o; "
                "mutually exclusive with --compiler-owned"
            )
        reference = Path(args.kernel_bridge).with_name(
            Path(args.kernel_bridge).name.removesuffix("_bridge.h") + "_reference.c"
        )
        if not reference.is_file():
            raise CppComError(
                "kernel owner requires the compiler-validated reference beside its header"
            )
        manifest = validate_reference(reference, required=True)
        assert manifest is not None
        base_path = Path(args.kernel_base).resolve().relative_to(REACTOS_ROOT.resolve())
        if (
            (REACTOS_ROOT / manifest["source"]).resolve() != source.resolve()
            or (REACTOS_ROOT / manifest["header"]).resolve() != Path(args.kernel_bridge).resolve()
            or manifest["class"] != args.class_name
            or str(base_path) not in manifest["sha256"]
        ):
            raise CppComError("kernel owner inputs do not match the compiler-validated bridge")
        generated = generate_owner(
            source.read_text(),
            Path(args.kernel_base).read_text(),
            scan,
            args.class_name,
            Path(args.kernel_bridge).read_text(),
        )
        Path(args.output).write_text(generated)
        print(
            f"Rust kernel owner written to {args.output}; "
            "QI/factory/platform hooks must compile before activation"
        )
        return

    if getattr(args, "compiler_owned", False):
        write_bridge(source, scan, args)
        return

    object_map = Path(args.object_map) if args.object_map else source.parents[0] / "shell32.cpp"
    if not object_map.is_absolute():
        object_map = REACTOS_ROOT / object_map
    # Nested shell32 dirs register in the root shell32.cpp.
    if not object_map.is_file() and "shell32" in source.parts:
        shell_index = source.parts.index("shell32")
        object_map = Path(*source.parts[: shell_index + 1]) / "shell32.cpp"

    cls = analyze_class(
        source,
        args.class_name,
        object_map_source=object_map,
        activation_source=_resolve_activation_source(args),
        method_names=[f.name for f in class_methods],
        island_files=[SHELL32_ISLAND_FILES[a] for a in ("i386", "amd64", "arm64")],
    )
    cls.require_supported(allow_internal=args.internal, allow_plain=getattr(args, "plain", False))

    if args.sdk_header:
        sdk_header = Path(args.sdk_header).resolve()
        interfaces = [parse_widl_interface(sdk_header, name) for name in cls.interface_names()]
    else:
        interfaces = [
            parse_widl_interface(
                discover_widl_header(REACTOS_ROOT, args.arch, interface=name),
                name,
            )
            for name in cls.interface_names()
        ]

    implemented = {f.name for f in class_methods if not f.is_ctor and not f.is_dtor}
    required = {
        method.name
        for interface in interfaces
        for method in interface.methods
        if method.name not in ("QueryInterface", "AddRef", "Release")
    }
    missing = sorted(required - implemented)
    if missing:
        raise CppComError(
            f"{cls.name}: COM map requires method bodies the scanner did not extract: "
            + ", ".join(missing)
        )

    if args.json:
        payload = json.loads(cls.to_json())
        payload["sdk_headers"] = sorted({interface.source for interface in interfaces})
        payload["interfaces"] = {
            interface.name: [method.name for method in interface.methods]
            for interface in interfaces
        }
        print(json.dumps(payload, indent=2, sort_keys=True))
        return

    island_slots = island_lpvtbl_interfaces(
        [SHELL32_ISLAND_FILES[a] for a in ("i386", "amd64", "arm64")]
    )
    chunks = [
        render_interface(interface, island_slots.get(interface.name)) for interface in interfaces
    ]
    chunks.append(
        render_class_scaffold(
            cls,
            interfaces,
            allow_internal=args.internal,
            allow_plain=getattr(args, "plain", False),
            island_slots=island_slots,
            module_lock=_module_lock_of(args),
        )
    )
    output = "\n\n".join(chunks) + "\n"
    if args.output:
        out = Path(args.output)
        out.parent.mkdir(parents=True, exist_ok=True)
        out.write_text(output, encoding="utf-8")
        print(f"wrote deterministic scaffold: {out}")
    else:
        print(output, end="")
    if args.adapter_output:
        if args.internal:
            raise CppComError("--adapter-output is invalid for internal COM construction")
        adapter = Path(args.adapter_output)
        adapter.parent.mkdir(parents=True, exist_ok=True)
        adapter.write_text(
            render_object_map_adapter(cls, module_lock=_module_lock_of(args)) + "\n",
            encoding="utf-8",
        )
        print(f"wrote deterministic ATL adapter: {adapter}")


def cmd_semdiff(args):

    only = [s.strip() for s in args.only.split(",")] if args.only else None
    rc = semdiff.run(
        args.file,
        rust_file=args.rust,
        model=args.model,
        only=only,
        max_fns=args.max_fns,
        use_cache=not args.no_cache,
        workers=args.workers,
    )
    sys.exit(rc)


def cmd_resolve(args):

    path = resolve_c_path(args.file)
    target = resolve_target(args, path)

    print(f"Building catalog... (target={target.name})", file=sys.stderr)
    catalog = build_catalog(arch=args.arch, target=target)
    print_catalog_stats(catalog)
    print()

    spec = (
        parse_spec(target.spec_file)
        if target.spec_file and target.spec_file.exists()
        else parse_spec()
    )
    scan = scan_file(path)
    result = resolve(scan, catalog, spec)
    print_resolve_result(result)


def cmd_prompt(args):

    path = resolve_c_path(args.file)
    target = resolve_target(args, path)
    catalog = build_catalog(arch=args.arch, target=target)
    spec = (
        parse_spec(target.spec_file)
        if target.spec_file and target.spec_file.exists()
        else parse_spec()
    )
    scan = scan_file(path)
    res = resolve(scan, catalog, spec)

    if args.estimate:
        prompt = build_full_prompt(scan, res, target)
        tokens = estimate_tokens(prompt)
        print(f"Estimated tokens: ~{tokens:,}", file=sys.stderr)
    else:
        prompt = build_full_prompt(scan, res, target)
        print(prompt)


def cmd_submit(args):
    if not getattr(args, "unsafe_unprotected", False):
        raise ValueError(
            "submit bypasses per-function seals and anchors. Use translate, or explicitly "
            "pass --unsafe-unprotected for the legacy whole-file paid path."
        )
    path = resolve_c_path(args.file)
    target = resolve_target(args, path)
    catalog = build_catalog(arch=args.arch, target=target)
    spec = (
        parse_spec(target.spec_file)
        if target.spec_file and target.spec_file.exists()
        else parse_spec()
    )
    scan = scan_file(path)
    _gate_unextracted(scan, getattr(args, "allow_partial", False))
    res = resolve(scan, catalog, spec)

    model = resolve_model(args.model)

    # For Claude, use separate system prompt (for caching)
    # For Gemini, combine into single prompt
    if model.startswith("claude-"):
        system_prompt = load_system_prompt(target)
        file_prompt = build_file_prompt(scan, res, target)
    else:
        system_prompt = ""
        file_prompt = build_full_prompt(scan, res, target)

    # Durable archive + cost banner + kill guard for this one paid call.
    warn_if_tmp_output(getattr(args, "output", None))
    archive = RunArchive(label=path.name, model=model, target=target.name)
    install_kill_guard(archive)
    paid_invocation_banner(
        model, 1, estimate_tokens(file_prompt) + estimate_tokens(system_prompt), archive
    )
    sink = ResponseSink(archive.stream_path(0, path.stem))

    print(f"Submitting to {model}...", file=sys.stderr)
    try:
        result = submit_single(file_prompt, model=model, system_prompt=system_prompt, sink=sink)
    finally:
        sink.close()
    result.source_file = path.name
    if result.success:
        archive.commit(
            0,
            path.stem,
            result.rust_code,
            result.input_tokens,
            result.output_tokens,
            attempt_id=result.attempt_id,
        )
        print(
            f"Cost: ${cost_usd(model, result.input_tokens, result.output_tokens):.2f}  "
            f"| archived: {archive.dir}",
            file=sys.stderr,
        )

    if not result.success:
        print(f"Error: {result.error}", file=sys.stderr)
        sys.exit(1)

    print(f"Tokens: {result.input_tokens} in / {result.output_tokens} out", file=sys.stderr)

    # Determine subsystem from path
    subsystem = path.parent.name

    if args.output:
        # Post-process before writing
        code = postprocess(
            result.rust_code,
            exports=res.exports,
            globals=scan.globals,
            target_name=target.name,
            sys_crate=target.sys_crate,
            sys_umbrella=target.sys_umbrella,
            crate_module=getattr(target, "crate_module", "") if target.crate_bindgen_files else "",
        )
        Path(args.output).write_text(code)
        print(f"Written to {args.output}", file=sys.stderr)
    elif args.assemble:
        out_path = assemble_file(
            result,
            subsystem,
            scan.raw_source,
            exports=res.exports,
            scan=scan,
            target=target,
            c_source_path=path,
        )
        print(f"Assembled to {out_path}", file=sys.stderr)
    else:
        # Post-process before printing
        code = postprocess(
            result.rust_code,
            exports=res.exports,
            globals=scan.globals,
            target_name=target.name,
            sys_crate=target.sys_crate,
            sys_umbrella=target.sys_umbrella,
            c_source_path=path,
            com_method_macro=target.com_method_macro,
            concrete_stdcall_slots=target.concrete_stdcall_slots,
            crate_module=getattr(target, "crate_module", "") if target.crate_bindgen_files else "",
        )
        print(code)


def cmd_batch(args):
    if not getattr(args, "dry_run", False) and not getattr(args, "unsafe_unprotected", False):
        raise ValueError(
            "batch bypasses per-function seals and anchors. Use translate <directory>, or "
            "explicitly pass --unsafe-unprotected for the legacy whole-file paid path."
        )

    dir_path = Path(args.directory)
    if not dir_path.is_absolute():
        dir_path = NTOSKRNL_DIR / args.directory
    if not dir_path.is_dir():
        print(f"Error: not a directory: {dir_path}", file=sys.stderr)
        sys.exit(1)

    c_files = sorted([*dir_path.glob("*.c"), *dir_path.glob("*.cpp")])
    if args.exclude:
        exclude_stems = set(args.exclude)
        c_files = [f for f in c_files if f.stem not in exclude_stems]
    if not c_files:
        print(f"No .c files found in {dir_path}", file=sys.stderr)
        sys.exit(1)

    print(f"Found {len(c_files)} C files in {dir_path.name}/", file=sys.stderr)

    # Resolve the target from the path like every other command, instead of silently
    # defaulting to ntoskrnl: `batch` built its catalog and parsed its spec with no target
    # at all, so batching a user-mode DLL got the KERNEL catalog and the KERNEL spec. For
    # an ntoskrnl path this resolves to the same thing it always did (no behaviour change).
    target = resolve_target(args, dir_path)
    catalog = build_catalog(arch=args.arch, target=target)
    spec = parse_spec(target.spec_file) if target.spec_file else {}

    model = resolve_model(args.model)
    use_claude = model.startswith("claude-")

    system_prompt = load_system_prompt() if use_claude else ""

    # Build prompts
    prompts = {}
    scans = {}
    resolves = {}
    for cf in c_files:
        scan = scan_file(cf)
        _gate_unextracted(scan, getattr(args, "allow_partial", False))
        scans[cf.name] = scan
        res = resolve(scan, catalog, spec)
        resolves[cf.name] = res
        if use_claude:
            prompts[cf.name] = build_file_prompt(scan, res, target)
        else:
            prompts[cf.name] = build_full_prompt(scan, res, target)

    print(f"\nSubmitting {len(prompts)} files to {model}...", file=sys.stderr)
    results = submit_batch(prompts, model=model, system_prompt=system_prompt)

    # Assemble results
    subsystem = args.subsystem if args.subsystem else dir_path.name
    module_names = []
    for filename, result in results.items():
        if result.success:
            c_source = scans[filename].raw_source
            res = resolves[filename]
            assemble_file(result, subsystem, c_source, exports=res.exports, scan=scans[filename])
            module_names.append(Path(filename).stem)

    if module_names:
        generate_mod_rs(subsystem, module_names)
        # Only update lib.rs for top-level subsystems (not nested like ke/i386)
        if "/" not in subsystem:
            generate_lib_rs([subsystem])
        print(
            f"\nAssembled {len(module_names)} modules into ntoskrnl-rust/src/{subsystem}/",
            file=sys.stderr,
        )

    # Report failures
    failures = [f for f, r in results.items() if not r.success]
    if failures:
        print(f"\nFailed ({len(failures)}):", file=sys.stderr)
        for f in failures:
            print(f"  {f}: {results[f].error}", file=sys.stderr)


def cmd_check(args):

    results = check_all_targets()
    print_check_report(results)


def cmd_shims(args):

    subsystem = args.subsystem

    if args.check:
        matches, desc = check_shims(subsystem)
        if matches:
            print(f"OK: {subsystem}_rust_shims.c matches manifest", file=sys.stderr)
        else:
            print(f"MISMATCH: {desc}", file=sys.stderr)
            sys.exit(1)
    else:
        output = generate_shims(subsystem)
        if args.output:
            Path(args.output).write_text(output)
            print(f"Written to {args.output}", file=sys.stderr)
        elif args.write:
            out_path = NTOSKRNL_DIR / subsystem / f"{subsystem}_rust_shims.c"
            out_path.write_text(output)
            print(f"Written to {out_path}", file=sys.stderr)
        else:
            print(output)


def cmd_seh_classify(args):

    target = Path(args.path)
    if not target.is_absolute():
        target = NTOSKRNL_DIR / args.path

    if target.is_file():
        infos = classify_file(target)
    elif target.is_dir():
        infos = classify_directory(target)
    else:
        print(f"Error: not found: {target}", file=sys.stderr)
        sys.exit(1)

    if args.json:
        print(to_json(infos))
    else:
        print_classification(infos, verbose=args.verbose)


def cmd_seh_inventory(args):

    if args.oracle:
        winsta = REACTOS_ROOT / "win32ss/user/ntuser/winsta.c"
        sys.exit(seh_inventory.run_oracle(winsta))

    target = Path(args.path)
    if not target.is_absolute():
        target = REACTOS_ROOT / args.path

    if target.is_file():
        sites = seh_inventory.inventory_file(target)
        if args.write_sidecars:
            print(f"wrote {seh_inventory.write_sidecar(target, sites)}")
        if args.json:
            print(
                json.dumps(
                    {
                        "source": str(target),
                        "sites": [s.__dict__ | {"category": s.category.value} for s in sites],
                    },
                    indent=2,
                    default=str,
                )
            )
        else:
            for s in sites:
                sig = (" " + ",".join(s.signals)) if s.signals else ""
                print(
                    f"  ({s.category.value}) {s.function} "
                    f"L{s.line_start}-{s.line_end}{sig}"
                    f"{('  — ' + s.note) if s.note else ''}"
                )
    elif target.is_dir():
        inv = seh_inventory.inventory_keepset(target)
        if args.write_sidecars:
            for p, ss in inv.items():
                seh_inventory.write_sidecar(p, ss)
            print(f"wrote {len(inv)} sidecars")
        seh_inventory.print_summary(inv)
    else:
        print(f"Error: not found: {target}", file=sys.stderr)
        sys.exit(1)


def cmd_seh_rewrite(args):

    if args.oracle:
        sys.exit(seh_rewrite.run_oracle(REACTOS_ROOT / "win32ss/user/ntuser/winsta.c"))

    target = Path(args.path)
    if not target.is_absolute():
        target = REACTOS_ROOT / args.path

    def do_one(p: Path):
        orig = p.read_text(errors="replace")
        res = seh_rewrite.rewrite_file(p)
        if args.apply and res.rewritten:
            p.write_text(res.text)
        if not args.apply:
            diff = difflib.unified_diff(
                orig.splitlines(True), res.text.splitlines(True), f"a/{p.name}", f"b/{p.name}"
            )
            sys.stdout.writelines(diff)
        return res

    if target.is_file():
        res = do_one(target)
        print(
            f"\n{target.name}: rewritten={res.rewritten} skipped={len(res.skipped)}",
            file=sys.stderr,
        )
        for ln, why in res.skipped:
            print(f"  skip L{ln}: {why}", file=sys.stderr)
    elif target.is_dir():
        inv = seh_inventory.inventory_keepset(target)
        tot_rw = tot_sk = 0
        hist: dict[str, int] = collections.Counter()
        manifest = []
        for p in inv:
            r = do_one(p)
            tot_rw += r.rewritten
            tot_sk += len(r.skipped)
            for ln, why in r.skipped:
                hist[why] += 1
                manifest.append({"file": str(p.relative_to(target)), "line": ln, "reason": why})
        if args.apply:
            mpath = target / "win32k_pass1_manual_followup.json"
            mpath.write_text(
                json.dumps({"rewritten": tot_rw, "skipped": tot_sk, "sites": manifest}, indent=2)
            )
            print(f"manual-followup ledger: {mpath}", file=sys.stderr)
        print(
            f"\n{'APPLIED' if args.apply else 'DRY-RUN'}: "
            f"{tot_rw} sites rewritten, {tot_sk} skipped across "
            f"{len(inv)} files",
            file=sys.stderr,
        )
        print("skip reasons:", file=sys.stderr)
        for why, n in sorted(hist.items(), key=lambda x: -x[1]):
            print(f"  {n:>4}  {why}", file=sys.stderr)
    else:
        print(f"Error: not found: {target}", file=sys.stderr)
        sys.exit(1)


def cmd_seh_belt(args):
    """Pass 1c: delete (d) BELT_SUSPENDERS SEH wrappers (live re-classify)."""

    if args.oracle:
        sys.exit(seh_belt.run_oracle())

    target = Path(args.path)
    if not target.is_absolute():
        target = REACTOS_ROOT / args.path

    def do_one(p: Path):
        orig = p.read_text(errors="replace")
        res = seh_belt.rewrite_file(p)
        if args.apply and res.rewritten:
            p.write_text(res.text)
        if not args.apply and res.text != orig:
            diff = difflib.unified_diff(
                orig.splitlines(True), res.text.splitlines(True), f"a/{p.name}", f"b/{p.name}"
            )
            sys.stdout.writelines(diff)
        return res

    if target.is_file():
        res = do_one(target)
        print(
            f"\n{target.name}: rewritten={res.rewritten} skipped={len(res.skipped)}",
            file=sys.stderr,
        )
        for ln, why in res.skipped:
            print(f"  skip L{ln}: {why}", file=sys.stderr)
    elif target.is_dir():
        tot_rw = tot_sk = files_touched = 0
        skip_reasons: list[tuple[str, int, str]] = []
        for p in seh_belt.iter_keepset(target):
            r = do_one(p)
            if r.rewritten or r.skipped:
                if r.rewritten:
                    files_touched += 1
                tot_rw += r.rewritten
                tot_sk += len(r.skipped)
                for ln, why in r.skipped:
                    skip_reasons.append((str(p.relative_to(target)), ln, why))
        print(
            f"\n{'APPLIED' if args.apply else 'DRY-RUN'}: "
            f"{tot_rw} BELT_SUSPENDERS sites deleted across "
            f"{files_touched} files; {tot_sk} skipped",
            file=sys.stderr,
        )
        for f, ln, why in skip_reasons:
            print(f"  skip {f}:L{ln} — {why}", file=sys.stderr)
    else:
        print(f"Error: not found: {target}", file=sys.stderr)
        sys.exit(1)


def cmd_callout_annotate(args):

    if args.oracle:
        sys.exit(callout_annotate.run_oracle(REACTOS_ROOT / "win32ss/user/ntuser/callback.c"))

    target = Path(args.path)
    if not target.is_absolute():
        target = REACTOS_ROOT / args.path

    if target.is_file():
        sites = callout_annotate.extract_file(target)
        if args.write_sidecars:
            print(f"wrote {callout_annotate.write_sidecar(target, sites)}")
        if args.json:
            print(
                json.dumps(
                    {"source": str(target), "sites": [s.__dict__ for s in sites]},
                    indent=2,
                    default=str,
                )
            )
        else:
            for s in sites:
                sig = (" " + ",".join(s.signals)) if s.signals else ""
                print(
                    f"  {s.function} L{s.func_line} "
                    f"{s.callback_index} @KUMC L{s.kumc_line} "
                    f"[Leave L{s.leave_co_line} / Enter L{s.enter_co_line} "
                    f"/ probe L{s.result_probe_line}]{sig}"
                )
    elif target.is_dir():
        inv = callout_annotate.extract_keepset(target)
        if args.write_sidecars:
            for p, ss in inv.items():
                callout_annotate.write_sidecar(p, ss)
            print(f"wrote {len(inv)} sidecars")
        callout_annotate.print_summary(inv)
    else:
        print(f"Error: not found: {target}", file=sys.stderr)
        sys.exit(1)


def cmd_seh_llm_lower(args):

    if args.oracle:
        sys.exit(seh_llm_lower.run_oracle())

    units = seh_llm_lower.load_units(args.only)
    print(
        f"{len(units)} dedup'd functions "
        f"({sum(len(u.reasons) for u in units)} ledger sites) "
        f"{'[--only ' + args.only + ']' if args.only else ''}",
        file=sys.stderr,
    )
    outcomes = seh_llm_lower.run(apply=args.apply, model=args.model, only=args.only, units=units)
    if args.diff:
        for o in outcomes:
            if o.applied and o.diff:
                sys.stdout.write(o.diff)
    sys.exit(seh_llm_lower.print_report(outcomes, args.apply))


def cmd_split_plan(args):

    target = Path(args.path)
    if not target.is_absolute():
        target = REACTOS_ROOT / args.path

    if target.is_file():
        plans = [file_splitter.analyze_file(target, args.target_loc)]
    else:
        plans = file_splitter.analyze_scope(target, args.min_loc, args.target_loc)

    if args.json:
        print(
            json.dumps(
                [
                    {
                        "file": str(p.path),
                        "loc": p.loc,
                        "nfuncs": p.nfuncs,
                        "classification": p.classification,
                        "groups": p.groups,
                        "blocking": p.blocking,
                        "note": p.note,
                    }
                    for p in plans
                ],
                indent=2,
                default=str,
            )
        )
    else:
        file_splitter.print_report(plans)


def cmd_switch_extract(args):

    target = Path(args.path)
    if not target.is_absolute():
        target = REACTOS_ROOT / args.path
    if args.oracle:
        sys.exit(switch_extract.run_oracle(target, args.function))

    r = switch_extract.analyze(target, args.function)
    print(f"{target.as_posix().split('/win32ss/')[-1]}:{args.function} — {r.note}", file=sys.stderr)
    for g in r.groups:
        tag = ("SAFE→" + g.tag) if g.safe else "inline"
        print(
            f"  [{tag}] {'/'.join(g.labels[:3])}{'…' if len(g.labels) > 3 else ''}  {g.reason}",
            file=sys.stderr,
        )
    if args.apply and r.new_text:
        target.write_text(r.new_text)
        print(f"APPLIED → {target}", file=sys.stderr)
    elif not r.new_text:
        print("nothing applied (no safe arms / oracle fail)", file=sys.stderr)


def cmd_split_apply(args):

    targets: list[Path] = []
    if args.path:
        t = Path(args.path)
        if not t.is_absolute():
            t = REACTOS_ROOT / args.path
        targets = [t]
    else:
        for pl in file_splitter.analyze_scope(
            REACTOS_ROOT / "win32ss", args.min_loc, args.target_loc
        ):
            if pl.classification == "CLEAN_SPLIT":
                targets.append(pl.path)

    for t in targets:
        em = file_splitter.plan_emit(t, args.target_loc)
        rel = t.as_posix().split("/win32ss/")[-1]
        if not em.fragments:
            print(f"  skip {rel}: {em.note}")
            continue
        names = ", ".join(p.name for p, _ in em.fragments)
        print(
            f"  {'APPLY' if args.apply else 'plan'} {rel} → "
            f"{rel.rsplit('/', 1)[-1]} + [{names}]  ({em.note})"
        )
        if args.apply:
            file_splitter.apply_emit(em)


def cmd_gdi_locks(args):

    if args.oracle:
        sys.exit(gdi_lock_inventory.run_oracle(REACTOS_ROOT / "win32ss/gdi/ntgdi/dcutil.c"))

    target = Path(args.path)
    if not target.is_absolute():
        target = REACTOS_ROOT / args.path

    if target.is_file():
        sites = gdi_lock_inventory.extract_file(target)
        if args.write_sidecars:
            print(f"wrote {gdi_lock_inventory.write_sidecar(target, sites)}")
        if args.json:
            print(
                json.dumps(
                    {"source": str(target), "sites": [s.__dict__ for s in sites]},
                    indent=2,
                    default=str,
                )
            )
        else:
            for s in sites:
                sig = (" " + ",".join(s.signals)) if s.signals else ""
                print(
                    f"  {s.function} L{s.func_line} "
                    f"{s.lock_fn}→{s.unlock_fn} {s.var}@L{s.lock_line} "
                    f"[{s.shape}] unlock@{s.unlock_lines}{sig}"
                )
    elif target.is_dir():
        inv = gdi_lock_inventory.extract_keepset(target)
        if args.write_sidecars:
            for p, ss in inv.items():
                gdi_lock_inventory.write_sidecar(p, ss)
            print(f"wrote {len(inv)} sidecars")
        gdi_lock_inventory.print_summary(inv)
    else:
        print(f"Error: not found: {target}", file=sys.stderr)
        sys.exit(1)


def cmd_user_locks(args):

    if args.oracle:
        sys.exit(user_lock_inventory.run_oracle(REACTOS_ROOT / "win32ss/user/ntuser/simplecall.c"))

    target = Path(args.path)
    if not target.is_absolute():
        target = REACTOS_ROOT / args.path

    if target.is_file():
        sites = user_lock_inventory.extract_file(target)
        if args.write_sidecars:
            print(f"wrote {user_lock_inventory.write_sidecar(target, sites)}")
        if args.json:
            print(
                json.dumps(
                    {"source": str(target), "sites": [s.__dict__ for s in sites]},
                    indent=2,
                    default=str,
                )
            )
        else:
            for s in sites:
                sig = (" " + ",".join(s.signals)) if s.signals else ""
                print(
                    f"  {s.function} L{s.func_line} {s.mode} "
                    f"enter@L{s.enter_line}"
                    f"{'(top)' if s.enter_at_top else ''} "
                    f"[{s.shape}] leave@{s.leave_lines}{sig}"
                )
    elif target.is_dir():
        inv = user_lock_inventory.extract_keepset(target)
        if args.write_sidecars:
            for p, ss in inv.items():
                user_lock_inventory.write_sidecar(p, ss)
            print(f"wrote {len(inv)} sidecars")
        user_lock_inventory.print_summary(inv)
    else:
        print(f"Error: not found: {target}", file=sys.stderr)
        sys.exit(1)


def cmd_probe_migrate(args):

    # Resolve paths
    target = Path(args.path)
    if not target.is_absolute():
        target = OUTPUT_CRATE / "src" / args.path

    if target.is_file():
        rust_files = [target]
    elif target.is_dir():
        # Find all .rs files with probe calls or SEH-affected code
        rust_files = []
        for rs_file in sorted(target.glob("*.rs")):
            if rs_file.name == "mod.rs":
                continue
            scan = scan_rust_file(rs_file)
            if scan.probe_call_count > 0 or scan.seh_context:
                rust_files.append(rs_file)
    else:
        print(f"Error: not found: {target}", file=sys.stderr)
        sys.exit(1)

    if not rust_files:
        print("No files needing migration found.", file=sys.stderr)
        sys.exit(0)

    print(f"Found {len(rust_files)} files needing migration:", file=sys.stderr)
    for f in rust_files:
        scan = scan_rust_file(f)
        parts = []
        if scan.probe_call_count > 0:
            parts.append(f"{scan.probe_call_count} probes")
        if scan.seh_context:
            seh_fns = scan.seh_context.count("Function: ")
            parts.append(f"{seh_fns} SEH fns")
        print(f"  {f.name}: {', '.join(parts)}", file=sys.stderr)

    if args.scan_only:
        return

    model = resolve_model(args.model)

    if args.prompt:
        # Just print the prompt for the first file
        system_prompt = load_probe_system_prompt()
        scan = scan_rust_file(rust_files[0])
        file_prompt = build_probe_file_prompt(scan)
        print(system_prompt + "\n\n---\n\n" + file_prompt)
        return

    print(f"\nSubmitting to {model}...", file=sys.stderr)
    results = run_probe_migration(
        rust_files,
        model=model,
        dry_run=args.dry_run,
    )

    # Summary
    ok = sum(1 for v in results.values() if v)
    fail = sum(1 for v in results.values() if not v)
    print(f"\nDone: {ok} succeeded, {fail} failed", file=sys.stderr)

    if not args.dry_run and ok > 0:
        print("\nRun `python -m rustconv check` to verify compilation.", file=sys.stderr)


def _module_pinned_macros(c_file: Path, args) -> dict[str, int]:
    """`-DNAME=<int>` the build puts on EVERY compile of `c_file`, from its CMakeLists.

    Feeds `never_compiled_lines`, so it decides what is on the bill: msvcrt pins
    `_MSVCR_VER=0` and ~200 `#if _MSVCR_VER >= 80/100/110/120/140` bodies across the module
    are dead at that value. Getting it WRONG in the permissive direction costs a paid call;
    wrong in the other direction silently drops a live body. So the rule is agreement:
    a name survives only if every target that compiles this TU, on every arch, pins it to
    the same value. A macro pinned only under `if(ARCH STREQUAL ...)`, or only in one of the
    three targets an msvcrt TU belongs to, decides nothing and the bodies stay live.

    Best-effort by design — an unparseable or absent CMakeLists yields `{}`, which is
    exactly the pre-existing behaviour (more paid calls, never a dropped body).
    """
    cmake = c_file.resolve().parent / "CMakeLists.txt"
    # A TU can live one directory down from the CMakeLists that lists it
    # (msvcrt's `reactos/misc.c`), so walk up until one turns up.
    for _ in range(3):
        if cmake.is_file():
            break
        cmake = cmake.parent.parent / "CMakeLists.txt"
    if not cmake.is_file():
        return {}

    repo_root = REACTOS_ROOT
    wanted = c_file.resolve()
    agreed: dict[str, int] | None = None
    for arch in ("i386", "amd64", "arm64"):
        try:
            names = sorted(_target_names(cmake))
        except ProfileDeriveError:
            return {}
        for name in names:
            if getattr(args, "target", None) and name != args.target:
                continue
            try:
                module = parse_cmakelists(cmake, repo_root, arch=arch, target=name)
            except ProfileDeriveError:
                continue
            if wanted not in {p.resolve() for p in module.compiled_tus}:
                continue
            here = pinned_int_macros(module.definitions)
            if agreed is None:
                agreed = here
            else:
                agreed = {k: v for k, v in agreed.items() if here.get(k) == v}
    return agreed or {}


def skip_fn_for_file(requested: Iterable[str], present: Iterable[str]) -> list[str]:
    """Which ``--skip-fn`` names apply to ONE file: the ones that file actually declares.

    A function name belongs to exactly one translation unit, so on a directory run every OTHER
    file legitimately lacks the name. Treating that as a per-file error — and skipping the file —
    is how asking to drop one BODY silently dropped eleven FILES: unbought, unassembled, and
    reported only as one stderr line among eleven identical ones. The name that matched NOTHING
    across the whole worklist is the real error, and only the caller of the loop can know it;
    see ``unmatched_skip_fn``.
    """
    return sorted(set(requested) & set(present))


def unmatched_skip_fn(requested: Iterable[str], matched: Iterable[str]) -> list[str]:
    """``--skip-fn`` names no file in the worklist carried — i.e. misspelled.

    Loud on purpose and in the AUDIT report rather than a log line: the caller believes a body
    was left to them to hand-write, and the truth is the opposite — it was translated and
    SHIPPED. That inverts the whole guarantee of the flag.
    """
    return sorted(set(requested) - set(matched))


def _already_translated(
    c_file: Path, output_crate: Path, c_names: set[str]
) -> tuple[list[Path], int, int] | None:
    """Are this TU's exported functions ALREADY served from Rust in the output crate?

    This guard exists because a lane doc said "Owed: a Rust port of file.c" long after
    file.c had been ported, wired and bug-fixed on top — and $2.06 of DeepSeek was spent
    re-buying all 232 bodies before anyone looked at the crate. A stale "owed" line is
    not evidence; the crate is. Nothing else catches it: `estimate` reads the resume
    cache, which is keyed by seal-flag namespace, so an earlier translation bought under
    different flags reports "0 banked" and looks like virgin work.

    Matched by SYMBOL, not by filename. The obvious `<stem>.rs` check looks right and
    silently misses the renames that integration actually does — msvcrt's wcs.c ships as
    `wcs_rest.rs`, and a TU split across two modules matches neither. Symbols are what
    the link resolves and what a second translation would collide with.

    `#[no_mangle]`/`#[export_name]` is the operative property: those are the definitions
    that win the link and displace the C. A module full of ordinary `fn`s is internal
    scaffolding and does not block.

    Same intent as `--reassemble`, which has always refused to OVERWRITE a committed
    .rs — but that refusal fires at assembly, after every function has been bought. This
    one fires before the first API call, and `--reassemble` waives both.
    """
    src = output_crate / "src"
    if not src.is_dir() or not c_names:
        return None
    served = exported_symbols(src)
    hits = sorted(n for n in c_names if n in served)
    if not hits:
        return None
    # A handful of shared helpers overlapping is normal; a majority means the TU is
    # already ported. Below the threshold we say nothing and let the run proceed.
    if len(hits) * 2 < len(c_names):
        return None
    where = sorted({served[n] for n in hits})
    return where, len(hits), len(c_names)


def _unmatched_fn_names(fn_arg: str, found: set[str]) -> set[str]:
    """`--fn` names (comma-separated) that matched in NO translated file."""
    return {n.strip() for n in fn_arg.split(",") if n.strip()} - found


def _class_declaration_scope(cls) -> str:
    """The text that selects a class's support closure: its HEADER — unless the class is
    declared inside the translation unit itself, in which case only its own declaration.

    Using the whole .cpp for an in-TU class over-selects: shell32.cpp's IDefClFImpl pulled in
    extern bridges for three helpers named only by the Shell32Rust* trampolines elsewhere in that
    1,200-line file — one of which then collided (E0255) with the crate's own Rust definition
    the import resolver found for the same name."""
    text = cls.header.read_text(encoding="utf-8", errors="replace")
    if cls.header.resolve() != cls.source.resolve():
        return text
    m = re.search(r"\bclass\s+" + re.escape(cls.name) + r"\b[^;{]*\{", text)
    if not m:
        return text
    depth, i = 0, m.end() - 1
    while i < len(text):
        depth += text[i] == "{"
        depth -= text[i] == "}"
        i += 1
        if depth == 0:
            break
    return text[m.start() : i]


def cmd_translate(args):
    """Function-by-function translation pipeline."""
    # `--fn` names that matched in SOME file. A name missing from one file of a multi-file run is
    # normal; a name that matched NOWHERE is a failed request, and used to end with rc=0 after an
    # "Error:" line — which a script checking the exit code reads as success. (It was hit when a
    # `#if 0` had hidden the bodies from the scanner: nothing was written, and nothing said so.)
    fn_names_found: set[str] = set()

    # The CLI contract says --prompt is inspection-only. Enforce it here before
    # creating a paid archive or resolving a backend so a prompt preview can
    # never drift into an API call.
    if getattr(args, "prompt", False):
        args.dry_run = True

    # Resolve target: single file or directory
    target_path = Path(args.path)
    if not target_path.is_absolute():
        # Try repo-root-relative, ntoskrnl-relative, hal-relative.

        candidates = [
            REACTOS_ROOT / args.path,
            NTOSKRNL_DIR / args.path,
            HAL_DIR / args.path,
        ]
        target_path = next((c for c in candidates if c.exists()), candidates[0])

    # Detect target (ntoskrnl vs hal) from the path
    rustconv_target = resolve_target(args, target_path)
    print(f"Target: {rustconv_target.name}", file=sys.stderr)

    # Sealing dialect. shell32/--cpp-com keeps the FROZEN "cpp" dialect so its cached
    # paid workers stay valid; everything else is plain C and gets the "c" dialect,
    # which additionally protects CamelCase DDK enum constants (NonPagedPool,
    # KernelMode, Executive, the Ndis* vocabulary) that the ALL_CAPS matcher misses.
    # Sealing used to be gated to shell32 --cpp-com purely out of caution; the
    # transform is lexical and language-neutral. Leaving it off for C shipped 13,503
    # unprotected constant occurrences, the same class as the 61 invented kernel
    # bugcheck codes (field manual §9.113).
    #
    # The dialect follows the SOURCE LANGUAGE, not the flag: a C++ TU translated via
    # --free-functions is still C++, and handing it the C dialect would seal it under
    # rules written for a different grammar.
    free_functions = bool(getattr(args, "free_functions", False))
    is_cpp_source = target_path.suffix.lower() in (".cpp", ".cc", ".cxx", ".c++")
    seal_dialect = "cpp" if (getattr(args, "cpp_com", False) or is_cpp_source) else "c"

    # Every COM-machinery branch below keys off THIS, not off the target name: the
    # deterministic scaffold, the `<name>_body` worker audit, the interface renderer
    # and the abi_enforce opt-out all exist to serve a translated COM class. A
    # --free-functions run has no class, so it takes the ordinary per-function path
    # that every other user-mode DLL uses.
    # Does this run translate a C++ CLASS? shell32 always does — its whole remaining tail is
    # C++ — and any other module does when the caller asks for one. This used to read
    # `rustconv_target.name == "shell32"`, which made the class path unreachable everywhere
    # else: `cpp_com_mode` was False for e.g. shellrecyclebin, and with it went the class
    # scaffold, the interface rendering and EVERY audit_worker_body call. Nothing said so.
    # The module still produced bodies, so it looked like it had worked.
    cpp_class_target = (
        is_cpp_source
        or rustconv_target.name == "shell32"
        or bool(getattr(args, "cpp_com", False) or free_functions)
    )
    # `--plain` is a MODIFIER of the class path (it drops the three COM preflight requirements
    # for a file-local helper), not a mode of its own. Alone it silently fell through to the
    # whole-TU path, which is how four hand-rolled COM classes got translated with no vtable
    # instance, no _new/_delete and no lpVtbl wiring — a wild jump on the first method call.
    if getattr(args, "plain", False) and not getattr(args, "cpp_com", False):
        raise CppComError(
            "--plain modifies the C++ class path; it is not a mode. Pass --cpp-com --class "
            "<Name> with it for a file-local helper class, or drop it."
        )
    cpp_com_mode = cpp_class_target and not free_functions

    cpp_com_cls = None
    cpp_com_interfaces = []
    if cpp_class_target and free_functions:
        if getattr(args, "cpp_com", False):
            raise CppComError(
                "--free-functions and --cpp-com are mutually exclusive: one translates the "
                "TU's classless functions, the other one COM class after ABI preflight."
            )
        if target_path.is_dir():
            raise CppComError(
                "--free-functions accepts one C++ implementation file, not a directory"
            )
    elif cpp_class_target:
        if not getattr(args, "cpp_com", False) or not getattr(args, "class_name", None):
            raise CppComError(
                "C++ translation is class-atomic: pass --cpp-com --class <Name> "
                "for a COM class, or --free-functions for the TU's classless functions. "
                "The paid class path is disabled until deterministic COM preflight succeeds."
            )
        if target_path.is_dir():
            raise CppComError("--cpp-com accepts one C++ implementation file, not a directory")
        if getattr(args, "c2rust", False):
            raise CppComError(
                "c2rust does not provide a C++/ATL behavior anchor. Refusing --cpp-com "
                "--c2rust rather than silently running unanchored."
            )
        # Analyze first: scan-extend needs the flattened local-base list to merge and
        # re-own base method bodies (concrete-local-base inherited-layout mapping).
        cpp_com_cls = analyze_class(
            target_path,
            args.class_name,
            object_map_source=rustconv_target.com_object_map,
            activation_source=_resolve_activation_source(args),
            island_files=list(rustconv_target.com_island_files.values()),
        )
        scan = _extend_cpp_class_scan(
            scan_file(target_path),
            target_path,
            args.class_name,
            local_bases=tuple(cpp_com_cls.local_base_classes),
        )
        cpp_com_cls.base_dtor_workers = [
            f.name
            for f in scan.functions
            if f.cpp_class == args.class_name and f.name.endswith("_base_dtor")
        ]
        if scan.unextracted:
            raise CppComError(
                f"{target_path.name}: {len(scan.unextracted)} body/bodies remain unextractable; "
                "fix scanner coverage before any paid class translation"
            )
        cpp_com_cls.method_names.extend(
            f.name for f in scan.functions if f.cpp_class == args.class_name
        )
        cpp_com_cls.require_supported(
            allow_internal=getattr(args, "internal", False),
            allow_plain=getattr(args, "plain", False),
        )
        if getattr(args, "sdk_header", None):
            sdk_header = Path(args.sdk_header).resolve()
            cpp_com_interfaces = [
                parse_widl_interface(sdk_header, name) for name in cpp_com_cls.interface_names()
            ]
        else:
            cpp_com_interfaces = [
                parse_widl_interface(
                    discover_widl_header(
                        REACTOS_ROOT,
                        args.arch,
                        interface=name,
                        extra_roots=[target_path.parent],
                    ),
                    name,
                )
                for name in cpp_com_cls.interface_names()
            ]
        # Parameter interfaces are ABI too.  Expand the WIDL closure
        # recursively so callback/service vtables referenced by a COM-map
        # method are generated from SDK authority instead of guessed by the
        # body model (for example IEmptyVolumeCacheCallBack).
        class_interface_names = set(cpp_com_cls.interface_names())
        known_interfaces = {item.name for item in cpp_com_interfaces}
        queue = list(cpp_com_interfaces)
        while queue:
            interface = queue.pop(0)
            referenced = {
                name
                for method in interface.methods
                for c_type, _param in method.params
                for name in re.findall(r"\b(I[A-Z][A-Za-z0-9_]*)\s*\*", c_type)
            }
            for name in sorted(referenced - known_interfaces):
                try:
                    header = discover_widl_header(
                        REACTOS_ROOT, args.arch, interface=name, extra_roots=[target_path.parent]
                    )
                    dependency = parse_widl_interface(header, name)
                except CppComError:
                    # Not every I-prefixed pointer is a COM interface
                    # (ITEMIDLIST is the common shell counterexample).
                    continue
                cpp_com_interfaces.append(dependency)
                known_interfaces.add(name)
                queue.append(dependency)
        implemented = {
            f.name
            for f in scan.functions
            if f.cpp_class == args.class_name and not f.is_ctor and not f.is_dtor
        }
        required = {
            method.name
            for interface in cpp_com_interfaces
            if interface.name in class_interface_names
            for method in interface.methods
            if method.name not in ("QueryInterface", "AddRef", "Release")
        }
        missing = sorted(required - implemented)
        if missing:
            raise CppComError(
                f"{args.class_name}: deterministic vtables require missing bodies: "
                + ", ".join(missing)
            )
        print(
            f"  [cpp-com] preflight: {len(cpp_com_cls.com_map)} interface(s), "
            # Only the class's OWN interfaces: `cpp_com_interfaces` also holds every interface
            # the TU merely mentions (IShellItem as a parameter type, ...), and summing those
            # printed 84 slots for IShellLibrary's 20 — a number that reads like a broken
            # scaffold and sends you auditing one that is fine.
            f"{sum(len(i.methods) for i in cpp_com_interfaces if i.name in class_interface_names)}"
            " vtable slot(s), "
            f"{len(implemented)} method body/bodies",
            file=sys.stderr,
        )

    if target_path.is_file():
        c_files = [target_path]
    elif target_path.is_dir():
        c_files = sorted([*target_path.glob("*.c"), *target_path.glob("*.cpp")])
        if args.exclude:
            exclude_stems = set(args.exclude)
            c_files = [f for f in c_files if f.stem not in exclude_stems]
    else:
        print(f"Error: not found: {target_path}", file=sys.stderr)
        sys.exit(1)

    if not c_files:
        print(f"No .c files found in {target_path}", file=sys.stderr)
        sys.exit(1)

    # HAL doesn't currently support arm64 catalog/translation.
    if rustconv_target.name == "hal" and args.arch not in rustconv_target.arches:
        print(
            f"HAL target does not support arch={args.arch}; supported: {rustconv_target.arches}",
            file=sys.stderr,
        )
        sys.exit(1)

    print("Building catalog...", file=sys.stderr)
    catalog = build_catalog(arch=args.arch, target=rustconv_target)
    # HAL has no .spec file — exports come from C source headers + Cargo no_mangle.
    if rustconv_target.spec_file is not None:
        spec = parse_spec(rustconv_target.spec_file)
    else:
        spec = {}
    catalog_fn_names = set(catalog.functions.keys())

    model = resolve_model(args.model)
    mode = getattr(args, "mode", "idiomatic")
    # `use_claude` here means "this backend takes a separate system prompt"
    # (Claude cached block / DeepSeek system message). Gemini gets the rules
    # prepended inline (unchanged path).
    use_claude = model.startswith("claude-") or model.startswith("deepseek-")
    system_prompt = (
        load_function_system_prompt(
            rustconv_target,
            mode=mode,
            cpp_com=cpp_com_mode,
            cpp_source=is_cpp_source and not cpp_com_mode,
        )
        if use_claude
        else ""
    )

    # Concurrency: fan out per-function calls. ONLY for mechanical mode on DeepSeek —
    # mechanical functions are independent (each forward-declares its callees), and
    # DeepSeek is cheap enough that a bounded fan-out won't black-hole the budget the
    # way Claude could. Claude/Gemini/idiomatic stay strictly sequential by policy.
    conc = max(1, getattr(args, "concurrency", 1) or 1)
    if conc > 1 and not (mode == "mechanical" and model.startswith("deepseek-")):
        print(
            f"  (--concurrency {conc} ignored: only for --mode mechanical on DeepSeek)",
            file=sys.stderr,
        )
        conc = 1

    # Durable archive + kill guard for this paid invocation. Every streamed
    # response lands here as it generates, and a Ctrl-C points back at the
    # preserved work instead of feeling like it threw away what you paid for.
    archive = None
    outcome_ledger = None
    running_cost = 0.0
    # Hard safety brake: stop before the next paid call once cumulative spend
    # crosses this. None = no cap. Prevents a runaway bill.
    cost_cap = getattr(args, "cost_cap", None)
    no_resume = getattr(args, "no_resume", False)

    # --local-raii NAME=PATH: hand-verified Rust lowerings for file-local C++ RAII
    # structs. Parsed and read up front so a bad path fails before any paid call.
    local_raii_impls: dict[str, str] = {}
    # NAME -> module path, for lowerings that live inside the output crate already.
    local_raii_uses: dict[str, str] = {}
    for spec_arg in getattr(args, "local_raii", []) or []:
        if "=" not in spec_arg:
            raise CppComError(f"--local-raii expects NAME=PATH, got: {spec_arg}")
        raii_name, _, raii_path = spec_arg.partition("=")
        raii_name = raii_name.strip()
        raii_file = Path(raii_path.strip()).expanduser()
        # Both C++ paths hit body-local RAII and both run require_supported_worker_raii,
        # so both need the escape hatch. Gating it to --cpp-com made the free-function
        # path a dead end: `provided` was always empty there, so the refusal message
        # named a flag that the same invocation would then reject (shlfolder.cpp /
        # shldataobject.cpp / item_prop.cpp all stalled on CDataObjectHIDA this way).
        # ...and --plain is the third such path, so it belongs here too. The gate is now on the
        # MODE alone: body-local RAII is a C++ language feature, not a shell32 feature, and the
        # old `target.name == "shell32"` clause hard-coded the only module that happened to have
        # C++ TUs when this was written. shellrecyclebin (its own CMake target, so target.name
        # == "recyclebin") hit exactly the dead end the comment above describes — refused for
        # naming a flag it was already passing — on CZZWStr, a file-local RAII guard owning an
        # SHAlloc'd double-NUL string for SHFILEOPSTRUCTW.
        cpp_mode = (
            getattr(args, "cpp_com", False)
            or getattr(args, "free_functions", False)
            or getattr(args, "plain", False)
        )
        if not cpp_mode:
            raise CppComError(
                "--local-raii requires a C++ mode: --cpp-com, --plain or --free-functions"
            )
        if not raii_file.is_file():
            raise CppComError(f"--local-raii {raii_name}: no such file: {raii_file}")
        local_raii_impls[raii_name] = raii_file.read_text()
        # SEAL from the file, but SPLICE only when the lowering is genuinely file-local.
        # The splice is verbatim, so pointing several runs at one SHARED support module
        # copies it into each output: N incompatible `CStringW` types, and N copies of a
        # `//!` module doc that is only legal at the top of a file. (Five recyclebin
        # classes did exactly that -- 863 errors, ~600 of them this one root.) When the
        # lowering already lives inside the output crate it is a real module, so import
        # it instead; the worker contract is unaffected either way.
        crate_src = (rustconv_target.output_crate / "src").resolve()
        try:
            relative = raii_file.resolve().relative_to(crate_src)
        except ValueError:
            continue
        module = "::".join(("crate", *relative.with_suffix("").parts))
        local_raii_uses[raii_name] = module
    if not args.dry_run:
        warn_if_tmp_output(getattr(args, "output", None))
        archive = RunArchive(label=target_path.name, model=model, target=rustconv_target.name)
        install_kill_guard(archive)
        outcome_ledger = OutcomeLedger(
            archive.dir.name,
            {
                "model": model,
                "target": rustconv_target.name,
                "arch": args.arch,
                "mode": mode,
                "seal_facts": bool(getattr(args, "seal_facts", False)),
                "seal_calls": bool(getattr(args, "seal_calls", False)),
                "seal_wide": bool(getattr(args, "seal_wide", False)),
                "cache_tag": getattr(args, "cache_tag", None),
            },
        )

    total_files = len(c_files)
    audit_findings: dict[str, list[str]] = {}
    #: `--skip-fn` names, and the ones some file actually carried. A name belongs to exactly one
    #: TU, so on a DIRECTORY run every OTHER file legitimately does not have it — see the
    #: per-file handling for why that must not be an error there, and the post-loop check for
    #: where a typo is still caught.
    skip_fn_requested = {
        name.strip() for name in (getattr(args, "skip_fn", None) or []) if name.strip()
    }
    skip_fn_matched: set[str] = set()
    rebuy_fn_requested = {
        name.strip() for name in (getattr(args, "rebuy_fn", None) or []) if name.strip()
    }
    rebuy_fn_matched: set[str] = set()

    def _rebuy_requested(name: str) -> bool:
        if name in rebuy_fn_requested:
            rebuy_fn_matched.add(name)
            return True
        return False

    # Decided ONCE, not per file, so the refusal below is stated once and not twenty times.
    # Never on the COM path: shell32's C++ literals are already lowered deterministically by
    # `rewrite_wide_literals`, and its prompts are banked under exactly that behaviour.
    wide_seal_enabled = bool(getattr(args, "seal_wide", False))
    if wide_seal_enabled and (cpp_com_mode or free_functions):
        print(
            '  --seal-wide IGNORED: the shell32 C++ path lowers L"..." deterministically '
            "before the model sees the body (cpp_com.rewrite_wide_literals), and its prompts "
            "are banked under that. Nothing is unprotected by this.",
            file=sys.stderr,
        )
        wide_seal_enabled = False
    for file_idx, c_file in enumerate(c_files, 1):
        print(f"\n{'=' * 60}", file=sys.stderr)
        print(f"[{file_idx}/{total_files}] {c_file.name}", file=sys.stderr)
        print(f"{'=' * 60}", file=sys.stderr)

        capped = False  # set if --cost-cap is hit mid-file
        scan = scan_file(c_file)
        if c_file.suffix.lower() == ".c":
            validate_reference(c_file)
        if c_file.suffix.lower() == ".c" and not getattr(args, "deterministic_only", False):
            derived = profile_derive.derive_source(
                c_file,
                REACTOS_ROOT,
                arch=args.arch,
                preferred_target=rustconv_target.name,
                usermode=rustconv_target.sys_crate != "ntkrnl_sys",
            )
            # Scope the gate to the file actually being translated. `derive_source`
            # is MODULE-scoped, so an ERROR about a different TU used to block this
            # one: msvcrt's handler4.c and concurrency.c are permanently ~98% dead,
            # their DEAD_TU findings are correct and say "DO NOT PAY TO TRANSLATE
            # IT" about THEMSELVES, and with no waiver flag they made every other
            # file in msvcrt unpayable too.
            # Same spelling profile_derive uses for a Finding's `subject`, so the
            # two strings compare equal.
            rel_c = profile_derive._rel(c_file, REACTOS_ROOT)
            if derived.blocking_errors(rel_c):
                raise ValueError(profile_derive.emit_report(derived))
            # Not blocking is not the same as not true — say them anyway.
            for finding in derived.deferred_errors(rel_c):
                print(
                    f"  [substrate] NOT BLOCKING {c_file.name} (it is about another "
                    f"TU), but still true: {finding.code}: {finding.detail}",
                    file=sys.stderr,
                )
            print(f"  [substrate] {derived.module.name}: profile audit passed", file=sys.stderr)
        if c_file.suffix.lower() == ".c" and not getattr(args, "reassemble", False):
            shipping = _already_translated(
                c_file,
                rustconv_target.output_crate,
                {f.name for f in scan.functions},
            )
            if shipping is not None:
                where, hits, total = shipping
                listed = "\n      ".join(str(w.relative_to(REACTOS_ROOT)) for w in where[:6])
                raise ValueError(
                    f"{c_file.name} is ALREADY TRANSLATED and shipping: {hits} of its "
                    f"{total} functions are already defined as #[no_mangle] in "
                    f"{rustconv_target.output_crate.name}:\n      {listed}\n"
                    "    Re-translating re-pays for work that exists, and the shipping "
                    "module has had fixes landed on top of the original output that a "
                    "fresh translation does not contain.\n"
                    "    AGENTS.md: never re-translate a file that already has output "
                    "without explicit approval. --reassemble already means exactly that "
                    "and already gates the WRITE; this is the same gate moved ahead of "
                    "the MONEY."
                )
        if cpp_com_mode:
            scan = _extend_cpp_class_scan(
                scan,
                c_file,
                args.class_name,
                local_bases=tuple(cpp_com_cls.local_base_classes) if cpp_com_cls else (),
            )
        try:
            source_file_label = str(c_file.relative_to(REACTOS_ROOT))
        except ValueError:
            source_file_label = str(c_file)

        def _record_outcome(
            dep,
            outcome: str,
            *,
            outcome_ledger=outcome_ledger,
            source_file_label=source_file_label,
            **fields,
        ):
            if outcome_ledger is None:
                return
            outcome_ledger.record(
                unit=dep.name,
                source=dep.body_text,
                outcome=outcome,
                source_file=source_file_label,
                cpp_class=dep.cpp_class,
                **fields,
            )

        def _record_preamble(
            source: str,
            outcome: str,
            *,
            outcome_ledger=outcome_ledger,
            source_file_label=source_file_label,
            **fields,
        ):
            if outcome_ledger is not None:
                outcome_ledger.record(
                    unit=PREAMBLE_KEY,
                    source=source,
                    outcome=outcome,
                    source_file=source_file_label,
                    cpp_class=None,
                    **fields,
                )

        def _result_fields(result, elapsed_ms: int, model=model) -> dict:
            return {
                "input_tokens": result.input_tokens,
                "output_tokens": result.output_tokens,
                "cache_read_tokens": result.cache_read_tokens,
                "cache_write_tokens": result.cache_write_tokens,
                "cost_usd": round(cost_usd(model, result.input_tokens, result.output_tokens), 8),
                "elapsed_ms": elapsed_ms,
                "attempt_id": getattr(result, "attempt_id", None),
            }

        _gate_unextracted(scan, getattr(args, "allow_partial", False))
        file_res = file_resolve(scan, catalog, spec)
        subsystem = c_file.parent.name

        # Extract call graph and topologically sort
        deps = extract_call_graph(scan, spec_exports=file_res.exports)
        if cpp_com_mode:
            deps = apply_interface_method_signatures(
                deps,
                args.class_name,
                cpp_com_interfaces,
                cpp_com_cls.sdk_type_aliases if cpp_com_cls is not None else None,
            )
        sorted_deps = topological_sort(deps)
        if c_file.suffix.lower() == ".c":
            # Independent textual inventory, before selectors or any paid work.
            # Scanner diagnostics alone cannot account for a function it never saw.
            original_source = c_file.read_text(errors="replace")
            original_dead = never_compiled_lines(
                original_source, _module_pinned_macros(c_file, args)
            )
            live_source = "\n".join(
                "" if line_no in original_dead else line
                for line_no, line in enumerate(original_source.splitlines(), 1)
            )
            require_c_plan(live_source, (dep.name for dep in sorted_deps))
        cpp_com_wide_statics = ""

        if cpp_class_target and free_functions:
            # Free functions only: a class method needs the deterministic COM scaffold
            # (vtable slot, `this` recovery, state layout) that only --cpp-com builds.
            # Name every skipped method rather than dropping it silently — an
            # invisible omission is exactly how iphlpapi shipped 14 missing Pf*
            # bodies from a run that exited 0.
            skipped = sorted({dep.name for dep in sorted_deps if dep.cpp_class is not None})
            sorted_deps = [dep for dep in sorted_deps if dep.cpp_class is None]
            if skipped:
                print(
                    f"  [free-functions] skipping {len(skipped)} class method(s) — these need "
                    f"--cpp-com --class: {', '.join(skipped)}",
                    file=sys.stderr,
                )
            if not sorted_deps:
                raise CppComError(
                    f"{c_file.name}: --free-functions found no classless function to translate "
                    f"({len(skipped)} class method(s) present). Use --cpp-com --class instead."
                )
            # These bodies are C++, so they carry C++ constructs that have no Rust
            # spelling. The COM path lowers them deterministically before paying;
            # routing free functions through the ordinary (plain-C) contract skipped
            # that layer entirely, and the model duly invented `u"..."` — which is not
            # Rust syntax — for every wide literal. Same code, same guarantees, $0.
            require_supported_worker_raii(sorted_deps, provided=set(local_raii_impls))
            sorted_deps = [
                replace(dep, body_text=rewrite_wide_literals(dep.body_text)) for dep in sorted_deps
            ]
            cpp_com_wide_statics = render_wide_literal_statics(deps, sorted_deps)
        elif cpp_com_mode:
            assert cpp_com_cls is not None
            sorted_deps = prepare_helper_closure(
                deps,
                args.class_name,
                cpp_com_cls.plain_state_fields(),
                cpp_com_cls.island_struct_field_types,
                cpp_com_cls.sdk_type_aliases,
            )
            # The mirror of the --free-functions report above, and it was missing for years.
            # prepare_helper_closure keeps the class closure, so the TU's file-scope functions
            # fall out with no diagnostic — shellrecyclebin lost all six of its `extern "C"`
            # entry points (the *_Constructor family) that way. Silence is what makes it
            # dangerous: cargo check only catches the ones another Rust module happens to call,
            # and once the TU is cohab-flipped the rest link-resolve to the still-compiled C,
            # leaving a half-Rust module that reports nothing. shell32 gets this right purely by
            # convention (cidldataobj_free.rs, CDefaultContextMenu_tu.rs) — a convention
            # documented only in a CMakeLists comment, which is a missing mechanism.
            kept = {dep.name for dep in sorted_deps}
            dropped = sorted(
                dep.name for dep in deps if dep.cpp_class is None and dep.name not in kept
            )
            if dropped:
                print(
                    f"  [cpp-com] {args.class_name}: skipping {len(dropped)} file-scope "
                    f"function(s) — these need a separate --free-functions run on the same "
                    f"file: {', '.join(dropped)}",
                    file=sys.stderr,
                )
            require_supported_worker_raii(sorted_deps, provided=set(local_raii_impls))
            cpp_com_wide_statics = render_wide_literal_statics(deps, sorted_deps)

        fact_seal_enabled = bool(getattr(args, "seal_facts", False))
        call_seal_enabled = bool(getattr(args, "seal_calls", False))
        model_bodies = {dep.name: strip_debug_statements(dep.body_text) for dep in sorted_deps}
        # FIRST of the three, so the fact seal sees an identifier where a literal was and
        # never has to reason about string contents, and so a wide literal's own array
        # BOUND (`WCHAR k[200] = L"..."`) is still a plain integer when this reads it.
        # `L"AddReg.NT" INF_ARCH` folds only with the macro's real per-arch value.
        string_macros = (
            string_literal_macros(REACTOS_ROOT, c_file.resolve()) if wide_seal_enabled else {}
        )
        wide_seals = (
            {
                dep.name: seal_wide_literals(model_bodies[dep.name], string_macros)
                for dep in sorted_deps
            }
            if wide_seal_enabled
            else {}
        )
        if wide_seals:
            model_bodies = {
                name: wide_seals[name].text if name in wide_seals else body
                for name, body in model_bodies.items()
            }
        fact_seals = (
            {
                dep.name: prepare_sealed_source(model_bodies[dep.name], dialect=seal_dialect)
                for dep in sorted_deps
            }
            if fact_seal_enabled
            else {}
        )
        model_sources = {
            dep.name: fact_seals[dep.name].text
            if dep.name in fact_seals
            else model_bodies[dep.name]
            for dep in sorted_deps
        }
        # The seal must know which callee-shaped names are function-like MACROS: they
        # have no 1:1 Rust call to carry a token, so sealing one makes the envelope
        # unsatisfiable and one such worker voids the whole TU. Derived from the real
        # preprocessor, and a HARD FAIL when unavailable — "seal everything anyway" is
        # exactly the silent degradation this replaced.
        seal_macros = (
            function_like_macros(REACTOS_ROOT, c_file.resolve())
            if call_seal_enabled
            else frozenset()
        )
        # Sibling-method callees of the class under --cpp-com. Their `_body` lowerings take an
        # explicit leading `this`, so the sealed arity has to expect one more argument than the
        # C call site shows (see seal_calls). A class STATIC would not take a receiver and would
        # now be rejected rather than accepted -- deliberately the safe direction, since a
        # rejection is loud and re-runs from cache for $0 while a wrong accept ships silently.
        # `dep.callees` already carries the `_body` suffix for a method lowering, so these are
        # taken as-is; appending another one yields `_body_body` and matches nothing.
        receiver_callees = (
            frozenset(
                callee
                for dep in sorted_deps
                for callee in dep.callees
                if callee.startswith(f"{args.class_name}_") and callee.endswith("_body")
            )
            if cpp_com_mode and getattr(args, "class_name", None)
            else frozenset()
        )
        # ...and the same thing for every OTHER C++ path, which the clause above cannot reach:
        # it needs a single `--class-name`, and `cpp_com_mode` is `target.name == "shell32"`.
        # --plain discovers its classes instead of being told one, and a non-shell32 C++ module
        # (shellrecyclebin) fails both halves — so `receiver_callees` came out EMPTY and every
        # implicit-receiver sibling call kept the C site's arity. That is not the safe direction
        # the note above assumes: `AddRef();` inside a member function is `this->AddRef()`, argc
        # 0, while the only correct --plain lowering is `RecycleBinGeneric_AddRef(this)`, argc 1.
        # The envelope then rejected CORRECT output — RecycleBinGeneric's QueryInterface and
        # EmptyRecycleBin both, i.e. the COM boilerplate every such class has, so --plain COM
        # classes rejected by construction and a retry could only re-bill the same verdict.
        #
        # Derived per-dep from the TU's own class membership, so a TU with several classes
        # (recyclebin_v5.cpp: CZZWStr + RecycleBin5) marks each call against the class that
        # actually owns the CALLER. A free function in the same TU keeps argc as-is.
        methods_by_class: dict[str, set[str]] = {}
        for fn in scan.functions:
            owner = getattr(fn, "cpp_class", None)
            if owner:
                methods_by_class.setdefault(owner, set()).add(fn.name)
        receivers_by_dep = {}
        for dep in sorted_deps:
            if receiver_callees:
                receivers_by_dep[dep.name] = receiver_callees
                continue
            own = methods_by_class.get(getattr(dep, "cpp_class", None) or "", ())
            receivers_by_dep[dep.name] = frozenset(
                callee
                for callee in dep.callees
                if callee in own or (callee.endswith("_body") and callee[: -len("_body")] in own)
            )

        # The C++ spelling behind each lowered sibling name, so the default-argument lookup
        # in `seal_calls` can find a declaration that only ever names the METHOD.
        callee_cpp_names: dict[str, str] = {}
        for dep in sorted_deps:
            for callee in dep.callees:
                for lowered in (callee, f"{callee}_body"):
                    cpp_name = lowered[: -len("_body")] if lowered.endswith("_body") else lowered
                    for owner in methods_by_class:
                        prefix = f"{owner}_"
                        if cpp_name.startswith(prefix) and cpp_name[len(prefix) :]:
                            callee_cpp_names[lowered] = cpp_name[len(prefix) :]
                            break

        # ATL CWindow methods, for classes that derive from CWindow. See
        # `cpp_core.cwindow_method_arity` for why their arity is off by the window handle.
        atlwin = REACTOS_ROOT / "sdk" / "lib" / "atl" / "atlwin.h"
        window_classes = [
            owner
            for owner in methods_by_class
            if cpp_core.derives_from_cwindow(scan.raw_source, owner)
        ]
        cwindow_arity = (
            cpp_core.cwindow_method_arity(atlwin.read_text(encoding="utf-8", errors="replace"))
            if window_classes and atlwin.is_file()
            else {}
        )
        hwnd_by_dep = {
            dep.name: (
                cwindow_arity
                if cwindow_arity and getattr(dep, "cpp_class", None) in window_classes
                else {}
            )
            for dep in sorted_deps
        }

        call_seals = (
            {
                dep.name: seal_calls(
                    model_sources[dep.name],
                    set(dep.callees) | {f"{callee}_body" for callee in dep.callees},
                    seal_macros,
                    tu_source=scan.raw_source,
                    receiver_callees=receivers_by_dep[dep.name],
                    callee_cpp_names=callee_cpp_names,
                    hwnd_receiver_callees=hwnd_by_dep[dep.name],
                )
                for dep in sorted_deps
            }
            if call_seal_enabled
            else {}
        )

        model_deps = {}
        for dep in sorted_deps:
            sealed_calls = call_seals.get(dep.name)
            body_text = sealed_calls.text if sealed_calls is not None else model_sources[dep.name]
            debug_macros = {"TRACE", "FIXME", "WARN", "ERR", "MESSAGE"}
            callees = dep.callees - debug_macros
            model_deps[dep.name] = (
                replace(dep, body_text=body_text, callees=callees)
                if body_text != dep.body_text or callees != dep.callees
                else dep
            )

        deterministic_contracts = {
            dep.name: (fact_contract(fact_seals[dep.name]) if dep.name in fact_seals else "")
            + (call_contract(call_seals[dep.name]) if dep.name in call_seals else "")
            + (wide_contract(wide_seals[dep.name]) if dep.name in wide_seals else "")
            for dep in sorted_deps
        }
        for raii_name, raii_rust in local_raii_impls.items():
            raii_block = provided_raii_contract(raii_name, raii_rust)
            for dep in sorted_deps:
                if re.search(rf"\b{raii_name}\b", dep.body_text):
                    deterministic_contracts[dep.name] += raii_block
        if cpp_com_cls is not None:
            container_fields = [
                state
                for state in cpp_com_cls.plain_state_fields()
                if state.raii_container is not None
            ]
            for dep in sorted_deps:
                touched = [
                    state
                    for state in container_fields
                    if re.search(rf"\b{re.escape(state.name)}\b", dep.body_text)
                ]
                if touched:
                    deterministic_contracts[dep.name] += raii_container_contract(touched)

        # WHAT each worker had hidden from it, as names rather than token indices.
        # The response cache compares this directionally: an entry sealed at least as
        # tightly as this run wants is still sound, so tightening a seal rule (or
        # excluding a macro from one) costs nothing already paid for.
        sealed_names = {
            dep.name: frozenset(
                [f"call:{call.original}" for call in call_seals[dep.name].calls]
                if dep.name in call_seals
                else []
            )
            | frozenset(
                [f"fact:{fact.original}" for fact in fact_seals[dep.name].facts]
                if dep.name in fact_seals
                else []
            )
            for dep in sorted_deps
        }

        # `--fn` selects the paid/output unit, not its dependency knowledge.
        # Preserve the complete prepared closure for local-worker signatures.
        context_deps = list(sorted_deps)

        # Filter to one or a comma-separated bounded worker set. Keeping this
        # in one invocation lets the normal concurrency/cache/ledger machinery
        # run representative giant-class experiments without racing multiple
        # processes against the same response manifest.
        if args.fn:
            requested = {name.strip() for name in args.fn.split(",") if name.strip()}
            selected = [d for d in sorted_deps if d.name in requested]
            fn_names_found.update(d.name for d in selected)
            missing = requested - {d.name for d in selected}
            sorted_deps = selected
            if missing:
                print(
                    f"  Error: function(s) {', '.join(sorted(missing))} not found in {c_file.name}",
                    file=sys.stderr,
                )
                func_names = [d.name for d in deps]
                print(f"  Available: {', '.join(func_names)}", file=sys.stderr)
                continue

        # `--skip-fn`: drop a named body from the worklist so the REST of the TU can
        # assemble. Without it, one function the model cannot get past the call/fact
        # envelope throws away every other function in the file — `--exclude` filters
        # FILE stems, not function names, so it is no help here (process.c's
        # `msvcrt_spawn` and `_wpopen` blocked 48 accepted bodies across nine runs).
        # The skip is never silent: each name becomes an audit finding, so the assembled
        # file always says out loud which bodies a human still owes.
        # Bodies the BUILD never compiles are dropped before anything is bought. A function
        # inside `#ifndef __REACTOS__` is in no object file we produce, so a translation of
        # it can never be called — and `translate` bills one paid call PER FUNCTION, so
        # leaving them in the worklist is a bill for code that does not exist.
        # except_i386.c is 7 of 23 that way: `_global_unwind2`, `_except_handler2`,
        # `_except_handler3`, `_local_unwind2`, `longjmp` and the two `__regs__setjmp*`,
        # every one confirmed absent from the object with `nm`. The rule is the narrow one
        # `dead_regions.cond_truth` can prove; anything it cannot settle stays live and gets
        # translated, because paying for a live body is a cost and dropping one is a bug.
        # TWO classes of gate, and the second is the bigger one in msvcrt: `#ifndef
        # __REACTOS__` is tree-wide, but the module's own `-D_MSVCR_VER=0` decides ~200
        # `#if _MSVCR_VER >= 80/100/110/120/140` directives — concurrency.c is 3,839 lines
        # wholly inside one. `_module_pinned_macros` reads those off the CMakeLists, and
        # only where every target and every arch agrees on the value.
        dead_lines = never_compiled_lines(scan.raw_source, _module_pinned_macros(c_file, args))
        skipped_dead = sorted(d.name for d in sorted_deps if d.line_start in dead_lines)
        if skipped_dead:
            sorted_deps = [d for d in sorted_deps if d.name not in set(skipped_dead)]
            print(
                f"  dead branch: {len(skipped_dead)} body/bodies the build never compiles, "
                f"not translated and not billed: {', '.join(skipped_dead)}",
                file=sys.stderr,
            )

        # A `--skip-fn` name belongs to exactly ONE translation unit, so on a directory run
        # every other file legitimately does not carry it. Treating that as a per-file error and
        # `continue`ing skipped those files ENTIRELY — unbought, unassembled, and reported only
        # as a stderr line among eleven others. Asking to drop one BODY silently dropped eleven
        # FILES; the run looked like it worked and cost a whole cycle to notice. So a name that
        # this file does not have is simply not applicable here. A name NO file has is still a
        # typo, and the post-loop check below is loud about it — that is the case the original
        # error was really for, and it belongs where the whole worklist is known.
        skipped_by_request = skip_fn_for_file(skip_fn_requested, (d.name for d in sorted_deps))
        if skipped_by_request:
            skip_fn_matched |= set(skipped_by_request)
            sorted_deps = [d for d in sorted_deps if d.name not in set(skipped_by_request)]
            print(
                f"  --skip-fn: {len(skipped_by_request)} body/bodies left to hand-writing: "
                f"{', '.join(skipped_by_request)}",
                file=sys.stderr,
            )

        print(f"  Functions: {len(sorted_deps)} (topo-sorted)", file=sys.stderr)

        # c2rust behavior anchors: a compiler-exact transpilation per function, handed
        # to the model so it can't silently stub/drop logic. Built once per file.
        c2rust_anchors: dict[str, str] = {}
        c2rust_preamble = ""
        anchor_enabled = getattr(args, "c2rust", None)
        if anchor_enabled is None:
            anchor_enabled = c_file.suffix.lower() == ".c" and not getattr(
                args, "deterministic_only", False
            )
        if anchor_enabled:
            # C anchoring is on by default: missing anchors are a HARD FAIL
            # (strict=True) instead of silently degrading to a worse pure-LLM translation.
            # Fails before any paid API call, so no money is wasted on unanchored output.
            try:
                try:
                    c2rust_anchors = build_anchor_map(
                        c_file,
                        strict=True,
                        required_names={dep.name for dep in sorted_deps},
                        arch=args.arch,
                    )
                except CompiledOut as gone:
                    # The compiler's own verdict, finer than `never_compiled_lines`: a
                    # condition only the preprocessor could settle. Same treatment.
                    sorted_deps = [d for d in sorted_deps if d.name not in gone.names]
                    print(
                        f"  dead branch (per c2rust, every built arch): {len(gone.names)} "
                        f"body/bodies not translated and not billed: "
                        f"{', '.join(sorted(gone.names))}",
                        file=sys.stderr,
                    )
                    c2rust_anchors = build_anchor_map(
                        c_file,
                        strict=True,
                        required_names={dep.name for dep in sorted_deps},
                        arch=args.arch,
                    )
                # File-scope static/const DATA (tables/S-boxes) compiler-exact — prepended
                # to the preamble so the LLM never has to (mis)transcribe a lookup table.
                c2rust_preamble = build_preamble(c_file, strict=True, arch=args.arch)
            except C2RustError as e:
                print(
                    f"\n  ✗ FATAL: --c2rust requested but c2rust failed: {e}\n"
                    "    Refusing to fall back to a pure-LLM translation (no dumb fallbacks).\n"
                    "    Fix the cause above, or explicitly pass --no-c2rust for pure-LLM.\n",
                    file=sys.stderr,
                )
                sys.exit(2)

        if args.dry_run:
            # Just show the plan
            print("\n  Translation order:", file=sys.stderr)
            all_deps = (
                context_deps
                if cpp_com_mode
                else extract_call_graph(scan, spec_exports=file_res.exports)
            )
            local_names = {d.name for d in all_deps}
            # The same $0 pass the real run does, so the plan says what you will actually
            # PAY for rather than how many functions the file has. Free on mbcs.c is 76 of
            # 192 — a dry-run that hid that would misprice the file by 40%.
            # Mirror the REAL run's branch exactly — it lowers on `cpp_com_mode`, not on the
            # target name. Keying this on `target != "shell32"` meant every shell32 C++ run
            # previewed its free trivial workers as PAID: filetypes' one-line DoExtractIcon
            # forwarder showed as "1 to buy" on a reassembly that has always cost $0.00, which
            # is exactly the number you read before deciding whether to spend.
            preview: dict[str, str] = {}
            if cpp_com_mode:
                for dep in sorted_deps:
                    sig = worker_signature(dep.name, dep.cpp_class, dep.params, dep.return_type)
                    if lower_trivial_worker(dep.body_text, sig) is not None:
                        preview[dep.name] = "trivial worker"
            else:
                preview = {
                    name: low.shape
                    for name, low in trivial_c.lower_trivial_c_functions(
                        sorted_deps, typedefs.pointer_typedefs_for_source(c_file)
                    ).items()
                }
            for i, dep in enumerate(sorted_deps, 1):
                exported = " [EXPORTED]" if dep.is_exported else ""
                static = " [static]" if dep.is_static else ""
                local_callees = dep.callees & local_names
                callee_info = (
                    f" → calls: {', '.join(sorted(local_callees))}" if local_callees else ""
                )
                seh = f" (SEH×{dep.seh_count})" if dep.seh_count else ""
                free = f"  [$0 {preview[dep.name]}]" if dep.name in preview else ""
                print(
                    f"    {i}. {dep.name}{exported}{static}{seh}{callee_info}{free}",
                    file=sys.stderr,
                )
            if preview:
                shapes = collections.Counter(preview.values())
                detail = ", ".join(f"{n}× {s}" for s, n in shapes.most_common())
                print(
                    f"\n  Deterministic (no API call, $0): {len(preview)} of {len(sorted_deps)}"
                    f"  —  {detail}",
                    file=sys.stderr,
                )
                print(f"  PAID: {len(sorted_deps) - len(preview)} functions", file=sys.stderr)

            # What is ALREADY BANKED. Without this the plan prices a re-run of a
            # partly-translated file as if nothing had been paid for, and the only
            # way to learn otherwise was to start spending. It also makes a cache
            # invalidation visible BEFORE it re-bills: a seal or contract change
            # that voids banked work shows up here as a collapsed hit count.
            dry_cache = _open_response_cache(
                args, subsystem, c_file, fact_seal_enabled, call_seal_enabled, wide_seal_enabled
            )
            banked = [
                dep.name
                for dep in sorted_deps
                if dep.name not in preview
                and dry_cache.get(
                    dep.name,
                    dep.body_text,
                    deterministic_contracts.get(dep.name, ""),
                    sealed_names.get(dep.name),
                )
                is not None
            ]
            unbanked = len(sorted_deps) - len(preview) - len(banked)
            print(
                f"  Resume cache: {len(banked)} banked ($0 replay), {unbanked} to buy"
                f"   [{dry_cache.dir.name}]",
                file=sys.stderr,
            )

            # Show estimated tokens for first function
            if sorted_deps:
                func_res = resolve_function(sorted_deps[0], catalog, file_res.exports, scan.defines)
                prompt = build_function_prompt(
                    model_deps[sorted_deps[0].name],
                    func_res,
                    {},
                    all_deps,
                    scan,
                    catalog_fn_names,
                    file_res.exports,
                    target=rustconv_target,
                    deterministic_contract=deterministic_contracts[sorted_deps[0].name],
                    cpp_com=cpp_com_mode,
                )
                if not use_claude:
                    prompt = system_prompt + "\n\n---\n\n" + prompt if system_prompt else prompt
                tokens = estimate_tokens(prompt)
                print(f"\n  First function prompt: ~{tokens:,} tokens", file=sys.stderr)

            if args.prompt:
                # Print full prompt for first function
                func_res = resolve_function(sorted_deps[0], catalog, file_res.exports, scan.defines)
                prompt = build_function_prompt(
                    model_deps[sorted_deps[0].name],
                    func_res,
                    {},
                    all_deps,
                    scan,
                    catalog_fn_names,
                    file_res.exports,
                    target=rustconv_target,
                    c2rust_anchor=c2rust_anchors.get(sorted_deps[0].name),
                    deterministic_contract=deterministic_contracts[sorted_deps[0].name],
                    cpp_com=cpp_com_mode,
                )
                full = (system_prompt + "\n\n---\n\n" + prompt) if system_prompt else prompt
                print(full)

            continue

        # Translate function by function
        all_deps_for_context = (
            context_deps
            if cpp_com_mode
            else extract_call_graph(scan, spec_exports=file_res.exports)
        )
        translated: dict[str, str] = {}
        # Banked plain-C bodies that fail a check added since they were bought (see
        # _stale_c_body); only these may be re-bought under --rebuy-rejected.
        reaudit_failed: set[str] = set()
        deterministic_workers: set[str] = set()
        if cpp_com_mode:
            for dep in sorted_deps:
                signature = worker_signature(
                    dep.name,
                    dep.cpp_class,
                    dep.params,
                    dep.return_type,
                )
                lowered = lower_trivial_worker(dep.body_text, signature)
                if lowered is not None:
                    translated[dep.name] = lowered
                    deterministic_workers.add(dep.name)
        else:
            # Plain C: deterministically lower the trivial band for $0. Each lowering
            # re-verifies itself against an INDEPENDENT tree-sitter oracle (control-free
            # on both sides, identical ordered identifier sequence, equal constant
            # multisets) and RAISES on disagreement rather than shipping — nobody reviews
            # a free translation, so it has to prove itself. Anything it cannot prove
            # returns None and is paid for as normal.
            #
            # The typedef pass runs first because it decides how much the lowering can
            # reach: a literal NULL is only spellable at a position PROVEN to be a
            # pointer, and the CRT's sentinel (`_locale_t`) hides its star in a header.
            # Printed, not silent — a $0 layer quietly proving nothing would just bill
            # the model instead, which is exactly the kind of degradation nobody notices.
            ptr_typedefs = typedefs.pointer_typedefs_for_source(c_file)
            print(typedefs.describe(ptr_typedefs, c_file))
            lowerings = trivial_c.lower_trivial_c_functions(sorted_deps, ptr_typedefs)
            for name, lowering in lowerings.items():
                translated[name] = lowering.rust
                deterministic_workers.add(name)
        for dep in sorted_deps:
            if dep.name in deterministic_workers:
                _record_outcome(dep, "deterministic", route="trivial_worker")
        total_in = 0
        total_out = 0

        # Per-file SHARED context (import superset + gap fns + globals), built
        # ONCE and sent as a second cached system block on every function call.
        # It's byte-identical across the file's N functions, so functions 2..N
        # read it from cache (~0.1x) instead of re-sending it uncached each call.
        file_context = build_file_context(
            file_res, scan, subsystem, target=rustconv_target, cpp_com=cpp_com_mode
        )

        cache_suffix = _cache_suffix(args, fact_seal_enabled, call_seal_enabled, wide_seal_enabled)
        cache = _open_response_cache(
            args, subsystem, c_file, fact_seal_enabled, call_seal_enabled, wide_seal_enabled
        )

        # A completed model stream can fail a deterministic audit and remain as
        # ``.partial``. When the harness improves, replay those already-paid bytes
        # through the current seals/normalizers before buying another response.
        # Every acceptance check is identical to a live response; incompatible,
        # truncated, or genuinely bad historical streams remain rejected.
        partial_replayed: set[str] = set()
        partial_rejects: dict[str, str] = {}
        if cpp_com_mode and not no_resume:
            run_dirs = sorted(RESPONSES_DIR.glob(f"*-{c_file.name}"), reverse=True)
            for dep in sorted_deps:
                if (
                    dep.name in deterministic_workers
                    or cache.get(
                        dep.name,
                        dep.body_text,
                        deterministic_contracts.get(dep.name, ""),
                        sealed_names.get(dep.name),
                    )
                    is not None
                ):
                    continue
                pattern = re.compile(rf"^[0-9]+_{re.escape(dep.name)}\.partial$")
                recovered = None
                for run_dir in run_dirs:
                    candidate = next(
                        (path for path in run_dir.iterdir() if pattern.fullmatch(path.name)), None
                    )
                    if candidate is None:
                        continue
                    try:
                        raw = candidate.read_text(encoding="utf-8")
                        if not raw.strip():
                            continue
                        normalized = _normalize_unseal_cpp_worker(dep, raw, fact_seals, call_seals)
                        findings = audit_worker_body(
                            dep.body_text,
                            normalized,
                            f"{dep.name}_body",
                            worker_signature(dep.name, dep.cpp_class, dep.params, dep.return_type),
                        )
                    except (CppComError, OSError) as exc:
                        partial_rejects.setdefault(dep.name, f"{candidate}: {exc}")
                        continue
                    if findings:
                        partial_rejects.setdefault(dep.name, f"{candidate}: {'; '.join(findings)}")
                    if not findings:
                        recovered = normalized
                        break
                if recovered is not None:
                    cache.put(
                        dep.name,
                        dep.body_text,
                        recovered,
                        deterministic_contracts.get(dep.name, ""),
                        sealed_names.get(dep.name),
                    )
                    partial_replayed.add(dep.name)
                    _record_outcome(dep, "partial_replay", route="current_deterministic_audit")
            if partial_replayed:
                print(
                    f"  [cpp-com] recovered {len(partial_replayed)} paid partial(s) "
                    "under the current deterministic audit ($0)",
                    file=sys.stderr,
                )
            for name, reason in sorted(partial_rejects.items()):
                if name not in partial_replayed:
                    print(
                        f"  [cpp-com] paid partial still rejected: {name}: {reason}",
                        file=sys.stderr,
                    )
        translation_memory = None
        memory_seeded: set[str] = set()
        memory_identity: Callable[..., dict[str, object]] | None = None
        if (
            cpp_com_mode
            and cpp_com_cls is not None
            and mode == "mechanical"
            and fact_seal_enabled
            and call_seal_enabled
        ):
            translation_memory = ExactBodyTranslationMemory()
            state_shape = [
                (field.name, field.rust_type, field.com_ptr_interface)
                for field in cpp_com_cls.plain_state_fields()
            ]
            class_shape = json.dumps(
                {
                    "interfaces": [entry.interface for entry in cpp_com_cls.com_map],
                    "state": state_shape,
                    "final_construct": cpp_com_cls.final_construct,
                },
                sort_keys=True,
            )

            def _memory_identity(dep, *, class_shape=class_shape, cache_suffix=cache_suffix):
                assert dep.cpp_class is not None
                return {
                    "body_text": dep.body_text,
                    "class_name": dep.cpp_class,
                    "method": dep.cpp_method,
                    "return_type": dep.return_type,
                    "params": dep.params,
                    "class_shape": class_shape,
                    "namespace": f"mechanical{cache_suffix}",
                }

            memory_identity = _memory_identity

        resumed = 0
        if cache.count() and not no_resume:
            print(
                f"  Resume cache: {cache.count()} prior unit(s) on disk (unchanged → $0)",
                file=sys.stderr,
            )

        # One paid banner per file: n calls = functions + an optional preamble. Rough input
        # estimate = first-function prompt × function count (each call re-sends
        # context, so this is a floor, not a ceiling).
        paid_deps = [dep for dep in sorted_deps if dep.name not in deterministic_workers]
        # Take the proven $0 band and stop, without an API key or a cent. The rest of the
        # file assembles as normal (missing bodies are simply absent), so the free band can
        # be integrated, gated and landed on its own before anyone decides to pay for the
        # rest. Enforced at EVERY paid entry point below, not here: `paid_deps` only feeds
        # the cost banner and the resume pre-pass, while the translation loops iterate
        # `sorted_deps` — emptying this list looks like it disables paying and does not.
        # (It did not. That mistake spent $0.90 on mbcs.c.)
        deterministic_only = bool(getattr(args, "deterministic_only", False))
        if deterministic_only:
            print(
                f"  --deterministic-only: keeping {len(deterministic_workers)} proven "
                f"lowering(s), SKIPPING {len(paid_deps)} paid function(s) and the preamble",
                file=sys.stderr,
            )
            paid_deps = []
        if translation_memory is not None and not no_resume:
            for dep in paid_deps:
                if dep.cpp_class is None:
                    continue
                if (
                    cache.get(
                        dep.name,
                        dep.body_text,
                        deterministic_contracts.get(dep.name, ""),
                        sealed_names.get(dep.name),
                    )
                    is not None
                ):
                    continue
                assert memory_identity is not None
                hit = translation_memory.get(**memory_identity(dep))
                if hit is None:
                    continue
                signature = worker_signature(dep.name, dep.cpp_class, dep.params, dep.return_type)
                hit = normalize_worker_body(hit, signature)
                hit = normalize_worker_semantics(dep.body_text, hit)
                findings = audit_worker_body(dep.body_text, hit, f"{dep.name}_body", signature)
                if findings:
                    continue
                cache.put(
                    dep.name,
                    dep.body_text,
                    hit,
                    deterministic_contracts.get(dep.name, ""),
                    sealed_names.get(dep.name),
                )
                memory_seeded.add(dep.name)
        pending_paid_deps = []
        for dep in paid_deps:
            hit = (
                None
                if no_resume
                else cache.get(
                    dep.name,
                    dep.body_text,
                    deterministic_contracts.get(dep.name, ""),
                    sealed_names.get(dep.name),
                )
            )
            if hit is not None and cpp_com_mode:
                hit = normalize_worker_body(
                    hit, worker_signature(dep.name, dep.cpp_class, dep.params, dep.return_type)
                )
                hit = normalize_worker_semantics(dep.body_text, hit)
                if audit_worker_body(
                    dep.body_text,
                    hit,
                    f"{dep.name}_body",
                    worker_signature(dep.name, dep.cpp_class, dep.params, dep.return_type),
                ):
                    hit = None
            elif hit is not None and is_cpp_source:
                # A cached body predates (or was produced without) the C++ repairs;
                # normalization is idempotent, so applying it here is free insurance
                # against a stale entry replaying a raw IID_PPV_ARG forever.
                hit = normalize_cpp_idioms(dep.body_text, hit)
                hit = _cpp_free_half_semantics(dep, hit)
            if hit is None:
                pending_paid_deps.append(dep)
        if deterministic_workers:
            print(
                f"  [cpp-com] lowered {len(deterministic_workers)} proven stub body/bodies "
                "deterministically ($0)",
                file=sys.stderr,
            )
        cap_prevent_paid = bool(pending_paid_deps and _cost_cap_reached(cost_cap, running_cost))
        if cap_prevent_paid:
            capped = True
            print(
                f"  ⛔ COST CAP ${running_cost:.2f} ≥ ${cost_cap:.2f} "
                "— refusing to start any paid calls",
                file=sys.stderr,
            )
        if archive is not None and pending_paid_deps and not cap_prevent_paid:
            _fr0 = resolve_function(pending_paid_deps[0], catalog, file_res.exports, scan.defines)
            _p0 = build_function_prompt(
                model_deps[pending_paid_deps[0].name],
                _fr0,
                {},
                all_deps_for_context,
                scan,
                catalog_fn_names,
                file_res.exports,
                target=rustconv_target,
                deterministic_contract=deterministic_contracts[pending_paid_deps[0].name],
                cpp_com=cpp_com_mode,
            )
            _est_in = (estimate_tokens(_p0) + estimate_tokens(system_prompt)) * len(
                pending_paid_deps
            )
            preamble_calls = 0 if cpp_com_mode else 1
            paid_invocation_banner(model, len(pending_paid_deps) + preamble_calls, _est_in, archive)

        # PARALLEL translate (mechanical + DeepSeek): fan out the paid per-function
        # calls. Workers do ONLY the submit (pure — they read shared state, write only
        # their own durable sink); ALL bookkeeping runs serially on the main thread as
        # futures complete, so no shared-state locking is needed. Resume-cached fns are
        # collected first (free). Cost-cap sets a stop Event that blocks new submits
        # (in-flight calls — at most `conc` — may still land).
        if conc > 1:
            todo = []
            for func_idx, dep in enumerate(sorted_deps, 1):
                if dep.name in deterministic_workers or deterministic_only:
                    continue
                if not no_resume:
                    hit = cache.get(
                        dep.name,
                        dep.body_text,
                        deterministic_contracts.get(dep.name, ""),
                        sealed_names.get(dep.name),
                    )
                    if hit is not None and cpp_com_mode:
                        hit = normalize_worker_body(
                            hit,
                            worker_signature(dep.name, dep.cpp_class, dep.params, dep.return_type),
                        )
                        hit = normalize_worker_semantics(dep.body_text, hit)
                        findings = audit_worker_body(
                            dep.body_text,
                            hit,
                            f"{dep.name}_body",
                            worker_signature(
                                dep.name,
                                dep.cpp_class,
                                dep.params,
                                dep.return_type,
                            ),
                        )
                        if findings:
                            print(
                                f"  [{func_idx}] {dep.name} CACHE REJECTED: " + "; ".join(findings),
                                file=sys.stderr,
                            )
                            hit = None
                    elif hit is not None and is_cpp_source:
                        hit = normalize_cpp_idioms(dep.body_text, hit)
                        hit = _cpp_free_half_semantics(dep, hit)
                    elif hit is not None and _stale_c_body(dep, hit, args, reaudit_failed):
                        hit = None
                    # No unseal on a cache hit. What is banked is the ACCEPTED text, already
                    # unsealed (`_audit_paid_body` mutates `rust_code` before `cache.put`),
                    # and `ResponseCache.get` refuses any entry that still holds a token —
                    # so re-running the closed-envelope check here judged token-free Rust
                    # and could only false-reject, sending banked work back to be re-paid.
                    if hit is not None:
                        translated[dep.name] = hit
                        resumed += 1
                        _record_outcome(
                            dep,
                            "translation_memory_hit" if dep.name in memory_seeded else "cache_hit",
                            cache_key=cache.dir.name,
                        )
                        continue
                if not no_resume:
                    cache.refuse_repurchase(
                        dep.name,
                        allow_rejected=getattr(args, "rebuy_rejected", False),
                        failed_audit=dep.name in reaudit_failed,
                        body_text=dep.body_text,
                        explicit=_rebuy_requested(dep.name),
                    )
                if getattr(args, "replay_only", False):
                    raise ValueError(f"--replay-only: no accepted cached body for {dep.name}")
                todo.append((func_idx, dep))
            ctx = dict(translated)  # static callee context (mechanical fns are independent)
            stop = threading.Event()
            if cap_prevent_paid:
                stop.set()

            # Bind the per-file loop vars as defaults so the closure captures this
            # iteration's values (B023): the executor joins within the iteration, so
            # these never change under it, but the explicit binding makes that provable.
            def _work(
                item,
                *,
                stop=stop,
                ctx=ctx,
                file_res=file_res,
                scan=scan,
                all_deps_for_context=all_deps_for_context,
                c2rust_anchors=c2rust_anchors,
                file_context=file_context,
                model_deps=model_deps,
                deterministic_contracts=deterministic_contracts,
            ):
                fi, dp = item
                if stop.is_set():
                    return (fi, dp, None, 0)
                fres = resolve_function(dp, catalog, file_res.exports, scan.defines)
                up = build_function_prompt(
                    model_deps[dp.name],
                    fres,
                    ctx,
                    all_deps_for_context,
                    scan,
                    catalog_fn_names,
                    file_res.exports,
                    target=rustconv_target,
                    include_shared=False,
                    c2rust_anchor=c2rust_anchors.get(dp.name),
                    deterministic_contract=deterministic_contracts[dp.name],
                    cpp_com=cpp_com_mode,
                )
                sk = ResponseSink(archive.stream_path(fi, dp.name)) if archive is not None else None
                started = time.monotonic()
                try:
                    r = submit_single(
                        up,
                        model=model,
                        system_prompt=system_prompt,
                        file_context=file_context,
                        sink=sk,
                    )
                finally:
                    if sk is not None:
                        sk.close()
                return (fi, dp, r, int((time.monotonic() - started) * 1000))

            print(
                f"  ⚡ parallel: {len(todo)} fn(s) @ concurrency={conc} ({resumed} cached)",
                file=sys.stderr,
            )
            parallel_errors = []
            with ThreadPoolExecutor(max_workers=conc) as ex:
                for fut in as_completed([ex.submit(_work, it) for it in todo]):
                    try:
                        fi, dp, r, elapsed_ms = fut.result()
                    except Exception as error:
                        parallel_errors.append(str(error))
                        print(f"  parallel worker FAIL: {error}", file=sys.stderr)
                        continue
                    if r is None:
                        continue  # skipped after cost cap
                    if r.success:
                        try:
                            if cpp_com_mode:
                                r.rust_code = _normalize_unseal_cpp_worker(
                                    dp, r.rust_code, fact_seals, call_seals
                                )
                                findings = audit_worker_body(
                                    dp.body_text,
                                    r.rust_code,
                                    f"{dp.name}_body",
                                    worker_signature(
                                        dp.name,
                                        dp.cpp_class,
                                        dp.params,
                                        dp.return_type,
                                    ),
                                )
                                if findings:
                                    raise CppComError(
                                        f"{dp.name}: paid body failed deterministic audit:\n  - "
                                        + "\n  - ".join(findings)
                                    )
                            else:
                                # Sealing is language-neutral; unsealing must be too — and
                                # exactly once, for the reason in `_audit_paid_body`.
                                r.rust_code = _unseal_worker(
                                    dp, r.rust_code, fact_seals, call_seals, wide_seals
                                )
                                if is_cpp_source:
                                    r.rust_code = normalize_cpp_idioms(dp.body_text, r.rust_code)
                                    r.rust_code = _cpp_free_half_semantics(dp, r.rust_code)
                            body = r.rust_code
                        except CppComError as error:
                            _record_outcome(
                                dp,
                                "audit_reject",
                                stage="body_audit",
                                error=str(error),
                                **_result_fields(r, elapsed_ms),
                            )
                            cache.put_rejected(dp.name, r.rust_code, str(error))
                            parallel_errors.append(str(error))
                            print(f"  [{fi}] {dp.name} AUDIT REJECT: {error}", file=sys.stderr)
                            continue
                        translated[dp.name] = body
                        # The cache keeps the RAW sealed reply: it is what the model said,
                        # and re-unsealing on resume re-checks the seal contract instead of
                        # trusting a once-processed copy.
                        cache.put(
                            dp.name,
                            dp.body_text,
                            r.rust_code,
                            deterministic_contracts.get(dp.name, ""),
                            sealed_names.get(dp.name),
                        )
                        if translation_memory is not None and dp.cpp_class is not None:
                            assert memory_identity is not None
                            translation_memory.put(r.rust_code, **memory_identity(dp))
                        total_in += r.input_tokens
                        total_out += r.output_tokens
                        artifact = None
                        if archive is not None:
                            artifact = archive.commit(
                                fi,
                                dp.name,
                                r.rust_code,
                                r.input_tokens,
                                r.output_tokens,
                                attempt_id=r.attempt_id,
                            )
                            running_cost += cost_usd(model, r.input_tokens, r.output_tokens)
                        _record_outcome(
                            dp,
                            "generated",
                            stage="body_audit" if cpp_com_mode else "api",
                            artifact=str(artifact) if artifact is not None else None,
                            **_result_fields(r, elapsed_ms),
                        )
                        print(
                            f"  [{fi}] {dp.name} OK ({r.input_tokens}/{r.output_tokens})"
                            f"  [running ${running_cost:.2f}]",
                            file=sys.stderr,
                        )
                        if cost_cap is not None and running_cost >= cost_cap:
                            stop.set()
                            capped = True
                            print(
                                f"  ⛔ COST CAP ${running_cost:.2f} ≥ ${cost_cap:.2f} "
                                f"— halting new calls",
                                file=sys.stderr,
                            )
                    else:
                        _record_outcome(
                            dp,
                            "api_error",
                            stage="submit",
                            error=r.error,
                            **_result_fields(r, elapsed_ms),
                        )
                        print(f"  [{fi}] {dp.name} FAIL: {r.error}", file=sys.stderr)
                        parallel_errors.append(f"{dp.name}: {r.error}")
            if parallel_errors:
                raise CppComError(
                    "parallel translation completed with rejected/failed worker(s); "
                    "all other accepted paid artifacts were preserved in the resume cache:\n  - "
                    + "\n  - ".join(parallel_errors)
                )

        # Sequential-path reject collector — mirrors the parallel path's
        # `parallel_errors`: a per-worker audit reject is recorded and the run
        # CONTINUES (all other accepted paid artifacts stream to cache), then a
        # single CppComError surfaces every reject at once after the loop. The
        # old behavior `raise`d on the first reject, abandoning the rest of an
        # already-paid sonnet run (sonnet is sequential — conc is ignored).
        sequential_errors: list[str] = []
        for func_idx, dep in enumerate(sorted_deps, 1):
            if conc > 1:
                break  # already handled by the parallel path above
            print(
                f"  [{func_idx}/{len(sorted_deps)}] {dep.name}...",
                file=sys.stderr,
                end=" ",
                flush=True,
            )

            if dep.name in deterministic_workers:
                print("DETERMINISTIC ($0)", file=sys.stderr)
                continue

            if deterministic_only:
                print("SKIPPED (--deterministic-only)", file=sys.stderr)
                continue

            if _cost_cap_reached(cost_cap, running_cost):
                print(
                    f"COST CAP ${running_cost:.2f} ≥ ${cost_cap:.2f} (no call made)",
                    file=sys.stderr,
                )
                capped = True
                break

            # Resume: reuse a prior unchanged-body translation for free. It still
            # feeds `translated` (callee context for later functions).
            if not no_resume:
                hit = cache.get(
                    dep.name,
                    dep.body_text,
                    deterministic_contracts.get(dep.name, ""),
                    sealed_names.get(dep.name),
                )
                if hit is not None and cpp_com_mode:
                    hit = normalize_worker_body(
                        hit,
                        worker_signature(dep.name, dep.cpp_class, dep.params, dep.return_type),
                    )
                    hit = normalize_worker_semantics(dep.body_text, hit)
                    findings = audit_worker_body(
                        dep.body_text,
                        hit,
                        f"{dep.name}_body",
                        worker_signature(
                            dep.name,
                            dep.cpp_class,
                            dep.params,
                            dep.return_type,
                        ),
                    )
                    if findings:
                        print("CACHE REJECTED: " + "; ".join(findings), file=sys.stderr)
                        hit = None
                elif hit is not None and is_cpp_source:
                    hit = normalize_cpp_idioms(dep.body_text, hit)
                    hit = _cpp_free_half_semantics(dep, hit)
                elif hit is not None and _stale_c_body(dep, hit, args, reaudit_failed):
                    hit = None
                # No unseal on a cache hit — see the same site on the parallel path.
                if hit is None and dep.name not in reaudit_failed:
                    hit = _adopt_rejected(
                        dep,
                        cache,
                        fact_seals,
                        call_seals,
                        wide_seals,
                        cpp_com_mode,
                        func_idx,
                        deterministic_contracts.get(dep.name, ""),
                        sealed_names.get(dep.name),
                    )
                if hit is not None:
                    translated[dep.name] = hit
                    resumed += 1
                    _record_outcome(
                        dep,
                        "translation_memory_hit" if dep.name in memory_seeded else "cache_hit",
                        cache_key=cache.dir.name,
                    )
                    print("CACHED ($0)", file=sys.stderr)
                    continue

            # Per-function resolve
            if not no_resume:
                cache.refuse_repurchase(
                    dep.name,
                    allow_rejected=getattr(args, "rebuy_rejected", False),
                    failed_audit=dep.name in reaudit_failed,
                    body_text=dep.body_text,
                    explicit=_rebuy_requested(dep.name),
                )
            if getattr(args, "replay_only", False):
                raise ValueError(f"--replay-only: no accepted cached body for {dep.name}")
            func_res = resolve_function(dep, catalog, file_res.exports, scan.defines)

            # Build the function-specific prompt only — the shared import/gap/
            # globals context is sent separately as a cached system block
            # (file_context), so include_shared=False keeps it out of the
            # full-price per-function user message.
            user_prompt = build_function_prompt(
                model_deps[dep.name],
                func_res,
                translated,
                all_deps_for_context,
                scan,
                catalog_fn_names,
                file_res.exports,
                target=rustconv_target,
                include_shared=False,
                c2rust_anchor=c2rust_anchors.get(dep.name),
                deterministic_contract=deterministic_contracts[dep.name],
                cpp_com=cpp_com_mode,
            )

            # A rejected worker assembles NOTHING for the whole file, so one
            # single-token slip in function 63 of 73 throws away the other 72. The
            # envelope failures that survive the deterministic fixes are sampling
            # noise on the largest bodies — re-rolling the same function by hand
            # converged 4 rejects → 2 → 1 on shlfileop — so do it here instead of
            # leaving it as a recipe. Each attempt is a real paid call and is billed;
            # only the LAST reject is reported.
            attempts_allowed = 1 + max(0, int(getattr(args, "retry_rejects", 0) or 0))
            body = None
            reject_error: CppComError | None = None
            for attempt in range(1, attempts_allowed + 1):
                # Stream this function's output straight to the durable archive.
                sink = None
                if archive is not None:
                    sink = ResponseSink(archive.stream_path(func_idx, dep.name))
                started = time.monotonic()
                attempt_prompt = user_prompt + _reject_correction(reject_error)
                try:
                    if use_claude:
                        result = submit_single(
                            attempt_prompt,
                            model=model,
                            system_prompt=system_prompt,
                            file_context=file_context,
                            sink=sink,
                        )
                    else:
                        prefix = "\n\n---\n\n".join(p for p in (system_prompt, file_context) if p)
                        full = (
                            (prefix + "\n\n---\n\n" + attempt_prompt) if prefix else attempt_prompt
                        )
                        result = submit_single(full, model=model, sink=sink)
                finally:
                    if sink is not None:
                        sink.close()
                elapsed_ms = int((time.monotonic() - started) * 1000)
                if not result.success:
                    break
                try:
                    body = _audit_paid_body(
                        dep,
                        result,
                        cpp_com_mode=cpp_com_mode,
                        is_cpp_source=is_cpp_source,
                        fact_seals=fact_seals,
                        call_seals=call_seals,
                        wide_seals=wide_seals,
                        target_name=rustconv_target.name,
                        strict=attempt < attempts_allowed,
                    )
                    reject_error = None
                    break
                except CppComError as error:
                    reject_error = error
                    cache.put_rejected(dep.name, result.rust_code, str(error))
                    if attempt >= attempts_allowed:
                        break
                    print(
                        f"  [{func_idx}] {dep.name} AUDIT REJECT "
                        f"(attempt {attempt}/{attempts_allowed}, retrying): {error}",
                        file=sys.stderr,
                    )
                    total_in += result.input_tokens
                    total_out += result.output_tokens
                    if archive is not None:
                        running_cost += cost_usd(model, result.input_tokens, result.output_tokens)

            if result.success:
                try:
                    if reject_error is not None:
                        raise reject_error
                    assert body is not None
                except CppComError as error:
                    _record_outcome(
                        dep,
                        "audit_reject",
                        stage="body_audit",
                        error=str(error),
                        **_result_fields(result, elapsed_ms),
                    )
                    sequential_errors.append(str(error))
                    print(
                        f"  [{func_idx}] {dep.name} AUDIT REJECT: {error}",
                        file=sys.stderr,
                    )
                    continue
                translated[dep.name] = body
                # Persist the RAW sealed reply for cross-run resume (see the parallel path).
                cache.put(
                    dep.name,
                    dep.body_text,
                    result.rust_code,
                    deterministic_contracts.get(dep.name, ""),
                    sealed_names.get(dep.name),
                )
                if translation_memory is not None and dep.cpp_class is not None:
                    assert memory_identity is not None
                    translation_memory.put(result.rust_code, **memory_identity(dep))
                total_in += result.input_tokens
                total_out += result.output_tokens
                artifact = None
                if archive is not None:
                    artifact = archive.commit(
                        func_idx,
                        dep.name,
                        result.rust_code,
                        result.input_tokens,
                        result.output_tokens,
                        attempt_id=result.attempt_id,
                    )
                    running_cost += cost_usd(model, result.input_tokens, result.output_tokens)
                    print(
                        f"OK ({result.input_tokens} in / {result.output_tokens} out)"
                        f"  [running ${running_cost:.2f}]",
                        file=sys.stderr,
                    )
                else:
                    print(
                        f"OK ({result.input_tokens} in / {result.output_tokens} out)",
                        file=sys.stderr,
                    )
                _record_outcome(
                    dep,
                    "generated",
                    stage="body_audit" if cpp_com_mode else "api",
                    artifact=str(artifact) if artifact is not None else None,
                    **_result_fields(result, elapsed_ms),
                )
            else:
                sequential_errors.append(f"{dep.name}: API failed: {result.error}")
                _record_outcome(
                    dep,
                    "api_error",
                    stage="submit",
                    error=result.error,
                    **_result_fields(result, elapsed_ms),
                )
                print(f"FAIL: {result.error}", file=sys.stderr)

            # Hard cost cap — stop before the next (paid) call if we've crossed it.
            if cost_cap is not None and running_cost >= cost_cap:
                print(
                    f"  ⛔ COST CAP HIT: ${running_cost:.2f} ≥ ${cost_cap:.2f} after "
                    f"{func_idx}/{len(sorted_deps)} functions — stopping.",
                    file=sys.stderr,
                )
                capped = True
                break

        if sequential_errors:
            # Name the file that is now STALE. This raise aborts the TU before assembly, so
            # the output on disk is whatever the last successful run left there — and nothing
            # said so. RecycleBin5.rs sat two full regenerations behind its four siblings that
            # way, which made a new deterministic lowering look like it had a hole in exactly
            # one module when in fact nothing had rewritten that module at all.
            stale = ""
            if getattr(args, "output", None) and Path(args.output).exists():
                stale = (
                    f"\n  ⚠ {args.output} was NOT rewritten and is now STALE — it still holds "
                    "the last successful run's output, so any deterministic fix made since "
                    "then is absent from it."
                )
            raise CppComError(
                "sequential translation completed with rejected/failed worker(s); "
                "all other accepted paid artifacts were preserved in the resume cache:\n  - "
                + "\n  - ".join(sequential_errors)
                + stale
            )

        file_scope = "" if cpp_com_mode else (scan.preamble_text or scan.globals_text or "")
        if not translated and not (
            file_scope.strip() and _preamble_has_translatable_content(file_scope)
        ):
            print(f"  No functions translated for {c_file.name}", file=sys.stderr)
            continue
        # A data-only TU (setuplib fileqsup.c: one exported global, no functions) still owns
        # its file-scope definitions; skipping it here left the symbol undefined.

        # Translate the file-scope preamble (structs / typedefs /
        # #define→const / static tables) as ONE synthetic unit. The
        # stitcher otherwise emits no preamble at all, so files with a
        # non-trivial preamble (most drivers) would not compile.
        preamble_rust = ""
        preamble_src = (
            "" if cpp_com_mode else (scan.preamble_text or scan.globals_text or "").strip()
        )
        derived_preamble = (
            derive_kernel_preamble(preamble_src)
            if preamble_src and rustconv_target.sys_crate == "ntkrnl_sys"
            else None
        )
        if derived_preamble is not None:
            preamble_rust = derived_preamble
            _record_preamble(preamble_src, "derived", stage="deterministic")
            print("  [preamble] source-derived kernel assertions ($0)", file=sys.stderr)
            preamble_src = ""

        # A preamble that is nothing but #include / comments / #pragma once carries no
        # file-scope C at all, so its CORRECT Rust translation is the empty string — the
        # includes are resolved by the crate prelude, not per-file. Sending it costs a call
        # and then trips the empty-output hard-fail below, because "model emitted nothing"
        # and "there was nothing to emit" look identical from the outside. Decide it here,
        # from the SOURCE, where the two are distinguishable.
        #
        # Deliberately conservative: ANY surviving token (a typedef, a static, a #define, a
        # WINE_DEFAULT_DEBUG_CHANNEL) makes it non-vacuous and it goes to the model as before.
        # This does not soften the hard-fail — a genuinely-failed non-empty preamble still
        # raises. (Found on wkssvc domain.c/rpcserver.c, whose preamble is one #include: the
        # model correctly returned nothing and the run died AFTER 38/38 paid functions.)
        if preamble_src and not _preamble_has_translatable_content(preamble_src):
            print(
                f"  [preamble] {len(preamble_src)} C chars — no file-scope C to carry over "
                "(includes/comments/debug-channel only), nothing to translate ($0)",
                file=sys.stderr,
            )
            _record_preamble(preamble_src, "vacuous", stage="skipped")
            preamble_src = ""

        # The preamble is a SEPARATE paid call with its own prompt, so the seals set up for
        # the function bodies do not reach it — and it is where the wide seal matters most:
        # every one of the 18 arrays that shipped without the terminator its readers scan
        # for was a file-scope `static const WCHAR X[] = L"...";`, which only this pass sees.
        preamble_wide_seal = (
            seal_wide_literals(preamble_src, string_literal_macros(REACTOS_ROOT, c_file.resolve()))
            if (wide_seal_enabled and preamble_src)
            else None
        )

        pre_hit = None if (no_resume or not preamble_src) else cache.get(PREAMBLE_KEY, preamble_src)
        if pre_hit is not None:
            preamble_rust = pre_hit
            resumed += 1
            _record_preamble(preamble_src, "cache_hit", cache_key=cache.dir.name)
            print("  [preamble] CACHED ($0)", file=sys.stderr)
        elif preamble_src and deterministic_only:
            print("  [preamble] SKIPPED (--deterministic-only)", file=sys.stderr)
        elif preamble_src and not capped:
            print(
                f"  [preamble] {len(preamble_src)} C chars...", file=sys.stderr, end=" ", flush=True
            )
            pre_prompt = build_preamble_prompt(
                scan,
                file_res,
                target=rustconv_target,
                sealed_c=None if preamble_wide_seal is None else preamble_wide_seal.text,
                extra_contract=(
                    "" if preamble_wide_seal is None else wide_contract(preamble_wide_seal)
                ),
            )
            pre_sink = None
            if not no_resume:
                cache.refuse_repurchase(PREAMBLE_KEY)
            if getattr(args, "replay_only", False):
                raise ValueError("--replay-only: no accepted cached preamble")
            if archive is not None:
                pre_sink = ResponseSink(archive.stream_path(0, "preamble"))
            pre_started = time.monotonic()
            try:
                if use_claude:
                    pre_res = submit_single(
                        pre_prompt, model=model, system_prompt=system_prompt, sink=pre_sink
                    )
                else:
                    pre_full = (
                        (system_prompt + "\n\n---\n\n" + pre_prompt)
                        if system_prompt
                        else pre_prompt
                    )
                    pre_res = submit_single(pre_full, model=model, sink=pre_sink)
            finally:
                if pre_sink is not None:
                    pre_sink.close()
            pre_elapsed_ms = int((time.monotonic() - pre_started) * 1000)
            if pre_res.success:
                if preamble_wide_seal is not None:
                    # Unseal BEFORE banking, exactly as a paid body does: the cache must
                    # hold what was accepted, not the raw reply. A damaged envelope raises
                    # here and the run stops rather than assembling a module whose tables
                    # hold `__RUSTCONV_WIDE_nnnn__`.
                    pre_res.rust_code = unseal_wide_literals(
                        scrub_comment_token_echoes(pre_res.rust_code), preamble_wide_seal
                    )
                preamble_rust = pre_res.rust_code
                cache.put(PREAMBLE_KEY, preamble_src, pre_res.rust_code)
                total_in += pre_res.input_tokens
                total_out += pre_res.output_tokens
                pre_artifact = None
                if archive is not None:
                    pre_artifact = archive.commit(
                        0,
                        "preamble",
                        pre_res.rust_code,
                        pre_res.input_tokens,
                        pre_res.output_tokens,
                        attempt_id=pre_res.attempt_id,
                    )
                    running_cost += cost_usd(model, pre_res.input_tokens, pre_res.output_tokens)
                _record_preamble(
                    preamble_src,
                    "generated",
                    stage="api",
                    artifact=str(pre_artifact) if pre_artifact is not None else None,
                    **_result_fields(pre_res, pre_elapsed_ms),
                )
                print(
                    f"OK ({pre_res.input_tokens} in / {pre_res.output_tokens} out)", file=sys.stderr
                )
            else:
                _record_preamble(
                    preamble_src,
                    "api_error",
                    stage="submit",
                    error=pre_res.error,
                    **_result_fields(pre_res, pre_elapsed_ms),
                )
                # The preamble is the ONLY pass that sees file-scope C — every
                # struct, typedef and lookup table in the file rides on it.
                # "Stitching without preamble" silently ships a crate missing all
                # of them, so this is a hard failure, not a warning.
                raise RuntimeError(
                    f"preamble translation FAILED for {c_file.name}: {pre_res.error}\n"
                    "  Refusing to stitch without it — the result would be missing every\n"
                    "  file-scope struct/typedef/table. Re-run (the per-function work is\n"
                    "  cached in .responses/, so the retry only re-pays for the preamble)."
                )

        # c2rust's vtbl tables are NOT usable: it hardcodes each slot as
        # `f as unsafe extern "C" fn(..)`, which is the amd64 spelling. On i386 the
        # bindgen slot is `extern "stdcall"`, so the cast is simply wrong there.
        # vtbl_gen regenerates them per-arch from the bindgen struct (and agrees
        # 149/149 with our hand-written tables), so let it own them exclusively.
        if c2rust_preamble.strip():
            c2rust_preamble = _strip_vtbl_statics(c2rust_preamble)

        # c2rust DATA preamble wins over the LLM's: prepend the compiler-exact
        # static/const tables, and strip any LLM-emitted item with the same name
        # (the LLM copy is the fabrication-prone one). Deterministic, $0.
        if c2rust_preamble.strip():
            names = set(
                re.findall(r"\b(?:static(?:\s+mut)?|const)\s+([A-Za-z_]\w*)\s*:", c2rust_preamble)
            )
            if names and preamble_rust.strip():
                kept, drop_depth, dropping = [], 0, False
                for ln in preamble_rust.splitlines():
                    if not dropping:
                        m = re.match(
                            r"\s*(?:pub\s+)?(?:static(?:\s+mut)?|const)\s+([A-Za-z_]\w*)\s*:", ln
                        )
                        if m and m.group(1) in names:
                            dropping, drop_depth = True, 0
                        else:
                            kept.append(ln)
                    if dropping:
                        drop_depth += ln.count("[") - ln.count("]") + ln.count("{") - ln.count("}")
                        if drop_depth <= 0 and ln.rstrip().endswith(";"):
                            dropping = False
                preamble_rust = "\n".join(kept)
            preamble_rust = (
                "// @c2rust file-scope data (compiler-exact tables/statics)\n"
                + c2rust_preamble
                + "\n\n"
                + preamble_rust
            )
            print(
                f"  [c2rust] prepended {len(names)} exact static/const item(s) to preamble",
                file=sys.stderr,
            )

        # A `--local-raii` type is OWNED by one module and imported everywhere else, but the
        # file-scope preamble the free-functions half translates re-declares it from the C++
        # header — so the module ended up with both `use …::CStringW;` and its own
        # `pub struct CStringW`. That is E0255, and had it compiled it would be worse: two
        # distinct Rust types for one C++ class, silently incompatible across the two halves.
        # cpp_com mode sidesteps this by skipping the preamble outright; the free half needs
        # the rest of it, so drop just the duplicated items.
        if preamble_rust.strip() and local_raii_uses:
            preamble_rust, deduped = strip_top_level_items(preamble_rust, set(local_raii_uses))
            if deduped:
                print(
                    f"  [preamble] dropped {len(deduped)} re-declaration(s) of "
                    f"--local-raii type(s): {', '.join(sorted(set(deduped)))}",
                    file=sys.stderr,
                )

        print(f"\n  Stitching {len(translated)} functions...", file=sys.stderr)
        if resumed:
            print(
                f"  Resumed {resumed} unit(s) from cache this run (re-translation skipped, $0)",
                file=sys.stderr,
            )
        print(f"  Total tokens: {total_in:,} in / {total_out:,} out", file=sys.stderr)

        # Stitch functions into a single file
        stitched = stitch_file(
            translated, sorted_deps, scan, file_res, target=rustconv_target, preamble=preamble_rust
        )
        if cpp_com_mode:
            if cpp_com_cls is None:
                raise CppComError("internal error: shell32 COM preflight state was lost")
            # Interfaces the bindgen island already provides in the C `lpVtbl` idiom are NOT
            # re-rendered -- a second same-named struct is a distinct Rust type, and the
            # worker bodies see the island's through the prelude glob.
            island_slots = island_lpvtbl_interfaces(list(rustconv_target.com_island_files.values()))
            deterministic = [
                render_interface(item, island_slots.get(item.name)) for item in cpp_com_interfaces
            ]
            deterministic.append(
                render_class_scaffold(
                    cpp_com_cls,
                    cpp_com_interfaces,
                    allow_internal=getattr(args, "internal", False),
                    allow_plain=getattr(args, "plain", False),
                    island_slots=island_slots,
                    module_lock=_module_lock_of(args, rustconv_target),
                )
            )
            support_selection = "\n".join(
                [
                    _class_declaration_scope(cpp_com_cls),
                    *(dep.body_text for dep in sorted_deps),
                    *deterministic,
                ]
            )
            deterministic.append(
                render_cpp_support(
                    c_file,
                    {"IID_IUnknown"}
                    | {f"IID_{interface.name}" for interface in cpp_com_interfaces},
                    support_selection,
                    {pod.name for pod in cpp_com_cls.pod_state_types},
                    provided_types=set(local_raii_impls),
                    island_types=set(island_slots),
                    select_from_source=(
                        cpp_com_cls.header.resolve() != cpp_com_cls.source.resolve()
                    ),
                )
            )
            if cpp_com_wide_statics:
                deterministic.append(cpp_com_wide_statics)
            for raii_name, raii_rust in local_raii_impls.items():
                if raii_name in local_raii_uses:
                    deterministic.append(
                        f"// --local-raii {raii_name}: owned by {local_raii_uses[raii_name]} "
                        "in this crate; imported, not copied (workers received this API as a "
                        "sealed contract either way)\n"
                        f"use {local_raii_uses[raii_name]}::{raii_name};"
                    )
                    continue
                deterministic.append(
                    f"// --local-raii {raii_name}: hand-verified lowering (spliced verbatim; "
                    "workers received this API as a sealed contract)\n" + raii_rust.rstrip()
                )
            stitched = "\n\n".join(deterministic) + "\n\n" + stitched
        elif free_functions:
            # Free-function path: the bodies now reference CPP_COM_WSTR_* by name and
            # call into the RAII lowering, so both have to be in the module or every
            # reference dangles.
            prelude = []
            # The definitions the REWRITES introduced (`LOWORD!`, `StringCbPrintfW!`). The
            # rewrites fire on both paths but only --cpp-com ever emitted the definitions, so
            # free-functions TUs hand-wrote them — filetypes.cpp reinvented LOWORD and a printf
            # helper that way. See render_rewrite_support for why this is not the whole closure.
            support = render_rewrite_support(stitched, preamble_rust)
            if support.strip():
                prelude.append(support)
            if cpp_com_wide_statics:
                prelude.append(cpp_com_wide_statics)
            for raii_name, raii_rust in local_raii_impls.items():
                if raii_name in local_raii_uses:
                    prelude.append(
                        f"// --local-raii {raii_name}: owned by {local_raii_uses[raii_name]} "
                        "in this crate; imported, not copied (workers received this API as a "
                        "sealed contract either way)\n"
                        f"use {local_raii_uses[raii_name]}::{raii_name};"
                    )
                    continue
                prelude.append(
                    f"// --local-raii {raii_name}: hand-verified lowering (spliced verbatim; "
                    "workers received this API as a sealed contract)\n" + raii_rust.rstrip()
                )
            if prelude:
                stitched = "\n\n".join(prelude) + "\n\n" + stitched

        if wide_seals or preamble_wide_seal is not None:
            # The unsealed bodies now reference `RUSTCONV_WSTR_*` by name, so the data has
            # to be in the module or every reference dangles. Filtered by what the module
            # actually cites, and a citation with no source raises rather than reaching the
            # linker. The preamble's own seal is in the pool because a file-scope table of
            # `LPCWSTR` cites the same statics the bodies do.
            wide_sources = list(wide_seals.values())
            if preamble_wide_seal is not None:
                wide_sources.append(preamble_wide_seal)
            wide_statics = render_wide_statics(wide_sources, stitched)
            if wide_statics:
                stitched = wide_statics + "\n\n" + stitched

        # COM vtbl tables, GENERATED — never asked of the model. `translate` is
        # function-granular, so file-scope `static const IXVtbl` tables fall to the
        # one preamble call, and it drops them: all 10 of storage32's went missing
        # in the ole32 run, which is a COM object with no methods at all. Slot order
        # comes from the bindgen'd struct; an arity mismatch raises rather than
        # zipping a table into the wrong methods.
        # The module's OWN island is the best field-order source after com-sys: bindgen has
        # already expanded `DECLARE_INTERFACE_(INetCfgBindingInterface, IUnknown)` into an
        # exact `INetCfgBindingInterfaceVtbl` struct, which a C-header regex cannot do
        # reliably. Without it netcfgx died with `VtblResolveError: cannot resolve field
        # order` on interfaces that were sitting in its own types_<arch>.rs: they are
        # ReactOS-private (sdk/include/psdk/netcfgx.h), so neither com-sys nor the C file's
        # SIBLING headers — the previous default — have them.
        # Pass c_headers explicitly too: find_vtbl_tables only applies its sibling-header
        # default when BOTH are None, so supplying just rust_sources would silently drop
        # the fallback every existing caller depends on.
        _island_dir = rustconv_target.output_crate / "src"
        _islands = sorted(_island_dir.glob("types*.rs")) if _island_dir.is_dir() else []
        vtbls = find_vtbl_tables(
            c_file,
            rust_sources=_islands,
            c_headers=sorted(c_file.parent.glob("*.h")),
        )
        new_vtbls = [t for t in vtbls if not re.search(rf"\b{re.escape(t.name)}\s*:", stitched)]
        if new_vtbls:
            body = "\n\n".join(t.render(c_name=c_file.name) for t in new_vtbls)
            stitched += f"\n\n// COM vtable tables — generated from {c_file.name} by vtbl_gen.py\n"
            stitched += body + "\n"
            print(
                f"  [vtbl] generated {len(new_vtbls)} vtable table(s) "
                f"({sum(len(t.slots) for t in new_vtbls)} slots)",
                file=sys.stderr,
            )

        # Post-process
        code = postprocess(
            stitched,
            exports=file_res.exports,
            globals=scan.globals,
            target_name=rustconv_target.name,
            sys_crate=rustconv_target.sys_crate,
            sys_umbrella=rustconv_target.sys_umbrella,
            c_source_path=None if cpp_com_mode else c_file,
            com_method_macro=rustconv_target.com_method_macro,
            concrete_stdcall_slots=rustconv_target.concrete_stdcall_slots,
            crate_module=(
                getattr(rustconv_target, "crate_module", "")
                if rustconv_target.crate_bindgen_files
                else ""
            ),
        )

        # A local DEFINITION shadows a glob import, but a second GLOB does not — it
        # COLLIDES. The COM scaffold opens island-first modules with `use crate::prelude::*;`
        # while the stitcher adds shellcom/types/win32, and every name the two share
        # (REFIID, HRESULT, ULONG, ...) is then an E0659 ambiguity. Three assembled shell32
        # TUs opened with 339, 339 and 399 errors that were nothing else, and each was
        # collapsed by hand before anyone read the next one. Done here rather than in the
        # stitcher because the prelude line lives in the scaffold, which the stitcher never
        # sees; only the composed text knows both are present. The prelude is the wider and
        # newer authority, so it wins — the narrow globs go.
        #
        # The prelude line itself comes from crate_integrate, which only runs on the
        # crate-assembly path — so a `-o` slice never got one. That matters because a TU
        # holding more than one class MUST use `-o` for every slice after the first (the
        # assembler refuses to let two classes overwrite one file), which handed the slices
        # of a single TU two different substrates. CIDLDataObj.cpp is three slices and hit
        # exactly that. shell32's COM path is island-first by decision, so put every slice
        # on the prelude regardless of how it was written out.
        if cpp_class_target and (cpp_com_mode or args.free_functions):
            narrow = r"^use crate::(?:shellcom|types|win32)::\*;\n"
            # ...and the SYS-crate glob is just as narrow when the crate keeps its own island.
            # shell32 has no sys glob to collapse, so this only bites a target whose substrate
            # is `crate::types` reached through a prelude -- there the class path was importing
            # `<sys>::<umbrella>::*` while every other module in the same crate imported
            # `crate::prelude::*`, so the module's OWN generated types (recyclebin's
            # DELETED_FILE_RECORD, INFO2_HEADER, IRecycleBin5) were simply not in scope. A
            # `src/prelude.rs` is what makes a crate that shape, so key on it rather than on a
            # crate name.
            if (rustconv_target.output_crate / "src" / "prelude.rs").is_file():
                umbrella = rustconv_target.sys_umbrella
                # A FLAT substrate has no umbrella segment, and `if umbrella:` then skipped
                # the collapse entirely -- so a crate with its OWN prelude still got
                # `use <sys>::*;` and none of its island. That is menuband-rust over com-sys:
                # 71 errors for five functions, every one a shell type the prelude has and the
                # COM ABI does not. "" and None mean different things here (see
                # collapse_sys_crate): "" is a real substrate that simply has no segment.
                if umbrella is not None:
                    sys_path = (
                        f"{rustconv_target.sys_crate}::{umbrella}"
                        if umbrella
                        else rustconv_target.sys_crate
                    )
                    sys_glob = re.escape(sys_path)
                    narrow = rf"(?:{narrow}|^use {sys_glob}::\*;\n)"
            if re.search(narrow, code, re.M):
                code = re.sub(narrow, "", code, flags=re.M)
                if not re.search(r"^use crate::prelude::\*;$", code, re.M):
                    code = "use crate::prelude::*;\n" + code
        elif re.search(r"^use crate::prelude::\*;$", code, re.M):
            code = re.sub(r"^use crate::(?:shellcom|types|win32)::\*;\n", "", code, flags=re.M)

        # Cross-module names, resolved from the crate on disk rather than left to the model.
        # Runs here, on the ASSEMBLED file, so it also repairs bodies replayed from the resume
        # cache -- the prompt hint that used to be the only source of these `use` lines cannot,
        # because prompt prose is out of the cache key.
        # Resolve against the crate the file will actually LIVE IN, which `-o` can make a
        # different crate from the one `--target` names. Translating shellmenu's TUs with
        # `--target shell32 -o .../menuband-rust/src/...` indexed shell32-rust and emitted
        # `use crate::wine::shlmenu::HMENU;` into a crate that has no `wine` module — an
        # unresolvable import, generated, so unfixable in the file.
        # The materializer HOISTS a body-level `use` to file scope, so the same import written
        # inside several function bodies becomes several identical file-scope `use` lines and
        # every one after the first is `E0252: the name ... is defined multiple times`. That is
        # not a body defect — writing the import per-function is the only way to make it survive
        # materialization at all — so collapse the duplicates here.
        import_crate = _crate_of(getattr(args, "output", None)) or rustconv_target.output_crate
        if import_crate.is_dir():
            code, import_notes = resolve_in_crate_imports(code, import_crate)
            for note in import_notes:
                print(f"  [imports] {note}", file=sys.stderr)

        # AFTER the resolver, not before: it adds its own `use` lines, and a name it imports
        # singly can collide with the same name inside a brace list hoisted out of a body.
        code = _dedupe_hoisted_imports(code)

        # Per-line trailing whitespace, stripped once here so BOTH writers below get it --
        # the `-o` path writes `code` directly and the crate path hands it to the assembler.
        # A model that wraps a call argument list leaves a space after the comma, which
        # `git diff --check` flags on every commit of generated output; stripping it by hand
        # does not stick, because the next regeneration (a $0 cache replay, so it happens
        # often) puts it straight back. Indentation and blank lines are untouched.
        code = "\n".join(line.rstrip() for line in code.split("\n"))

        # --- Post-translation audits -------------------------------------
        # These catch the SILENT class: output that compiles, links and boots but
        # is wrong. A dropped struct is a compile error and finds itself; a table
        # with one wrong byte does not.
        file_findings: list[str] = []
        for name in skipped_dead:
            file_findings.append(
                f"dead branch: '{name}' is inside a preprocessor arm this tree never "
                f"compiles, so it is absent from the output ON PURPOSE — nothing is owed"
            )
        for name in skipped_by_request:
            file_findings.append(
                f"--skip-fn: '{name}' was NOT translated and is absent from the output — "
                f"hand-write it before this file is flipped"
            )
        # Wide literals the seal deliberately left to the model. Reported per occurrence
        # because "the seal was on" must never be read as "every literal was protected":
        # these are exactly the ones a reviewer still has to diff against the C.
        wide_skip_sources = sorted(wide_seals.items())
        if preamble_wide_seal is not None:
            wide_skip_sources.append(("the file-scope preamble", preamble_wide_seal))
        for name, sealed_wide in wide_skip_sources:
            for skip in sealed_wide.skipped:
                file_findings.append(
                    f"UNSEALED WIDE LITERAL in {name}: {skip.source[:60]} — {skip.reason}"
                )
        if rustconv_target.name != "shell32":
            for name in audit_preamble(c_file, preamble_rust, rust_full=code):
                file_findings.append(f"preamble: {name}")
            for d in audit_tables(c_file, code):
                if d.kind in ("missing", "count", "value"):
                    file_findings.append(f"table {d.kind}: {d.name}: {d.detail}")
                elif d.kind == "unparsed":
                    file_findings.append(f"table UNVERIFIED (read by hand): {d.name}")
        if file_findings:
            audit_findings[c_file.name] = file_findings
            print(f"\n  ⚠ {len(file_findings)} audit finding(s) in {c_file.name}:", file=sys.stderr)
            for f in file_findings[:10]:
                print(f"      {f}", file=sys.stderr)

        expected_names = {
            dep.name
            for dep in sorted_deps
            if not deterministic_only or dep.name in deterministic_workers
        }
        require_rust_definitions(
            {f"{name}_body" for name in expected_names} if cpp_com_mode else expected_names,
            code,
        )
        if args.output and total_files == 1:
            output_path = Path(args.output)
            output_path.parent.mkdir(parents=True, exist_ok=True)
            output_path.write_text(code)
            print(f"  Written to {args.output}", file=sys.stderr)
        else:
            # Assemble into crate
            fake_result = TranslationResult(
                source_file=c_file.name,
                rust_code=code,
                model=model,
                input_tokens=total_in,
                output_tokens=total_out,
                success=True,
            )
            # Name the SLICE of the TU this run covers, so a second run over the same TU
            # (another class, or the free functions) cannot silently overwrite this file.
            if getattr(args, "cpp_com", False) and getattr(args, "class_name", None):
                selector = f"class:{args.class_name}"
            elif getattr(args, "free_functions", False):
                selector = "free-functions"
            else:
                selector = "whole-tu"
            try:
                out_path = assemble_file(
                    fake_result,
                    subsystem,
                    scan.raw_source,
                    exports=file_res.exports,
                    scan=scan,
                    target=rustconv_target,
                    c_source_path=c_file,
                    selector=selector,
                    assembled_fns={dep.name for dep in sorted_deps},
                    reassemble=getattr(args, "reassemble", False),
                    expected_definitions=(
                        {f"{name}_body" for name in expected_names}
                        if cpp_com_mode
                        else expected_names
                    ),
                )
            except AssemblyClobberError as exc:
                print(f"\n\u26d4 {exc}", file=sys.stderr)
                sys.exit(1)
            print(f"  Assembled to {out_path}", file=sys.stderr)

        # Which arm of a preprocessor conditional did each body translate?
        #
        # Nothing else in the pipeline asks. The seals treat a whole `#if/#else/#endif` as
        # one dead span -- every arm's tokens optional -- so a body that renders the `#else`
        # of a conditional whose `#if` is live drops the difference silently and compiles.
        # CStartMenu's `GetMergedFolder` did exactly that against
        # `shellmenu.h:69 #define MERGE_FOLDERS 1`: the shipped translation returned the USER
        # start menu unmerged and left the entire merge tail of the function dead. The only
        # thing that ever objected was a rustc `unreachable statement` warning, a build and a
        # link after the money was spent.
        #
        # Report-only, like the free-half audit was on its first pass: a finding a reviewer
        # settles by reading two lines is worth more than a rejection that bills a retry.
        arm_pinned = sibling_header_int_macros(c_file)
        for dep in sorted_deps:
            for note in conditional_arm_notes(dep.body_text, arm_pinned):
                CONDITIONAL_ARM_BODIES.append(f"{c_file.name}:{dep.name}: {note}")

        # Respect the cost cap across files too, not just within one file.
        if capped:
            print(
                f"\n⛔ Stopping run: cost cap ${cost_cap:.2f} reached "
                f"(run total ≈ ${running_cost:.2f}). Remaining files NOT translated.",
                file=sys.stderr,
            )
            break

    # Audit summary. Printed LAST and written to disk, because the per-file
    # warnings scroll away under a long run — and these are the findings that do
    # NOT announce themselves later: a dropped struct is a compile error, but a
    # lookup table with one wrong byte compiles, links, boots, and formats a date
    # wrong forever.
    # A body accepted on its last attempt despite ignoring the fact envelope is an
    # audit finding, not a log line: its constants are unverified, and unverified
    # constants are precisely what scrolls away and ships. Folded in here so they
    # reach AUDIT.md alongside the table-count and struct findings.
    if UNVERIFIED_CONSTANT_BODIES:
        audit_findings.setdefault("UNVERIFIED CONSTANTS (fact envelope ignored)", []).extend(
            UNVERIFIED_CONSTANT_BODIES
        )

    # Same bargain, one envelope over: every UTF-16 token came back, but not in the order
    # the C used them. The closed-set check cannot tell a reorder from a mix-up, so it says
    # so here instead of guessing (wide_seal, "THE BLIND SPOT WE DO NOT INHERIT").
    if REORDERED_WIDE_LITERAL_BODIES:
        audit_findings.setdefault("WIDE LITERALS REORDERED (check each slot)", []).extend(
            REORDERED_WIDE_LITERAL_BODIES
        )

    # The `--free-functions` half, audited for the first time. Report-only: these have never
    # been graded, so the first runs are calibrating the CHECK as much as the bodies.
    if CONDITIONAL_ARM_BODIES:
        audit_findings.setdefault(
            "PREPROCESSOR ARMS (verify the body took the live one)", []
        ).extend(CONDITIONAL_ARM_BODIES)

    if FREE_HALF_AUDIT_FINDINGS:
        audit_findings.setdefault("FREE-HALF AUDIT (report-only, first pass)", []).extend(
            FREE_HALF_AUDIT_FINDINGS
        )

    # A `--skip-fn` name no file in the worklist carried. Per-file this is normal (the name
    # belongs to one TU); across the WHOLE worklist it means the name is misspelled, and the
    # caller believes a body was left to them that was in fact translated and shipped. That is
    # the inverse of the skip's own guarantee, so it is an audit finding, not a log line.
    unmatched_rebuy = sorted(rebuy_fn_requested - rebuy_fn_matched)
    if unmatched_rebuy:
        audit_findings.setdefault("--rebuy-fn NAMES THAT MATCHED NOTHING", []).extend(
            f"'{name}' was never a cache miss in this worklist (misspelled, or it replayed)"
            for name in unmatched_rebuy
        )
    unmatched_skip = unmatched_skip_fn(skip_fn_requested, skip_fn_matched)
    if unmatched_skip:
        audit_findings.setdefault("--skip-fn NAMES THAT MATCHED NOTHING", []).extend(
            f"'{name}' is in no file of this worklist — misspelled? Nothing was skipped for it, "
            "so any body you expected to hand-write was translated and SHIPPED."
            for name in unmatched_skip
        )

    if audit_findings:
        total = sum(len(v) for v in audit_findings.values())
        print(f"\n{'=' * 60}", file=sys.stderr)
        print(
            f"⚠ AUDIT: {total} finding(s) across {len(audit_findings)} file(s) — "
            "review before trusting this output",
            file=sys.stderr,
        )
        for fname, findings in audit_findings.items():
            print(f"\n  {fname}:", file=sys.stderr)
            for f in findings:
                print(f"    - {f}", file=sys.stderr)
        if archive is not None:
            report = archive.dir / "AUDIT.md"
            lines = ["# rustconv post-translation audit\n"]
            for fname, findings in audit_findings.items():
                lines.append(f"\n## {fname}\n")
                lines.extend(f"- {f}\n" for f in findings)
            report.write_text("".join(lines))
            print(f"\n  Written to {report}", file=sys.stderr)
        print(f"{'=' * 60}", file=sys.stderr)

    # Generate mod.rs and lib.rs for batch runs (ntoskrnl only — HAL handles
    # mod.rs incrementally via _hal_register_mod and has a single src/x86 dir).
    if total_files > 1 and not args.dry_run and rustconv_target.name == "ntoskrnl":
        subsystem = c_files[0].parent.name
        module_names = [f.stem for f in c_files]
        generate_mod_rs(subsystem, module_names)
        generate_lib_rs([subsystem])
        print(f"\nUpdated mod.rs and lib.rs for {subsystem}/", file=sys.stderr)

    if archive is not None:
        print(
            f"\n  Total spent this run: ${running_cost:.2f}  "
            f"({len(archive.committed)} unit(s) archived)",
            file=sys.stderr,
        )
        print(f"  Durable archive: {archive.dir}", file=sys.stderr)

    if not args.dry_run:
        print("\nRun `python -m rustconv check` to verify compilation.", file=sys.stderr)

    if getattr(args, "fn", None):
        unmatched = _unmatched_fn_names(args.fn, fn_names_found)
        if unmatched:
            print(
                f"\nERROR: --fn named function(s) found in no translated file: "
                f"{', '.join(sorted(unmatched))}. A body inside `#if 0` is invisible to the "
                f"scanner — lift the guard to translate it.",
                file=sys.stderr,
            )
            sys.exit(2)


def cmd_estimate(args):

    target = Path(args.path)
    if not target.is_absolute():
        for candidate in (REACTOS_ROOT / args.path, NTOSKRNL_DIR / args.path, HAL_DIR / args.path):
            if candidate.exists():
                target = candidate
                break
        else:
            target = NTOSKRNL_DIR / args.path

    if target.is_file():
        c_files = [target]
    elif target.is_dir():
        c_files = sorted([*target.glob("*.c"), *target.glob("*.cpp")])
    else:
        print(f"Error: not found: {target}", file=sys.stderr)
        sys.exit(1)

    rustconv_target = resolve_target(args, target)
    catalog = build_catalog(arch=args.arch, target=rustconv_target)
    spec = parse_spec(rustconv_target.spec_file) if rustconv_target.spec_file else {}

    total_input = 0
    total_lines = 0
    for cf in c_files:
        scan = scan_file(cf)
        res = resolve(scan, catalog, spec)
        prompt = build_full_prompt(scan, res, rustconv_target)
        tokens = estimate_tokens(prompt)
        lines = len(scan.raw_source.splitlines())
        total_input += tokens
        total_lines += lines
        print(f"  {cf.name}: {lines} lines, ~{tokens:,} tokens")

    # Output runs ~0.6x of input for C→Rust (Rust expands the source).
    total_output = int(total_input * 0.6)

    print(f"\nTotal: {len(c_files)} files, {total_lines:,} lines")
    print(
        f"Estimated tokens (single full-file prompt): "
        f"~{total_input:,} input, ~{total_output:,} output"
    )
    print("\n  ⚠️  NOTE: `submit`/`batch` send ONE prompt per file. But `translate`")
    print("     (the path used for big files) sends one paid call PER FUNCTION,")
    print("     each RE-SENDING the system prompt + growing context — real input")
    print("     is commonly 2–5× the figure above. These estimates are a FLOOR.")

    # DeepSeek FIRST, and flagged, because it is what an unflagged run actually bills. The
    # table used to list only Claude and Gemini, so the one number a spend decision is made
    # from was never the price of the default lane — a re-translate got priced at Sonnet's
    # $1.22 and declined as too expensive when the real bill was ~$0.10. Every other row is
    # an escalation from the first one.
    rows = [
        ("DeepSeek v4-pro  (DEFAULT)", DEEPSEEK_MODEL_V4_PRO),
        ("DeepSeek v4-flash", "deepseek-v4-flash"),
        ("Opus   (claude-opus-4-8)", "claude-opus-4-8"),
        ("Sonnet (claude-sonnet-4-5)", "claude-sonnet-4-5-20250929"),
        ("Haiku  (claude-haiku-4-5)", "claude-haiku-4-5-20251001"),
        ("Gemini 2.5 Pro", "gemini-2.5-pro"),
        ("Gemini 2.5 Flash", "gemini-2.5-flash"),
    ]
    print("\nEstimated cost (sync = translate/submit; batch = `batch` cmd, 50% off):")
    print(f"  {'model':<28} {'sync':>10}  {'batch':>10}   sync @5x")
    for label, mid in rows:
        pin, pout = MODEL_PRICING[mid]
        sync = total_input * pin / 1_000_000 + total_output * pout / 1_000_000
        batch = sync * BATCH_DISCOUNT
        print(
            f"  {label:<28} {'$' + format(sync, '.2f'):>10}  "
            f"{'$' + format(batch, '.2f'):>10}   ${sync * 5:.2f}"
        )


def cmd_objdiff(args):
    """Compare C vs Rust object files for symbol/ABI differences."""

    run_objdiff(
        arch=args.arch,
        target=args.target if args.target else None,
        verbose=args.verbose,
        critical_only=args.critical_only,
        json_output=args.json,
        baseline=args.baseline,
        deep=args.deep,
    )


def cmd_extern_audit(args):
    """Audit extern declarations vs #[no_mangle] definitions."""
    if args.fix or args.dry_run:
        run_extern_fix(dry_run=args.dry_run)
    else:
        run_extern_audit(json_output=args.json)


def cmd_body_audit(args):
    """Did each pre-seal translated body keep what its C body does? (lead generator)"""
    argv = [str(m) for m in args.modules]
    if args.all:
        argv.append("--all")
    for flag in ("json", "provenance", "show_reviewed"):
        if getattr(args, flag):
            argv.append("--" + flag.replace("_", "-"))
    argv += ["--limit", str(args.limit), "--min-severity", args.min_severity]
    return body_audit_main(argv)


def cmd_bank_audit(args):
    """Report paid translation output that never made it into the tree."""
    responses = Path(args.responses) if args.responses else RUSTCONV_RESPONSES_DIR
    rust_root = Path(args.tree) if args.tree else RUST_DIR
    if not responses.is_dir():
        raise ValueError(f"no response bank at {responses}")
    if not rust_root.is_dir():
        raise ValueError(f"no Rust tree at {rust_root}")

    reports = bank_audit(responses, rust_root)
    print(format_bank_report(reports, verbose=args.verbose))
    # Unintegrated paid work is a finding, not a status line — exit nonzero so a
    # caller that wired this into a gate actually notices.
    if any(r.absent for r in reports):
        sys.exit(1)


def cmd_trace_catalog(args):
    """Generate debugcon trace catalog from Rust source."""

    print_catalog(args.arch)
    out = generate_catalog(args.arch)
    print(f"\nWritten to {out}")


def cmd_postprocess(args):
    """Run postprocessor on existing .rs files."""

    NTOSKRNL_RUST_SRC = OUTPUT_CRATE / "src"

    target = Path(args.path)
    if not target.is_absolute():
        # Try as relative to ntoskrnl-rust/src/
        candidate = NTOSKRNL_RUST_SRC / args.path
        if candidate.exists():
            target = candidate
        elif not target.exists():
            print(f"Error: not found: {args.path}", file=sys.stderr)
            sys.exit(1)

    if target.is_file():
        rs_files = [target]
    elif target.is_dir():
        rs_files = sorted(target.glob("*.rs"))
        # Exclude mod.rs
        rs_files = [f for f in rs_files if f.name != "mod.rs"]
    else:
        print(f"Error: not found: {target}", file=sys.stderr)
        sys.exit(1)

    if not rs_files:
        print(f"No .rs files found in {target}", file=sys.stderr)
        sys.exit(1)

    total_changed = 0
    for rs_file in rs_files:
        original = rs_file.read_text()
        # detect target from path so win32k/hal/freeldr-gated passes fire
        target_name = "ntoskrnl"
        p = rs_file.as_posix()
        if "/win32k-rust/" in p:
            target_name = "win32k"
        elif "/hal-rust/" in p:
            target_name = "hal"
        elif "/freeldr-rust/" in p:
            target_name = "freeldr"
        processed = postprocess(original, target_name=target_name)

        if processed == original:
            print(f"  {rs_file.name}: unchanged", file=sys.stderr)
            continue

        total_changed += 1
        # Count what changed
        orig_lines = set(original.splitlines())
        proc_lines = set(processed.splitlines())
        removed = len(orig_lines - proc_lines)
        added = len(proc_lines - orig_lines)

        if args.dry_run:
            print(
                f"  {rs_file.name}: would change (~{removed} removed, ~{added} added)",
                file=sys.stderr,
            )
        else:
            rs_file.write_text(processed)
            print(
                f"  {rs_file.name}: updated (~{removed} removed, ~{added} added)", file=sys.stderr
            )

    verb = "would change" if args.dry_run else "changed"
    print(f"\n{total_changed}/{len(rs_files)} files {verb}", file=sys.stderr)


def main():
    parser = argparse.ArgumentParser(
        prog="rustconv",
        description="C-to-Rust translation pipeline for ReactOS ntoskrnl",
    )
    parser.add_argument(
        "--arch",
        default="arm64",
        choices=["arm64", "amd64", "i386"],
        help="Architecture for catalog/type resolution",
    )

    sub = parser.add_subparsers(dest="command", required=True)

    # scan
    p = sub.add_parser("scan", help="Parse C file, print analysis")
    p.add_argument("file", help="C source file (relative to ntoskrnl/ or absolute)")

    # dup-census — read-only opportunity measurement; never reuses output.
    p = sub.add_parser(
        "dup-census",
        help="Report exact and alpha-normalized duplicate C/C++ function bodies",
    )
    p.add_argument("paths", nargs="+", help="Source file/directory scopes relative to repo root")
    p.add_argument("--top", type=int, default=20, help="Largest groups to include (default 20)")
    p.add_argument("--json", action="store_true", help="Machine-readable report")

    p = sub.add_parser(
        "shell32-tail-inventory",
        help="Check the zero-C++ lane ledger against shell32's live CMake source lists",
    )
    p.add_argument("--ledger", help="Override the shell32 tail TOML ledger")
    p.add_argument("--json", action="store_true", help="Machine-readable report")

    hybrid_closure.build_arg_parser(
        sub.add_parser(
            "hybrid-closure",
            help="Which symbols of a hybrid TU are still needed elsewhere, measured from the link",
        )
    )

    # candidate-check — N isolated worker replacements, one cargo check per arch.
    p = sub.add_parser(
        "candidate-check",
        help="Batch cargo-BUILD candidate workers against the real shell32 substrate",
    )
    p.add_argument("manifest", help="JSON candidate/base/function manifest")
    p.add_argument(
        "--arches",
        nargs="+",
        choices=["i386", "amd64", "arm64"],
        help="Architectures to check (default all three)",
    )
    p.add_argument("--json", action="store_true", help="Machine-readable report")
    p.add_argument(
        "--repair",
        action="store_true",
        help="Apply only deterministic base-authority worker repairs before checking",
    )
    p.add_argument(
        "--check-only",
        dest="check_only",
        action="store_true",
        help="Use `cargo check` instead of `cargo build` — FASTER but WEAKER: check "
        "stops before codegen, so the deny-by-default unconditional_panic lint never "
        "runs and a constant out-of-bounds index (the bindgen [T; 1] flex-array shape) "
        "passes silently. Iteration only; never for acceptance.",
    )

    # difftest-gen — derive a differential harness from the C's own signatures.
    p = sub.add_parser(
        "difftest-gen",
        help="Generate a differential test harness for a translated C file (no LLM)",
    )
    p.add_argument("c_file", help="the translated C source, e.g. dll/win32/msvcrt/mbcs.c")
    p.add_argument("--out", help="write the cases here (default: stdout)")

    p = sub.add_parser(
        "difftest-contract",
        help="Generate and run a reviewed scalar/byte-buffer differential contract (no LLM)",
    )
    p.add_argument("contract", help="JSON contract with genuine C and existing Rust source paths")
    p.add_argument("--out", required=True, help="fresh durable artifact directory")
    p.add_argument("--json", action="store_true", help="print the full measurement report")

    # crate-repair — deterministic Stage 1.5. Closes the MECHANICAL rustc roots of a freshly
    # assembled crate for $0, before any paid repair pass, and leaves a bucketed RED residual.
    p = sub.add_parser(
        "crate-repair",
        help="Bounded deterministic repair of a whole-crate cargo check (no LLM)",
    )
    p.add_argument("crate", help="cargo package name, e.g. wkssvc-rust")
    p.add_argument(
        "--arches",
        nargs="+",
        default=["i386", "amd64", "arm64"],
        choices=["i386", "amd64", "arm64"],
        help="Targets to analyse. >=2 required to apply a pointer-width-dependent cast",
    )
    p.add_argument(
        "--rules",
        nargs="+",
        default=list(crate_repair.ALL_RULES),
        choices=list(crate_repair.ALL_RULES),
    )
    p.add_argument("--crate-dir", help="Crate root (default <rust-workspace>/<crate>)")
    p.add_argument("--types-module", default="types.rs", help="bindgen types module in src/")
    p.add_argument("--max-passes", type=int, default=6)
    p.add_argument("--dry-run", action="store_true", help="Report edits without writing")
    p.add_argument(
        "--allow-single-target-width",
        action="store_true",
        help="LOUD override: bake a pointer-width cast proven on ONE target only",
    )
    p.add_argument("--json", action="store_true", help="Machine-readable ledger")

    p = sub.add_parser(
        "substrate-plan",
        help="Cluster a candidate oracle report into canonical substrate authority roots",
    )
    p.add_argument("report", help="candidate-oracle report.json")
    p.add_argument("--json", action="store_true", help="Machine-readable authority plan")
    p.add_argument("--limit", type=int, default=25, help="Roots to print (default 25)")
    p.add_argument(
        "--refresh",
        action="store_true",
        help="Recompute and update substrate_analysis in the durable report",
    )

    p = sub.add_parser(
        "skeleton-check",
        help="Compare C++/Rust control and mutation AST skeletons (experimental, non-gating)",
    )
    p.add_argument("manifest", help="JSON routing corpus manifest")
    p.add_argument("--json", action="store_true", help="Machine-readable report")

    # cpp-com — deterministic C++/ATL class ABI preflight + scaffold. No LLM.
    p = sub.add_parser(
        "cpp-com",
        help="Generate one ATL COM class's interfaces/vtables/factory scaffold deterministically",
    )
    p.add_argument("path", help="C++ implementation file")
    p.add_argument("--class", dest="class_name", required=True, help="C++ class name")
    p.add_argument(
        "--sdk-header",
        help="WIDL-generated C header containing interface Vtbl structs (default build-<arch>)",
    )
    p.add_argument(
        "--object-map",
        help="C++ source containing OBJECT_ENTRY registrations (default shell32.cpp)",
    )
    p.add_argument("-o", "--output", help="Write combined deterministic Rust scaffold")
    p.add_argument("--adapter-output", help="Write the deterministic C++ object-map adapter")
    p.add_argument(
        "--kernel-owned",
        action="store_true",
        help="Emit Rust storage/lifetime ownership for one CUnknownImpl interface",
    )
    p.add_argument("--kernel-base", help="Source header defining the atomic CUnknownImpl base")
    p.add_argument(
        "--kernel-bridge", help="Compiler-validated bridge header for retained method workers"
    )
    p.add_argument(
        "--compiler-owned",
        action="store_true",
        help="Keep C++ object mechanics; emit C reference workers and compiler thunks",
    )
    p.add_argument(
        "--bridge-methods", help="Comma-separated ordinary methods for compiler-owned glue"
    )
    p.add_argument(
        "--bridge-include",
        action="append",
        help="Real C-compatible SDK header for bridge declarations",
    )
    p.add_argument("--bridge-macro-header", help="Source header containing exact reference macros")
    p.add_argument(
        "--bridge-macro", action="append", help="Macro to copy verbatim from --bridge-macro-header"
    )

    p.add_argument(
        "--internal",
        action="store_true",
        help="Allow exact internal ShellObjectCreator<T> activation; never emits an adapter",
    )
    p.add_argument("--json", action="store_true", help="Print the proven class/interface IR")
    p.add_argument(
        "--plain",
        action="store_true",
        help="The class is a file-local C++ helper, NOT COM: no vtable, no QI, no activation. "
        "Skips the three COM preflight requirements (which are vacuous for it) and keeps every "
        "state-layout/RAII check. Refuses if the class actually has a BEGIN_COM_MAP.",
    )

    p = sub.add_parser(
        "cpp-tu",
        help="Preflight every class method and free function in one shell32 C++ TU",
    )
    p.add_argument("path", help="C++ implementation file")
    p.add_argument("--json", action="store_true", help="Machine-readable ownership/layout IR")

    # semdiff (C↔Rust semantic-divergence finder; from cdrom-port f29579d1119)
    p = sub.add_parser("semdiff", help="Find C↔Rust semantic/behavioral divergences (LLM)")
    p.add_argument("file", help="C source file (.c)")
    p.add_argument("--rust", default=None, help="Rust file (.rs); auto-located by stem if omitted")
    p.add_argument("--model", default="deepseek", help="Model (default deepseek-v4-pro)")
    p.add_argument("--only", default=None, help="Comma-separated function names to diff")
    p.add_argument("--max-fns", type=int, default=None, help="Cap number of functions diffed")
    p.add_argument("--no-cache", action="store_true", help="Ignore per-pair response cache")
    p.add_argument("--workers", type=int, default=6, help="Concurrent API calls (default 6)")

    # resolve
    p = sub.add_parser("resolve", help="Resolve dependencies, print import manifest")
    p.add_argument("file", help="C source file")

    # prompt
    p = sub.add_parser("prompt", help="Generate translation prompt")
    p.add_argument("file", help="C source file")
    p.add_argument("--estimate", action="store_true", help="Print token estimate instead of prompt")

    # submit
    p = sub.add_parser("submit", help="Submit single file for translation")
    p.add_argument(
        "--unsafe-unprotected",
        action="store_true",
        help="Explicitly use the legacy paid path without per-function protection",
    )
    p.add_argument("file", help="C source file")
    p.add_argument("-o", "--output", help="Write Rust output to file")
    p.add_argument("--assemble", action="store_true", help="Assemble into crate structure")
    p.add_argument(
        "--target", help="Override target: a known target name or a dll/win32/<name> crate name"
    )
    p.add_argument(
        "-m", "--model", help="Model: sonnet, haiku, opus, gemini-pro, gemini-flash, or full ID"
    )

    # batch
    p.add_argument(
        "--allow-partial",
        dest="allow_partial",
        action="store_true",
        help="Proceed when C++ bodies are unextractable (free-function subset only)",
    )
    p = sub.add_parser("batch", help="Batch-submit entire subsystem directory")
    p.add_argument(
        "--unsafe-unprotected",
        action="store_true",
        help="Explicitly use the legacy paid path without per-function protection",
    )
    p.add_argument("directory", help="Subsystem directory (relative to ntoskrnl/ or absolute)")
    p.add_argument(
        "-m", "--model", help="Model: sonnet, haiku, opus, gemini-pro, gemini-flash, or full ID"
    )
    p.add_argument(
        "--exclude", nargs="*", default=[], help="File stems to skip (e.g., obhandle ob_rust_shims)"
    )
    p.add_argument(
        "--allow-partial",
        dest="allow_partial",
        action="store_true",
        help="Proceed when C++ bodies are unextractable (free-function subset only)",
    )
    p.add_argument(
        "-s",
        "--subsystem",
        default=None,
        help="Override subsystem name (e.g., ke/i386 for nested arch dirs)",
    )

    # check
    sub.add_parser("check", help="Run cargo check on all 3 targets")

    # shims
    p = sub.add_parser("shims", help="Generate C shim file from TOML manifest")
    p.add_argument("subsystem", help="Subsystem name (e.g., ob, ex)")
    p.add_argument("-o", "--output", help="Write output to file")
    p.add_argument(
        "--write",
        action="store_true",
        help="Write directly to ntoskrnl/{subsys}/{subsys}_rust_shims.c",
    )
    p.add_argument("--check", action="store_true", help="Check if generated file matches existing")

    # estimate
    p = sub.add_parser("estimate", help="Estimate token count and cost")
    p.add_argument("path", help="File or directory to estimate")
    # Same three knobs as `translate`. estimate resolves the target exactly as
    # translate does, so a module whose crate cannot be guessed (msvcrt ->
    # msvcrt-crt-rust) raised TargetResolveError here with no flag to answer it —
    # the one command whose entire job is to price a run refused to price the
    # modules most in need of pricing.
    p.add_argument(
        "--target", help="Override target: a known target name or a dll/win32/<name> crate name"
    )
    p.add_argument(
        "--output-crate",
        help="Rust crate the translation is assembled into, when it is not the "
        "conventional <target>-rust (e.g. msvcrt -> msvcrt-crt-rust).",
    )
    p.add_argument(
        "--sys-crate",
        help="FFI substrate crate to translate against (e.g. ntdll-sys, msvcrt-sys), "
        "when the output crate declares none or several.",
    )

    # seh-classify
    p = sub.add_parser("seh-classify", help="Classify SEH blocks in C files by pattern")
    p.add_argument("path", help="C file or directory (relative to ntoskrnl/ or absolute)")
    p.add_argument("-v", "--verbose", action="store_true", help="Show per-function details")
    p.add_argument("--json", action="store_true", help="Output JSON")

    # seh-inventory (win32k Pass 0 triage)
    p = sub.add_parser("seh-inventory", help="Win32k Pass 0: triage SEH sites a/b/c/d/e + sidecars")
    p.add_argument(
        "path",
        nargs="?",
        default="win32ss",
        help="C file or dir (relative to repo root or absolute); default win32ss/ keep-set",
    )
    p.add_argument("--json", action="store_true", help="Output JSON (file mode)")
    p.add_argument(
        "--write-sidecars",
        action="store_true",
        help="Write <file>.win32k_meta.json next to each source",
    )
    p.add_argument(
        "--oracle", action="store_true", help="Run the winsta.c spike self-test and exit"
    )

    # seh-rewrite (win32k Pass 1 mechanical probe lowering)
    p = sub.add_parser(
        "seh-rewrite",
        help="Win32k Pass 1: mechanically lower category-(a) probe SEH sites to SafeProbeFor*",
    )
    p.add_argument(
        "path",
        nargs="?",
        default="win32ss",
        help="C file or dir (relative to repo root or absolute)",
    )
    p.add_argument(
        "--apply", action="store_true", help="Write changes in place (default: print unified diff)"
    )
    p.add_argument(
        "--oracle", action="store_true", help="Run the winsta.c spike self-test and exit"
    )

    # seh-belt (win32k Pass 1c: delete (d) BELT_SUSPENDERS SEH wrappers)
    p = sub.add_parser(
        "seh-belt",
        help="Win32k Pass 1c: delete category-(d) BELT_SUSPENDERS "
        "SEH wrappers (user write, no probe, dead handler "
        "under DUMMY_PSEH)",
    )
    p.add_argument(
        "path",
        nargs="?",
        default="win32ss",
        help="C file or dir (relative to repo root or absolute)",
    )
    p.add_argument(
        "--apply", action="store_true", help="Write changes in place (default: print unified diff)"
    )
    p.add_argument("--oracle", action="store_true", help="Run the synthetic self-test and exit")

    # callout-annotate (win32k Pass 3 USER callout extraction — no rewrite)
    p = sub.add_parser(
        "callout-annotate",
        help="Win32k Pass 3: extract KeUserModeCallback "
        "callout sites + sidecars (no source rewrite)",
    )
    p.add_argument(
        "path",
        nargs="?",
        default="win32ss",
        help="C file or dir (relative to repo root or absolute); default win32ss/ keep-set",
    )
    p.add_argument("--json", action="store_true", help="Output JSON (file mode)")
    p.add_argument(
        "--write-sidecars",
        action="store_true",
        help="Write <file>.win32k_callouts.json next to each source",
    )
    p.add_argument(
        "--oracle", action="store_true", help="Run the callback.c ground-truth self-test and exit"
    )

    # seh-llm-lower (win32k Pass 1b: LLM lowering of the 64-site ledger)
    p = sub.add_parser(
        "seh-llm-lower",
        help="Win32k Pass 1b: LLM-lower the Pass-1 manual-followup ledger (cached, validated)",
    )
    p.add_argument(
        "--apply", action="store_true", help="Splice validated rewrites in place (default: dry-run)"
    )
    p.add_argument("--diff", action="store_true", help="Print unified diffs for the ready rewrites")
    p.add_argument("--model", default="", help="Model id/shortcut (default: Sonnet)")
    p.add_argument("--only", default=None, help="Restrict to ledger files whose path contains this")
    p.add_argument(
        "--oracle", action="store_true", help="Structural self-test (no API spend) and exit"
    )

    # split-plan (win32k giant-file split analyzer — analyze-only)
    p = sub.add_parser(
        "split-plan",
        help="Win32k: analyze giant .c for safe split seams (static-coupling graph; no mutation)",
    )
    p.add_argument(
        "path",
        nargs="?",
        default="win32ss",
        help="C file or dir (default win32ss/ translation scope)",
    )
    p.add_argument(
        "--min-loc",
        type=int,
        default=1000,
        help="Only consider files with ≥ this LOC (default 1000)",
    )
    p.add_argument(
        "--target-loc",
        type=int,
        default=900,
        help="Target per-group LOC for the split (default 900)",
    )
    p.add_argument("--json", action="store_true", help="Output JSON")

    # switch-extract (win32k tree-sitter switch-dispatcher de-monstering)
    p = sub.add_parser(
        "switch-extract",
        help="Win32k: extract proven-safe switch arms of a giant dispatcher fn into static helpers",
    )
    p.add_argument("path", help="C file (relative to repo root or absolute)")
    p.add_argument("--function", required=True, help="Target function name (e.g. SpiGetSet)")
    p.add_argument(
        "--apply", action="store_true", help="Write the transform (default: dry-run report)"
    )
    p.add_argument(
        "--oracle", action="store_true", help="Self-test (label-set + reparse invariants) and exit"
    )

    # split-apply (win32k giant-file #include-back emitter)
    p = sub.add_parser(
        "split-apply",
        help="Win32k: emit the #include-back split for CLEAN_SPLIT files (one TU, no CMake change)",
    )
    p.add_argument(
        "path", nargs="?", default=None, help="One C file; omit to do ALL CLEAN_SPLIT in scope"
    )
    p.add_argument(
        "--apply", action="store_true", help="Write fragments + rewrite base (default: dry-run)"
    )
    p.add_argument("--min-loc", type=int, default=1000)
    p.add_argument("--target-loc", type=int, default=900)

    # gdi-locks (win32k Pass 2 GDI manual-lock-bracket detection — no rewrite)
    p = sub.add_parser(
        "gdi-locks",
        help="Win32k Pass 2: detect GDI manual lock brackets + sidecars (no source rewrite)",
    )
    p.add_argument(
        "path",
        nargs="?",
        default="win32ss",
        help="C file or dir (relative to repo root or absolute); default win32ss/ keep-set",
    )
    p.add_argument("--json", action="store_true", help="Output JSON (file mode)")
    p.add_argument(
        "--write-sidecars",
        action="store_true",
        help="Write <file>.win32k_gdilocks.json next to each source",
    )
    p.add_argument(
        "--oracle", action="store_true", help="Run the dcutil.c ground-truth self-test and exit"
    )

    # user-locks (win32k Pass 2b USER critsec-bracket detection — no rewrite)
    p = sub.add_parser(
        "user-locks",
        help="Win32k Pass 2b: detect USER critical-section brackets + sidecars (no source rewrite)",
    )
    p.add_argument(
        "path",
        nargs="?",
        default="win32ss",
        help="C file or dir (relative to repo root or absolute); default win32ss/ keep-set",
    )
    p.add_argument("--json", action="store_true", help="Output JSON (file mode)")
    p.add_argument(
        "--write-sidecars",
        action="store_true",
        help="Write <file>.win32k_userlocks.json next to each source",
    )
    p.add_argument(
        "--oracle", action="store_true", help="Run the simplecall.c ground-truth self-test and exit"
    )

    # translate (function-by-function)
    p = sub.add_parser("translate", help="Function-by-function translation (improved pipeline)")
    p.add_argument("path", help="C file or directory (relative to ntoskrnl/ or absolute)")
    p.add_argument(
        "--target", help="Override target: a known target name or a dll/win32/<name> crate name"
    )
    p.add_argument(
        "--output-crate",
        help="Rust crate the translation is assembled into, when it is not the "
        "conventional <target>-rust (e.g. msvcrt -> msvcrt-crt-rust). Named by the "
        "TargetResolveError that refuses to guess.",
    )
    p.add_argument(
        "--sys-crate",
        help="FFI substrate crate to translate against (e.g. ntdll-sys, msvcrt-sys), when "
        "the output crate declares none or several.",
    )
    p.add_argument(
        "-m",
        "--model",
        help="Model: sonnet, haiku, opus, gemini-pro, gemini-flash, deepseek, "
        "deepseek-r1, or full ID",
    )
    p.add_argument(
        "--mode",
        default="idiomatic",
        choices=["idiomatic", "mechanical"],
        help="idiomatic (default): translate + idiomize in one pass. "
        "mechanical: faithful transliteration only — compiles, "
        "preserves C control flow/pointers, no idiom, no stubs "
        "(pass 1 of a two-pass cheap-model pipeline).",
    )
    p.add_argument("-o", "--output", help="Write Rust output to file (single file only)")
    p.add_argument(
        "--allow-partial",
        dest="allow_partial",
        action="store_true",
        help="Proceed when C++ bodies are unextractable (free-function subset only)",
    )
    p.add_argument(
        "--cpp-com",
        action="store_true",
        help="Translate one shell32 C++ COM class after deterministic ABI preflight",
    )
    p.add_argument("--class", dest="class_name", help="Class selected by --cpp-com")
    p.add_argument(
        "--free-functions",
        dest="free_functions",
        action="store_true",
        help="shell32: translate the TU's FREE functions (no class, no vtable) through "
        "the ordinary sealed per-function path. Class methods are skipped and listed by "
        "name — they still need --cpp-com --class. Mutually exclusive with --cpp-com.",
    )
    p.add_argument(
        "--internal",
        action="store_true",
        help="shell32 cpp-com: allow exact internal ShellObjectCreator<T> activation",
    )
    p.add_argument(
        "--activation-source",
        help=(
            "Another .cpp that CONSTRUCTS this class, when the construction is not in the "
            "class's own TU (e.g. CFolderItems.cpp does `new CComObject<CFolderItemVerbs>()` "
            "for a class defined in CFolderItemVerbs.cpp). Scanned for internal-activation "
            "shapes only; it never supplies initializer arguments."
        ),
    )
    p.add_argument(
        "--plain",
        action="store_true",
        help="shell32 cpp-com: the class is a file-local C++ HELPER, not COM — no vtable, no "
        "QI, no activation. Use with --cpp-com --class for a class --free-functions skips but "
        "the COM preflight refuses. Refuses if the class actually has a BEGIN_COM_MAP.",
    )
    p.add_argument(
        "--seal-facts",
        dest="seal_facts",
        action=argparse.BooleanOptionalAction,
        default=True,
        help="Replace each constant occurrence with an audited opaque token during the "
        "paid call, so the model never copies a value (and lower HRESULT predicates). "
        "Works for plain C and for shell32 --cpp-com; the C dialect additionally protects "
        "CamelCase DDK enum constants. Strongly recommended: leaving this off is how 61 "
        "invented kernel bugcheck codes shipped (field manual 9.113).",
    )
    p.add_argument(
        "--seal-calls",
        dest="seal_calls",
        action=argparse.BooleanOptionalAction,
        default=True,
        help="Replace each known runtime callee occurrence with an audited opaque token "
        "during the paid call, so a dropped, duplicated, or invented call fails loudly "
        "instead of silently. Works for plain C and for shell32 --cpp-com.",
    )
    p.add_argument(
        "--seal-wide",
        dest="seal_wide",
        action=argparse.BooleanOptionalAction,
        default=True,
        help="Replace each `L\"...\"` / `L'x'` occurrence with an audited opaque token during "
        "the paid call, and substitute deterministic UTF-16 afterward. Rust has no wide "
        "literal, so a model asked for one INVENTS an encoding — a byte string behind a wide "
        "pointer, an alignment-1 interleave, or a hand-counted [u16; N] missing its "
        "terminator; five modules shipped one of those. Plain C only: shell32 --cpp-com has "
        "its own frozen lowering. 98%% of the tree's literals seal; the rest are reported as "
        "UNSEALED WIDE LITERAL audit findings rather than passed off as protected.",
    )
    p.add_argument(
        "--retry-rejects",
        dest="retry_rejects",
        type=int,
        default=0,
        metavar="N",
        help="Re-roll a worker that fails the deterministic audit, up to N extra PAID "
        "calls (default 0). One rejected worker assembles no file at all, so a single "
        "token slip in one function throws away every other function in the TU; the "
        "survivors after the deterministic fixes are sampling noise on the biggest "
        "bodies. Each retry is billed and applies the identical acceptance checks.",
    )
    p.add_argument(
        "--rebuy-rejected",
        action="store_true",
        help="Re-buy functions whose only banked state is a body the deterministic audit "
        "REJECTED (inspect the reject first: suspect the audit before the model). Accepted "
        "bodies are never re-bought. `rustconv port translate` uses it to retry a TU whose "
        "parallel run ended with rejects.",
    )
    p.add_argument(
        "--sdk-header",
        help="WIDL-generated C header used as COM vtable ground truth",
    )
    p.add_argument(
        "--local-raii",
        dest="local_raii",
        action="append",
        default=[],
        metavar="NAME=PATH",
        help="shell32 COM mode: hand-verified Rust lowering for a file-local C++ RAII "
        "struct (e.g. CDrop=tools/rustconv/provided_raii/CDrop.rs). Lifts the "
        "pre-payment reject for NAME and seals its Rust API into each affected "
        "worker prompt. Repeatable.",
    )
    p.add_argument(
        "--fn",
        dest="fn",
        help="Translate only this function, or a comma-separated bounded worker set",
    )
    p.add_argument(
        "--skip-fn",
        dest="skip_fn",
        # `extend`, not the bare `nargs="*"` this used to be: with plain nargs a REPEATED
        # `--skip-fn a --skip-fn b` silently keeps only `b`, so ten of eleven names asked
        # for were dropped with no message. That is the one failure this flag is not
        # allowed to have -- its whole contract below is "never silent".
        action="extend",
        nargs="*",
        default=[],
        metavar="NAME",
        help=(
            "FUNCTION names to leave out of the worklist so the rest of the TU still "
            "assembles (--exclude filters FILE stems and cannot do this). For a body the "
            "model cannot get past the call/fact envelope: skip it, ship the other "
            "functions, hand-write this one. Never silent — each name becomes an audit "
            "finding, and an unknown name is an error rather than a no-op. Repeatable, "
            "and accepts several names per flag."
        ),
    )
    p.add_argument(
        "--rebuy-fn",
        dest="rebuy_fn",
        action="extend",
        nargs="*",
        default=[],
        metavar="NAME",
        help=(
            "FUNCTION names whose ACCEPTED banked body may be bought again, e.g. after a seal "
            "got stricter and the banked body no longer replays. Named, never inferred; a name "
            "that was never a cache miss is reported. Repeatable."
        ),
    )
    p.add_argument(
        "--reassemble",
        action="store_true",
        help="Allow overwriting a COMMITTED, unmodified .rs. Off by default: a re-translation "
        "sees only the C, so it reverts every repair made after integration. Also waives the "
        "pre-payment refusal when the output crate already ships a #[no_mangle] port of this TU.",
    )
    p.add_argument("--dry-run", action="store_true", help="Show translation plan without API calls")
    p.add_argument(
        "--prompt", action="store_true", help="Print prompt for first function (implies --dry-run)"
    )
    p.add_argument("--exclude", nargs="*", default=[], help="File stems to skip (batch mode)")
    p.add_argument(
        "--cost-cap",
        dest="cost_cap",
        type=float,
        default=None,
        help="Hard USD spend cap for this invocation; stops before the next "
        "call once cumulative cost reaches it (safety brake).",
    )
    p.add_argument(
        "--concurrency",
        type=int,
        default=1,
        help="Fan out per-function calls N-at-a-time (DeepSeek + --mode "
        "mechanical ONLY; ignored otherwise). Default 1 (sequential). "
        "Try 5; pair with --cost-cap as the runaway brake.",
    )
    p.add_argument(
        "--deterministic-only",
        dest="deterministic_only",
        action="store_true",
        help="Assemble ONLY the functions that lower deterministically ($0, no API "
        "call at all) and skip every paid one. The free band on a CRT file is "
        "large (mbcs.c: 76 of 192) and is proven rather than reviewed, so it can "
        "be integrated and gated on its own before deciding to pay for the rest.",
    )
    p.add_argument(
        "--no-resume",
        dest="no_resume",
        action="store_true",
        help="Ignore the .responses/ resume cache and re-translate every "
        "function fresh (cache still refreshed). Default resumes "
        "unchanged-body functions for $0.",
    )
    p.add_argument(
        "--replay-only",
        action="store_true",
        help="Reassemble cached/recoverable workers for $0; fail before any new paid call",
    )
    p.add_argument(
        "--cache-tag",
        default=None,
        help="Append an isolated namespace tag to the response-cache key. Intended for "
        "controlled model/prompt experiments that must not overwrite production entries.",
    )
    p.add_argument(
        "--c2rust",
        action=argparse.BooleanOptionalAction,
        default=None,
        help="Require c2rust anchors for every selected function (default on for C). "
        "Missing tools, build inputs or function anchors fail before payment. "
        "C++ uses its separate class preflight; c2rust cannot anchor C++. "
        "--no-c2rust explicitly opts out of C anchoring.",
    )

    # probe-migrate
    p = sub.add_parser(
        "probe-migrate", help="Migrate Rust files from ProbeForRead/Write to probe::*"
    )
    p.add_argument(
        "path", help="Rust file or directory (relative to ntoskrnl-rust/src/ or absolute)"
    )
    p.add_argument(
        "-m", "--model", help="Model: sonnet, haiku, opus, gemini-pro, gemini-flash, or full ID"
    )
    p.add_argument(
        "--dry-run", action="store_true", help="Don't write, just show what would change"
    )
    p.add_argument(
        "--scan-only", action="store_true", help="Only scan for probe calls, don't migrate"
    )
    p.add_argument(
        "--prompt", action="store_true", help="Print the prompt for the first file and exit"
    )

    # objdiff
    p = sub.add_parser("objdiff", help="Compare C vs Rust object files for ABI differences")
    p.add_argument(
        "target",
        nargs="?",
        default=None,
        help="Subsystem (ex/) or file (ex/atom) to compare (default: all)",
    )
    p.add_argument("--json", action="store_true", help="JSON output for CI")
    p.add_argument("-v", "--verbose", action="store_true", help="Per-function details")
    p.add_argument("--critical-only", action="store_true", help="Only show critical issues")
    p.add_argument(
        "--baseline",
        default=None,
        metavar="COMMIT",
        help="Git commit to extract C files from (compare Rust vs historical C)",
    )
    p.add_argument(
        "--deep",
        action="store_true",
        help="Analyze function prologues (stack frame size, callee-saved regs)",
    )

    # extern-audit
    p = sub.add_parser(
        "extern-audit", help="Audit extern declarations vs definitions for convention mismatches"
    )
    p.add_argument("--json", action="store_true", help="JSON output")
    p.add_argument("--fix", action="store_true", help="Fix critical mismatches automatically")
    p.add_argument(
        "--dry-run", action="store_true", help="Show what --fix would change without writing"
    )

    # body-audit
    p = sub.add_parser(
        "body-audit",
        help="Per function: C callees a translated Rust body no longer performs",
    )
    p.add_argument(
        "modules", nargs="*", help="module source dirs, e.g. drivers/filesystems/fastfat"
    )
    p.add_argument("--all", action="store_true", help="every module that links a Rust crate")
    p.add_argument("--json", action="store_true", help="JSON output")
    p.add_argument("--limit", type=int, default=0, help="at most N findings per module")
    p.add_argument(
        "--min-severity",
        default="stub-loud",
        choices=("stub-silent", "high", "medium", "low", "stub-loud"),
        help="hide findings ranked below this",
    )
    p.add_argument(
        "--provenance",
        action="store_true",
        help="mark dropped callees the C gained after the Rust was committed (git)",
    )
    p.add_argument(
        "--show-reviewed",
        action="store_true",
        help="also list leads data/body_audit_reviewed.toml marks false/benign",
    )

    # bank-audit
    p = sub.add_parser(
        "bank-audit",
        help="Report paid translation output (.responses) that is not in the tree",
    )
    p.add_argument("--responses", help="Response bank root (default: tools/rustconv/.responses)")
    p.add_argument("--tree", help="Rust crate root to check against (default: sdk/lib/rust)")
    p.add_argument(
        "--verbose", action="store_true", help="Also list functions resolved via an alias rule"
    )

    # pp (postprocess existing .rs files)
    p = sub.add_parser("pp", help="Run postprocessor on existing .rs files")
    p.add_argument("path", help="Rust file or directory (subsystem under ntoskrnl-rust/src/)")
    p.add_argument("--dry-run", action="store_true", help="Show changes without writing")

    # trace-catalog
    p = sub.add_parser("trace-catalog", help="Generate debugcon trace catalog from Rust source")
    p.add_argument("--arch", default="i386", choices=["i386", "amd64", "arm64"])

    port_mod.add_parser(sub)

    args = parser.parse_args()

    commands = {
        "scan": cmd_scan,
        "dup-census": cmd_dup_census,
        "shell32-tail-inventory": cmd_shell32_tail_inventory,
        "hybrid-closure": cmd_hybrid_closure,
        "candidate-check": cmd_candidate_check,
        "crate-repair": cmd_crate_repair,
        "difftest-gen": cmd_difftest_gen,
        "difftest-contract": cmd_difftest_contract,
        "substrate-plan": cmd_substrate_plan,
        "skeleton-check": cmd_skeleton_check,
        "cpp-com": cmd_cpp_com,
        "cpp-tu": cmd_cpp_tu,
        "resolve": cmd_resolve,
        "semdiff": cmd_semdiff,
        "prompt": cmd_prompt,
        "submit": cmd_submit,
        "batch": cmd_batch,
        "check": cmd_check,
        "shims": cmd_shims,
        "estimate": cmd_estimate,
        "translate": cmd_translate,
        "seh-classify": cmd_seh_classify,
        "seh-inventory": cmd_seh_inventory,
        "seh-rewrite": cmd_seh_rewrite,
        "seh-belt": cmd_seh_belt,
        "callout-annotate": cmd_callout_annotate,
        "gdi-locks": cmd_gdi_locks,
        "user-locks": cmd_user_locks,
        "seh-llm-lower": cmd_seh_llm_lower,
        "split-plan": cmd_split_plan,
        "split-apply": cmd_split_apply,
        "switch-extract": cmd_switch_extract,
        "probe-migrate": cmd_probe_migrate,
        "objdiff": cmd_objdiff,
        "extern-audit": cmd_extern_audit,
        "bank-audit": cmd_bank_audit,
        "body-audit": cmd_body_audit,
        "pp": cmd_postprocess,
        "trace-catalog": cmd_trace_catalog,
        "port": lambda a: sys.exit(port_mod.main(a)),
    }

    try:
        commands[args.command](args)
    except (ValueError, CppComError, ProfileDeriveError, crate_repair.RepairError) as e:
        # target auto-detect failure (unrecognized path) surfaces here as a clean
        # message instead of a traceback — tells the user to pass --target.
        print(str(e), file=sys.stderr)
        sys.exit(2)


if __name__ == "__main__":
    main()
