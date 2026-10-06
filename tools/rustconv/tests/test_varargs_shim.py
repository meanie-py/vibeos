from rustconv import varargs_shim as vs

C = """
NTSTATUS
ConcatPathsV(PWSTR b, SIZE_T n, ULONG k, va_list l);

NTSTATUS
CombinePaths(
    OUT PWSTR PathBuffer,
    IN SIZE_T cchPathSize,
    IN ULONG NumberOfPathComponents,
    IN /* PCWSTR */ ...)
{
    return 0;
}

static VOID __cdecl Trace(_In_ PCSTR Format, ...)
{
}

int Proto(int a, ...);
"""


def test_only_variadic_definitions_are_found():
    defs = vs.variadic_definitions(C)
    assert [d.name for d in defs] == ["CombinePaths", "Trace"]
    cp, tr = defs
    assert cp.ret == "NTSTATUS" and tr.ret == "VOID"
    assert cp.names == ["PathBuffer", "cchPathSize", "NumberOfPathComponents"]
    assert tr.names == ["Format"]


def test_a_trampoline_only_does_va_start_and_hands_over():
    cp, tr = vs.variadic_definitions(C)
    t = vs.trampoline(cp, "f.c")
    assert "NTSTATUS __cdecl CombinePaths_body(OUT PWSTR PathBuffer," in t
    assert "va_start(ap, NumberOfPathComponents);" in t
    assert (
        "r = CombinePaths_body(PathBuffer, cchPathSize, NumberOfPathComponents, (void *)&ap);" in t
    )
    assert t.index("va_end(ap);") < t.index("return r;")
    v = vs.trampoline(tr, "f.c")
    assert "    Trace_body(Format, (void *)&ap);" in v and "return" not in v
