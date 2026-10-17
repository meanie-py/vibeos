use std::io::{self, Write};
use std::path::PathBuf;
use vibeos_lint::{rules, Corpus, Rule, SourceNeeds};

fn run() -> Result<(), String> {
    let mut root = PathBuf::from(".");
    let mut inputs = Vec::<PathBuf>::new();
    let mut names = None;
    let mut all = false;
    let mut list = false;
    let mut args = std::env::args().skip(1);
    while let Some(arg) = args.next() {
        match arg.as_str() {
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
                return Ok(());
            }
            _ => return Err(format!("unknown argument: {arg}")),
        }
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
    let jobs = std::env::var("VIBEOS_LINT_JOBS")
        .ok()
        .map(|raw| {
            raw.parse::<usize>()
                .ok()
                .filter(|&n| n > 0)
                .ok_or("VIBEOS_LINT_JOBS must be positive")
        })
        .transpose()?;
    let mut builder = rayon::ThreadPoolBuilder::new();
    if let Some(jobs) = jobs {
        builder = builder.num_threads(jobs);
    }
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
        let corpus = Corpus::load_scopes_with_sets(&root, &inputs, needs, &extras, &source_sets)?;
        if std::env::var_os("VIBEOS_LINT_TIMINGS").is_some() {
            eprintln!(
                "TIMING\tsource_snapshot\t{:.3}\t{} files\t{} bytes",
                started.elapsed().as_secs_f64(),
                corpus.files.len(),
                corpus.files.iter().map(|f| f.bytes().len()).sum::<usize>()
            );
        }
        vibeos_lint::execute(&corpus, &selected)
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
