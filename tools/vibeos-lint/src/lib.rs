//! Statically linked lint plugins over an immutable, shared source snapshot.
mod ast_facts;
pub mod big_math;
pub mod binary;
pub mod catalog;
pub mod const_shared;
pub mod const_values;
mod crate_cfg;
pub mod vendor_registry;
pub mod yaml;
pub use crate_cfg::CrateCfg;
mod history;
pub mod integers;
pub use history::VersionHistory;
mod inventory;
pub use inventory::SourceSet;
pub mod lexical;
pub mod link_model;
pub mod module_inputs;
pub mod modules;
pub mod regexes;
pub mod rules;
pub mod syntax;
mod syntax_cache;
pub mod twins;

use rayon::prelude::*;
use regex::Regex;
use std::collections::{BTreeMap, HashMap};
use std::fs;
use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex, OnceLock, Weak};

pub trait Rule: Sync {
    fn name(&self) -> &'static str;
    fn run(&self, corpus: &Corpus) -> Result<Vec<String>, String>;
    /// Registry-only checks also support standalone trees without sdk/lib/rust.
    fn needs_workspace_rust(&self) -> bool {
        true
    }
    fn source_sets(&self) -> &'static [SourceSet] {
        &[]
    }
    fn needs_c_signature_asts(&self) -> &'static [&'static str] {
        &[]
    }
    fn needs_module_inputs(&self) -> bool {
        false
    }
    fn needs_version_history(&self) -> bool {
        false
    }
    fn needs_c_twins(&self) -> bool {
        false
    }
    fn needs_module_tree(&self) -> bool {
        false
    }
    fn extra_files(&self) -> &'static [&'static str] {
        &[]
    }
    fn needs_c_sources(&self) -> bool {
        false
    }
    fn needs_sdk_rust(&self) -> bool {
        false
    }
    fn needs_cdecl_sources(&self) -> bool {
        false
    }
}

pub struct SourceFile {
    pub path: PathBuf,
    pub text: String,
    resolved: Option<PathBuf>,
    raw: Option<Arc<[u8]>>,
    groups: Vec<&'static str>,
    directory: bool,
    inventory_order: usize,
    rust_tree: OnceLock<Result<Arc<tree_sitter::Tree>, String>>,
    large_rust_tree: Mutex<Weak<tree_sitter::Tree>>,
    syntax_cache: Option<(Arc<syntax_cache::Cache>, usize)>,
    twin_functions: OnceLock<Vec<twins::Function>>,
    scopes: u8,
    noise: OnceLock<Arc<str>>,
    lines: Vec<usize>,
    views: Mutex<HashMap<(String, Option<String>, bool), Arc<OnceLock<Arc<str>>>>>,
}

impl SourceFile {
    pub fn new(path: PathBuf, text: String) -> Self {
        let mut lines = vec![0];
        lines.extend(text.match_indices('\n').map(|(at, _)| at + 1));
        Self {
            path,
            text,
            resolved: None,
            raw: None,
            groups: Vec::new(),
            directory: false,
            inventory_order: 0,
            rust_tree: OnceLock::new(),
            large_rust_tree: Mutex::new(Weak::new()),
            syntax_cache: None,
            twin_functions: OnceLock::new(),
            scopes: u8::MAX,
            noise: OnceLock::new(),
            lines,
            views: Mutex::new(HashMap::new()),
        }
    }

    pub fn bytes(&self) -> &[u8] {
        self.raw.as_deref().unwrap_or(self.text.as_bytes())
    }
    pub fn is_directory(&self) -> bool {
        self.directory
    }
    pub fn rust_tree(&self) -> Result<Arc<tree_sitter::Tree>, String> {
        if let Some((cache, file)) = &self.syntax_cache {
            let mut weak = self.large_rust_tree.lock().expect("syntax cache poisoned");
            let (tree, parsed) = match weak.upgrade() {
                Some(tree) => (tree, false),
                None => {
                    let tree = Arc::new(syntax::parse(&self.text)?);
                    *weak = Arc::downgrade(&tree);
                    (tree, true)
                }
            };
            cache.retain(*file, &tree, parsed, self.text.len() <= 256 * 1024);
            return Ok(tree);
        }
        // Retaining every bindgen AST costs many times the source bytes. Large
        // parses are shared while consumers hold them, then released; ordinary
        // source parses stay cached for the whole run.
        if self.text.len() <= 256 * 1024 {
            return self
                .rust_tree
                .get_or_init(|| syntax::parse(&self.text).map(Arc::new))
                .clone();
        }
        let mut cached = self.large_rust_tree.lock().expect("syntax cache poisoned");
        if let Some(tree) = cached.upgrade() {
            return Ok(tree);
        }
        let tree = Arc::new(syntax::parse(&self.text)?);
        *cached = Arc::downgrade(&tree);
        Ok(tree)
    }
    pub fn twin_functions(&self, root: &Path) -> &[twins::Function] {
        self.twin_functions.get_or_init(|| {
            twins::rust_functions(&twins::blank_rust(&self.text), &root.join(&self.path))
        })
    }

    /// Some legacy scanners resolve symlinked file names for diagnostics while
    /// others intentionally retain lexical paths. Capture both in the snapshot.
    pub fn resolved_path(&self) -> &Path {
        self.resolved.as_deref().unwrap_or(&self.path)
    }

    /// Legacy rules disagree about which lexemes to mask and about whether
    /// offsets count UTF-8 bytes or Unicode characters. Cache each exact view;
    /// equivalent users share its allocation and never tokenize it again.
    pub fn masked(&self, regex: &Regex, cutoff: Option<&str>, characters: bool) -> Arc<str> {
        // Absent cutoff markers leave the entire input intact. On ASCII input,
        // character and byte masking also produce exactly the same view. Reuse
        // one allocation for these equivalent legacy scanner configurations.
        let (raw, cutoff) = cutoff
            .and_then(|marker| {
                self.text
                    .split_once(marker)
                    .map(|(raw, _)| (raw, Some(marker)))
            })
            .unwrap_or((&self.text, None));
        let characters = characters && !raw.is_ascii();
        let key = (
            regex.as_str().to_owned(),
            cutoff.map(str::to_owned),
            characters,
        );
        let slot = {
            let mut views = self.views.lock().unwrap();
            views
                .entry(key)
                .or_insert_with(|| Arc::new(OnceLock::new()))
                .clone()
        };
        slot.get_or_init(|| {
            regex
                .replace_all(raw, |caps: &regex::Captures<'_>| {
                    let text = caps.get(0).unwrap().as_str();
                    if characters {
                        text.chars()
                            .map(|c| if c == '\n' { '\n' } else { ' ' })
                            .collect::<String>()
                    } else {
                        text.bytes()
                            .map(|b| if b == b'\n' { '\n' } else { ' ' })
                            .collect::<String>()
                    }
                })
                .into_owned()
                .into()
        })
        .clone()
    }

    pub fn rust_noise(&self) -> &str {
        self.noise
            .get_or_init(|| lexical::strip_noise(&self.text).into())
    }

    pub fn line(&self, byte: usize) -> usize {
        self.lines.partition_point(|&at| at <= byte)
    }

    pub fn line_text(&self, line: usize) -> &str {
        let Some(&start) = line.checked_sub(1).and_then(|i| self.lines.get(i)) else {
            return "";
        };
        let end = self.lines.get(line).copied().unwrap_or(self.text.len());
        self.text[start..end].trim_end_matches(['\r', '\n'])
    }

    /// Zero-based line with exactly `str::lines()` ending semantics.
    pub fn source_line(&self, row: usize) -> Option<&str> {
        let start = *self.lines.get(row)?;
        if start == self.text.len() {
            return None;
        }
        let end = self.lines.get(row + 1).copied().unwrap_or(self.text.len());
        let line = &self.text[start..end];
        Some(match line.strip_suffix('\n') {
            Some(line) => line.strip_suffix('\r').unwrap_or(line),
            None => line,
        })
    }
}

