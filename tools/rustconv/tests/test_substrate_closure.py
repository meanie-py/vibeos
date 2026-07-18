from pathlib import Path

from rustconv.substrate_closure import analyze_report, diagnostic_root


def test_diagnostic_root_extracts_language_neutral_unresolved_names():
    assert diagnostic_root({"message": "cannot find type `POINTL` in this scope"}) == (
        "type",
        "POINTL",
    )
    assert diagnostic_root({"message": "cannot find macro `countof` in this scope"}) == (
        "macro",
        "countof",
    )
    assert diagnostic_root({"message": "mismatched types"}) is None


def test_substrate_report_clusters_arches_and_routes_authorities(tmp_path: Path):
    repo = tmp_path / "repo"
    shell = repo / "sdk/lib/rust/shell32-rust/src"
    project = repo / "sdk/lib/rust/other/src"
    include = repo / "sdk/include/psdk"
    shell.mkdir(parents=True)
    project.mkdir(parents=True)
    include.mkdir(parents=True)
    (shell / "types.rs").write_text("pub type HICON = *mut core::ffi::c_void;\n")
    (project / "bindings.rs").write_text("pub struct POINTL { pub x: i32, pub y: i32 }\n")
    (include / "winbase.h").write_text("typedef struct _SYSTEMTIME SYSTEMTIME;\n")
    source = tmp_path / "worker.c"
    source.write_text("static int InternalHelper(void) { return 1; }\n")
    candidate = tmp_path / "candidate.rs"
    candidate.write_text("pub unsafe fn worker() {}\n")

    messages = [
        ("type", "HICON"),
        ("type", "POINTL"),
        ("type", "SYSTEMTIME"),
        ("function", "InternalHelper"),
        ("value", "MISSING"),
    ]
    arches = {}
    for arch in ("i386", "amd64", "arm64"):
        diagnostics = [
            {
                "category": "unresolved_symbol",
                "message": f"cannot find {kind} `{symbol}` in this scope",
            }
            for kind, symbol in messages
        ]
        arches[arch] = {"status": "failed", "diagnostics": diagnostics}
    report = {"results": [{"name": "fixture", "candidate": str(candidate), "arches": arches}]}
    analysis = analyze_report(report, candidate_sources={"fixture": source}, repo_root=repo)
    assert analysis["unique_unresolved_roots"] == 5
    by_symbol = {item["symbol"]: item for item in analysis["roots"]}
    assert by_symbol["HICON"]["route"] == "import_existing"
    assert by_symbol["POINTL"]["route"] == "reuse_project_binding"
    assert by_symbol["SYSTEMTIME"]["route"] == "generate_from_sdk"
    assert by_symbol["InternalHelper"]["route"] == "generate_local_or_bridge"
    assert by_symbol["MISSING"]["route"] == "authority_backlog"
    assert all(item["cross_arch_stable"] for item in analysis["roots"])
    assert analysis["generation_plan"]["sdk_types"] == ["SYSTEMTIME"]
    assert analysis["generation_plan"]["shell32_authorities"] == ["HICON"]


def test_source_authority_is_scoped_to_its_candidate(tmp_path: Path):
    repo = tmp_path / "repo"
    (repo / "sdk/include").mkdir(parents=True)
    first_source = tmp_path / "first.c"
    second_source = tmp_path / "second.c"
    first_source.write_text("static int LocalOnly(void) { return 1; }")
    second_source.write_text("int other(void) { return LocalOnly(); }")
    candidates = []
    for name in ("first", "second"):
        candidate = tmp_path / f"{name}.rs"
        candidate.write_text("pub unsafe fn worker() {}")
        candidates.append(
            {
                "name": name,
                "candidate": str(candidate),
                "arches": {
                    "amd64": {
                        "diagnostics": [
                            {
                                "category": "unresolved_symbol",
                                "message": "cannot find function `LocalOnly` in this scope",
                            }
                        ]
                    }
                },
            }
        )
    analysis = analyze_report(
        {"results": candidates},
        candidate_sources={"first": first_source, "second": second_source},
        repo_root=repo,
    )
    by_candidate = {item["candidate"]: item for item in analysis["roots"]}
    assert by_candidate["first"]["route"] == "generate_local_or_bridge"
    assert by_candidate["second"]["route"] == "authority_backlog"
