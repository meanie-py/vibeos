"""local_macros.header_functions: inline functions in a module's own headers."""


def test_header_functions_skip_a_struct_the_scanner_misreads(tmp_path):
    """efilter.h: semicolon-less macro invocations, then `typedef struct ETHI_FILTER {`."""
    from rustconv.local_macros import header_functions

    h = tmp_path / "efilter.h"
    h.write_text(
        "#define DECLARE_UNKNOWN_PROTOTYPE(Name) typedef VOID (*(Name))(VOID);\n"
        "DECLARE_UNKNOWN_PROTOTYPE(ETH_ADDRESS_CHANGE)\n\n"
        "typedef struct ETHI_FILTER {\n  PVOID Lock;\n} ETHI_FILTER, *PETHI_FILTER;\n"
        "static __inline int Twice(int x) { return x * 2; }\n"
    )
    names = set(header_functions([h], tmp_path))
    assert "ETHI_FILTER" not in names
    assert "Twice" in names


def test_a_helper_named_after_its_struct_is_kept(tmp_path):
    """wined3d_gl.h: the struct and its container-of helper share a name, as wine always does."""
    from rustconv.local_macros import header_functions

    h = tmp_path / "wined3d_gl.h"
    h.write_text(
        "struct wined3d_context_gl\n{\n    struct wined3d_context c;\n};\n\n"
        "static inline struct wined3d_context_gl *\n"
        "wined3d_context_gl(struct wined3d_context *context)\n"
        "{\n    return CONTAINING_RECORD(context, struct wined3d_context_gl, c);\n}\n"
    )
    assert list(header_functions([h], tmp_path)) == ["wined3d_context_gl"]
