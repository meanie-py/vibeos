use std::io::{self, Write};
use std::path::PathBuf;
use vibeos_lint::{rules, Corpus, Rule};

fn run() -> Result<(), String> {
    let mut root = PathBuf::from(".");
    let mut inputs = Vec::<PathBuf>::new();
    let mut names = None;
    let mut all = false;
    let mut list = false;
    let mut args = std::env::args().skip(1);
    while let Some(arg) = args.next() {
        match arg.as_str() {
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
        let corpus =
            Corpus::load_paths(&root, &inputs, selected.iter().any(|r| r.needs_c_sources()))?;
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
