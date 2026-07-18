"""CLI entry point for rustconv."""

import argparse
import collections
import difflib
import json
import re
import sys
import threading
import time
from collections.abc import Callable
from concurrent.futures import ThreadPoolExecutor, as_completed
from dataclasses import replace
from pathlib import Path

from . import (
    callout_annotate,
    candidate_oracle,
    dup_census,
    file_splitter,
    gdi_lock_inventory,
    seh_belt,
    seh_inventory,
    seh_llm_lower,
    seh_rewrite,
    semantic_skeleton,
    semdiff,
    substrate_closure,
    switch_extract,
    user_lock_inventory,
)
from .assembler import assemble_file, generate_lib_rs, generate_mod_rs
from .batch_runner import TranslationResult, submit_batch, submit_single
from .c2rust_anchor import C2RustError, build_anchor_map, build_preamble
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
    DEFAULT_MODELS,
    HAL_DIR,
    MODEL_PRICING,
    NTOSKRNL_DIR,
    OUTPUT_CRATE,
    REACTOS_ROOT,
    TARGETS,
    cost_usd,
    detect_target,
    get_backend,
    make_user_dll_target,
)
from .cpp_com import (
    CppComError,
    analyze_class,
    apply_interface_method_signatures,
    audit_worker_body,
    discover_widl_header,
    find_class_header,
    lower_trivial_worker,
    normalize_worker_body,
    normalize_worker_semantics,
    parse_widl_interface,
    prepare_helper_closure,
    render_class_scaffold,
    render_cpp_support,
    render_interface,
    render_object_map_adapter,
    render_wide_literal_statics,
    require_supported_worker_raii,
    strip_debug_statements,
    worker_signature,
)
from .extern_audit import run_extern_audit, run_extern_fix
from .fact_seal import (
    FactSealError,
    canonicalize_hresult_predicates,
    fact_contract,
    prepare_sealed_source,
    unseal_facts,
)
from .function_prompt import (
    build_file_context,
    build_function_prompt,
    build_preamble_prompt,
    load_function_system_prompt,
)
from .function_resolver import resolve_function
from .objdiff import run_objdiff
from .outcome_ledger import OutcomeLedger
from .postprocess import postprocess
from .preamble_audit import audit_preamble
from .probe_migrate import (
    build_probe_file_prompt,
    load_probe_system_prompt,
    run_probe_migration,
    scan_rust_file,
)
from .prompt_builder import (
    build_file_prompt,
    build_full_prompt,
    estimate_tokens,
    load_system_prompt,
)
from .resolver import print_resolve_result, resolve
from .resolver import resolve as file_resolve
from .response_cache import PREAMBLE_KEY, RESPONSES_DIR, ExactBodyTranslationMemory, ResponseCache
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
        src = REACTOS_ROOT / "dll" / "win32" / name
        if not src.exists():
            src = REACTOS_ROOT / "dll" / name
        if not src.exists():
            # Standalone sdk/lib/<name> static libs (dnslib, conutils, skiplist,
            # ...) → <name>-rust on the ntdll-sys user substrate. Only on the
            # explicit --target path; detect_target (rtl/cmlib routing) untouched.
            src = REACTOS_ROOT / "sdk" / "lib" / name
        return make_user_dll_target(name, src)
    return detect_target(path)


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
        # Defined in THIS file? A definition is `name` at the head of a line followed by
        # `(` — the ReactOS/wine house style puts the return type + convention on their own
        # lines above it. Cheap and precise enough to distinguish from a call site.
        if re.search(rf"^{re.escape(name)}\s*\(", text, re.M):
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


