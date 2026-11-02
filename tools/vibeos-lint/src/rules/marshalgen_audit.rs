//! Adapter over the shared authoritative WoW64 model and audit.
use crate::{Corpus, Rule};
pub use vibeos_codegen::marshal::*;
pub struct MarshalgenAudit;
#[cfg(test)]
use std::{collections::HashMap, fs, path::Path};
#[cfg(test)]
use vibeos_codegen::wow64::{compute_thunk_table, parse_sysfuncs};
impl Rule for MarshalgenAudit {
    fn needs_workspace_rust(&self) -> bool {
        false
    }
    fn name(&self) -> &'static str {
        "marshalgen_audit"
    }
    fn run(&self, c: &Corpus) -> Result<Vec<String>, String> {
        audit(&c.root, None, None, "both")
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn audit_verdict_matches_python_contract() {
        assert_eq!(audit_verdict("WIDENING", "-> UNICODE_STRING"), "DIVERGENT");
        assert_eq!(
            audit_verdict("SCALAR_WIDEN", "-> HWND: 4->8 scalar pointee"),
            "DIVERGENT"
        );
        assert_eq!(
            audit_verdict("SEMANTIC", "union/bitfield with divergent layout"),
            "DIVERGENT"
        );
        assert_eq!(
            audit_verdict("SEMANTIC", "-> FOO: layout not in dump (add header to TU?)"),
            "UNKNOWN"
        );
        assert_eq!(audit_verdict("SEMANTIC", "unknown type 'BAR'"), "UNKNOWN");
        assert_eq!(audit_verdict("FLAT", "scalar"), "OK");
        assert_eq!(audit_verdict("IDENTICAL", ""), "OK")
    }
    #[test]
    fn source_table_parser_tracks_win64_guest_conditionals() {
        let s="SVC_(A, 1)\n#ifdef _WIN64\nSVC_(Only64, 1)\n#else\nSVC_(B, 2)\n#endif\n#if defined(_WIN64)\nSVC_(Only64b, 3)\n#endif\n#if !defined(_WIN64)\nSVC_(C, 4)\n#endif\n";
        assert_eq!(
            parse_sysfuncs(s),
            vec![("NtA".into(), 1), ("NtB".into(), 2), ("NtC".into(), 4)]
        )
    }
    #[test]
    fn checked_in_win32k_table_is_source_derived_with_manual_entries_disabled() {
        let root = Path::new(env!("CARGO_MANIFEST_DIR"))
            .parent()
            .and_then(Path::parent)
            .expect("crate lives in tools/vibeos-lint");
        let table = compute_thunk_table(root, "win32k", "amd64").unwrap();
        assert!(table.len() > 500);
        assert_eq!(table.get("NtGdiCreateDIBSection"), Some(&false));
        assert_eq!(table.get("NtUserGetAtomName"), Some(&false));
        assert_eq!(table.get("NtGdiAbortDoc"), Some(&true));
    }
    #[test]
    fn stale_built_table_hard_fails_with_actionable_diff() {
        let c = HashMap::from([("NtFoo".to_owned(), false)]);
        let table = "{ \"NtFoo\", wow64_thunk_NtFoo, 1 },\n";
        let err = crosscheck(&c, table, Path::new("build/x.gen.c"), "ntdll").unwrap_err();
        assert!(err.contains("STALE") && err.contains("NtFoo"))
    }
    #[test]
    fn layout_parser_and_widening_classification() {
        let a=parse_layout_dump("*** Dumping AST Record Layout\n         0 | struct _X\n         0 |   ULONG A\n         4 |   PVOID P\n           | [sizeof=8, align=4]\n");
        let b=parse_layout_dump("*** Dumping AST Record Layout\n         0 | struct _X\n         0 |   ULONG A\n         8 |   PVOID P\n           | [sizeof=16, align=8]\n");
        assert_eq!(diff_struct("X", &a, &b).cls, "WIDENING")
    }
    #[test]
    fn prototype_extraction_strips_comments_and_sal() {
        let p=extract_prototypes("/* NtFake(VOID); */\nNTSTATUS NTAPI NtReal(_In_ HANDLE Handle, _Out_writes_(Count) PULONG Buffer, ULONG Count);\n",r"Nt\w+").unwrap();
        assert!(!p.contains_key("NtFake"));
        assert_eq!(p["NtReal"].params.len(), 3);
        assert_eq!(p["NtReal"].params[0].name, "Handle");
        assert_eq!(p["NtReal"].params[1].sal, "out")
    }
    #[test]
    fn no_build_means_empty_stdout() {
        let root = std::env::temp_dir().join(format!("marshalgen-no-build-{}", std::process::id()));
        let _ = fs::create_dir_all(&root);
        assert!(audit(&root, None, None, "both").unwrap().is_empty());
        let _ = fs::remove_dir_all(root);
    }
}