pub struct Corpus {
    pub root: PathBuf,
    pub files: Vec<SourceFile>,
    syntax_cache: Arc<syntax_cache::Cache>,
    explicit_paths: Vec<PathBuf>,
    c_signature_asts: OnceLock<BTreeMap<String, Result<serde_json::Value, String>>>,
    twins: OnceLock<twins::CIndex>,
    modules: OnceLock<modules::ModuleTree>,
    version_history: OnceLock<Result<VersionHistory, String>>,
    module_inputs: OnceLock<module_inputs::ModuleModel>,
    crate_configs: OnceLock<BTreeMap<PathBuf, CrateCfg>>,
    normalized_paths: OnceLock<HashMap<PathBuf, usize>>,
    files_by_path: OnceLock<HashMap<PathBuf, usize>>,
    link_scans: Mutex<
        BTreeMap<
            (String, Vec<PathBuf>),
            Arc<(Vec<link_model::Def>, std::collections::BTreeSet<PathBuf>)>,
        >,
    >,
}

const RUST: u8 = 1;
const SDK_RUST: u8 = 2;
const MODULE_C: u8 = 4;
const CDECL: u8 = 8;
const TRACKED_C: u8 = 16;
const MANIFEST: u8 = 32;

#[derive(Clone, Copy)]
pub struct SourceNeeds {
    pub workspace_rust: bool,
    pub module_c: bool,
    pub sdk_rust: bool,
    pub cdecl_sources: bool,
    pub tracked_c: bool,
    pub cargo_manifests: bool,
}
impl Default for SourceNeeds {
    fn default() -> Self {
        Self {
            workspace_rust: true,
            module_c: false,
            sdk_rust: false,
            cdecl_sources: false,
            tracked_c: false,
            cargo_manifests: false,
        }
    }
}

