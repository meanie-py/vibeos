"""cpp_templates: argument-independent template layouts for cppapp's external bases."""

import pytest

from rustconv import cpp_app, cpp_templates
from rustconv.cpp_model import ModuleModel


def test_a_registered_template_maps_its_arguments():
    tb = cpp_templates.template_base("CicArray<CUIFObject *>", lambda a: f"*mut {a[:-1].strip()}")
    assert tb == cpp_templates.TemplateBase("CicArray<*mut CUIFObject>", "cicero-rust")


def test_an_unregistered_template_is_none():
    assert cpp_templates.template_base("CUiWindow<CWindow>", str) is None


def test_external_base_and_field_types_use_the_registry():
    model = ModuleModel()
    assert cpp_app._external_base("CicArray<HICON>", model) == "CicArray<HICON>"
    assert cpp_app.rust_type("CicArray<DWORD>", model, field=True) == "CicArray<DWORD>"
    with pytest.raises(cpp_app.CppAppError, match="no Rust layout"):
        cpp_app._external_base("CUiWindow<CWindow>", model)
