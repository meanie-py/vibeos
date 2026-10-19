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
