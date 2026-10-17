//! Declarative extra inputs, collected by the corpus rather than individual rules.
use std::path::Path;

#[derive(Clone, Copy, Debug)]
pub struct SourceSet {
    pub name: &'static str,
    pub roots: &'static [&'static str],
    pub extensions: &'static [&'static str],
    pub filenames: &'static [&'static str],
    pub tracked: bool,
    pub pruning: &'static [&'static str],
    pub inputs: bool,
    pub directories: bool,
}

impl SourceSet {
    pub const fn tree(
        name: &'static str,
        roots: &'static [&'static str],
        extensions: &'static [&'static str],
        filenames: &'static [&'static str],
    ) -> Self {
        Self {
            name,
            roots,
            extensions,
            filenames,
            tracked: false,
            pruning: &["target"],
            inputs: false,
            directories: false,
        }
    }
    pub const fn tracked(
        name: &'static str,
        extensions: &'static [&'static str],
        filenames: &'static [&'static str],
    ) -> Self {
        Self {
            name,
            roots: &[""],
            extensions,
            filenames,
            tracked: true,
            pruning: &[],
            inputs: false,
            directories: false,
        }
    }
    pub const fn with_pruning(mut self, pruning: &'static [&'static str]) -> Self {
        self.pruning = pruning;
        self
    }
    pub const fn with_directories(mut self) -> Self {
        self.directories = true;
        self
    }
    pub const fn with_inputs(mut self) -> Self {
        self.inputs = true;
        self
    }
    fn pruned(&self, path: &Path) -> bool {
        path.components().any(|part| {
            let name = part.as_os_str().to_string_lossy();
            self.pruning.iter().any(|rule| {
                rule.strip_suffix('*')
                    .map_or(name == *rule, |prefix| name.starts_with(prefix))
            })
        })
    }
    pub(crate) fn visits(&self, directory: &Path, explicit: bool) -> bool {
        !self.tracked
            && !self.pruned(directory)
            && (explicit && self.inputs
                || self.roots.iter().any(|root| {
                    let root = Path::new(if *root == "." { "" } else { root });
                    root.starts_with(directory) || directory.starts_with(root)
                }))
    }
    pub(crate) fn accepts(&self, path: &Path, explicit: bool) -> bool {
        if explicit && !self.inputs {
            return false;
        }
        let matches_name = path
            .extension()
            .is_some_and(|ext| self.extensions.iter().any(|want| ext == *want))
            || path.file_name().is_some_and(|name| {
                self.filenames.iter().any(|want| {
                    want.split_once('*')
                        .map_or(name == *want, |(prefix, suffix)| {
                            let name = name.to_string_lossy();
                            name.len() >= prefix.len() + suffix.len()
                                && name.starts_with(prefix)
                                && name.ends_with(suffix)
                        })
                })
            });
        if !matches_name {
            return false;
        }
        let parent = path.parent().unwrap_or(Path::new(""));
        if !self.tracked {
            // `visits(parent)` and the path-root check are both required: the
            // former keeps ancestor directories alive when a root is nested
            // below them, while the latter rejects siblings of that root.
            // Evaluate both predicates in one roots pass instead of walking
            // the static root list twice for every candidate file.
            if self.pruned(parent) {
                return false;
            }
            if !(explicit && self.inputs) {
                let mut parent_in_scope = false;
                let mut path_in_scope = false;
                for root in self.roots {
                    let root = Path::new(if *root == "." { "" } else { root });
                    parent_in_scope |= root.starts_with(parent) || parent.starts_with(root);
                    path_in_scope |= path.starts_with(root);
                    if parent_in_scope && path_in_scope {
                        break;
                    }
                }
                if !parent_in_scope || !path_in_scope {
                    return false;
                }
            }
        }
        true
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn scope_roots_pruning_and_explicit_overrides_are_separate() {
        let headers = SourceSet::tree("h", &["sdk/include"], &["h"], &[]);
        assert!(headers.visits(Path::new("sdk"), false));
        assert!(headers.accepts(Path::new("sdk/include/a.h"), false));
        assert!(!headers.accepts(Path::new("sdk/elsewhere/a.h"), false));
        assert!(!headers.accepts(Path::new("sdk/include/target/a.h"), false));
        assert!(!headers.accepts(Path::new("a.h"), true));
        assert!(headers.with_inputs().accepts(Path::new("a.h"), true));
        assert!(!headers
            .with_pruning(&["build-*"])
            .accepts(Path::new("sdk/include/build-test/a.h"), false));
        assert!(
            SourceSet::tracked("r", &["res"], &[]).accepts(Path::new("sdk/lib/rust/a.res"), false)
        );
        assert!(SourceSet::tree("stub", &["."], &[], &["rust_cohab*.c"])
            .accepts(Path::new("drivers/a/rust_cohab_stub.c"), false));
        assert!(!SourceSet::tree("stub", &["."], &[], &["rust_cohab*.c"])
            .accepts(Path::new("drivers/a/rust_cohab_stub.h"), false));
    }

    #[test]
    fn accepts_keeps_the_independent_parent_and_file_root_checks() {
        let split_roots = SourceSet::tree(
            "split",
            &["sdk/include/foo", "sdk/include/bar"],
            &["h"],
            &[],
        );
        // The parent is retained because a configured root lies below it;
        // the file itself belongs to the other configured root.
        assert!(split_roots.accepts(Path::new("sdk/include/bar/a.h"), false));
        assert!(!split_roots.accepts(Path::new("sdk/include/baz/a.h"), false));
    }
}
