pub mod apitest;
pub mod winetest;
#[derive(Debug, PartialEq, Eq)]
pub struct Generated {
    pub suites: String,
    pub skips: String,
    pub warnings: Vec<String>,
    pub status: String,
}
fn read_optional(p: &std::path::Path) -> Result<String, String> {
    match std::fs::read_to_string(p) {
        Ok(v) => Ok(v),
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => Ok(String::new()),
        Err(e) => Err(format!("{}: {e}", p.display())),
    }
}
fn valid_arch(arch: &str) -> Result<(), String> {
    if ["i386", "amd64", "arm64"].contains(&arch) {
        Ok(())
    } else {
        Err(format!("unknown architecture: {arch}"))
    }
}

fn within(path: &std::path::Path, root: &std::path::Path) -> Result<(), String> {
    let target = path
        .canonicalize()
        .map_err(|e| format!("{}: {e}", path.display()))?;
    let root = root
        .canonicalize()
        .map_err(|e| format!("{}: {e}", root.display()))?;
    if !target.starts_with(&root) {
        return Err(format!(
            "{} resolves outside {}",
            path.display(),
            root.display()
        ));
    }
    Ok(())
}