impl Corpus {
    /// Retained payloads after execution; counts exclude allocator overhead.
    /// Inspect initialized caches only, without parsing or creating new views.
    pub fn cache_stats(&self) -> serde_json::Value {
        let mut text_bytes = 0;
        let mut raw_bytes = 0;
        let mut line_bytes = 0;
        let mut noise_bytes = 0;
        let mut twin_bytes = 0;
        let mut tree_count = 0;
        let mut tree_nodes = 0;
        let mut views = BTreeMap::<String, usize>::new();
        for file in &self.files {
            text_bytes += file.text.len();
            raw_bytes += file.raw.as_ref().map_or(0, |raw| raw.len());
            line_bytes += file.lines.capacity() * std::mem::size_of::<usize>();
            noise_bytes += file.noise.get().map_or(0, |text| text.len());
            if let Some(functions) = file.twin_functions.get() {
                twin_bytes += functions
                    .iter()
                    .map(|f| f.body.len() + f.params.len() + f.ret.len())
                    .sum::<usize>();
            }
            if let Some(Ok(tree)) = file.rust_tree.get() {
                tree_count += 1;
                tree_nodes += tree.root_node().descendant_count();
            }
            for ((pattern, cutoff, characters), slot) in file.views.lock().unwrap().iter() {
                if let Some(text) = slot.get() {
                    *views
                        .entry(format!(
                            "{pattern}; cutoff={cutoff:?}; characters={characters}"
                        ))
                        .or_default() += text.len();
                }
            }
        }
        let cache = self.syntax_cache.stats();
        tree_count += cache["retained_trees"].as_u64().unwrap() as usize;
        tree_nodes += cache["retained_nodes"].as_u64().unwrap() as usize;
        serde_json::json!({"source_text_bytes": text_bytes, "additional_raw_bytes": raw_bytes, "line_index_capacity_bytes": line_bytes, "noise_bytes": noise_bytes, "rust_twin_payload_bytes": twin_bytes, "retained_rust_trees": tree_count, "retained_rust_tree_nodes": tree_nodes, "masked_view_bytes": views, "c_signature_arches": self.c_signature_asts.get().map_or(0, |asts| asts.len()), "syntax_cache": cache})
    }
    pub fn new(root: PathBuf, mut files: Vec<SourceFile>) -> Self {
        let syntax_cache = Arc::new(syntax_cache::Cache::new(syntax_cache::DEFAULT_NODE_LIMIT));
        for (index, file) in files.iter_mut().enumerate() {
            file.syntax_cache = Some((syntax_cache.clone(), index));
            if let Some(Ok(tree)) = file.rust_tree.take() {
                *file
                    .large_rust_tree
                    .get_mut()
                    .expect("syntax cache poisoned") = Arc::downgrade(&tree);
                syntax_cache.retain(index, &tree, false, true);
            }
        }
        Self {
            root,
            files,
            syntax_cache,
            explicit_paths: Vec::new(),
            c_signature_asts: OnceLock::new(),
            twins: OnceLock::new(),
            modules: OnceLock::new(),
            version_history: OnceLock::new(),
            module_inputs: OnceLock::new(),
            crate_configs: OnceLock::new(),
            normalized_paths: OnceLock::new(),
            files_by_path: OnceLock::new(),
            link_scans: Mutex::new(BTreeMap::new()),
        }
    }
    /// Bound retained syntax by node count; active consumers keep their Arcs.
    pub fn with_ast_cache_nodes(self, nodes: usize) -> Self {
        self.syntax_cache.set_limit(nodes);
        self
    }
    pub fn c_signature_ast(&self, arch: &str) -> Result<&serde_json::Value, String> {
        self.c_signature_asts
            .get()
            .and_then(|all| all.get(arch))
            .ok_or_else(|| format!("C signature AST {arch} was not requested"))?
            .as_ref()
            .map_err(Clone::clone)
    }
    pub fn explicit_paths(&self) -> &[PathBuf] {
        &self.explicit_paths
    }
    pub fn with_explicit_paths(mut self, paths: Vec<PathBuf>) -> Self {
        self.explicit_paths = paths;
        self
    }
    pub fn file(&self, path: &Path) -> Option<&SourceFile> {
        let paths = self.files_by_path.get_or_init(|| {
            let mut paths = HashMap::with_capacity(self.files.len());
            for (index, file) in self.files.iter().enumerate() {
                paths.entry(file.path.clone()).or_insert(index);
            }
            paths
        });
        paths.get(path).map(|&index| &self.files[index])
    }
    pub(crate) fn normalized_file_index(&self) -> &HashMap<PathBuf, usize> {
        self.normalized_paths.get_or_init(|| {
            let mut paths = HashMap::with_capacity(self.files.len());
            for (index, file) in self.files.iter().enumerate() {
                paths.insert(crate::modules::normalize(&file.path), index);
            }
            paths
        })
    }
    pub fn crate_cfg(&self, path: &Path) -> &CrateCfg {
        static EMPTY: std::sync::LazyLock<CrateCfg> = std::sync::LazyLock::new(CrateCfg::default);
        let all = self.crate_configs.get_or_init(|| crate_cfg::configs(self));
        for parent in path.ancestors().skip(1) {
            if let Some(cfg) = all.get(parent) {
                return cfg;
            }
        }
        &EMPTY
    }
    pub fn module_inputs(&self) -> &module_inputs::ModuleModel {
        self.module_inputs
            .get_or_init(|| module_inputs::ModuleModel::from_corpus(self))
    }
    pub fn version_history(&self) -> Result<&VersionHistory, String> {
        self.version_history
            .get()
            .ok_or("version history was not requested")?
            .as_ref()
            .map_err(Clone::clone)
    }
    pub fn module_tree(&self) -> &modules::ModuleTree {
        self.modules
            .get_or_init(|| modules::ModuleTree::from_files(&self.files))
    }
    pub fn c_twins(&self) -> &twins::CIndex {
        self.twins.get_or_init(|| {
            twins::CIndex::from_files(self.files.iter().filter(|f| f.scopes & TRACKED_C != 0))
        })
    }
    pub fn sources<'a>(&'a self, name: &'a str) -> impl Iterator<Item = &'a SourceFile> + 'a {
        self.files
            .iter()
            .filter(move |file| file.groups.contains(&name) || file.scopes == u8::MAX)
    }

    /// Directory traversal order for scanners whose first definition wins.
    pub fn ordered_sources<'a>(&'a self, name: &'a str) -> Vec<&'a SourceFile> {
        let mut files: Vec<_> = self.sources(name).collect();
        files.sort_by_key(|f| f.inventory_order);
        files
    }

    pub fn load(root: &Path) -> Result<Self, String> {
        Self::load_with_c(root, false)
    }
    pub fn load_with_c(root: &Path, include_c: bool) -> Result<Self, String> {
        Self::load_paths(root, &[], include_c)
    }
    pub fn load_paths(root: &Path, paths: &[PathBuf], include_c: bool) -> Result<Self, String> {
        Self::load_scopes(
            root,
            paths,
            SourceNeeds {
                module_c: include_c,
                ..SourceNeeds::default()
            },
        )
    }
    pub fn load_scopes(
        root: &Path,
        inputs: &[PathBuf],
        needs: SourceNeeds,
    ) -> Result<Self, String> {
        Self::load_scopes_with_extra(root, inputs, needs, &[])
    }
    pub fn load_scopes_with_extra(
        root: &Path,
        inputs: &[PathBuf],
        needs: SourceNeeds,
        extras: &[PathBuf],
    ) -> Result<Self, String> {
        Self::load_scopes_with_sets(root, inputs, needs, extras, &[])
    }
    pub fn load_scopes_with_sets(
        root: &Path,
        inputs: &[PathBuf],
        needs: SourceNeeds,
        extras: &[PathBuf],
        sets: &[SourceSet],
    ) -> Result<Self, String> {
        let inventory_started = std::time::Instant::now();
        let root = root
            .canonicalize()
            .map_err(|e| format!("{}: {e}", root.display()))?;
        let mut paths = BTreeMap::<PathBuf, u8>::new();
        let mut groups = BTreeMap::<PathBuf, Vec<&'static str>>::new();
        let mut traversal = BTreeMap::<PathBuf, usize>::new();
        if inputs.is_empty() {
            collect_scopes_with_sets(
                &root,
                &root,
                needs,
                false,
                &mut paths,
                sets,
                &mut groups,
                &mut traversal,
            )
            .map_err(|e| format!("source traversal: {e}"))?;
            // Missing the default Rust corpus is an incomplete check, even if
            // another scope happens to exist and supplies files.
            if needs.workspace_rust {
                fs::read_dir(root.join("sdk/lib/rust"))
                    .map_err(|e| format!("Rust source root: {e}"))?;
            }
            if needs.sdk_rust {
                fs::read_dir(root.join("sdk")).map_err(|e| format!("SDK source root: {e}"))?;
            }
        } else {
            for input in inputs {
                let input = if input.is_absolute() {
                    input.clone()
                } else {
                    root.join(input)
                };
                let metadata =
                    fs::metadata(&input).map_err(|e| format!("{}: {e}", input.display()))?;
                if metadata.is_dir() {
                    collect_scopes_with_sets(
                        &input,
                        &input,
                        needs,
                        true,
                        &mut paths,
                        sets,
                        &mut groups,
                        &mut traversal,
                    )
                    .map_err(|e| format!("source traversal: {e}"))?;
                } else {
                    for set in sets.iter().filter(|set| !set.tracked && set.inputs) {
                        if set.accepts(input.strip_prefix(&root).unwrap_or(&input), true) {
                            groups.entry(input.clone()).or_default().push(set.name);
                        }
                    }
                    paths.insert(input, RUST | SDK_RUST | CDECL);
                }
            }
            // Global catalog inputs remain repository-scoped when fixture Rust
            // inputs are narrowed. Input-relative requests override their roots.
            let global_sets: Vec<_> = sets
                .iter()
                .copied()
                .filter(|set| !set.tracked && !set.inputs)
                .collect();
            if !global_sets.is_empty() {
                let no_defaults = SourceNeeds {
                    workspace_rust: false,
                    ..SourceNeeds::default()
                };
                collect_scopes_with_sets(
                    &root,
                    &root,
                    no_defaults,
                    false,
                    &mut paths,
                    &global_sets,
                    &mut groups,
                    &mut traversal,
                )
                .map_err(|e| format!("catalog traversal: {e}"))?;
            }
            if needs.module_c {
                // Fixture paths narrow Rust inputs, but the HRESULT oracle still
                // comes from this repository just as in the original scanner.
                let c_only = SourceNeeds {
                    module_c: true,
                    workspace_rust: false,
                    ..SourceNeeds::default()
                };
                let mut extra = BTreeMap::new();
                collect_scopes(&root, &root, c_only, false, &mut extra)
                    .map_err(|e| format!("C source traversal: {e}"))?;
                for (path, scope) in extra {
                    if scope & MODULE_C != 0 {
                        *paths.entry(path).or_default() |= MODULE_C;
                    }
                }
            }
        }
        if needs.module_c {
            *paths
                .entry(root.join("sdk/include/psdk/winerror.h"))
                .or_default() |= MODULE_C;
        }
        for path in extras {
            *paths.entry(root.join(path)).or_default() |= 0;
        }
        if needs.tracked_c || sets.iter().any(|set| set.tracked) {
            let mut patterns = std::collections::BTreeSet::new();
            if needs.tracked_c {
                patterns.extend(["*.c".to_owned(), "*.cpp".to_owned()]);
            }
            for set in sets.iter().filter(|set| set.tracked) {
                patterns.extend(set.extensions.iter().map(|ext| format!("*.{ext}")));
                patterns.extend(set.filenames.iter().map(|name| format!("*{name}")));
            }
            let output = std::process::Command::new("git")
                .arg("-C")
                .arg(&root)
                .args(["ls-files", "-z", "--"])
                .args(patterns)
                .output()
                .map_err(|e| format!("tracked C inventory: {e}"))?;
            if !output.status.success() {
                return Err(format!(
                    "tracked C inventory: {}",
                    String::from_utf8_lossy(&output.stderr)
                ));
            }
            #[cfg(unix)]
            use std::os::unix::ffi::OsStrExt;
            for raw in output
                .stdout
                .split(|&b| b == 0)
                .filter(|raw| !raw.is_empty())
            {
                #[cfg(unix)]
                let relative = Path::new(std::ffi::OsStr::from_bytes(raw));
                #[cfg(not(unix))]
                let relative = Path::new(
                    std::str::from_utf8(raw).map_err(|e| format!("tracked C path: {e}"))?,
                );
                let path = root.join(relative);
                match fs::metadata(&path) {
                    Ok(_) => {
                        if needs.tracked_c
                            && !relative.starts_with("sdk/lib/rust")
                            && relative
                                .extension()
                                .is_some_and(|ext| ext == "c" || ext == "cpp")
                        {
                            *paths.entry(path.clone()).or_default() |= TRACKED_C;
                        }
                        for set in sets
                            .iter()
                            .filter(|set| set.tracked && set.accepts(relative, false))
                        {
                            groups.entry(path.clone()).or_default().push(set.name);
                            paths.entry(path.clone()).or_default();
                        }
                    }
                    Err(e) if e.kind() == std::io::ErrorKind::NotFound => {} // staged/unstaged deletion, as in Python
                    Err(e) => return Err(format!("{}: {e}", path.display())),
                }
            }
        }
        let entries: Vec<_> = paths.into_iter().collect();
        if std::env::var_os("VIBEOS_LINT_TIMINGS").is_some() {
            eprintln!(
                "TIMING\tsource_inventory\t{:.3}",
                inventory_started.elapsed().as_secs_f64()
            );
        }
        let materialize_started = std::time::Instant::now();
        let files: Result<Vec<_>, String> = entries
            .par_iter()
            .map(|(path, scopes)| {
                let directory = fs::metadata(path)
                    .map_err(|e| format!("{}: {e}", path.display()))?
                    .is_dir();
                let bytes = if directory {
                    Vec::new()
                } else {
                    fs::read(path).map_err(|e| format!("{}: {e}", path.display()))?
                };
                // Path.read_text() uses universal newlines in the Python oracles.
                let text = String::from_utf8_lossy(&bytes);
                let text = if text.contains('\r') {
                    text.replace("\r\n", "\n").replace('\r', "\n")
                } else {
                    text.into_owned()
                };
                let mut file =
                    SourceFile::new(path.strip_prefix(&root).unwrap_or(path).to_owned(), text);
                let resolved = path
                    .canonicalize()
                    .map_err(|e| format!("{}: {e}", path.display()))?;
                file.resolved = Some(resolved.strip_prefix(&root).unwrap_or(&resolved).to_owned());
                file.directory = directory;
                file.inventory_order = traversal.get(path).copied().unwrap_or(usize::MAX);
                file.groups = groups.get(path).cloned().unwrap_or_default();
                if (needs.cargo_manifests && path.extension().is_some_and(|ext| ext == "rs")
                    || path.extension().is_some_and(|ext| ext == "res"))
                    && bytes != file.text.as_bytes()
                {
                    file.raw = Some(bytes.into());
                }
                file.scopes = *scopes;
                Ok(file)
            })
            .collect();
        if std::env::var_os("VIBEOS_LINT_TIMINGS").is_some() {
            eprintln!(
                "TIMING\tsource_materialize\t{:.3}",
                materialize_started.elapsed().as_secs_f64()
            );
        }
        let explicit_paths = inputs
            .iter()
            .map(|p| {
                let path = if p.is_absolute() {
                    p.clone()
                } else {
                    root.join(p)
                };
                let resolved = path
                    .canonicalize()
                    .map_err(|e| format!("{}: {e}", path.display()))?;
                Ok(resolved.strip_prefix(&root).unwrap_or(&resolved).to_owned())
            })
            .collect::<Result<Vec<_>, String>>()?;
        Ok(Self::new(root, files?).with_explicit_paths(explicit_paths))
    }
    pub fn rust_files(&self) -> impl Iterator<Item = &SourceFile> {
        self.files
            .iter()
            .filter(|f| f.scopes & RUST != 0 && f.path.extension().is_some_and(|ext| ext == "rs"))
    }
    pub fn cargo_manifests(&self) -> impl Iterator<Item = &SourceFile> {
        self.files.iter().filter(|file| {
            file.scopes & MANIFEST != 0
                && file
                    .path
                    .file_name()
                    .is_some_and(|name| name == "Cargo.toml")
        })
    }
    pub fn sdk_rust_files(&self) -> impl Iterator<Item = &SourceFile> {
        self.files.iter().filter(|f| {
            f.scopes & SDK_RUST != 0 && f.path.extension().is_some_and(|ext| ext == "rs")
        })
    }
    pub fn cdecl_files(&self) -> impl Iterator<Item = &SourceFile> {
        self.files.iter().filter(|f| f.scopes & CDECL != 0)
    }
}

