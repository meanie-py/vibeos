"""CLI entry point for rustconv."""

import argparse
import sys
from pathlib import Path

from .config import NTOSKRNL_DIR, get_backend, DEFAULT_MODELS

# Model shortcuts for CLI convenience
MODEL_ALIASES = {
    "sonnet": "claude-sonnet-4-5-20250929",
    "haiku": "claude-haiku-4-5-20251001",
    "opus": "claude-opus-4-6-20250908",
    "gemini-pro": "gemini-2.5-pro",
    "gemini-flash": "gemini-2.5-flash",
}


def resolve_model(model_str: str | None) -> str:
    """Resolve model name, supporting aliases."""
    if not model_str:
        return DEFAULT_MODELS[get_backend()]
    return MODEL_ALIASES.get(model_str, model_str)


def resolve_c_path(path_str: str) -> Path:
    """Resolve a C file path, allowing relative paths within ntoskrnl."""
    p = Path(path_str)
    if p.exists():
        return p.resolve()
    # Try as relative to ntoskrnl/
    candidate = NTOSKRNL_DIR / path_str
    if candidate.exists():
        return candidate.resolve()
    # Try with .c extension
    candidate = NTOSKRNL_DIR / (path_str + ".c") if not path_str.endswith('.c') else candidate
    if candidate.exists():
        return candidate.resolve()
    print(f"Error: file not found: {path_str}", file=sys.stderr)
    print(f"  Tried: {p}, {NTOSKRNL_DIR / path_str}", file=sys.stderr)
    sys.exit(1)


def cmd_scan(args):
    from .scanner import scan_file, print_scan_result
    path = resolve_c_path(args.file)
    result = scan_file(path)
    print_scan_result(result)


def cmd_resolve(args):
    from .scanner import scan_file
    from .catalog import build_catalog, print_catalog_stats
    from .spec_parser import parse_spec
    from .resolver import resolve, print_resolve_result

    path = resolve_c_path(args.file)

    print("Building catalog...", file=sys.stderr)
    catalog = build_catalog(arch=args.arch)
    print_catalog_stats(catalog)
    print()

    spec = parse_spec()
    scan = scan_file(path)
    result = resolve(scan, catalog, spec)
    print_resolve_result(result)


def cmd_prompt(args):
    from .scanner import scan_file
    from .catalog import build_catalog
    from .spec_parser import parse_spec
    from .resolver import resolve
    from .prompt_builder import build_full_prompt, build_file_prompt, load_system_prompt, estimate_tokens

    path = resolve_c_path(args.file)
    catalog = build_catalog(arch=args.arch)
    spec = parse_spec()
    scan = scan_file(path)
    res = resolve(scan, catalog, spec)

    if args.estimate:
        prompt = build_full_prompt(scan, res)
        tokens = estimate_tokens(prompt)
        print(f"Estimated tokens: ~{tokens:,}", file=sys.stderr)
    else:
        prompt = build_full_prompt(scan, res)
        print(prompt)


def cmd_submit(args):
    from .scanner import scan_file
    from .catalog import build_catalog
    from .spec_parser import parse_spec
    from .resolver import resolve
    from .prompt_builder import build_file_prompt, load_system_prompt, build_full_prompt
    from .batch_runner import submit_single
    from .assembler import assemble_file
    from .postprocess import postprocess

    path = resolve_c_path(args.file)
    catalog = build_catalog(arch=args.arch)
    spec = parse_spec()
    scan = scan_file(path)
    res = resolve(scan, catalog, spec)

    model = resolve_model(args.model)

    # For Claude, use separate system prompt (for caching)
    # For Gemini, combine into single prompt
    if model.startswith("claude-"):
        system_prompt = load_system_prompt()
        file_prompt = build_file_prompt(scan, res)
    else:
        system_prompt = ""
        file_prompt = build_full_prompt(scan, res)

    print(f"Submitting to {model}...", file=sys.stderr)
    result = submit_single(file_prompt, model=model, system_prompt=system_prompt)
    result.source_file = path.name

    if not result.success:
        print(f"Error: {result.error}", file=sys.stderr)
        sys.exit(1)

    print(f"Tokens: {result.input_tokens} in / {result.output_tokens} out", file=sys.stderr)

    # Determine subsystem from path
    subsystem = path.parent.name

    if args.output:
        # Post-process before writing
        code = postprocess(result.rust_code, exports=res.exports, globals=scan.globals)
        Path(args.output).write_text(code)
        print(f"Written to {args.output}", file=sys.stderr)
    elif args.assemble:
        out_path = assemble_file(result, subsystem, scan.raw_source,
                                 exports=res.exports, scan=scan)
        print(f"Assembled to {out_path}", file=sys.stderr)
    else:
        # Post-process before printing
        code = postprocess(result.rust_code, exports=res.exports, globals=scan.globals)
        print(code)