def _extend_cpp_class_scan(scan, source: Path, class_name: str):
    """Merge source-owned in-class method definitions from the defining header.

    C++ permits method bodies in the class header.  They are part of the same
    class-atomic translation and vtable contract even when the implementation
    file contains most out-of-line methods.  Force the C++ grammar for `.h` and
    merge only this class's definitions; unrelated header helpers stay outside
    the paid closure.
    """
    header = find_class_header(source, class_name)
    if header == source:
        return scan
    header_scan = scan_file(header, force_cpp=True)
    inline = [item for item in header_scan.functions if item.cpp_class == class_name]
    # A method is uniquely identified by (name, arity): C++ overloads that differ
    # in parameter count are distinct methods, not duplicate bodies (e.g. a static
    # inline `GetClassID()` helper in the header alongside the virtual
    # `GetClassID(CLSID*)` defined out-of-line). Only a genuine same-name AND
    # same-arity body present in BOTH header and impl is a real double-definition
    # conflict; true overloads are resolved downstream by the arity-based binder.
    existing = {
        (item.name, len(item.params)) for item in scan.functions if item.cpp_class == class_name
    }
    duplicate = sorted(item.name for item in inline if (item.name, len(item.params)) in existing)
    if duplicate:
        raise CppComError(
            f"{class_name}: method body defined in both implementation and header: "
            + ", ".join(duplicate)
        )
    scan.functions.extend(inline)
    scan.identifiers.update(header_scan.identifiers)
    scan.local_names.update(header_scan.local_names)
    scan.unextracted.extend(item for item in header_scan.unextracted if class_name in item)
    return scan


def _normalize_unseal_cpp_worker(dep, rust: str, fact_seals, call_seals) -> str:
    """Apply the deterministic shell32 worker boundary to a model response."""
    signature = worker_signature(dep.name, dep.cpp_class, dep.params, dep.return_type)
    rust = normalize_worker_body(rust, signature)
    rust = scrub_comment_token_echoes(rust)
    try:
        calls = call_seals.get(dep.name)
        if calls is not None:
            rust = unseal_calls(rust, calls)
        facts = fact_seals.get(dep.name)
        if facts is not None:
            rust = canonicalize_hresult_predicates(rust)
            rust = unseal_facts(rust, facts)
    except (CallSealError, FactSealError) as error:
        raise CppComError(f"{dep.name}: {error}") from error
    rust = normalize_worker_body(rust, signature)
    return normalize_worker_semantics(dep.body_text, rust)


def cmd_candidate_check(args):
    """Batch-compile candidate workers against real target substrate."""
    manifest = Path(args.manifest)
    if not manifest.is_absolute():
        manifest = (Path.cwd() / manifest).resolve()
    arches = args.arches or ["i386", "amd64", "arm64"]
    report = candidate_oracle.run_oracle(manifest, arches, repair=args.repair)
    candidate_oracle.print_report(report, as_json=args.json)
    if any(item["status"] != "ok" for item in report["results"]):
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
        method_names=[f.name for f in class_methods],
    )
    cls.require_supported()

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

    chunks = [render_interface(interface) for interface in interfaces]
    chunks.append(render_class_scaffold(cls, interfaces))
    output = "\n\n".join(chunks) + "\n"
    if args.output:
        out = Path(args.output)
        out.parent.mkdir(parents=True, exist_ok=True)
        out.write_text(output, encoding="utf-8")
        print(f"wrote deterministic scaffold: {out}")
    else:
        print(output, end="")
    if args.adapter_output:
        adapter = Path(args.adapter_output)
        adapter.parent.mkdir(parents=True, exist_ok=True)
        adapter.write_text(render_object_map_adapter(cls) + "\n", encoding="utf-8")
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
        archive.commit(0, path.stem, result.rust_code, result.input_tokens, result.output_tokens)
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
            crate_module=getattr(target, "crate_module", "") if target.crate_bindgen_files else "",
        )
        print(code)


def cmd_batch(args):

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


