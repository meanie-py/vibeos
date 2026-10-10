from rustconv.__main__ import _extend_cpp_class_scan
from rustconv.config import REACTOS_ROOT
from rustconv.cpp_tu import _find_rust_factory, analyze_translation_unit
from rustconv.scanner import scan_file

SHELL32 = REACTOS_ROOT / "dll/win32/shell32"


def test_cpp_tu_recognizes_internal_nonaggregated_enum_object():
    report = analyze_translation_unit(SHELL32 / "CEnumIDListBase.cpp", REACTOS_ROOT)
    assert report.ok
    assert report.free_functions == ("IEnumIDList_Constructor",)
    assert len(report.classes) == 1
    cls = report.classes[0]
    assert cls.name == "CEnumIDListBase"
    assert cls.ownership == "translate"
    assert cls.activation == "internal-shell-object-creator"
    assert cls.interfaces == ("IEnumIDList",)
    assert cls.state_fields == (
        "ENUMLIST *mpFirst",
        "ENUMLIST *mpLast",
        "ENUMLIST *mpCurrent",
    )


def test_cpp_class_extension_does_not_claim_uninstantiated_header_template():
    source = SHELL32 / "CEnumIDListBase.cpp"
    scan = _extend_cpp_class_scan(scan_file(source), source, "CEnumIDListBase")
    methods = [item.name for item in scan.functions if item.cpp_class == "CEnumIDListBase"]
    assert "HasItemWithCLSIDImpl" not in methods


def test_cpp_tu_peels_rust_owned_class_and_keeps_free_closure():
    report = analyze_translation_unit(SHELL32 / "CDropTargetHelper.cpp", REACTOS_ROOT)
    assert report.ok
    assert report.free_functions == ("SH32_SimulateDropWithSite", "SHSimulateDropOnClsid")
    assert len(report.classes) == 1
    cls = report.classes[0]
    assert cls.name == "CDropTargetHelper"
    assert cls.ownership == "rust-owned"
    assert cls.rust_owner == "sdk/lib/rust/shell32-rust/src/drop_target_helper.rs"


def test_cpp_tu_accepts_proven_nested_atl_rust_factory_adapter():
    report = analyze_translation_unit(SHELL32 / "CRecycleBinCleaner.cpp", REACTOS_ROOT)
    assert report.ok
    assert report.free_functions == ()
    assert len(report.classes) == 1
    cls = report.classes[0]
    assert cls.name == "CRecycleBinCleaner"
    assert cls.ownership == "rust-owned"
    assert cls.rust_owner == "sdk/lib/rust/shell32-rust/src/recycle_bin_cleaner.rs"


def test_cpp_tu_accepts_exact_local_raii_marker_in_discovered_owner():
    report = analyze_translation_unit(SHELL32 / "dialogs/drvdefext.cpp", REACTOS_ROOT)
    assert report.ok
    assert [(cls.name, cls.ownership) for cls in report.classes] == [
        ("CDrvDefExt", "rust-owned"),
        ("CDrop", "rust-owned-local-raii"),
    ]
    assert report.free_functions == (
        "GetDriveNameWithLetter",
        "InitializeChkDskDialog",
        "ChkdskCallback",
        "ChkDskNow",
        "ChkDskDlg",
        "GetDriveTypeAndCharacteristics",
        "IsDriveFloppyW",
        "IsDriveFloppyA",
        "IsValidDrivePath",
        "SH_FormatByteSize_RustBridge",
        "ChkDskDlg_RustBridge",
    )


def test_cpp_tu_finds_integrated_printers_enum_factory():
    report = analyze_translation_unit(SHELL32 / "folders/CPrinterFolder.cpp", REACTOS_ROOT)
    assert any(
        cls.name == "CPrinterFolder" and cls.ownership == "rust-owned" for cls in report.classes
    )
    assert any(
        cls.name == "CPrintersEnum" and cls.ownership == "rust-owned" for cls in report.classes
    )


def test_cpp_tu_does_not_treat_extern_factory_import_as_rust_ownership(tmp_path):
    source = tmp_path / "sdk/lib/rust/shell32-rust/src/import.rs"
    source.parent.mkdir(parents=True)
    source.write_text('extern "system" { fn CImported_CreateInstance(outer: *mut IUnknown); }\n')
    assert _find_rust_factory(tmp_path, "CImported") is None