def cmd_batch(args):
    from .scanner import scan_file
    from .catalog import build_catalog
    from .spec_parser import parse_spec
    from .resolver import resolve
    from .prompt_builder import build_file_prompt, load_system_prompt, build_full_prompt
    from .batch_runner import submit_batch
    from .assembler import assemble_file, generate_mod_rs, generate_lib_rs

    dir_path = Path(args.directory)
    if not dir_path.is_absolute():
        dir_path = NTOSKRNL_DIR / args.directory
    if not dir_path.is_dir():
        print(f"Error: not a directory: {dir_path}", file=sys.stderr)
        sys.exit(1)

    c_files = sorted(dir_path.glob("*.c"))
    if args.exclude:
        exclude_stems = set(args.exclude)
        c_files = [f for f in c_files if f.stem not in exclude_stems]
    if not c_files:
        print(f"No .c files found in {dir_path}", file=sys.stderr)
        sys.exit(1)

    print(f"Found {len(c_files)} C files in {dir_path.name}/", file=sys.stderr)

    catalog = build_catalog(arch=args.arch)
    spec = parse_spec()

    model = resolve_model(args.model)
    use_claude = model.startswith("claude-")

    system_prompt = load_system_prompt() if use_claude else ""

    # Build prompts
    prompts = {}
    scans = {}
    resolves = {}
    for cf in c_files:
        scan = scan_file(cf)
        scans[cf.name] = scan
        res = resolve(scan, catalog, spec)
        resolves[cf.name] = res
        if use_claude:
            prompts[cf.name] = build_file_prompt(scan, res)
        else:
            prompts[cf.name] = build_full_prompt(scan, res)

    print(f"\nSubmitting {len(prompts)} files to {model}...", file=sys.stderr)
    results = submit_batch(prompts, model=model, system_prompt=system_prompt)

    # Assemble results
    subsystem = args.subsystem if args.subsystem else dir_path.name
    module_names = []
    for filename, result in results.items():
        if result.success:
            c_source = scans[filename].raw_source
            res = resolves[filename]
            assemble_file(result, subsystem, c_source,
                          exports=res.exports, scan=scans[filename])
            module_names.append(Path(filename).stem)

    if module_names:
        generate_mod_rs(subsystem, module_names)
        # Only update lib.rs for top-level subsystems (not nested like ke/i386)
        if '/' not in subsystem:
            generate_lib_rs([subsystem])
        print(f"\nAssembled {len(module_names)} modules into ntoskrnl-rust/src/{subsystem}/", file=sys.stderr)

    # Report failures
    failures = [f for f, r in results.items() if not r.success]
    if failures:
        print(f"\nFailed ({len(failures)}):", file=sys.stderr)
        for f in failures:
            print(f"  {f}: {results[f].error}", file=sys.stderr)


def cmd_check(args):
    from .checker import check_all_targets, print_check_report
    results = check_all_targets()
    print_check_report(results)


