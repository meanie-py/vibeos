"""CLI entry point for rustconv."""

import argparse
import sys
from pathlib import Path

from .config import (NTOSKRNL_DIR, REACTOS_ROOT, get_backend, DEFAULT_MODELS,
                     detect_target, make_user_dll_target, TARGETS)


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
    "deepseek": "deepseek-v4-pro",        # default to pro for translation quality
    "deepseek-pro": "deepseek-v4-pro",
    "deepseek-flash": "deepseek-v4-flash",
    "deepseek-v4-pro": "deepseek-v4-pro",
    "deepseek-v4-flash": "deepseek-v4-flash",
    "deepseek-chat": "deepseek-chat",     # legacy aliases (route server-side)
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
    from .config import HAL_DIR
    hal_candidate = HAL_DIR / path_str
    if hal_candidate.exists():
        return hal_candidate.resolve()
    # Try with .c extension
    candidate_c = NTOSKRNL_DIR / (path_str + ".c") if not path_str.endswith('.c') else candidate
    if candidate_c.exists():
        return candidate_c.resolve()
    hal_candidate_c = HAL_DIR / (path_str + ".c") if not path_str.endswith('.c') else hal_candidate
    if hal_candidate_c.exists():
        return hal_candidate_c.resolve()
    print(f"Error: file not found: {path_str}", file=sys.stderr)
    print(f"  Tried: {p}, {NTOSKRNL_DIR / path_str}, {HAL_DIR / path_str}", file=sys.stderr)
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
    target = resolve_target(args,path)

    print(f"Building catalog... (target={target.name})", file=sys.stderr)
    catalog = build_catalog(arch=args.arch, target=target)
    print_catalog_stats(catalog)
    print()

    spec = (parse_spec(target.spec_file)
            if target.spec_file and target.spec_file.exists()
            else parse_spec())
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
    target = resolve_target(args,path)
    catalog = build_catalog(arch=args.arch, target=target)
    spec = (parse_spec(target.spec_file)
            if target.spec_file and target.spec_file.exists()
            else parse_spec())
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
    from .sink import (RunArchive, ResponseSink, install_kill_guard,
                       paid_invocation_banner, warn_if_tmp_output)
    from .prompt_builder import estimate_tokens
    from .config import cost_usd

    path = resolve_c_path(args.file)
    target = resolve_target(args,path)
    catalog = build_catalog(arch=args.arch, target=target)
    spec = (parse_spec(target.spec_file)
            if target.spec_file and target.spec_file.exists()
            else parse_spec())
    scan = scan_file(path)
    res = resolve(scan, catalog, spec)

    model = resolve_model(args.model)

    # For Claude, use separate system prompt (for caching)
    # For Gemini, combine into single prompt
    if model.startswith("claude-"):
        system_prompt = load_system_prompt(target)
        file_prompt = build_file_prompt(scan, res)
    else:
        system_prompt = ""
        file_prompt = build_full_prompt(scan, res)

    # Durable archive + cost banner + kill guard for this one paid call.
    warn_if_tmp_output(getattr(args, "output", None))
    archive = RunArchive(label=path.name, model=model, target=target.name)
    install_kill_guard(archive)
    paid_invocation_banner(model, 1,
                           estimate_tokens(file_prompt) + estimate_tokens(system_prompt),
                           archive)
    sink = ResponseSink(archive.stream_path(0, path.stem))

    print(f"Submitting to {model}...", file=sys.stderr)
    try:
        result = submit_single(file_prompt, model=model,
                               system_prompt=system_prompt, sink=sink)
    finally:
        sink.close()
    result.source_file = path.name
    if result.success:
        archive.commit(0, path.stem, result.rust_code,
                       result.input_tokens, result.output_tokens)
        print(f"Cost: ${cost_usd(model, result.input_tokens, result.output_tokens):.2f}  "
              f"| archived: {archive.dir}", file=sys.stderr)

    if not result.success:
        print(f"Error: {result.error}", file=sys.stderr)
        sys.exit(1)

    print(f"Tokens: {result.input_tokens} in / {result.output_tokens} out", file=sys.stderr)

    # Determine subsystem from path
    subsystem = path.parent.name

    if args.output:
        # Post-process before writing
        code = postprocess(result.rust_code, exports=res.exports, globals=scan.globals,
                           target_name=target.name,
                           sys_crate=target.sys_crate, sys_umbrella=target.sys_umbrella)
        Path(args.output).write_text(code)
        print(f"Written to {args.output}", file=sys.stderr)
    elif args.assemble:
        out_path = assemble_file(result, subsystem, scan.raw_source,
                                 exports=res.exports, scan=scan,
                                 target=target, c_source_path=path)
        print(f"Assembled to {out_path}", file=sys.stderr)
    else:
        # Post-process before printing
        code = postprocess(result.rust_code, exports=res.exports, globals=scan.globals,
                           target_name=target.name,
                           sys_crate=target.sys_crate, sys_umbrella=target.sys_umbrella)
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