const CDECL_ROOTS: &[&str] = &[
    "boot",
    "base",
    "dll",
    "drivers",
    "hal",
    "modules",
    "ntoskrnl",
    "sdk",
    "subsystems",
    "win32ss",
];
const CDECL_EXTS: &[&str] = &["c", "h", "s", "S", "asm", "inc", "rs", "cpp"];
fn directory_scopes(relative: &Path, needs: SourceNeeds, explicit: bool) -> u8 {
    if relative.components().any(|p| p.as_os_str() == "target") {
        return 0;
    }
    let rust = Path::new("sdk/lib/rust");
    let mut scopes = 0;
    if (needs.workspace_rust || needs.cargo_manifests)
        && (explicit || relative.starts_with(rust) || rust.starts_with(relative))
    {
        scopes |= RUST;
    }
    if needs.sdk_rust
        && (explicit || relative.starts_with("sdk") || Path::new("sdk").starts_with(relative))
    {
        scopes |= SDK_RUST;
    }
    let rel = relative.to_string_lossy();
    if needs.module_c
        && !["build-", "modules/rostests", "sdk/lib/rust", ".git"]
            .iter()
            .any(|p| rel.starts_with(p))
    {
        scopes |= MODULE_C;
    }
    let cdecl_root = relative
        .components()
        .next()
        .map(|p| p.as_os_str().to_string_lossy());
    let cdecl_skipped = relative.components().any(|p| {
        let name = p.as_os_str().to_string_lossy();
        name.starts_with('.') || name.starts_with("build-") || name == "node_modules"
    });
    if needs.cdecl_sources
        && !cdecl_skipped
        && (explicit || cdecl_root.is_none_or(|p| CDECL_ROOTS.contains(&p.as_ref())))
    {
        scopes |= CDECL;
    }
    scopes
}
fn collect_scopes(
    base: &Path,
    dir: &Path,
    needs: SourceNeeds,
    explicit: bool,
    paths: &mut BTreeMap<PathBuf, u8>,
) -> std::io::Result<()> {
    collect_scopes_with_sets(
        base,
        dir,
        needs,
        explicit,
        paths,
        &[],
        &mut BTreeMap::new(),
        &mut BTreeMap::new(),
    )
}
fn collect_scopes_with_sets(
    base: &Path,
    dir: &Path,
    needs: SourceNeeds,
    explicit: bool,
    paths: &mut BTreeMap<PathBuf, u8>,
    sets: &[SourceSet],
    groups: &mut BTreeMap<PathBuf, Vec<&'static str>>,
    traversal: &mut BTreeMap<PathBuf, usize>,
) -> std::io::Result<()> {
    if base.components().any(|p| p.as_os_str() == "target") {
        return Ok(());
    }
    let relative = dir.strip_prefix(base).unwrap();
    let scopes = directory_scopes(relative, needs, explicit);
    // A set that cannot visit this directory cannot visit its descendants.
    // Retain each set's own pruning policy while dropping irrelevant root tests.
    let active_sets: Vec<_> = sets
        .iter()
        .copied()
        .filter(|set| set.visits(relative, explicit))
        .collect();
    if scopes == 0 && active_sets.is_empty() {
        return Ok(());
    }
    let mut directories = Vec::new();
    for entry in fs::read_dir(dir)? {
        let entry = entry?;
        let path = entry.path();
        let kind = entry.file_type()?;
        // A link to a directory is not followed, and a dangling link (a ramdisk
        // stage's link to a deleted payload under build-<arch>/) has nothing to
        // read: skip both rather than fail the whole traversal on its ENOENT.
        if kind.is_symlink() && fs::metadata(&path).map_or(true, |m| m.is_dir()) {
            continue;
        }
        let file_relative = path.strip_prefix(base).unwrap();
        let mut grouped = false;
        for set in active_sets.iter().filter(|set| {
            !set.tracked
                && (set.accepts(file_relative, explicit)
                    || kind.is_dir() && set.directories && set.visits(file_relative, explicit))
        }) {
            if !kind.is_dir()
                || set.directories
                || set
                    .filenames
                    .iter()
                    .any(|name| path.file_name().is_some_and(|n| n == *name))
            {
                groups.entry(path.clone()).or_default().push(set.name);
                grouped = true;
            }
        }
        if grouped {
            paths.entry(path.clone()).or_default();
            let next = traversal.len();
            traversal.entry(path.clone()).or_insert(next);
        }
        if kind.is_dir() {
            directories.push(path);
            continue;
        }
        let ext = path.extension().unwrap_or_default().to_string_lossy();
        let file_relative = path.strip_prefix(base).unwrap();
        let mut selected = 0;
        if needs.cargo_manifests
            && path.file_name().is_some_and(|n| n == "Cargo.toml")
            && scopes & RUST != 0
        {
            selected |= MANIFEST;
        }
        if ext == "rs" {
            if scopes & RUST != 0 && (explicit || file_relative.starts_with("sdk/lib/rust")) {
                selected |= RUST;
            }
            if scopes & SDK_RUST != 0 && (explicit || file_relative.starts_with("sdk")) {
                selected |= SDK_RUST;
            }
        }
        if ext == "c" && scopes & MODULE_C != 0 {
            selected |= MODULE_C;
        }
        if scopes & CDECL != 0
            && CDECL_EXTS.contains(&ext.as_ref())
            && (explicit
                || file_relative.components().next().is_some_and(|p| {
                    CDECL_ROOTS.contains(&p.as_os_str().to_string_lossy().as_ref())
                }))
        {
            selected |= CDECL;
        }
        if selected != 0 {
            if kind.is_symlink() && fs::metadata(&path).map_or(true, |m| m.is_dir()) {
                continue;
            }
            *paths.entry(path).or_default() |= selected;
        }
    }
    // pathlib.rglob visits all matches in a directory before its subdirectories.
    for path in directories {
        collect_scopes_with_sets(
            base,
            &path,
            needs,
            explicit,
            paths,
            &active_sets,
            groups,
            traversal,
        )?;
    }
    Ok(())
}

