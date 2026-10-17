use std::io::{self, Write};
use std::path::PathBuf;
use vibeos_lint::{rules, Corpus, Rule, SourceNeeds};

fn run() -> Result<(), String> {
    let mut root = PathBuf::from(".");
    let mut inputs = Vec::<PathBuf>::new();
    let mut names = None;
    let mut all = false;
    let mut list = false;
    let mut binary_names: Option<String> = None;
    let mut binary_explain = None;
    let mut build_dir = None;
    let mut arch = None;
    let mut target = None;
    let mut packages = Vec::new();
    let mut include_declarations = false;
    let mut crates = Vec::new();
    let mut args = std::env::args().skip(1);
    while let Some(arg) = args.next() {
        match arg.as_str() {
            "--crates" => {
                crates = args
                    .next()
                    .ok_or("--crates requires names")?
                    .split(',')
                    .map(String::from)
                    .collect()
            }
            "--target" => target = Some(args.next().ok_or("--target requires a triple")?),
            "--package" => packages.push(args.next().ok_or("--package requires a name")?),
            "--include-declarations" => include_declarations = true,
            "--explain" => {
                binary_explain = Some(args.next().ok_or("--explain requires substring")?)
            }
            "--binary-rules" => {
                binary_names = Some(args.next().ok_or("--binary-rules requires names")?)
            }
            "--build-dir" => {
                build_dir = Some(PathBuf::from(
                    args.next().ok_or("--build-dir requires path")?,
                ))
            }
            "--arch" => arch = Some(args.next().ok_or("--arch requires architecture")?),
            "--list-binary-rules" => {
                for name in vibeos_lint::binary::RULES {
                    println!("{name}");
                }
                return Ok(());
            }
            "--python" => std::env::set_var(
                "VIBEOS_LINT_PYTHON",
                args.next().ok_or("--python requires an executable")?,
            ),
            "--shellcheck" => std::env::set_var(
                "VIBEOS_LINT_SHELLCHECK",
                args.next().ok_or("--shellcheck requires an executable")?,
            ),
            "--root" => root = args.next().ok_or("--root requires a path")?.into(),
            "--path" => inputs.push(args.next().ok_or("--path requires a path")?.into()),
            "--rules" => names = Some(args.next().ok_or("--rules requires names")?),
            "--all-ported" => all = true,
            "--list-rules" => list = true,
            "--help" | "-h" => {
                println!("vibeos-lint [--root REPO] [--path SOURCE] (--rules a,b | --all-ported | --list-rules)");
                println!("vibeos-lint --root REPO --binary-rules a,b --arch ARCH [--build-dir PATH] [--path INPUT]");
                println!("vibeos-lint --list-binary-rules");
                return Ok(());
            }
            _ => return Err(format!("unknown argument: {arg}")),
        }
    }
    let source_selected = names.is_some();
    if let Some(names) = binary_names {
        if all || list || source_selected || names.is_empty() {
            return Err("binary rules require a separate invocation".into());
        }
        let root = std::fs::canonicalize(root).map_err(|e| e.to_string())?;
        let arch = arch.ok_or("--binary-rules requires --arch")?;
        if !["i386", "amd64", "arm64"].contains(&arch.as_str())
            && !(arch == "all" && names == "committed_drift")
        {
            return Err(format!("unsupported architecture: {arch}"));
        }
        let build_dir = build_dir.unwrap_or_else(|| root.join(format!("build-{arch}")));
        let mut ctx = vibeos_lint::binary::BinaryContext::new(root, build_dir, arch);
        ctx.inputs = inputs
            .into_iter()
            .map(|p| if p.is_absolute() { p } else { ctx.root.join(p) })
            .collect();
        if let Some(target) = target {
            ctx.target = target;
        }
        ctx.packages = packages;
        ctx.include_declarations = include_declarations;
        ctx.crates = crates;
        let names = names.split(',').map(String::from).collect::<Vec<_>>();
        if let Some(substring) = binary_explain {
            if names != ["scan_flex_array_panic"] {
                return Err("--explain requires only scan_flex_array_panic".into());
            }
            let output = vibeos_lint::binary::explain(&ctx, &substring)?;
            print!("{}", output.stdout);
            eprint!("{}", output.stderr);
            if output.returncode != 0 {
                return Err(format!("explain scanner exited {}", output.returncode));
            }
        } else {
            println!("{}", vibeos_lint::binary::run(&ctx, &names)?);
        }
        return Ok(());
    }
    if arch.is_some()
        || build_dir.is_some()
        || target.is_some()
        || !packages.is_empty()
        || include_declarations
        || binary_explain.is_some()
        || !crates.is_empty()
    {
        return Err("artifact options require --binary-rules".into());
    }
    let registry = rules::registry();
    if list {
        for rule in &registry {
            println!("{}", rule.name());
        }
        return Ok(());
    }
    if all == names.is_some() {
        return Err("select --rules or --all-ported".into());
    }
    let requested: Vec<_> = names
        .as_deref()
        .map(|s| s.split(',').collect())
        .unwrap_or_default();
    for name in &requested {
        if !registry.iter().any(|rule| rule.name() == *name) {
            return Err(format!("unknown rule: {name}"));
        }
    }
    let selected: Vec<&dyn Rule> = registry
        .iter()
        .filter(|r| all || requested.contains(&r.name()))
        .map(|r| r.as_ref())
        .collect();
    if selected.is_empty() {
        return Err("no rules selected".into());
    }
    let ast_cache_nodes = match std::env::var("VIBEOS_LINT_AST_CACHE_NODES") {
        Ok(value) => Some(
            value
                .parse::<usize>()
                .map_err(|_| "VIBEOS_LINT_AST_CACHE_NODES must be a nonnegative integer")?,
        ),
        Err(std::env::VarError::NotPresent) => None,
        Err(error) => return Err(format!("VIBEOS_LINT_AST_CACHE_NODES: {error}")),
    };
    let jobs = std::env::var("VIBEOS_LINT_JOBS")
        .ok()
        .map(|raw| {
            raw.parse::<usize>()
                .ok()
                .filter(|&n| n > 0)
                .ok_or("VIBEOS_LINT_JOBS must be positive")
        })
        .transpose()?;
    // Each worker may hold a large generated-source AST. Leave room for other
    // lanes and avoid multiplying parser peaks by every logical CPU on the host.
    let jobs = match jobs {
        Some(jobs) => jobs,
        None => std::thread::available_parallelism()
            .map_err(|e| format!("available CPU parallelism: {e}"))?
            .get()
            .min(8),
    };
    let builder = rayon::ThreadPoolBuilder::new().num_threads(jobs);
    let pool = builder.build().map_err(|e| e.to_string())?;
    let results = pool.install(|| {
        let needs = SourceNeeds {
            workspace_rust: selected.iter().any(|r| {
                r.needs_workspace_rust() && !r.needs_sdk_rust() && !r.needs_cdecl_sources()
            }),
            module_c: selected.iter().any(|r| r.needs_c_sources()),
            tracked_c: selected.iter().any(|r| r.needs_c_twins()),
            cargo_manifests: selected.iter().any(|r| r.needs_module_tree()),
            sdk_rust: selected.iter().any(|r| r.needs_sdk_rust()),
            cdecl_sources: selected.iter().any(|r| r.needs_cdecl_sources()),
        };
        let extras: Vec<PathBuf> = selected
            .iter()
            .flat_map(|r| r.extra_files().iter().map(PathBuf::from))
            .collect();
        let mut source_sets: Vec<_> = selected
            .iter()
            .flat_map(|rule| rule.source_sets().iter().copied())
            .collect();
        if selected.iter().any(|r| r.needs_module_inputs()) {
            source_sets.extend_from_slice(vibeos_lint::module_inputs::SOURCE_SETS);
        }
        let started = std::time::Instant::now();
        let mut corpus =
            Corpus::load_scopes_with_sets(&root, &inputs, needs, &extras, &source_sets)?;
        if let Some(nodes) = ast_cache_nodes {
            corpus = corpus.with_ast_cache_nodes(nodes);
        }
        if std::env::var_os("VIBEOS_LINT_TIMINGS").is_some() {
            eprintln!(
                "TIMING\tsource_snapshot\t{:.3}\t{} files\t{} bytes",
                started.elapsed().as_secs_f64(),
                corpus.files.len(),
                corpus.files.iter().map(|f| f.bytes().len()).sum::<usize>()
            );
        }
        let started = std::time::Instant::now();
        let results = vibeos_lint::execute(&corpus, &selected);
        if std::env::var_os("VIBEOS_LINT_TIMINGS").is_some() {
            eprintln!(
                "TIMING\tsource_rules\t{:.3}",
                started.elapsed().as_secs_f64()
            );
        }
        if std::env::var_os("VIBEOS_LINT_CACHE_STATS").is_some() {
            let started = std::time::Instant::now();
            eprintln!("CACHE_STATS\t{}", corpus.cache_stats());
            if std::env::var_os("VIBEOS_LINT_TIMINGS").is_some() {
                eprintln!(
                    "TIMING\tsource_cache_stats\t{:.3}",
                    started.elapsed().as_secs_f64()
                );
            }
        }
        // This is a one-shot CLI snapshot with no destructor side effects.
        // Cached trees/views can occupy gigabytes: walking their allocations
        // merely to free them faults swapped pages back in before process exit.
        // Let the OS reclaim them, including on a failed check. Library callers
        // still own and drop their Corpus normally.
        std::mem::forget(corpus);
        results
    })?;
    // Buffer all results until every plugin succeeds; never publish partial coverage.
    let stdout = io::stdout();
    let mut stdout = stdout.lock();
    for (name, lines) in results {
        for line in lines {
            writeln!(stdout, "{name}\t{line}").map_err(|e| e.to_string())?;
        }
    }
    Ok(())
}

fn main() {
    if let Err(error) = run() {
        eprintln!("FATAL: {error}");
        std::process::exit(2);
    }
}