def cmd_seh_inventory(args):
    from .seh_inventory import (
        inventory_file, inventory_keepset, write_sidecar,
        print_summary, run_oracle,
    )
    from .config import REACTOS_ROOT

    if args.oracle:
        winsta = REACTOS_ROOT / "win32ss/user/ntuser/winsta.c"
        sys.exit(run_oracle(winsta))

    target = Path(args.path)
    if not target.is_absolute():
        target = REACTOS_ROOT / args.path

    if target.is_file():
        sites = inventory_file(target)
        if args.write_sidecars:
            print(f"wrote {write_sidecar(target, sites)}")
        if args.json:
            print(json.dumps(
                {"source": str(target),
                 "sites": [s.__dict__ | {"category": s.category.value}
                           for s in sites]},
                indent=2, default=str))
        else:
            for s in sites:
                sig = (" " + ",".join(s.signals)) if s.signals else ""
                print(f"  ({s.category.value}) {s.function} "
                      f"L{s.line_start}-{s.line_end}{sig}"
                      f"{('  — ' + s.note) if s.note else ''}")
    elif target.is_dir():
        inv = inventory_keepset(target)
        if args.write_sidecars:
            for p, ss in inv.items():
                write_sidecar(p, ss)
            print(f"wrote {len(inv)} sidecars")
        print_summary(inv)
    else:
        print(f"Error: not found: {target}", file=sys.stderr)
        sys.exit(1)


def cmd_seh_rewrite(args):
    import difflib
    from .seh_rewrite import rewrite_file, run_oracle
    from .config import REACTOS_ROOT
    from .seh_inventory import inventory_keepset, Category

    if args.oracle:
        sys.exit(run_oracle(REACTOS_ROOT / "win32ss/user/ntuser/winsta.c"))

    target = Path(args.path)
    if not target.is_absolute():
        target = REACTOS_ROOT / args.path

    def do_one(p: Path):
        orig = p.read_text(errors="replace")
        res = rewrite_file(p)
        if args.apply and res.rewritten:
            p.write_text(res.text)
        if not args.apply:
            diff = difflib.unified_diff(
                orig.splitlines(True), res.text.splitlines(True),
                f"a/{p.name}", f"b/{p.name}")
            sys.stdout.writelines(diff)
        return res

    if target.is_file():
        res = do_one(target)
        print(f"\n{target.name}: rewritten={res.rewritten} "
              f"skipped={len(res.skipped)}", file=sys.stderr)
        for ln, why in res.skipped:
            print(f"  skip L{ln}: {why}", file=sys.stderr)
    elif target.is_dir():
        import json as _json
        import collections
        inv = inventory_keepset(target)
        tot_rw = tot_sk = 0
        hist: dict[str, int] = collections.Counter()
        manifest = []
        for p in inv:
            r = do_one(p)
            tot_rw += r.rewritten
            tot_sk += len(r.skipped)
            for ln, why in r.skipped:
                hist[why] += 1
                manifest.append({"file": str(p.relative_to(target)),
                                 "line": ln, "reason": why})
        if args.apply:
            mpath = target / "win32k_pass1_manual_followup.json"
            mpath.write_text(_json.dumps(
                {"rewritten": tot_rw, "skipped": tot_sk,
                 "sites": manifest}, indent=2))
            print(f"manual-followup ledger: {mpath}", file=sys.stderr)
        print(f"\n{'APPLIED' if args.apply else 'DRY-RUN'}: "
              f"{tot_rw} sites rewritten, {tot_sk} skipped across "
              f"{len(inv)} files", file=sys.stderr)
        print("skip reasons:", file=sys.stderr)
        for why, n in sorted(hist.items(), key=lambda x: -x[1]):
            print(f"  {n:>4}  {why}", file=sys.stderr)
    else:
        print(f"Error: not found: {target}", file=sys.stderr)
        sys.exit(1)


def cmd_seh_belt(args):
    """Pass 1c: delete (d) BELT_SUSPENDERS SEH wrappers (live re-classify)."""
    import difflib
    from .seh_belt import rewrite_file, iter_keepset, run_oracle
    from .config import REACTOS_ROOT

    if args.oracle:
        sys.exit(run_oracle())

    target = Path(args.path)
    if not target.is_absolute():
        target = REACTOS_ROOT / args.path

    def do_one(p: Path):
        orig = p.read_text(errors="replace")
        res = rewrite_file(p)
        if args.apply and res.rewritten:
            p.write_text(res.text)
        if not args.apply and res.text != orig:
            diff = difflib.unified_diff(
                orig.splitlines(True), res.text.splitlines(True),
                f"a/{p.name}", f"b/{p.name}")
            sys.stdout.writelines(diff)
        return res

    if target.is_file():
        res = do_one(target)
        print(f"\n{target.name}: rewritten={res.rewritten} "
              f"skipped={len(res.skipped)}", file=sys.stderr)
        for ln, why in res.skipped:
            print(f"  skip L{ln}: {why}", file=sys.stderr)
    elif target.is_dir():
        tot_rw = tot_sk = files_touched = 0
        skip_reasons: list[tuple[str, int, str]] = []
        for p in iter_keepset(target):
            r = do_one(p)
            if r.rewritten or r.skipped:
                if r.rewritten:
                    files_touched += 1
                tot_rw += r.rewritten
                tot_sk += len(r.skipped)
                for ln, why in r.skipped:
                    skip_reasons.append((str(p.relative_to(target)), ln, why))
        print(f"\n{'APPLIED' if args.apply else 'DRY-RUN'}: "
              f"{tot_rw} BELT_SUSPENDERS sites deleted across "
              f"{files_touched} files; {tot_sk} skipped",
              file=sys.stderr)
        for f, ln, why in skip_reasons:
            print(f"  skip {f}:L{ln} — {why}", file=sys.stderr)
    else:
        print(f"Error: not found: {target}", file=sys.stderr)
        sys.exit(1)