def cmd_shims(args):
    from .shim_generator import generate_shims, check_shims

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
            from .config import NTOSKRNL_DIR
            out_path = NTOSKRNL_DIR / subsystem / f"{subsystem}_rust_shims.c"
            out_path.write_text(output)
            print(f"Written to {out_path}", file=sys.stderr)
        else:
            print(output)


def cmd_seh_classify(args):
    from .seh_classifier import classify_file, classify_directory, print_classification, to_json

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


def cmd_probe_migrate(args):
    from .probe_migrate import (
        scan_rust_file, find_rust_files, run_probe_migration,
        load_probe_system_prompt, build_probe_file_prompt,
    )
    from .config import OUTPUT_CRATE

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
    from .scanner import scan_file
    from .catalog import build_catalog
    from .spec_parser import parse_spec
    from .resolver import resolve as file_resolve
    from .callgraph import extract_call_graph, topological_sort, print_call_graph
    from .function_resolver import resolve_function
    from .function_prompt import build_function_prompt, load_function_system_prompt
    from .stitcher import stitch_file
    from .batch_runner import submit_single, TranslationResult
    from .assembler import assemble_file, generate_mod_rs, generate_lib_rs
    from .postprocess import postprocess
    from .prompt_builder import estimate_tokens

    # Resolve target: single file or directory
    target = Path(args.path)
    if not target.is_absolute():
        target = NTOSKRNL_DIR / args.path

    if target.is_file():
        c_files = [target]
    elif target.is_dir():
        c_files = sorted(target.glob("*.c"))
        if args.exclude:
            exclude_stems = set(args.exclude)
            c_files = [f for f in c_files if f.stem not in exclude_stems]
    else:
        print(f"Error: not found: {target}", file=sys.stderr)
        sys.exit(1)

    if not c_files:
        print(f"No .c files found in {target}", file=sys.stderr)
        sys.exit(1)

    print(f"Building catalog...", file=sys.stderr)
    catalog = build_catalog(arch=args.arch)
    spec = parse_spec()
    catalog_fn_names = set(catalog.functions.keys())

    model = resolve_model(args.model)
    use_claude = model.startswith("claude-")
    system_prompt = load_function_system_prompt() if use_claude else ""

    total_files = len(c_files)
    for file_idx, c_file in enumerate(c_files, 1):
        print(f"\n{'='*60}", file=sys.stderr)
        print(f"[{file_idx}/{total_files}] {c_file.name}", file=sys.stderr)
        print(f"{'='*60}", file=sys.stderr)

        scan = scan_file(c_file)
        file_res = file_resolve(scan, catalog, spec)
        subsystem = c_file.parent.name

        # Extract call graph and topologically sort
        deps = extract_call_graph(scan, spec_exports=file_res.exports)
        sorted_deps = topological_sort(deps)

        # Filter to single function if requested
        if args.fn:
            sorted_deps = [d for d in sorted_deps if d.name == args.fn]
            if not sorted_deps:
                print(f"  Error: function '{args.fn}' not found in {c_file.name}", file=sys.stderr)
                func_names = [d.name for d in deps]
                print(f"  Available: {', '.join(func_names)}", file=sys.stderr)
                continue

        print(f"  Functions: {len(sorted_deps)} (topo-sorted)", file=sys.stderr)

        if args.dry_run:
            # Just show the plan
            print(f"\n  Translation order:", file=sys.stderr)
            all_deps = extract_call_graph(scan, spec_exports=file_res.exports)
            local_names = {d.name for d in all_deps}
            for i, dep in enumerate(sorted_deps, 1):
                exported = " [EXPORTED]" if dep.is_exported else ""
                static = " [static]" if dep.is_static else ""
                local_callees = dep.callees & local_names
                callee_info = f" → calls: {', '.join(sorted(local_callees))}" if local_callees else ""
                seh = f" (SEH×{dep.seh_count})" if dep.seh_count else ""
                print(f"    {i}. {dep.name}{exported}{static}{seh}{callee_info}", file=sys.stderr)

            # Show estimated tokens for first function
            if sorted_deps:
                func_res = resolve_function(sorted_deps[0], catalog, file_res.exports, scan.defines)
                prompt = build_function_prompt(
                    sorted_deps[0], func_res, {}, all_deps, scan, catalog_fn_names, file_res.exports
                )
                if not use_claude:
                    prompt = system_prompt + "\n\n---\n\n" + prompt if system_prompt else prompt
                tokens = estimate_tokens(prompt)
                print(f"\n  First function prompt: ~{tokens:,} tokens", file=sys.stderr)

            if args.prompt:
                # Print full prompt for first function
                func_res = resolve_function(sorted_deps[0], catalog, file_res.exports, scan.defines)
                prompt = build_function_prompt(
                    sorted_deps[0], func_res, {}, all_deps, scan, catalog_fn_names, file_res.exports
                )
                full = (system_prompt + "\n\n---\n\n" + prompt) if system_prompt else prompt
                print(full)

            continue

        # Translate function by function
        all_deps_for_context = extract_call_graph(scan, spec_exports=file_res.exports)
        translated: dict[str, str] = {}
        total_in = 0
        total_out = 0

        for func_idx, dep in enumerate(sorted_deps, 1):
            print(f"  [{func_idx}/{len(sorted_deps)}] {dep.name}...", file=sys.stderr, end=" ", flush=True)

            # Per-function resolve
            func_res = resolve_function(dep, catalog, file_res.exports, scan.defines)

            # Build prompt
            user_prompt = build_function_prompt(
                dep, func_res, translated, all_deps_for_context,
                scan, catalog_fn_names, file_res.exports,
            )

            if use_claude:
                result = submit_single(user_prompt, model=model, system_prompt=system_prompt)
            else:
                full_prompt = (system_prompt + "\n\n---\n\n" + user_prompt) if system_prompt else user_prompt
                result = submit_single(full_prompt, model=model)

            if result.success:
                translated[dep.name] = result.rust_code
                total_in += result.input_tokens
                total_out += result.output_tokens
                print(f"OK ({result.input_tokens} in / {result.output_tokens} out)", file=sys.stderr)
            else:
                print(f"FAIL: {result.error}", file=sys.stderr)

        if not translated:
            print(f"  No functions translated for {c_file.name}", file=sys.stderr)
            continue

        print(f"\n  Stitching {len(translated)} functions...", file=sys.stderr)
        print(f"  Total tokens: {total_in:,} in / {total_out:,} out", file=sys.stderr)

        # Stitch functions into a single file
        stitched = stitch_file(translated, sorted_deps, scan, file_res)

        # Post-process
        code = postprocess(stitched, exports=file_res.exports, globals=scan.globals)

        if args.output and total_files == 1:
            Path(args.output).write_text(code)
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
            out_path = assemble_file(fake_result, subsystem, scan.raw_source,
                                     exports=file_res.exports, scan=scan)
            print(f"  Assembled to {out_path}", file=sys.stderr)

    # Generate mod.rs and lib.rs for batch runs
    if total_files > 1 and not args.dry_run:
        subsystem = c_files[0].parent.name
        module_names = [f.stem for f in c_files]
        generate_mod_rs(subsystem, module_names)
        generate_lib_rs([subsystem])
        print(f"\nUpdated mod.rs and lib.rs for {subsystem}/", file=sys.stderr)

    if not args.dry_run:
        print(f"\nRun `python -m rustconv check` to verify compilation.", file=sys.stderr)


