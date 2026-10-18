"""port translate hands translate the canonical lowering of each RAII type a C++ TU names."""

from rustconv import port


def test_a_tu_naming_a_canonical_raii_type_gets_its_lowering(tmp_path):
    tu = tmp_path / "x_cpp.hpp"
    tu.write_text("HRESULT Drop(IDataObject *p) { CDataObjectHIDA pida(p); return pida.hr(); }\n")
    args = port._local_raii_args(tu)
    assert args == ["--local-raii", f"CDataObjectHIDA={port.LOCAL_RAII / 'CDataObjectHIDA.rs'}"]
    assert (port.LOCAL_RAII / "CDataObjectHIDA.rs").is_file()


def test_a_longer_name_is_not_a_mention(tmp_path):
    tu = tmp_path / "y_cpp.hpp"
    tu.write_text("void f() { CDataObjectHIDAEx x; }\n")
    assert port._local_raii_args(tu) == []