def cmd_callout_annotate(args):
    from .callout_annotate import (
        extract_file, extract_keepset, write_sidecar,
        print_summary, run_oracle,
    )
    from .config import REACTOS_ROOT

    if args.oracle:
        sys.exit(run_oracle(REACTOS_ROOT / "win32ss/user/ntuser/callback.c"))

    target = Path(args.path)
    if not target.is_absolute():
        target = REACTOS_ROOT / args.path

    if target.is_file():
        sites = extract_file(target)
        if args.write_sidecars:
            print(f"wrote {write_sidecar(target, sites)}")
        if args.json:
            print(json.dumps(
                {"source": str(target),
                 "sites": [s.__dict__ for s in sites]},
                indent=2, default=str))
        else:
            for s in sites:
                sig = (" " + ",".join(s.signals)) if s.signals else ""
                print(f"  {s.function} L{s.func_line} "
                      f"{s.callback_index} @KUMC L{s.kumc_line} "
                      f"[Leave L{s.leave_co_line} / Enter L{s.enter_co_line} "
                      f"/ probe L{s.result_probe_line}]{sig}")
    elif target.is_dir():
        inv = extract_keepset(target)
        if args.write_sidecars:
            for p, ss in inv.items():
                write_sidecar(p, ss)
            print(f"wrote {len(inv)} sidecars")
        print_summary(inv)
    else:
        print(f"Error: not found: {target}", file=sys.stderr)
        sys.exit(1)


def cmd_seh_llm_lower(args):
    from .seh_llm_lower import run, print_report, run_oracle, load_units

    if args.oracle:
        sys.exit(run_oracle())

    units = load_units(args.only)
    print(f"{len(units)} dedup'd functions "
          f"({sum(len(u.reasons) for u in units)} ledger sites) "
          f"{'[--only ' + args.only + ']' if args.only else ''}",
          file=sys.stderr)
    outcomes = run(apply=args.apply, model=args.model,
                   only=args.only, units=units)
    if args.diff:
        for o in outcomes:
            if o.applied and o.diff:
                sys.stdout.write(o.diff)
    sys.exit(print_report(outcomes, args.apply))


def cmd_split_plan(args):
    from .file_splitter import analyze_file, analyze_scope, print_report
    from .config import REACTOS_ROOT

    target = Path(args.path)
    if not target.is_absolute():
        target = REACTOS_ROOT / args.path

    if target.is_file():
        plans = [analyze_file(target, args.target_loc)]
    else:
        plans = analyze_scope(target, args.min_loc, args.target_loc)

    if args.json:
        print(json.dumps([{
            "file": str(p.path), "loc": p.loc, "nfuncs": p.nfuncs,
            "classification": p.classification, "groups": p.groups,
            "blocking": p.blocking, "note": p.note} for p in plans],
            indent=2, default=str))
    else:
        print_report(plans)


def cmd_switch_extract(args):
    from .switch_extract import analyze, run_oracle
    from .config import REACTOS_ROOT

    target = Path(args.path)
    if not target.is_absolute():
        target = REACTOS_ROOT / args.path
    if args.oracle:
        sys.exit(run_oracle(target, args.function))

    r = analyze(target, args.function)
    print(f"{target.as_posix().split('/win32ss/')[-1]}:{args.function} — "
          f"{r.note}", file=sys.stderr)
    for g in r.groups:
        tag = ("SAFE→" + g.tag) if g.safe else "inline"
        print(f"  [{tag}] {'/'.join(g.labels[:3])}"
              f"{'…' if len(g.labels) > 3 else ''}  {g.reason}",
              file=sys.stderr)
    if args.apply and r.new_text:
        target.write_text(r.new_text)
        print(f"APPLIED → {target}", file=sys.stderr)
    elif not r.new_text:
        print("nothing applied (no safe arms / oracle fail)",
              file=sys.stderr)


def cmd_split_apply(args):
    from .file_splitter import (analyze_scope, plan_emit, apply_emit)
    from .config import REACTOS_ROOT

    targets: list[Path] = []
    if args.path:
        t = Path(args.path)
        if not t.is_absolute():
            t = REACTOS_ROOT / args.path
        targets = [t]
    else:
        for pl in analyze_scope(REACTOS_ROOT / "win32ss",
                                args.min_loc, args.target_loc):
            if pl.classification == "CLEAN_SPLIT":
                targets.append(pl.path)

    for t in targets:
        em = plan_emit(t, args.target_loc)
        rel = t.as_posix().split("/win32ss/")[-1]
        if not em.fragments:
            print(f"  skip {rel}: {em.note}")
            continue
        names = ", ".join(p.name for p, _ in em.fragments)
        print(f"  {'APPLY' if args.apply else 'plan'} {rel} → "
              f"{rel.rsplit('/',1)[-1]} + [{names}]  ({em.note})")
        if args.apply:
            apply_emit(em)


def cmd_gdi_locks(args):
    from .gdi_lock_inventory import (
        extract_file, extract_keepset, write_sidecar,
        print_summary, run_oracle,
    )
    from .config import REACTOS_ROOT

    if args.oracle:
        sys.exit(run_oracle(REACTOS_ROOT / "win32ss/gdi/ntgdi/dcutil.c"))

    target = Path(args.path)
    if not target.is_absolute():
        target = REACTOS_ROOT / args.path

    if target.is_file():
        sites = extract_file(target)
        if args.write_sidecars:
            print(f"wrote {write_sidecar(target, sites)}")
        if args.json:
            print(json.dumps(
                {"source": str(target),
                 "sites": [s.__dict__ for s in sites]},
                indent=2, default=str))
        else:
            for s in sites:
                sig = (" " + ",".join(s.signals)) if s.signals else ""
                print(f"  {s.function} L{s.func_line} "
                      f"{s.lock_fn}→{s.unlock_fn} {s.var}@L{s.lock_line} "
                      f"[{s.shape}] unlock@{s.unlock_lines}{sig}")
    elif target.is_dir():
        inv = extract_keepset(target)
        if args.write_sidecars:
            for p, ss in inv.items():
                write_sidecar(p, ss)
            print(f"wrote {len(inv)} sidecars")
        print_summary(inv)
    else:
        print(f"Error: not found: {target}", file=sys.stderr)
        sys.exit(1)