def cmd_estimate(args):
    from .scanner import scan_file
    from .catalog import build_catalog
    from .spec_parser import parse_spec
    from .resolver import resolve
    from .prompt_builder import build_full_prompt, estimate_tokens

    target = Path(args.path)
    if not target.is_absolute():
        target = NTOSKRNL_DIR / args.path

    if target.is_file():
        c_files = [target]
    elif target.is_dir():
        c_files = sorted(target.glob("*.c"))
    else:
        print(f"Error: not found: {target}", file=sys.stderr)
        sys.exit(1)

    catalog = build_catalog(arch=args.arch)
    spec = parse_spec()

    total_input = 0
    total_lines = 0
    for cf in c_files:
        scan = scan_file(cf)
        res = resolve(scan, catalog, spec)
        prompt = build_full_prompt(scan, res)
        tokens = estimate_tokens(prompt)
        lines = len(scan.raw_source.splitlines())
        total_input += tokens
        total_lines += lines
        print(f"  {cf.name}: {lines} lines, ~{tokens:,} tokens")

    total_output = int(total_input * 0.35)

    print(f"\nTotal: {len(c_files)} files, {total_lines:,} lines")
    print(f"Estimated tokens: ~{total_input:,} input, ~{total_output:,} output")

    # Show costs for multiple models
    costs = [
        ("Claude Sonnet (batch)", 1.50, 7.50),
        ("Claude Haiku (batch)", 0.40, 2.00),
        ("Gemini 2.5 Pro", 1.25, 10.00),
        ("Gemini 2.5 Flash", 0.15, 0.60),
    ]
    print("\nEstimated costs:")
    for name, in_price, out_price in costs:
        cost = total_input * in_price / 1_000_000 + total_output * out_price / 1_000_000
        print(f"  {name}: ${cost:.2f}")


