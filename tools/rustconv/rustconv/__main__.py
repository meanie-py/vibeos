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
    subsystem = dir_path.name
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
    }

    commands[args.command](args)


if __name__ == "__main__":
    main()