def cmd_translate(args):
    """Function-by-function translation pipeline."""

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

    if (getattr(args, "seal_facts", False) or getattr(args, "seal_calls", False)) and not (
        rustconv_target.name == "shell32" and getattr(args, "cpp_com", False)
    ):
        raise CppComError("--seal-facts/--seal-calls currently require shell32 --cpp-com")

    cpp_com_cls = None
    cpp_com_interfaces = []
    if rustconv_target.name == "shell32":
        if not getattr(args, "cpp_com", False) or not getattr(args, "class_name", None):
            raise CppComError(
                "shell32 C++ translation is class-atomic: pass --cpp-com --class <Name>. "
                "The paid body path is disabled until deterministic COM preflight succeeds."
            )
        if target_path.is_dir():
            raise CppComError("--cpp-com accepts one C++ implementation file, not a directory")
        if getattr(args, "c2rust", False):
            raise CppComError(
                "c2rust does not provide a C++/ATL behavior anchor. Refusing --cpp-com "
                "--c2rust rather than silently running unanchored."
            )
        scan = _extend_cpp_class_scan(scan_file(target_path), target_path, args.class_name)
        if scan.unextracted:
            raise CppComError(
                f"{target_path.name}: {len(scan.unextracted)} body/bodies remain unextractable; "
                "fix scanner coverage before any paid class translation"
            )
        cpp_com_cls = analyze_class(
            target_path,
            args.class_name,
            object_map_source=rustconv_target.source_dir / "shell32.cpp",
            method_names=[f.name for f in scan.functions if f.cpp_class == args.class_name],
        )
        cpp_com_cls.require_supported()
        if getattr(args, "sdk_header", None):
            sdk_header = Path(args.sdk_header).resolve()
            cpp_com_interfaces = [
                parse_widl_interface(sdk_header, name) for name in cpp_com_cls.interface_names()
            ]
        else:
            cpp_com_interfaces = [
                parse_widl_interface(
                    discover_widl_header(REACTOS_ROOT, args.arch, interface=name),
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
                    header = discover_widl_header(REACTOS_ROOT, args.arch, interface=name)
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
            f"{sum(len(i.methods) for i in cpp_com_interfaces)} vtable slot(s), "
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
    system_prompt = load_function_system_prompt(rustconv_target, mode=mode) if use_claude else ""

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
                "cache_tag": getattr(args, "cache_tag", None),
            },
        )

    total_files = len(c_files)
    audit_findings: dict[str, list[str]] = {}
    for file_idx, c_file in enumerate(c_files, 1):
        print(f"\n{'=' * 60}", file=sys.stderr)
        print(f"[{file_idx}/{total_files}] {c_file.name}", file=sys.stderr)
        print(f"{'=' * 60}", file=sys.stderr)

        capped = False  # set if --cost-cap is hit mid-file
        scan = scan_file(c_file)
        if rustconv_target.name == "shell32":
            scan = _extend_cpp_class_scan(scan, c_file, args.class_name)
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
            }

        _gate_unextracted(scan, getattr(args, "allow_partial", False))
        file_res = file_resolve(scan, catalog, spec)
        subsystem = c_file.parent.name

        # Extract call graph and topologically sort
        deps = extract_call_graph(scan, spec_exports=file_res.exports)
        if rustconv_target.name == "shell32":
            deps = apply_interface_method_signatures(deps, args.class_name, cpp_com_interfaces)
        sorted_deps = topological_sort(deps)
        cpp_com_wide_statics = ""

        if rustconv_target.name == "shell32":
            assert cpp_com_cls is not None
            sorted_deps = prepare_helper_closure(
                deps, args.class_name, cpp_com_cls.plain_state_fields()
            )
            require_supported_worker_raii(sorted_deps)
            cpp_com_wide_statics = render_wide_literal_statics(deps, sorted_deps)

        fact_seal_enabled = bool(
            rustconv_target.name == "shell32" and getattr(args, "seal_facts", False)
        )
        call_seal_enabled = bool(
            rustconv_target.name == "shell32" and getattr(args, "seal_calls", False)
        )
        model_bodies = {dep.name: strip_debug_statements(dep.body_text) for dep in sorted_deps}
        fact_seals = (
            {dep.name: prepare_sealed_source(model_bodies[dep.name]) for dep in sorted_deps}
            if fact_seal_enabled
            else {}
        )
        model_sources = {
            dep.name: fact_seals[dep.name].text
            if dep.name in fact_seals
            else model_bodies[dep.name]
            for dep in sorted_deps
        }
        call_seals = (
            {
                dep.name: seal_calls(
                    model_sources[dep.name],
                    set(dep.callees) | {f"{callee}_body" for callee in dep.callees},
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

        print(f"  Functions: {len(sorted_deps)} (topo-sorted)", file=sys.stderr)

        # c2rust behavior anchors: a compiler-exact transpilation per function, handed
        # to the model so it can't silently stub/drop logic. Built once per file.
        c2rust_anchors: dict[str, str] = {}
        c2rust_preamble = ""
        if getattr(args, "c2rust", False):
            # --c2rust is an explicit opt-in: if c2rust can't anchor the file, HARD FAIL
            # (strict=True) instead of silently degrading to a worse pure-LLM translation.
            # Fails before any paid API call, so no money is wasted on unanchored output.
            try:
                c2rust_anchors = build_anchor_map(c_file, strict=True)
                # File-scope static/const DATA (tables/S-boxes) compiler-exact — prepended
                # to the preamble so the LLM never has to (mis)transcribe a lookup table.
                c2rust_preamble = build_preamble(c_file, strict=True)
            except C2RustError as e:
                print(
                    f"\n  ✗ FATAL: --c2rust requested but c2rust failed: {e}\n"
                    "    Refusing to fall back to a pure-LLM translation (no dumb fallbacks).\n"
                    "    Fix the cause above, or re-run WITHOUT --c2rust to opt into pure-LLM.\n",
                    file=sys.stderr,
                )
                sys.exit(2)

        if args.dry_run:
            # Just show the plan
            print("\n  Translation order:", file=sys.stderr)
            all_deps = (
                context_deps
                if rustconv_target.name == "shell32"
                else extract_call_graph(scan, spec_exports=file_res.exports)
            )
            local_names = {d.name for d in all_deps}
            for i, dep in enumerate(sorted_deps, 1):
                exported = " [EXPORTED]" if dep.is_exported else ""
                static = " [static]" if dep.is_static else ""
                local_callees = dep.callees & local_names
                callee_info = (
                    f" → calls: {', '.join(sorted(local_callees))}" if local_callees else ""
                )
                seh = f" (SEH×{dep.seh_count})" if dep.seh_count else ""
                print(f"    {i}. {dep.name}{exported}{static}{seh}{callee_info}", file=sys.stderr)

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
                )
                full = (system_prompt + "\n\n---\n\n" + prompt) if system_prompt else prompt
                print(full)

            continue

        # Translate function by function
        all_deps_for_context = (
            context_deps
            if rustconv_target.name == "shell32"
            else extract_call_graph(scan, spec_exports=file_res.exports)
        )
        translated: dict[str, str] = {}
        deterministic_workers: set[str] = set()
        if rustconv_target.name == "shell32":
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
        for dep in sorted_deps:
            if dep.name in deterministic_workers:
                _record_outcome(dep, "deterministic", route="trivial_worker")
        total_in = 0
        total_out = 0

        # Per-file SHARED context (import superset + gap fns + globals), built
        # ONCE and sent as a second cached system block on every function call.
        # It's byte-identical across the file's N functions, so functions 2..N
        # read it from cache (~0.1x) instead of re-sending it uncached each call.
        file_context = build_file_context(file_res, scan, subsystem, target=rustconv_target)

        # Durable per-function resume cache → re-run the same command to resume a
        # killed/capped run for $0 (skip functions whose C body is unchanged).
        # Coexists with the sink archive (which is per-run, kill-safe streaming);
        # this one is the stable, cross-run skip layer. --no-resume forces fresh.
        cache_tag = getattr(args, "cache_tag", None)
        cache_suffix = f"__{cache_tag}" if cache_tag else ""
        if fact_seal_enabled:
            cache_suffix += "__factseal_v1"
        if call_seal_enabled:
            cache_suffix += "__callseal_v1"
        cache = ResponseCache(f"{subsystem}__{c_file.stem}{cache_suffix}")

        # A completed model stream can fail a deterministic audit and remain as
        # ``.partial``. When the harness improves, replay those already-paid bytes
        # through the current seals/normalizers before buying another response.
        # Every acceptance check is identical to a live response; incompatible,
        # truncated, or genuinely bad historical streams remain rejected.
        partial_replayed: set[str] = set()
        if rustconv_target.name == "shell32" and not no_resume:
            run_dirs = sorted(RESPONSES_DIR.glob(f"*-{c_file.name}"), reverse=True)
            for dep in sorted_deps:
                if (
                    dep.name in deterministic_workers
                    or cache.get(dep.name, dep.body_text) is not None
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
                    except (CppComError, OSError):
                        continue
                    if not findings:
                        recovered = normalized
                        break
                if recovered is not None:
                    cache.put(dep.name, dep.body_text, recovered)
                    partial_replayed.add(dep.name)
                    _record_outcome(dep, "partial_replay", route="current_deterministic_audit")
            if partial_replayed:
                print(
                    f"  [cpp-com] recovered {len(partial_replayed)} paid partial(s) "
                    "under the current deterministic audit ($0)",
                    file=sys.stderr,
                )
        translation_memory = None
        memory_seeded: set[str] = set()
        memory_identity: Callable[..., dict[str, object]] | None = None
        if (
            rustconv_target.name == "shell32"
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
        if translation_memory is not None and not no_resume:
            for dep in paid_deps:
                if dep.cpp_class is None:
                    continue
                if cache.get(dep.name, dep.body_text) is not None:
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
                cache.put(dep.name, dep.body_text, hit)
                memory_seeded.add(dep.name)
        pending_paid_deps = []
        for dep in paid_deps:
            hit = None if no_resume else cache.get(dep.name, dep.body_text)
            if hit is not None and rustconv_target.name == "shell32":
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
            if hit is None:
                pending_paid_deps.append(dep)
        if deterministic_workers:
            print(
                f"  [cpp-com] lowered {len(deterministic_workers)} proven stub body/bodies "
                "deterministically ($0)",
                file=sys.stderr,
            )
        if archive is not None and pending_paid_deps:
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
            )
            _est_in = (estimate_tokens(_p0) + estimate_tokens(system_prompt)) * len(
                pending_paid_deps
            )
            preamble_calls = 0 if rustconv_target.name == "shell32" else 1
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
                if dep.name in deterministic_workers:
                    continue
                if not no_resume:
                    hit = cache.get(dep.name, dep.body_text)
                    if hit is not None and rustconv_target.name == "shell32":
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
                    if hit is not None:
                        translated[dep.name] = hit
                        resumed += 1
                        _record_outcome(
                            dep,
                            "translation_memory_hit" if dep.name in memory_seeded else "cache_hit",
                            cache_key=cache.dir.name,
                        )
                        continue
                todo.append((func_idx, dep))
            ctx = dict(translated)  # static callee context (mechanical fns are independent)
            stop = threading.Event()

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
                            if rustconv_target.name == "shell32":
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
                        except CppComError as error:
                            _record_outcome(
                                dp,
                                "audit_reject",
                                stage="body_audit",
                                error=str(error),
                                **_result_fields(r, elapsed_ms),
                            )
                            parallel_errors.append(str(error))
                            print(f"  [{fi}] {dp.name} AUDIT REJECT: {error}", file=sys.stderr)
                            continue
                        translated[dp.name] = r.rust_code
                        cache.put(dp.name, dp.body_text, r.rust_code)
                        if translation_memory is not None and dp.cpp_class is not None:
                            assert memory_identity is not None
                            translation_memory.put(r.rust_code, **memory_identity(dp))
                        total_in += r.input_tokens
                        total_out += r.output_tokens
                        artifact = None
                        if archive is not None:
                            artifact = archive.commit(
                                fi, dp.name, r.rust_code, r.input_tokens, r.output_tokens
                            )
                            running_cost += cost_usd(model, r.input_tokens, r.output_tokens)
                        _record_outcome(
                            dp,
                            "generated",
                            stage="body_audit" if rustconv_target.name == "shell32" else "api",
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

            # Resume: reuse a prior unchanged-body translation for free. It still
            # feeds `translated` (callee context for later functions).
            if not no_resume:
                hit = cache.get(dep.name, dep.body_text)
                if hit is not None and rustconv_target.name == "shell32":
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
            )

            # Stream this function's output straight to the durable archive.
            sink = None
            if archive is not None:
                sink = ResponseSink(archive.stream_path(func_idx, dep.name))
            started = time.monotonic()
            try:
                if use_claude:
                    result = submit_single(
                        user_prompt,
                        model=model,
                        system_prompt=system_prompt,
                        file_context=file_context,
                        sink=sink,
                    )
                else:
                    prefix = "\n\n---\n\n".join(p for p in (system_prompt, file_context) if p)
                    full_prompt = (prefix + "\n\n---\n\n" + user_prompt) if prefix else user_prompt
                    result = submit_single(full_prompt, model=model, sink=sink)
            finally:
                if sink is not None:
                    sink.close()
            elapsed_ms = int((time.monotonic() - started) * 1000)

            if result.success:
                try:
                    if rustconv_target.name == "shell32":
                        result.rust_code = _normalize_unseal_cpp_worker(
                            dep, result.rust_code, fact_seals, call_seals
                        )
                        findings = audit_worker_body(
                            dep.body_text,
                            result.rust_code,
                            f"{dep.name}_body",
                            worker_signature(
                                dep.name,
                                dep.cpp_class,
                                dep.params,
                                dep.return_type,
                            ),
                        )
                        if findings:
                            raise CppComError(
                                f"{dep.name}: paid body failed deterministic audit:\n  - "
                                + "\n  - ".join(findings)
                            )
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
                translated[dep.name] = result.rust_code
                # Persist for cross-run resume the instant it returns.
                cache.put(dep.name, dep.body_text, result.rust_code)
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
                    stage="body_audit" if rustconv_target.name == "shell32" else "api",
                    artifact=str(artifact) if artifact is not None else None,
                    **_result_fields(result, elapsed_ms),
                )
            else:
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
            raise CppComError(
                "sequential translation completed with rejected/failed worker(s); "
                "all other accepted paid artifacts were preserved in the resume cache:\n  - "
                + "\n  - ".join(sequential_errors)
            )

        if not translated:
            print(f"  No functions translated for {c_file.name}", file=sys.stderr)
            continue

        # Translate the file-scope preamble (structs / typedefs /
        # #define→const / static tables) as ONE synthetic unit. The
        # stitcher otherwise emits no preamble at all, so files with a
        # non-trivial preamble (most drivers) would not compile.
        preamble_rust = ""
        preamble_src = (
            ""
            if rustconv_target.name == "shell32"
            else (scan.preamble_text or scan.globals_text or "").strip()
        )
        pre_hit = None if (no_resume or not preamble_src) else cache.get(PREAMBLE_KEY, preamble_src)
        if pre_hit is not None:
            preamble_rust = pre_hit
            resumed += 1
            _record_preamble(preamble_src, "cache_hit", cache_key=cache.dir.name)
            print("  [preamble] CACHED ($0)", file=sys.stderr)
        elif preamble_src and not capped:
            print(
                f"  [preamble] {len(preamble_src)} C chars...", file=sys.stderr, end=" ", flush=True
            )
            pre_prompt = build_preamble_prompt(scan, file_res, target=rustconv_target)
            pre_sink = None
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
        if rustconv_target.name == "shell32":
            if cpp_com_cls is None:
                raise CppComError("internal error: shell32 COM preflight state was lost")
            deterministic = [render_interface(item) for item in cpp_com_interfaces]
            deterministic.append(render_class_scaffold(cpp_com_cls, cpp_com_interfaces))
            support_selection = "\n".join(
                [
                    cpp_com_cls.header.read_text(encoding="utf-8", errors="replace"),
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
                )
            )
            if cpp_com_wide_statics:
                deterministic.append(cpp_com_wide_statics)
            stitched = "\n\n".join(deterministic) + "\n\n" + stitched

        # COM vtbl tables, GENERATED — never asked of the model. `translate` is
        # function-granular, so file-scope `static const IXVtbl` tables fall to the
        # one preamble call, and it drops them: all 10 of storage32's went missing
        # in the ole32 run, which is a COM object with no methods at all. Slot order
        # comes from the bindgen'd struct; an arity mismatch raises rather than
        # zipping a table into the wrong methods.
        vtbls = find_vtbl_tables(c_file)
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
            c_source_path=None if rustconv_target.name == "shell32" else c_file,
            com_method_macro=rustconv_target.com_method_macro,
            crate_module=(
                getattr(rustconv_target, "crate_module", "")
                if rustconv_target.crate_bindgen_files
                else ""
            ),
        )

        # --- Post-translation audits -------------------------------------
        # These catch the SILENT class: output that compiles, links and boots but
        # is wrong. A dropped struct is a compile error and finds itself; a table
        # with one wrong byte does not.
        file_findings: list[str] = []
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
            out_path = assemble_file(
                fake_result,
                subsystem,
                scan.raw_source,
                exports=file_res.exports,
                scan=scan,
                target=rustconv_target,
                c_source_path=c_file,
            )
            print(f"  Assembled to {out_path}", file=sys.stderr)

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

    rows = [
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

    # candidate-check — N isolated worker replacements, one cargo check per arch.
    p = sub.add_parser(
        "candidate-check",
        help="Batch cargo-check candidate workers against the real shell32 substrate",
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
    p.add_argument("--json", action="store_true", help="Print the proven class/interface IR")

    # semdiff (C↔Rust semantic-divergence finder; from cdrom-port f29579d1119)
    p = sub.add_parser("semdiff", help="Find C↔Rust semantic/behavioral divergences (LLM)")
    p.add_argument("file", help="C source file (.c)")
    p.add_argument("--rust", default=None, help="Rust file (.rs); auto-located by stem if omitted")
    p.add_argument("--model", default="deepseek-reasoner", help="Model (default deepseek-reasoner)")
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
        "--seal-facts",
        dest="seal_facts",
        action="store_true",
        help="EXPERIMENTAL shell32 COM mode: lower HRESULT predicates and replace each "
        "constant occurrence with an audited opaque token during the paid call. Uses a "
        "separate response-cache namespace.",
    )
    p.add_argument(
        "--seal-calls",
        dest="seal_calls",
        action="store_true",
        help="EXPERIMENTAL shell32 COM mode: replace each known runtime callee occurrence "
        "with an audited opaque token during the paid call. Uses a separate response-cache "
        "namespace.",
    )
    p.add_argument(
        "--sdk-header",
        help="WIDL-generated C header used as COM vtable ground truth",
    )
    p.add_argument(
        "--fn",
        dest="fn",
        help="Translate only this function, or a comma-separated bounded worker set",
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
        "--no-resume",
        dest="no_resume",
        action="store_true",
        help="Ignore the .responses/ resume cache and re-translate every "
        "function fresh (cache still refreshed). Default resumes "
        "unchanged-body functions for $0.",
    )
    p.add_argument(
        "--cache-tag",
        default=None,
        help="Append an isolated namespace tag to the response-cache key. Intended for "
        "controlled model/prompt experiments that must not overwrite production entries.",
    )
    p.add_argument(
        "--c2rust",
        action="store_true",
        help="Anchor each function on its c2rust transpilation (compiler-exact, "
        "behavior-preserving) so the model can't silently stub/drop logic. "
        "Best-effort: needs the c2rust binary + a build-*/compile_commands.json; "
        "falls back to anchor-less translation if unavailable. Pairs with "
        "--mode mechanical.",
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

    # pp (postprocess existing .rs files)
    p = sub.add_parser("pp", help="Run postprocessor on existing .rs files")
    p.add_argument("path", help="Rust file or directory (subsystem under ntoskrnl-rust/src/)")
    p.add_argument("--dry-run", action="store_true", help="Show changes without writing")

    # trace-catalog
    p = sub.add_parser("trace-catalog", help="Generate debugcon trace catalog from Rust source")
    p.add_argument("--arch", default="i386", choices=["i386", "amd64", "arm64"])

    args = parser.parse_args()

    commands = {
        "scan": cmd_scan,
        "dup-census": cmd_dup_census,
        "candidate-check": cmd_candidate_check,
        "substrate-plan": cmd_substrate_plan,
        "skeleton-check": cmd_skeleton_check,
        "cpp-com": cmd_cpp_com,
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
        "pp": cmd_postprocess,
        "trace-catalog": cmd_trace_catalog,
    }

    try:
        commands[args.command](args)
    except (ValueError, CppComError) as e:
        # target auto-detect failure (unrecognized path) surfaces here as a clean
        # message instead of a traceback — tells the user to pass --target.
        print(str(e), file=sys.stderr)
        sys.exit(2)


if __name__ == "__main__":
    main()