def cmd_objdiff(args):
    """Compare C vs Rust object files for symbol/ABI differences."""
    from .objdiff import run_objdiff

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
        from .extern_audit import run_extern_fix
        run_extern_fix(dry_run=args.dry_run)
    else:
        from .extern_audit import run_extern_audit
        run_extern_audit(json_output=args.json)


def cmd_trace_catalog(args):
    """Generate debugcon trace catalog from Rust source."""
    from .trace_catalog import generate_catalog, print_catalog

    print_catalog(args.arch)
    out = generate_catalog(args.arch)
    print(f"\nWritten to {out}")


def cmd_postprocess(args):
    """Run postprocessor on existing .rs files."""
    from .postprocess import postprocess
    from .config import OUTPUT_CRATE
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
        processed = postprocess(original)

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
            print(f"  {rs_file.name}: would change (~{removed} removed, ~{added} added)", file=sys.stderr)
        else:
            rs_file.write_text(processed)
            print(f"  {rs_file.name}: updated (~{removed} removed, ~{added} added)", file=sys.stderr)

    verb = "would change" if args.dry_run else "changed"
    print(f"\n{total_changed}/{len(rs_files)} files {verb}", file=sys.stderr)


def main():
    parser = argparse.ArgumentParser(
        prog="rustconv",
        description="C-to-Rust translation pipeline for ReactOS ntoskrnl",
    )
    parser.add_argument("--arch", default="arm64", choices=["arm64", "amd64", "i386"],
                        help="Architecture for catalog/type resolution")

    sub = parser.add_subparsers(dest="command", required=True)

    # scan
    p = sub.add_parser("scan", help="Parse C file, print analysis")
    p.add_argument("file", help="C source file (relative to ntoskrnl/ or absolute)")

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
    p.add_argument("-m", "--model", help="Model: sonnet, haiku, opus, gemini-pro, gemini-flash, or full ID")

    # batch
    p = sub.add_parser("batch", help="Batch-submit entire subsystem directory")
    p.add_argument("directory", help="Subsystem directory (relative to ntoskrnl/ or absolute)")
    p.add_argument("-m", "--model", help="Model: sonnet, haiku, opus, gemini-pro, gemini-flash, or full ID")
    p.add_argument("--exclude", nargs="*", default=[],
                   help="File stems to skip (e.g., obhandle ob_rust_shims)")
    p.add_argument("-s", "--subsystem", default=None,
                   help="Override subsystem name (e.g., ke/i386 for nested arch dirs)")

    # check
    sub.add_parser("check", help="Run cargo check on all 3 targets")

    # shims
    p = sub.add_parser("shims", help="Generate C shim file from TOML manifest")
    p.add_argument("subsystem", help="Subsystem name (e.g., ob, ex)")
    p.add_argument("-o", "--output", help="Write output to file")
    p.add_argument("--write", action="store_true",
                   help="Write directly to ntoskrnl/{subsys}/{subsys}_rust_shims.c")
    p.add_argument("--check", action="store_true",
                   help="Check if generated file matches existing")

    # estimate
    p = sub.add_parser("estimate", help="Estimate token count and cost")
    p.add_argument("path", help="File or directory to estimate")

    # seh-classify
    p = sub.add_parser("seh-classify", help="Classify SEH blocks in C files by pattern")
    p.add_argument("path", help="C file or directory (relative to ntoskrnl/ or absolute)")
    p.add_argument("-v", "--verbose", action="store_true", help="Show per-function details")
    p.add_argument("--json", action="store_true", help="Output JSON")

    # translate (function-by-function)
    p = sub.add_parser("translate", help="Function-by-function translation (improved pipeline)")
    p.add_argument("path", help="C file or directory (relative to ntoskrnl/ or absolute)")
    p.add_argument("-m", "--model", help="Model: sonnet, haiku, opus, gemini-pro, gemini-flash, or full ID")
    p.add_argument("-o", "--output", help="Write Rust output to file (single file only)")
    p.add_argument("--fn", dest="fn", help="Translate only this function")
    p.add_argument("--dry-run", action="store_true", help="Show translation plan without API calls")
    p.add_argument("--prompt", action="store_true", help="Print prompt for first function (implies --dry-run)")
    p.add_argument("--exclude", nargs="*", default=[],
                   help="File stems to skip (batch mode)")

    # probe-migrate
    p = sub.add_parser("probe-migrate", help="Migrate Rust files from ProbeForRead/Write to probe::*")
    p.add_argument("path", help="Rust file or directory (relative to ntoskrnl-rust/src/ or absolute)")
    p.add_argument("-m", "--model", help="Model: sonnet, haiku, opus, gemini-pro, gemini-flash, or full ID")
    p.add_argument("--dry-run", action="store_true", help="Don't write, just show what would change")
    p.add_argument("--scan-only", action="store_true", help="Only scan for probe calls, don't migrate")
    p.add_argument("--prompt", action="store_true", help="Print the prompt for the first file and exit")

    # objdiff
    p = sub.add_parser("objdiff", help="Compare C vs Rust object files for ABI differences")
    p.add_argument("target", nargs="?", default=None,
                   help="Subsystem (ex/) or file (ex/atom) to compare (default: all)")
    p.add_argument("--json", action="store_true", help="JSON output for CI")
    p.add_argument("-v", "--verbose", action="store_true", help="Per-function details")
    p.add_argument("--critical-only", action="store_true", help="Only show critical issues")
    p.add_argument("--baseline", default=None, metavar="COMMIT",
                   help="Git commit to extract C files from (compare Rust vs historical C)")
    p.add_argument("--deep", action="store_true",
                   help="Analyze function prologues (stack frame size, callee-saved regs)")

    # extern-audit
    p = sub.add_parser("extern-audit", help="Audit extern declarations vs definitions for convention mismatches")
    p.add_argument("--json", action="store_true", help="JSON output")
    p.add_argument("--fix", action="store_true", help="Fix critical mismatches automatically")
    p.add_argument("--dry-run", action="store_true", help="Show what --fix would change without writing")

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
        "resolve": cmd_resolve,
        "prompt": cmd_prompt,
        "submit": cmd_submit,
        "batch": cmd_batch,
        "check": cmd_check,
        "shims": cmd_shims,
        "estimate": cmd_estimate,
        "translate": cmd_translate,
        "seh-classify": cmd_seh_classify,
        "probe-migrate": cmd_probe_migrate,
        "objdiff": cmd_objdiff,
        "extern-audit": cmd_extern_audit,
        "pp": cmd_postprocess,
        "trace-catalog": cmd_trace_catalog,
    }

    commands[args.command](args)


if __name__ == "__main__":
    main()