fn timed<T>(stage: &str, action: impl FnOnce() -> T) -> T {
    let started = std::time::Instant::now();
    let result = action();
    if std::env::var_os("VIBEOS_LINT_TIMINGS").is_some() {
        eprintln!("TIMING\t{stage}\t{:.3}", started.elapsed().as_secs_f64());
    }
    result
}

pub fn execute(
    corpus: &Corpus,
    selected: &[&dyn Rule],
) -> Result<Vec<(&'static str, Vec<String>)>, String> {
    // Build cross-file indexes before rules fan out. Initializing a OnceLock
    // from nested Rayon jobs can exhaust the pool with workers waiting on it.
    let ast_arches: std::collections::BTreeSet<_> = selected
        .iter()
        .flat_map(|r| r.needs_c_signature_asts().iter().copied())
        .collect();
    if !ast_arches.is_empty() {
        timed("c_signature_asts", || {
            corpus.c_signature_asts.get_or_init(|| {
                ast_arches
                    .iter()
                    .map(|a| (a.to_string(), ast_facts::c_signature_ast(&corpus.root, a)))
                    .collect()
            });
            for arch in ast_arches {
                corpus.c_signature_ast(arch)?;
            }
            Ok::<(), String>(())
        })?;
    }
    if selected.iter().any(|rule| rule.needs_module_inputs()) {
        timed("module_inputs", || {
            corpus.module_inputs();
        });
    }
    if selected.iter().any(|rule| rule.needs_version_history()) {
        let started = std::time::Instant::now();
        corpus
            .version_history
            .get_or_init(|| VersionHistory::capture(&corpus.root));
        corpus.version_history()?;
        if std::env::var_os("VIBEOS_LINT_TIMINGS").is_some() {
            eprintln!(
                "TIMING\tversion_history\t{:.3}",
                started.elapsed().as_secs_f64()
            );
        }
    }
    if selected.iter().any(|rule| rule.needs_c_twins()) {
        timed("c_twins", || {
            corpus.c_twins();
        });
    }
    if selected.iter().any(|rule| rule.needs_module_tree()) {
        timed("module_tree", || {
            corpus.module_tree();
        });
    }
    let execution_started = std::time::Instant::now();
    let results: Vec<_> = selected
        .par_iter()
        .map(|rule| {
            let started = std::time::Instant::now();
            let chunks = rule
                .run(corpus)
                .map_err(|e| format!("{}: {e}", rule.name()))?;
            if std::env::var_os("VIBEOS_LINT_TIMINGS").is_some() {
                eprintln!(
                    "TIMING\t{}\t{:.3}",
                    rule.name(),
                    started.elapsed().as_secs_f64()
                );
            }
            // Original stdout may include multiline types or report sections.
            // Match subprocess text-mode universal newlines, then tag each
            // physical output line independently. Blank lines carry no finding.
            let mut lines: Vec<String> = chunks
                .into_iter()
                .flat_map(|chunk| {
                    chunk
                        .split(|c| {
                            matches!(
                                c,
                                '\n' | '\r'
                                    | '\u{000b}'
                                    | '\u{000c}'
                                    | '\u{001c}'
                                    | '\u{001d}'
                                    | '\u{001e}'
                                    | '\u{0085}'
                                    | '\u{2028}'
                                    | '\u{2029}'
                            )
                        })
                        .filter(|line| !line.is_empty())
                        .map(str::to_owned)
                        .collect::<Vec<_>>()
                })
                .collect();
            lines.sort();
            lines.dedup();
            Ok((rule.name(), lines))
        })
        .collect();
    if std::env::var_os("VIBEOS_LINT_TIMINGS").is_some() {
        eprintln!(
            "TIMING\tsource_rule_execution\t{:.3}",
            execution_started.elapsed().as_secs_f64()
        );
    }
    results.into_iter().collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn equivalent_ascii_mask_views_share_the_same_allocation() {
        let file = SourceFile::new("a.rs".into(), "/* comment */ fn item() {}".into());
        let regex = Regex::new(r"/\*.*?\*/").unwrap();
        let bytes = file.masked(&regex, None, false);
        let characters = file.masked(&regex, Some("ABSENT"), true);
        let other_marker = file.masked(&regex, Some("ALSO_ABSENT"), false);
        assert!(Arc::ptr_eq(&bytes, &characters));
        assert!(Arc::ptr_eq(&bytes, &other_marker));
        assert_eq!(bytes.as_ref(), "              fn item() {}");
        // A real cutoff and non-ASCII masked text must keep their semantics.
        let unicode = SourceFile::new("u.rs".into(), "/*é*/ END tail".into());
        let bytes = unicode.masked(&regex, None, false);
        let characters = unicode.masked(&regex, None, true);
        assert_ne!(bytes.as_ref(), characters.as_ref());
        assert_eq!(unicode.masked(&regex, Some("END"), true).as_ref(), "      ");
    }

    #[test]
    fn indexed_source_lines_match_str_lines_including_unterminated_cr() {
        for text in ["", "a", "a\n", "\n", "\r", "a\r\nb\r\r\nc\r", "é\n猫"] {
            let file = SourceFile::new("fixture.rs".into(), text.into());
            for row in 0..text.lines().count() + 2 {
                assert_eq!(
                    file.source_line(row),
                    text.lines().nth(row),
                    "{text:?}, row {row}"
                );
            }
        }
    }

    #[test]
    fn concurrent_consumers_share_one_parse_even_without_retention() {
        let corpus = Corpus::new(
            "/fixture".into(),
            vec![SourceFile::new("a.rs".into(), "fn item() {}".into())],
        )
        .with_ast_cache_nodes(0);
        let barrier = std::sync::Barrier::new(8);
        std::thread::scope(|scope| {
            let file = &corpus.files[0];
            let threads: Vec<_> = (0..8)
                .map(|_| {
                    let barrier = &barrier;
                    scope.spawn(move || {
                        barrier.wait();
                        file.rust_tree().unwrap()
                    })
                })
                .collect();
            let trees: Vec<_> = threads
                .into_iter()
                .map(|thread| thread.join().unwrap())
                .collect();
            assert!(trees.iter().all(|tree| Arc::ptr_eq(tree, &trees[0])));
            assert_eq!(corpus.cache_stats()["syntax_cache"]["successful_parses"], 1);
            assert_eq!(corpus.cache_stats()["retained_rust_trees"], 0);
        });
    }
    #[test]
    fn cache_statistics_observe_only_initialized_payloads_and_eviction() {
        let corpus = Corpus::new(
            "/fixture".into(),
            vec![SourceFile::new(
                "a.rs".into(),
                "/* hi */ fn item() {}\n".into(),
            )],
        );
        let initial = corpus.cache_stats();
        assert_eq!(initial["retained_rust_trees"], 0);
        assert_eq!(initial["noise_bytes"], 0);
        assert_eq!(initial["masked_view_bytes"], serde_json::json!({}));
        let file = &corpus.files[0];
        let tree = file.rust_tree().unwrap();
        let nodes = tree.root_node().descendant_count();
        assert!(Arc::ptr_eq(&tree, &file.rust_tree().unwrap()));
        let masked = file.masked(&Regex::new(r"/\*.*?\*/").unwrap(), None, false);
        assert_eq!(masked.len(), file.text.len());
        let observed = corpus.cache_stats();
        assert_eq!(observed["retained_rust_tree_nodes"], nodes);
        assert_eq!(observed["syntax_cache"]["successful_parses"], 1);
        assert_eq!(observed["noise_bytes"], 0);
        let corpus = corpus.with_ast_cache_nodes(0);
        assert_eq!(corpus.cache_stats()["retained_rust_trees"], 0);
        assert_eq!(tree.root_node().descendant_count(), nodes);
    }
    #[test]
    fn multiline_reports_are_tagged_as_individual_output_lines() {
        struct Report;
        impl Rule for Report {
            fn name(&self) -> &'static str {
                "report"
            }
            fn run(&self, _: &Corpus) -> Result<Vec<String>, String> {
                Ok(vec![
                    "first\r\nsecond\rthird\n\n".into(),
                    String::new(),
                    "fourth\tvalue".into(),
                    "fifth\u{2028}sixth\u{0085}".into(),
                ])
            }
        }
        let c = Corpus::new(".".into(), vec![]);
        let got = execute(&c, &[&Report]).unwrap();
        assert_eq!(
            got[0].1,
            vec![
                "fifth",
                "first",
                "fourth\tvalue",
                "second",
                "sixth",
                "third"
            ]
        );
    }
    #[test]
    fn declared_inputs_keep_binary_bytes_tracking_boundaries_and_fixture_scopes() {
        let temp =
            std::env::temp_dir().join(format!("vibeos-lint-source-sets-{}", std::process::id()));
        struct Cleanup(PathBuf);
        impl Drop for Cleanup {
            fn drop(&mut self) {
                let _ = fs::remove_dir_all(&self.0);
            }
        }
        let _cleanup = Cleanup(temp.clone());
        fs::create_dir_all(temp.join("sdk/lib/rust/demo/src")).unwrap();
        fs::create_dir_all(temp.join("sdk/include")).unwrap();
        fs::create_dir_all(temp.join("sdk/lib/rust/clone/.git")).unwrap();
        fs::create_dir_all(temp.join("assets")).unwrap();
        fs::write(
            temp.join("sdk/lib/rust/demo/src/lib.rs"),
            "fn parsed() { /* nested /* comment */ */ }\n",
        )
        .unwrap();
        fs::write(temp.join("sdk/include/catalog.h"), "#define FLAG 1\n").unwrap();
        fs::write(
            temp.join("sdk/lib/rust/demo/Cargo.toml"),
            "[package]\nname=\"demo\"\n",
        )
        .unwrap();
        let raw = b"\xff\r\n\0\rresource";
        fs::write(temp.join("assets/tracked.res"), raw).unwrap();
        fs::write(temp.join("assets/untracked.res"), raw).unwrap();
        assert!(std::process::Command::new("git")
            .args(["init", "-q"])
            .arg(&temp)
            .status()
            .unwrap()
            .success());
        assert!(std::process::Command::new("git")
            .arg("-C")
            .arg(&temp)
            .args(["add", "assets/tracked.res", "sdk/lib/rust/demo/Cargo.toml"])
            .status()
            .unwrap()
            .success());
        const SETS: &[SourceSet] = &[
            SourceSet::tree("catalog", &["sdk/include"], &["h"], &[]),
            SourceSet::tracked("resources", &["res"], &[]),
            SourceSet::tracked("tracked-manifests", &[], &["Cargo.toml"]),
            SourceSet::tree("markers", &[""], &[], &[".git"]).with_pruning(&[".git", "target"]),
            SourceSet::tree("fixture-c", &["sdk/include"], &["h"], &[]).with_inputs(),
        ];
        let corpus =
            Corpus::load_scopes_with_sets(&temp, &[], SourceNeeds::default(), &[], SETS).unwrap();
        assert_eq!(corpus.sources("resources").count(), 1);
        assert_eq!(corpus.sources("tracked-manifests").count(), 1);
        assert_eq!(corpus.sources("resources").next().unwrap().bytes(), raw);
        assert_eq!(corpus.sources("catalog").count(), 1);
        assert_eq!(
            corpus
                .sources("markers")
                .filter(|file| file.is_directory())
                .count(),
            2
        );
        let file = corpus.rust_files().next().unwrap();
        assert!(Arc::ptr_eq(
            &file.rust_tree().unwrap(),
            &file.rust_tree().unwrap()
        ));
        fs::create_dir_all(temp.join("fixture")).unwrap();
        fs::write(temp.join("fixture/local.h"), "#define LOCAL 2").unwrap();
        let narrowed = Corpus::load_scopes_with_sets(
            &temp,
            &[temp.join("fixture")],
            SourceNeeds::default(),
            &[],
            SETS,
        )
        .unwrap();
        assert_eq!(
            narrowed.sources("catalog").next().unwrap().path,
            Path::new("sdk/include/catalog.h")
        );
        assert_eq!(
            narrowed.sources("fixture-c").next().unwrap().path,
            Path::new("fixture/local.h")
        );
    }

    #[test]
    fn large_syntax_is_shared_only_while_consumers_hold_it() {
        let corpus = Corpus::new(
            "/fixture".into(),
            vec![SourceFile::new(
                "large.rs".into(),
                format!("{}fn live() {{}}", " ".repeat(256 * 1024)),
            )],
        );
        let file = &corpus.files[0];
        let first = file.rust_tree().unwrap();
        let second = file.rust_tree().unwrap();
        assert!(Arc::ptr_eq(&first, &second));
        let weak = Arc::downgrade(&first);
        drop(first);
        drop(second);
        assert!(weak.upgrade().is_none());
        assert_eq!(file.rust_tree().unwrap().root_node().kind(), "source_file");
    }

    #[test]
    fn byte_offsets_and_unicode() {
        let file = SourceFile::new("x.rs".into(), "é\r\nnext\n".into());
        assert_eq!(file.line(4), 2);
        assert_eq!(file.line_text(1), "é");
        assert_eq!(file.line_text(2), "next");
        assert_eq!(file.line_text(0), "");
    }

    #[test]
    fn lexical_views_share_work_without_changing_legacy_offsets() {
        let file = SourceFile::new("x.rs".into(), "/* é */\nfn f() {} MARKER ignored".into());
        let regex = Regex::new(r"(?s)/\*.*?\*/").unwrap();
        let bytes = file.masked(&regex, Some("MARKER"), false);
        assert_eq!(bytes.as_ref(), "        \nfn f() {} ");
        assert!(Arc::ptr_eq(
            &bytes,
            &file.masked(&regex, Some("MARKER"), false)
        ));
        assert_eq!(
            file.masked(&regex, Some("MARKER"), true).as_ref(),
            "       \nfn f() {} "
        );
    }

    #[test]
    fn source_walk_prunes_targets_and_does_not_follow_directory_links() {
        let temp = std::env::temp_dir().join(format!(
            "vibeos-lint-corpus-{}-{}",
            std::process::id(),
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_nanos()
        ));
        struct Cleanup(PathBuf);
        impl Drop for Cleanup {
            fn drop(&mut self) {
                let _ = fs::remove_dir_all(&self.0);
            }
        }
        let _cleanup = Cleanup(temp.clone());
        let rust = temp.join("sdk/lib/rust");
        fs::create_dir_all(rust.join("crate/src")).unwrap();
        fs::create_dir_all(rust.join("target")).unwrap();
        fs::write(rust.join("crate/src/lib.rs"), "fn f() {}\n").unwrap();
        fs::write(rust.join("target/ignored.rs"), "ignored").unwrap();
        fs::write(rust.join("other.txt"), "ignored").unwrap();
        #[cfg(unix)]
        {
            std::os::unix::fs::symlink(&rust, rust.join("cycle")).unwrap();
            std::os::unix::fs::symlink(rust.join("crate/src/lib.rs"), rust.join("alias.rs"))
                .unwrap();
            // Dangling links, in scope and under a build tree, are skipped.
            std::os::unix::fs::symlink(rust.join("deleted.rs"), rust.join("dangling.rs")).unwrap();
            fs::create_dir_all(temp.join("build-arm64/boot/ramdisk")).unwrap();
            std::os::unix::fs::symlink(
                temp.join("gone/payload.exe"),
                temp.join("build-arm64/boot/ramdisk/payload.exe"),
            )
            .unwrap();
        }
        let corpus = Corpus::load(&temp).unwrap();
        assert!(corpus
            .files
            .iter()
            .all(|f| !f.path.components().any(|p| p.as_os_str() == "target")));
        assert!(corpus
            .files
            .iter()
            .any(|f| f.path == Path::new("sdk/lib/rust/crate/src/lib.rs")));
        #[cfg(unix)]
        assert_eq!(corpus.files.len(), 2);
        #[cfg(unix)]
        assert_eq!(
            corpus
                .files
                .iter()
                .find(|f| f.path.ends_with("alias.rs"))
                .unwrap()
                .resolved_path(),
            Path::new("sdk/lib/rust/crate/src/lib.rs")
        );
        assert!(Corpus::load_paths(&temp, &[temp.join("absent")], false).is_err());
        assert!(Corpus::load_paths(&temp, &[rust.join("target")], false)
            .unwrap()
            .files
            .is_empty());
        #[cfg(unix)]
        {
            let nested = temp.join("nested");
            fs::create_dir_all(nested.join("sdk/lib/rust")).unwrap();
            let target = rust.join("crate/src/lib.rs");
            std::os::unix::fs::symlink(&target, nested.join("sdk/lib/rust/external.rs")).unwrap();
            let outside = Corpus::load(&nested).unwrap();
            assert_eq!(outside.files[0].resolved_path(), target);
        }
    }

    #[test]
    fn union_scopes_share_files_without_widening_rule_inputs() {
        let temp = std::env::temp_dir().join(format!("vibeos-lint-scopes-{}", std::process::id()));
        fs::create_dir_all(&temp).unwrap();
        struct Cleanup(PathBuf);
        impl Drop for Cleanup {
            fn drop(&mut self) {
                let _ = fs::remove_dir_all(&self.0);
            }
        }
        let _cleanup = Cleanup(temp.clone());
        for name in [
            "sdk/lib/rust/a/src/lib.rs",
            "sdk/extra.rs",
            "dll/alias.c",
            "dll/target/ignored.rs",
            "dll/.hidden/ignored.asm",
            "dll/build-test/ignored.c",
            "tools/ignored.c",
            "root.asm",
        ] {
            let path = temp.join(name);
            fs::create_dir_all(path.parent().unwrap()).unwrap();
            fs::write(path, "source").unwrap();
        }
        let corpus = Corpus::load_scopes(
            &temp,
            &[],
            SourceNeeds {
                sdk_rust: true,
                cdecl_sources: true,
                ..SourceNeeds::default()
            },
        )
        .unwrap();
        let names = |files: Vec<&SourceFile>| {
            files
                .into_iter()
                .map(|f| f.path.to_string_lossy().into_owned())
                .collect::<Vec<_>>()
        };
        assert_eq!(
            names(corpus.rust_files().collect()),
            ["sdk/lib/rust/a/src/lib.rs"]
        );
        assert_eq!(
            names(corpus.sdk_rust_files().collect()),
            ["sdk/extra.rs", "sdk/lib/rust/a/src/lib.rs"]
        );
        assert_eq!(
            names(corpus.cdecl_files().collect()),
            ["dll/alias.c", "sdk/extra.rs", "sdk/lib/rust/a/src/lib.rs"]
        );
        assert_eq!(corpus.files.len(), 3);
        let explicit = Corpus::load_scopes(
            &temp,
            &[PathBuf::from("root.asm")],
            SourceNeeds {
                workspace_rust: false,
                cdecl_sources: true,
                ..SourceNeeds::default()
            },
        )
        .unwrap();
        assert_eq!(names(explicit.cdecl_files().collect()), ["root.asm"]);
        fs::write(temp.join("root.asm"), "first\rsecond\r\nthird\n").unwrap();
        let newlines = Corpus::load_scopes(
            &temp,
            &[PathBuf::from("root.asm")],
            SourceNeeds {
                workspace_rust: false,
                cdecl_sources: true,
                ..SourceNeeds::default()
            },
        )
        .unwrap();
        assert_eq!(newlines.files[0].text, "first\nsecond\nthird\n");
    }

    #[test]
    fn tracked_c_oracle_excludes_untracked_sources_and_keeps_raw_rust_bytes() {
        let temp = std::env::temp_dir().join(format!("vibeos-lint-tracked-{}", std::process::id()));
        struct Cleanup(PathBuf);
        impl Drop for Cleanup {
            fn drop(&mut self) {
                let _ = fs::remove_dir_all(&self.0);
            }
        }
        let _cleanup = Cleanup(temp.clone());
        for (name, text) in [
            ("sdk/lib/rust/x/Cargo.toml", "[package]\nname=\"x\"\n"),
            ("sdk/lib/rust/x/src/lib.rs", "pub fn Caller() {}\r\n"),
            ("source/oracle.c", "int Oracle(void) { return 1; }"),
            ("source/cpp.cpp", "int Cpp(void) { return 2; }"),
            ("source/untracked.c", "int Wrong(void) { return 0; }"),
            ("sdk/lib/rust/x/cohab.c", "int Hidden(void) { return 0; }"),
            (
                "ntoskrnl/include/internal/mm.h",
                "#define IS_DIRTY_SSE(E) ((E)&2)",
            ),
        ] {
            let path = temp.join(name);
            fs::create_dir_all(path.parent().unwrap()).unwrap();
            fs::write(path, text).unwrap();
        }
        assert!(std::process::Command::new("git")
            .args(["init", "--quiet"])
            .arg(&temp)
            .status()
            .unwrap()
            .success());
        assert!(std::process::Command::new("git")
            .arg("-C")
            .arg(&temp)
            .args([
                "add",
                "source/oracle.c",
                "source/cpp.cpp",
                "sdk/lib/rust/x/cohab.c"
            ])
            .status()
            .unwrap()
            .success());
        let needs = SourceNeeds {
            tracked_c: true,
            cargo_manifests: true,
            ..SourceNeeds::default()
        };
        let corpus = Corpus::load_scopes_with_extra(
            &temp,
            &[],
            needs,
            &[PathBuf::from("ntoskrnl/include/internal/mm.h")],
        )
        .unwrap();
        assert_eq!(corpus.files.len(), 5);
        assert_eq!(corpus.rust_files().count(), 1);
        let rust = corpus.rust_files().next().unwrap();
        assert_eq!(rust.text, "pub fn Caller() {}\n");
        assert_eq!(rust.bytes(), b"pub fn Caller() {}\r\n");
        assert!(corpus.c_twins().get("Wrong").is_empty());
        assert!(corpus.c_twins().get("Hidden").is_empty());
        assert_eq!(
            corpus.c_twins().get("Oracle")[0].path,
            Path::new("source/oracle.c")
        );
        assert_eq!(corpus.c_twins().get("Cpp").len(), 1);
        let rust_path = Path::new("sdk/lib/rust/x/src/lib.rs");
        fs::write(temp.join(rust_path), "pub fn Caller() {}\n").unwrap();
        let plain = Corpus::load_scopes(&temp, &[], needs).unwrap();
        let file = plain.file(rust_path).unwrap();
        assert_eq!(file.bytes(), b"pub fn Caller() {}\n");
        assert!(file.raw.is_none());
        fs::write(temp.join(rust_path), b"// \xff\r\n").unwrap();
        let invalid = Corpus::load_scopes(&temp, &[], needs).unwrap();
        let file = invalid.file(rust_path).unwrap();
        assert_eq!(file.text, "// \u{fffd}\n");
        assert_eq!(file.bytes(), b"// \xff\r\n");
        fs::remove_file(temp.join("source/cpp.cpp")).unwrap();
        assert!(Corpus::load_scopes(&temp, &[], needs)
            .unwrap()
            .c_twins()
            .get("Cpp")
            .is_empty());
    }
}
