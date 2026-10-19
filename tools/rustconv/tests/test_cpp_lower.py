def test_a_function_local_enum_becomes_consts():
    from rustconv.cpp_lower import local_enums_to_consts

    body = "{\n    enum { IDC_EXTRACT = 1, IDC_EXTRACTALL };\n    switch (id) { case IDC_EXTRACT: f(); }\n}"
    out = local_enums_to_consts(body)
    assert "const int IDC_EXTRACT = 1; const int IDC_EXTRACTALL = IDC_EXTRACT + 1;" in out
    assert "enum" not in out
    assert local_enums_to_consts("enum E { A, B };") == (
        "typedef int E; const int A = 0; const int B = A + 1;"
    )
    weird = "enum { A = sizeof(x), B };"
    assert local_enums_to_consts(weird) == "const int A = sizeof(x); const int B = A + 1;"


def test_a_hidden_stdcall_is_spelled_in_a_free_function_signature():
    from rustconv.cpp_model import _spell_stdcall

    assert _spell_stdcall("STDAPI_(BOOL) DllMain(HINSTANCE h)", "DllMain", "") == (
        "BOOL WINAPI DllMain(HINSTANCE h)"
    )
    assert _spell_stdcall("STDAPI DllCanUnloadNow(void)", "DllCanUnloadNow", "") == (
        "HRESULT WINAPI DllCanUnloadNow(void)"
    )
    qt = "int (int) __attribute__((stdcall))"
    assert _spell_stdcall("MYAPI int F(int x)", "F", qt) == "MYAPI int __stdcall F(int x)"
    assert _spell_stdcall("int WINAPI F(int x)", "F", qt) == "int WINAPI F(int x)"
    assert _spell_stdcall("static int F(int x)", "F", "int (int)") == "static int F(int x)"


def test_ccomobject_create_instance_is_the_class_itself():
    from rustconv.cpp_lower import com_object_create

    body = "_CComObject<CFolderViewCB> *pcb;\nHRESULT hr = _CComObject<CFolderViewCB>::CreateInstance(&pcb);"
    assert com_object_create(body) == (
        "CFolderViewCB *pcb;\nHRESULT hr = ComObject_CreateInstance<CFolderViewCB>(&pcb);"
    )


def test_a_module_conditional_in_a_body_is_decided_and_a_foreign_one_left():
    from rustconv.cpp_lower import resolve_module_conditionals

    body = (
        "{\n#if USE_CUSTOM_CONMGR\n    return mine(riid, ppv);\n#else\n"
        "    return theirs(riid, ppv);\n#endif\n}\n"
    )
    out = resolve_module_conditionals(body, {"USE_CUSTOM_CONMGR": "1"})
    assert "mine" in out and "theirs" not in out and "#if" not in out
    assert "theirs" in resolve_module_conditionals(body, {"USE_CUSTOM_CONMGR": "0"})
    arch = "{\n#if defined(_M_IX86)\n    a();\n#else\n    b();\n#endif\n}\n"
    assert resolve_module_conditionals(arch, {"USE_CUSTOM_CONMGR": "1"}) == arch
    nested = "{\n#ifdef FOO\n#if X\n x();\n#endif\n#else\n y();\n#endif\n}\n"
    out = resolve_module_conditionals(nested, {"FOO": ""})
    assert "#if X" in out and "y();" not in out
