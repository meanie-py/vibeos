"""cpp_templates: argument-independent template layouts for cppapp's external bases."""

import pytest

from rustconv import cpp_app, cpp_templates
from rustconv.cpp_model import ModuleModel


def test_a_registered_template_maps_its_arguments():
    tb = cpp_templates.template_base("CicArray<CUIFObject *>", lambda a: f"*mut {a[:-1].strip()}")
    assert tb == cpp_templates.TemplateBase(
        "CicArray<*mut CUIFObject>", "cicero-rust", polymorphic=True, drops=True
    )


def test_a_template_that_frees_on_destruction_drops():
    # CicArrayBase's destructor frees m_pb: an owner deriving Copy would double-free it.
    assert cpp_templates.drops("CicArray<HICON>")
    assert not cpp_templates.drops("CUiWindow<CWindow>")
    assert not cpp_templates.drops("DWORD")


def test_an_unregistered_template_is_none():
    assert cpp_templates.template_base("CUiWindow<CWindow>", str) is None


def test_external_base_and_field_types_use_the_registry():
    model = ModuleModel()
    assert cpp_app._external_base("CicArray<HICON>", model) == "CicArray<HICON>"
    assert cpp_app.rust_type("CicArray<DWORD>", model, field=True) == "CicArray<DWORD>"
    with pytest.raises(cpp_app.CppAppError, match="no Rust layout"):
        cpp_app._external_base("CUiWindow<CWindow>", model)


def test_the_island_copy_of_a_template_reexports_the_generic():
    own = cpp_templates.island_reexport("CicArray", "cicero-rust")
    assert own == "pub use crate::cicero::cicarray_t::CicArray; // cpp_templates owns the generic"
    other = cpp_templates.island_reexport("CicArray", "msctf-rust")
    assert other is not None and other.startswith("pub use cicero_rust::cicero::cicarray_t::")
    assert cpp_templates.island_reexport("CicArrayBase", "cicero-rust") is None


def test_a_typedef_of_an_instantiation_gets_its_arguments_back():
    island = "pub type CDispAttrPropCache = CicArray;\npub type Other = CicArray;\n"
    src = "typedef CicArray<GUID> CDispAttrPropCache;\n"
    out = cpp_templates.retype_aliases(island, src, str)
    assert "pub type CDispAttrPropCache = CicArray<GUID>;" in out
    assert "pub type Other = CicArray;" in out  # no typedef to read: left loud


def test_island_glue_reexports_class_enums_and_derefs_struct_bases():
    from rustconv.cpp_model import ClassModel

    model = ModuleModel()
    model.classes["CUIFWindow"] = ClassModel("CUIFWindow", "CUIFWindow", "class", "cicuif.h")
    island = (
        "pub const CUIFWindow_USER_TIMER_ID: CUIFWindow__bindgen_ty_1 = 21601;\n"
        "pub struct INPUTCONTEXTDX {\n    pub _base: INPUTCONTEXT,\n    pub nVKey: UINT,\n}\n"
    )
    out = cpp_app._render_island_glue(model, island)
    assert "pub const USER_TIMER_ID: CUIFWindow__bindgen_ty_1 = CUIFWindow_USER_TIMER_ID;" in out
    assert "impl core::ops::Deref for INPUTCONTEXTDX { type Target = INPUTCONTEXT;" in out