def cmd_user_locks(args):
    from .user_lock_inventory import (
        extract_file, extract_keepset, write_sidecar,
        print_summary, run_oracle,
    )
    from .config import REACTOS_ROOT

    if args.oracle:
        sys.exit(run_oracle(
            REACTOS_ROOT / "win32ss/user/ntuser/simplecall.c"))

    target = Path(args.path)
    if not target.is_absolute():
        target = REACTOS_ROOT / args.path

    if target.is_file():
        sites = extract_file(target)
        if args.write_sidecars:
            print(f"wrote {write_sidecar(target, sites)}")
        if args.json:
            print(json.dumps(
                {"source": str(target),
                 "sites": [s.__dict__ for s in sites]},
                indent=2, default=str))
        else:
            for s in sites:
                sig = (" " + ",".join(s.signals)) if s.signals else ""
                print(f"  {s.function} L{s.func_line} {s.mode} "
                      f"enter@L{s.enter_line}"
                      f"{'(top)' if s.enter_at_top else ''} "
                      f"[{s.shape}] leave@{s.leave_lines}{sig}")
    elif target.is_dir():
        inv = extract_keepset(target)
        if args.write_sidecars:
            for p, ss in inv.items():
                write_sidecar(p, ss)
            print(f"wrote {len(inv)} sidecars")
        print_summary(inv)
    else:
        print(f"Error: not found: {target}", file=sys.stderr)
        sys.exit(1)


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
    from .function_prompt import build_function_prompt, build_file_context, build_preamble_prompt, load_function_system_prompt
    from .response_cache import ResponseCache, PREAMBLE_KEY
    from .stitcher import stitch_file
    from .batch_runner import submit_single, TranslationResult
    from .assembler import assemble_file, generate_mod_rs, generate_lib_rs
    from .postprocess import postprocess
    from .prompt_builder import estimate_tokens
    from .sink import (RunArchive, ResponseSink, install_kill_guard,
                       paid_invocation_banner, warn_if_tmp_output)
    from .config import cost_usd

    # Resolve target: single file or directory
    target_path = Path(args.path)
    if not target_path.is_absolute():
        # Try repo-root-relative, ntoskrnl-relative, hal-relative.
        from .config import REACTOS_ROOT, HAL_DIR
        candidates = [
            REACTOS_ROOT / args.path,
            NTOSKRNL_DIR / args.path,
            HAL_DIR / args.path,
        ]
        target_path = next((c for c in candidates if c.exists()), candidates[0])

    # Detect target (ntoskrnl vs hal) from the path
    rustconv_target = resolve_target(args,target_path)
    print(f"Target: {rustconv_target.name}", file=sys.stderr)

    if target_path.is_file():
        c_files = [target_path]
    elif target_path.is_dir():
        c_files = sorted(target_path.glob("*.c"))
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
        print(f"HAL target does not support arch={args.arch}; supported: {rustconv_target.arches}", file=sys.stderr)
        sys.exit(1)

    print(f"Building catalog...", file=sys.stderr)
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
        print(f"  (--concurrency {conc} ignored: only for --mode mechanical on DeepSeek)",
              file=sys.stderr)
        conc = 1

    # Durable archive + kill guard for this paid invocation. Every streamed
    # response lands here as it generates, and a Ctrl-C points back at the
    # preserved work instead of feeling like it threw away what you paid for.
    archive = None
    running_cost = 0.0
    # Hard safety brake: stop before the next paid call once cumulative spend
    # crosses this. None = no cap. Prevents a runaway bill.
    cost_cap = getattr(args, "cost_cap", None)
    no_resume = getattr(args, "no_resume", False)
    if not args.dry_run:
        warn_if_tmp_output(getattr(args, "output", None))
        archive = RunArchive(label=target_path.name, model=model,
                             target=rustconv_target.name)
        install_kill_guard(archive)

    total_files = len(c_files)
    for file_idx, c_file in enumerate(c_files, 1):
        print(f"\n{'='*60}", file=sys.stderr)
        print(f"[{file_idx}/{total_files}] {c_file.name}", file=sys.stderr)
        print(f"{'='*60}", file=sys.stderr)

        capped = False  # set if --cost-cap is hit mid-file
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
                    sorted_deps[0], func_res, {}, all_deps, scan, catalog_fn_names, file_res.exports,
                    target=rustconv_target,
                )
                if not use_claude:
                    prompt = system_prompt + "\n\n---\n\n" + prompt if system_prompt else prompt
                tokens = estimate_tokens(prompt)
                print(f"\n  First function prompt: ~{tokens:,} tokens", file=sys.stderr)

            if args.prompt:
                # Print full prompt for first function
                func_res = resolve_function(sorted_deps[0], catalog, file_res.exports, scan.defines)
                prompt = build_function_prompt(
                    sorted_deps[0], func_res, {}, all_deps, scan, catalog_fn_names, file_res.exports,
                    target=rustconv_target,
                )
                full = (system_prompt + "\n\n---\n\n" + prompt) if system_prompt else prompt
                print(full)

            continue

        # Translate function by function
        all_deps_for_context = extract_call_graph(scan, spec_exports=file_res.exports)
        translated: dict[str, str] = {}
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
        cache = ResponseCache(f"{subsystem}__{c_file.stem}")
        resumed = 0
        if cache.count() and not no_resume:
            print(f"  Resume cache: {cache.count()} prior unit(s) on disk "
                  f"(unchanged → $0)", file=sys.stderr)

        # One paid banner per file: n calls = functions + preamble. Rough input
        # estimate = first-function prompt × function count (each call re-sends
        # context, so this is a floor, not a ceiling).
        if archive is not None and sorted_deps:
            _fr0 = resolve_function(sorted_deps[0], catalog, file_res.exports, scan.defines)
            _p0 = build_function_prompt(sorted_deps[0], _fr0, {}, all_deps_for_context,
                                        scan, catalog_fn_names, file_res.exports,
                                        target=rustconv_target)
            _est_in = (estimate_tokens(_p0) + estimate_tokens(system_prompt)) * len(sorted_deps)
            paid_invocation_banner(model, len(sorted_deps) + 1, _est_in, archive)

        # PARALLEL translate (mechanical + DeepSeek): fan out the paid per-function
        # calls. Workers do ONLY the submit (pure — they read shared state, write only
        # their own durable sink); ALL bookkeeping runs serially on the main thread as
        # futures complete, so no shared-state locking is needed. Resume-cached fns are
        # collected first (free). Cost-cap sets a stop Event that blocks new submits
        # (in-flight calls — at most `conc` — may still land).
        if conc > 1:
            import threading
            from concurrent.futures import ThreadPoolExecutor, as_completed
            todo = []
            for func_idx, dep in enumerate(sorted_deps, 1):
                if not no_resume:
                    hit = cache.get(dep.name, dep.body_text)
                    if hit is not None:
                        translated[dep.name] = hit
                        resumed += 1
                        continue
                todo.append((func_idx, dep))
            ctx = dict(translated)   # static callee context (mechanical fns are independent)
            stop = threading.Event()

            def _work(item):
                fi, dp = item
                if stop.is_set():
                    return (fi, dp, None)
                fres = resolve_function(dp, catalog, file_res.exports, scan.defines)
                up = build_function_prompt(dp, fres, ctx, all_deps_for_context, scan,
                                           catalog_fn_names, file_res.exports,
                                           target=rustconv_target, include_shared=False)
                sk = ResponseSink(archive.stream_path(fi, dp.name)) if archive is not None else None
                try:
                    r = submit_single(up, model=model, system_prompt=system_prompt,
                                      file_context=file_context, sink=sk)
                finally:
                    if sk is not None:
                        sk.close()
                return (fi, dp, r)

            print(f"  ⚡ parallel: {len(todo)} fn(s) @ concurrency={conc} "
                  f"({resumed} cached)", file=sys.stderr)
            with ThreadPoolExecutor(max_workers=conc) as ex:
                for fut in as_completed([ex.submit(_work, it) for it in todo]):
                    fi, dp, r = fut.result()
                    if r is None:
                        continue   # skipped after cost cap
                    if r.success:
                        translated[dp.name] = r.rust_code
                        cache.put(dp.name, dp.body_text, r.rust_code)
                        total_in += r.input_tokens
                        total_out += r.output_tokens
                        if archive is not None:
                            archive.commit(fi, dp.name, r.rust_code, r.input_tokens, r.output_tokens)
                            running_cost += cost_usd(model, r.input_tokens, r.output_tokens)
                        print(f"  [{fi}] {dp.name} OK ({r.input_tokens}/{r.output_tokens})"
                              f"  [running ${running_cost:.2f}]", file=sys.stderr)
                        if cost_cap is not None and running_cost >= cost_cap:
                            stop.set()
                            capped = True
                            print(f"  ⛔ COST CAP ${running_cost:.2f} ≥ ${cost_cap:.2f} "
                                  f"— halting new calls", file=sys.stderr)
                    else:
                        print(f"  [{fi}] {dp.name} FAIL: {r.error}", file=sys.stderr)

        for func_idx, dep in enumerate(sorted_deps, 1):
            if conc > 1:
                break   # already handled by the parallel path above
            print(f"  [{func_idx}/{len(sorted_deps)}] {dep.name}...", file=sys.stderr, end=" ", flush=True)

            # Resume: reuse a prior unchanged-body translation for free. It still
            # feeds `translated` (callee context for later functions).
            if not no_resume:
                hit = cache.get(dep.name, dep.body_text)
                if hit is not None:
                    translated[dep.name] = hit
                    resumed += 1
                    print("CACHED ($0)", file=sys.stderr)
                    continue

            # Per-function resolve
            func_res = resolve_function(dep, catalog, file_res.exports, scan.defines)

            # Build the function-specific prompt only — the shared import/gap/
            # globals context is sent separately as a cached system block
            # (file_context), so include_shared=False keeps it out of the
            # full-price per-function user message.
            user_prompt = build_function_prompt(
                dep, func_res, translated, all_deps_for_context,
                scan, catalog_fn_names, file_res.exports,
                target=rustconv_target, include_shared=False,
            )

            # Stream this function's output straight to the durable archive.
            sink = None
            if archive is not None:
                sink = ResponseSink(archive.stream_path(func_idx, dep.name))
            try:
                if use_claude:
                    result = submit_single(user_prompt, model=model,
                                           system_prompt=system_prompt,
                                           file_context=file_context, sink=sink)
                else:
                    prefix = "\n\n---\n\n".join(p for p in (system_prompt, file_context) if p)
                    full_prompt = (prefix + "\n\n---\n\n" + user_prompt) if prefix else user_prompt
                    result = submit_single(full_prompt, model=model, sink=sink)
            finally:
                if sink is not None:
                    sink.close()

            if result.success:
                translated[dep.name] = result.rust_code
                # Persist for cross-run resume the instant it returns.
                cache.put(dep.name, dep.body_text, result.rust_code)
                total_in += result.input_tokens
                total_out += result.output_tokens
                if archive is not None:
                    archive.commit(func_idx, dep.name, result.rust_code,
                                   result.input_tokens, result.output_tokens)
                    running_cost += cost_usd(model, result.input_tokens, result.output_tokens)
                    print(f"OK ({result.input_tokens} in / {result.output_tokens} out)"
                          f"  [running ${running_cost:.2f}]", file=sys.stderr)
                else:
                    print(f"OK ({result.input_tokens} in / {result.output_tokens} out)", file=sys.stderr)
            else:
                print(f"FAIL: {result.error}", file=sys.stderr)

            # Hard cost cap — stop before the next (paid) call if we've crossed it.
            if cost_cap is not None and running_cost >= cost_cap:
                print(f"  ⛔ COST CAP HIT: ${running_cost:.2f} ≥ ${cost_cap:.2f} after "
                      f"{func_idx}/{len(sorted_deps)} functions — stopping.", file=sys.stderr)
                capped = True
                break

        if not translated:
            print(f"  No functions translated for {c_file.name}", file=sys.stderr)
            continue

        # Translate the file-scope preamble (structs / typedefs /
        # #define→const / static tables) as ONE synthetic unit. The
        # stitcher otherwise emits no preamble at all, so files with a
        # non-trivial preamble (most drivers) would not compile.
        preamble_rust = ""
        preamble_src = (scan.preamble_text or scan.globals_text or "").strip()
        pre_hit = None if (no_resume or not preamble_src) else cache.get(PREAMBLE_KEY, preamble_src)
        if pre_hit is not None:
            preamble_rust = pre_hit
            resumed += 1
            print(f"  [preamble] CACHED ($0)", file=sys.stderr)
        elif preamble_src and not capped:
            print(f"  [preamble] {len(preamble_src)} C chars...",
                  file=sys.stderr, end=" ", flush=True)
            pre_prompt = build_preamble_prompt(scan, file_res, target=rustconv_target)
            pre_sink = None
            if archive is not None:
                pre_sink = ResponseSink(archive.stream_path(0, "preamble"))
            try:
                if use_claude:
                    pre_res = submit_single(pre_prompt, model=model,
                                            system_prompt=system_prompt, sink=pre_sink)
                else:
                    pre_full = (system_prompt + "\n\n---\n\n" + pre_prompt) if system_prompt else pre_prompt
                    pre_res = submit_single(pre_full, model=model, sink=pre_sink)
            finally:
                if pre_sink is not None:
                    pre_sink.close()
            if pre_res.success:
                preamble_rust = pre_res.rust_code
                cache.put(PREAMBLE_KEY, preamble_src, pre_res.rust_code)
                total_in += pre_res.input_tokens
                total_out += pre_res.output_tokens
                if archive is not None:
                    archive.commit(0, "preamble", pre_res.rust_code,
                                   pre_res.input_tokens, pre_res.output_tokens)
                    running_cost += cost_usd(model, pre_res.input_tokens, pre_res.output_tokens)
                print(f"OK ({pre_res.input_tokens} in / {pre_res.output_tokens} out)",
                      file=sys.stderr)
            else:
                print(f"FAIL: {pre_res.error} (stitching without preamble)",
                      file=sys.stderr)

        print(f"\n  Stitching {len(translated)} functions...", file=sys.stderr)
        if resumed:
            print(f"  Resumed {resumed} unit(s) from cache this run (re-translation skipped, $0)",
                  file=sys.stderr)
        print(f"  Total tokens: {total_in:,} in / {total_out:,} out", file=sys.stderr)

        # Stitch functions into a single file
        stitched = stitch_file(translated, sorted_deps, scan, file_res,
                               target=rustconv_target, preamble=preamble_rust)

        # Post-process
        code = postprocess(stitched, exports=file_res.exports, globals=scan.globals,
                           target_name=rustconv_target.name,
                           sys_crate=rustconv_target.sys_crate,
                           sys_umbrella=rustconv_target.sys_umbrella,
                           c_source_path=c_file)

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
                                     exports=file_res.exports, scan=scan,
                                     target=rustconv_target, c_source_path=c_file)
            print(f"  Assembled to {out_path}", file=sys.stderr)

        # Respect the cost cap across files too, not just within one file.
        if capped:
            print(f"\n⛔ Stopping run: cost cap ${cost_cap:.2f} reached "
                  f"(run total ≈ ${running_cost:.2f}). Remaining files NOT translated.",
                  file=sys.stderr)
            break

    # Generate mod.rs and lib.rs for batch runs (ntoskrnl only — HAL handles
    # mod.rs incrementally via _hal_register_mod and has a single src/x86 dir).
    if total_files > 1 and not args.dry_run and rustconv_target.name == "ntoskrnl":
        subsystem = c_files[0].parent.name
        module_names = [f.stem for f in c_files]
        generate_mod_rs(subsystem, module_names)
        generate_lib_rs([subsystem])
        print(f"\nUpdated mod.rs and lib.rs for {subsystem}/", file=sys.stderr)

    if archive is not None:
        print(f"\n  Total spent this run: ${running_cost:.2f}  "
              f"({len(archive.committed)} unit(s) archived)", file=sys.stderr)
        print(f"  Durable archive: {archive.dir}", file=sys.stderr)

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
        from .config import REACTOS_ROOT, HAL_DIR
        for candidate in (REACTOS_ROOT / args.path, NTOSKRNL_DIR / args.path, HAL_DIR / args.path):
            if candidate.exists():
                target = candidate
                break
        else:
            target = NTOSKRNL_DIR / args.path

    if target.is_file():
        c_files = [target]
    elif target.is_dir():
        c_files = sorted(target.glob("*.c"))
    else:
        print(f"Error: not found: {target}", file=sys.stderr)
        sys.exit(1)

    rustconv_target = resolve_target(args,target)
    catalog = build_catalog(arch=args.arch, target=rustconv_target)
    spec = parse_spec(rustconv_target.spec_file) if rustconv_target.spec_file else {}

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

    # Output runs ~0.6x of input for C→Rust (Rust expands the source).
    total_output = int(total_input * 0.6)

    print(f"\nTotal: {len(c_files)} files, {total_lines:,} lines")
    print(f"Estimated tokens (single full-file prompt): "
          f"~{total_input:,} input, ~{total_output:,} output")
    print("\n  ⚠️  NOTE: `submit`/`batch` send ONE prompt per file. But `translate`")
    print("     (the path used for big files) sends one paid call PER FUNCTION,")
    print("     each RE-SENDING the system prompt + growing context — real input")
    print("     is commonly 2–5× the figure above. These estimates are a FLOOR.")

    from .config import MODEL_PRICING, BATCH_DISCOUNT
    rows = [
        ("Opus   (claude-opus-4-8)",   "claude-opus-4-8"),
        ("Sonnet (claude-sonnet-4-5)", "claude-sonnet-4-5-20250929"),
        ("Haiku  (claude-haiku-4-5)",  "claude-haiku-4-5-20251001"),
        ("Gemini 2.5 Pro",             "gemini-2.5-pro"),
        ("Gemini 2.5 Flash",           "gemini-2.5-flash"),
    ]
    print("\nEstimated cost (sync = translate/submit; batch = `batch` cmd, 50% off):")
    print(f"  {'model':<28} {'sync':>10}  {'batch':>10}   sync @5x")
    for label, mid in rows:
        pin, pout = MODEL_PRICING[mid]
        sync = total_input * pin / 1_000_000 + total_output * pout / 1_000_000
        batch = sync * BATCH_DISCOUNT
        print(f"  {label:<28} {'$' + format(sync, '.2f'):>10}  "
              f"{'$' + format(batch, '.2f'):>10}   ${sync * 5:.2f}")


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
    p.add_argument("--target", help="Override target: a known target name or a dll/win32/<name> crate name")
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

    # seh-inventory (win32k Pass 0 triage)
    p = sub.add_parser("seh-inventory",
                       help="Win32k Pass 0: triage SEH sites a/b/c/d/e + sidecars")
    p.add_argument("path", nargs="?", default="win32ss",
                   help="C file or dir (relative to repo root or absolute); "
                        "default win32ss/ keep-set")
    p.add_argument("--json", action="store_true", help="Output JSON (file mode)")
    p.add_argument("--write-sidecars", action="store_true",
                   help="Write <file>.win32k_meta.json next to each source")
    p.add_argument("--oracle", action="store_true",
                   help="Run the winsta.c spike self-test and exit")

    # seh-rewrite (win32k Pass 1 mechanical probe lowering)
    p = sub.add_parser("seh-rewrite",
                       help="Win32k Pass 1: mechanically lower category-(a) "
                            "probe SEH sites to SafeProbeFor*")
    p.add_argument("path", nargs="?", default="win32ss",
                   help="C file or dir (relative to repo root or absolute)")
    p.add_argument("--apply", action="store_true",
                   help="Write changes in place (default: print unified diff)")
    p.add_argument("--oracle", action="store_true",
                   help="Run the winsta.c spike self-test and exit")

    # seh-belt (win32k Pass 1c: delete (d) BELT_SUSPENDERS SEH wrappers)
    p = sub.add_parser("seh-belt",
                       help="Win32k Pass 1c: delete category-(d) BELT_SUSPENDERS "
                            "SEH wrappers (user write, no probe, dead handler "
                            "under DUMMY_PSEH)")
    p.add_argument("path", nargs="?", default="win32ss",
                   help="C file or dir (relative to repo root or absolute)")
    p.add_argument("--apply", action="store_true",
                   help="Write changes in place (default: print unified diff)")
    p.add_argument("--oracle", action="store_true",
                   help="Run the synthetic self-test and exit")

    # callout-annotate (win32k Pass 3 USER callout extraction — no rewrite)
    p = sub.add_parser("callout-annotate",
                       help="Win32k Pass 3: extract KeUserModeCallback "
                            "callout sites + sidecars (no source rewrite)")
    p.add_argument("path", nargs="?", default="win32ss",
                   help="C file or dir (relative to repo root or absolute); "
                        "default win32ss/ keep-set")
    p.add_argument("--json", action="store_true", help="Output JSON (file mode)")
    p.add_argument("--write-sidecars", action="store_true",
                   help="Write <file>.win32k_callouts.json next to each source")
    p.add_argument("--oracle", action="store_true",
                   help="Run the callback.c ground-truth self-test and exit")

    # seh-llm-lower (win32k Pass 1b: LLM lowering of the 64-site ledger)
    p = sub.add_parser("seh-llm-lower",
                       help="Win32k Pass 1b: LLM-lower the Pass-1 "
                            "manual-followup ledger (cached, validated)")
    p.add_argument("--apply", action="store_true",
                   help="Splice validated rewrites in place (default: dry-run)")
    p.add_argument("--diff", action="store_true",
                   help="Print unified diffs for the ready rewrites")
    p.add_argument("--model", default="",
                   help="Model id/shortcut (default: Sonnet)")
    p.add_argument("--only", default=None,
                   help="Restrict to ledger files whose path contains this")
    p.add_argument("--oracle", action="store_true",
                   help="Structural self-test (no API spend) and exit")

    # split-plan (win32k giant-file split analyzer — analyze-only)
    p = sub.add_parser("split-plan",
                       help="Win32k: analyze giant .c for safe split seams "
                            "(static-coupling graph; no mutation)")
    p.add_argument("path", nargs="?", default="win32ss",
                   help="C file or dir (default win32ss/ translation scope)")
    p.add_argument("--min-loc", type=int, default=1000,
                   help="Only consider files with ≥ this LOC (default 1000)")
    p.add_argument("--target-loc", type=int, default=900,
                   help="Target per-group LOC for the split (default 900)")
    p.add_argument("--json", action="store_true", help="Output JSON")

    # switch-extract (win32k tree-sitter switch-dispatcher de-monstering)
    p = sub.add_parser("switch-extract",
                       help="Win32k: extract proven-safe switch arms of a "
                            "giant dispatcher fn into static helpers")
    p.add_argument("path", help="C file (relative to repo root or absolute)")
    p.add_argument("--function", required=True,
                   help="Target function name (e.g. SpiGetSet)")
    p.add_argument("--apply", action="store_true",
                   help="Write the transform (default: dry-run report)")
    p.add_argument("--oracle", action="store_true",
                   help="Self-test (label-set + reparse invariants) and exit")

    # split-apply (win32k giant-file #include-back emitter)
    p = sub.add_parser("split-apply",
                       help="Win32k: emit the #include-back split for "
                            "CLEAN_SPLIT files (one TU, no CMake change)")
    p.add_argument("path", nargs="?", default=None,
                   help="One C file; omit to do ALL CLEAN_SPLIT in scope")
    p.add_argument("--apply", action="store_true",
                   help="Write fragments + rewrite base (default: dry-run)")
    p.add_argument("--min-loc", type=int, default=1000)
    p.add_argument("--target-loc", type=int, default=900)

    # gdi-locks (win32k Pass 2 GDI manual-lock-bracket detection — no rewrite)
    p = sub.add_parser("gdi-locks",
                       help="Win32k Pass 2: detect GDI manual lock brackets "
                            "+ sidecars (no source rewrite)")
    p.add_argument("path", nargs="?", default="win32ss",
                   help="C file or dir (relative to repo root or absolute); "
                        "default win32ss/ keep-set")
    p.add_argument("--json", action="store_true", help="Output JSON (file mode)")
    p.add_argument("--write-sidecars", action="store_true",
                   help="Write <file>.win32k_gdilocks.json next to each source")
    p.add_argument("--oracle", action="store_true",
                   help="Run the dcutil.c ground-truth self-test and exit")

    # user-locks (win32k Pass 2b USER critsec-bracket detection — no rewrite)
    p = sub.add_parser("user-locks",
                       help="Win32k Pass 2b: detect USER critical-section "
                            "brackets + sidecars (no source rewrite)")
    p.add_argument("path", nargs="?", default="win32ss",
                   help="C file or dir (relative to repo root or absolute); "
                        "default win32ss/ keep-set")
    p.add_argument("--json", action="store_true", help="Output JSON (file mode)")
    p.add_argument("--write-sidecars", action="store_true",
                   help="Write <file>.win32k_userlocks.json next to each source")
    p.add_argument("--oracle", action="store_true",
                   help="Run the simplecall.c ground-truth self-test and exit")

    # translate (function-by-function)
    p = sub.add_parser("translate", help="Function-by-function translation (improved pipeline)")
    p.add_argument("path", help="C file or directory (relative to ntoskrnl/ or absolute)")
    p.add_argument("--target", help="Override target: a known target name or a dll/win32/<name> crate name")
    p.add_argument("-m", "--model", help="Model: sonnet, haiku, opus, gemini-pro, gemini-flash, deepseek, deepseek-r1, or full ID")
    p.add_argument("--mode", default="idiomatic", choices=["idiomatic", "mechanical"],
                   help="idiomatic (default): translate + idiomize in one pass. "
                        "mechanical: faithful transliteration only — compiles, "
                        "preserves C control flow/pointers, no idiom, no stubs "
                        "(pass 1 of a two-pass cheap-model pipeline).")
    p.add_argument("-o", "--output", help="Write Rust output to file (single file only)")
    p.add_argument("--fn", dest="fn", help="Translate only this function")
    p.add_argument("--dry-run", action="store_true", help="Show translation plan without API calls")
    p.add_argument("--prompt", action="store_true", help="Print prompt for first function (implies --dry-run)")
    p.add_argument("--exclude", nargs="*", default=[],
                   help="File stems to skip (batch mode)")
    p.add_argument("--cost-cap", dest="cost_cap", type=float, default=None,
                   help="Hard USD spend cap for this invocation; stops before the next "
                        "call once cumulative cost reaches it (safety brake).")
    p.add_argument("--concurrency", type=int, default=1,
                   help="Fan out per-function calls N-at-a-time (DeepSeek + --mode "
                        "mechanical ONLY; ignored otherwise). Default 1 (sequential). "
                        "Try 5; pair with --cost-cap as the runaway brake.")
    p.add_argument("--no-resume", dest="no_resume", action="store_true",
                   help="Ignore the .responses/ resume cache and re-translate every "
                        "function fresh (cache still refreshed). Default resumes "
                        "unchanged-body functions for $0.")

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
    except ValueError as e:
        # target auto-detect failure (unrecognized path) surfaces here as a clean
        # message instead of a traceback — tells the user to pass --target.
        print(str(e), file=sys.stderr)
        sys.exit(2)


if __name__ == "__main__":
    main()
