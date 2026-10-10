"""port macros: a module's own C macros are found, closed over, and the reply is held to the request."""

import pytest

from rustconv import local_macros as lm

FILESUP_H = """#define DoesDirExist(RootDirectory, DirName)    \\
    DoesPathExist((RootDirectory), (DirName), TRUE)

#define DoesFileExist(RootDirectory, FileName)  \\
    DoesPathExist((RootDirectory), (FileName), FALSE) /* comment */
#define NOT_A_FN_MACRO 3
"""

VOLUTIL_H = """#define IS_RAWFS(fs) \\
    ((fs) && wcsicmp((fs), L"RAW") == 0)
#define IsUnformatted(VolInfo) \\
    IS_RAWFS((VolInfo)->FileSystem)
#ifdef X
#define TWICE(a) (a)
#else
#define TWICE(a) (a + 1)
#endif
"""


@pytest.fixture
def defs(tmp_path):
    (tmp_path / "filesup.h").write_text(FILESUP_H)
    (tmp_path / "volutil.h").write_text(VOLUTIL_H)
    found, ambiguous = lm.module_macros(sorted(tmp_path.glob("*.h")), tmp_path)
    return found, ambiguous


def test_function_like_defines_are_found_with_continuations(defs):
    found, ambiguous = defs
    assert set(found) == {"DoesDirExist", "DoesFileExist", "IS_RAWFS", "IsUnformatted"}
    assert found["DoesFileExist"].params == ("RootDirectory", "FileName")
    assert found["DoesFileExist"].body == "DoesPathExist((RootDirectory), (FileName), FALSE)"
    assert found["DoesDirExist"].where == "filesup.h:1"
    assert ambiguous == {"TWICE"}


def test_closure_pulls_in_the_local_macros_a_body_uses(defs):
    found, _ = defs
    assert set(lm.closure({"IsUnformatted", "Unrelated"}, found)) == {"IsUnformatted", "IS_RAWFS"}


GOOD = """```rust
macro_rules! A {
    ($x:expr $(,)?) => { ($x + 1) };
}

#[allow(unused)]
macro_rules! B { ($p:expr) => {{ (*$p).f = 0; }}; }
```"""


def test_reply_must_be_exactly_the_requested_macros():
    got = lm.parse_reply(GOOD, {"A", "B"})
    assert set(got) == {"A", "B"} and got["B"].endswith("}") and "macro_rules! B" in got["B"]
    assert set(lm.parse_reply(GOOD.split("```rust\n")[1].rstrip("`"), {"A", "B"})) == {"A", "B"}
    with pytest.raises(ValueError, match="missing"):
        lm.parse_reply(GOOD, {"A", "B", "C"})
    with pytest.raises(ValueError, match="extra"):
        lm.parse_reply(GOOD, {"A"})
    with pytest.raises(ValueError, match="extra"):  # an unrequested helper fn
        lm.parse_reply(GOOD.replace("```rust\n", "```rust\nfn helper() {}\n"), {"A", "B"})
    with pytest.raises(ValueError, match="non-macro"):
        lm.parse_reply(GOOD.replace("```rust\n", "```rust\nconst X: u32 = 1;\n"), {"A", "B"})


def test_rendered_block_round_trips_and_replaces_in_place(tmp_path, defs):
    found, _ = defs
    ported = {"DoesFileExist": "macro_rules! DoesFileExist { ($r:expr, $f:expr) => { (1) }; }"}
    block = lm.render(ported, found)
    assert "pub(crate) use DoesFileExist;" in block and "filesup.h:4" in block
    p = tmp_path / "prelude.rs"
    p.write_text("// head\n")
    assert lm.write_block(p, block)
    assert not lm.write_block(p, block)
    assert lm.existing(p) == {"DoesFileExist"}
    assert lm.key({"DoesFileExist": found["DoesFileExist"]}) != lm.key(found)


def test_prompt_names_in_scope_macros_and_key_tracks_rules(tmp_path, defs):
    found, _ = defs
    shared = tmp_path / "win32macros-rust" / "src"
    shared.mkdir(parents=True)
    (shared / "lib.rs").write_text(
        "macro_rules! LOWORD { () => {} }\nmacro_rules! ZeroMemory { () => {} }\n"
        "pub fn max(a: i32, b: i32) -> i32 { a }\n"
    )
    (tmp_path / "x-rust" / "src").mkdir(parents=True)
    p = tmp_path / "x-rust" / "src" / "prelude.rs"
    # `max` is a FUNCTION the shared crate re-exports: listing it told comctl32's model `max!`.
    p.write_text(
        "macro_rules! CONTAINING_RECORD { () => {} }\n"
        "pub use win32macros_rust::{LOWORD, ZeroMemory, max};\n"
        + lm.render({"X": "macro_rules! X { () => {} }"}, {"X": found["IS_RAWFS"]})
    )
    assert lm.in_scope_macros(p) == {"CONTAINING_RECORD", "LOWORD", "ZeroMemory"}
    assert "CONTAINING_RECORD, LOWORD, ZeroMemory" in lm.prompt(found, lm.in_scope_macros(p))
    assert lm.key(found, "on") != lm.key(found, "off")


def test_object_like_expression_macros_are_found_but_literals_are_not(tmp_path):
    (tmp_path / "infsupp.h").write_text(
        "#define SpInfFindFirstLine      (SpInfExports.SpInfFindFirstLine)\n"
        '#define INF_STYLE_WIN4  0x00000002\n#define NDEBUG\n#define NAME L"x"\n'
        "#define WRAP(x) (SpInfFindFirstLine(x))\n"
    )
    found, _ = lm.module_macros([tmp_path / "infsupp.h"], tmp_path)
    assert set(found) == {"SpInfFindFirstLine", "WRAP"}
    assert found["SpInfFindFirstLine"].params is None
    assert found["SpInfFindFirstLine"].c_text() == (
        "#define SpInfFindFirstLine (SpInfExports.SpInfFindFirstLine)"
    )
    # WRAP's body uses it, but an object-like macro joins only by request
    assert set(lm.closure({"WRAP"}, found)) == {"WRAP"}


def test_header_inline_functions_round_trip(tmp_path):
    h = tmp_path / "infsupp.h"
    h.write_text(
        "FORCEINLINE\nVOID\nINF_FreeData(\n    IN PCWSTR InfData)\n{\n    UNREFERENCED_PARAMETER(InfData);\n}\n"
    )
    fns = lm.header_functions([h], tmp_path)
    assert set(fns) == {"INF_FreeData"} and fns["INF_FreeData"].kind == "fn"
    assert "INF_FreeData(" in lm.prompt(fns) and "pub unsafe fn" in lm.prompt(fns)
    reply = (
        "```rust\n#[inline]\npub unsafe fn INF_FreeData(InfData: PCWSTR) {\n"
        "    UNREFERENCED_PARAMETER!(InfData);\n}\n```"
    )
    got = lm.parse_reply(reply, {"INF_FreeData"})
    assert got["INF_FreeData"].startswith("#[inline]\npub unsafe fn INF_FreeData")
    p = tmp_path / "prelude.rs"
    p.write_text("")
    lm.write_block(p, lm.render(got, fns))
    assert lm.existing(p) == {"INF_FreeData"}
    assert "pub(crate) use INF_FreeData" not in p.read_text()


def test_expand_follows_c_rules():
    from rustconv.local_macros import CMacro, expand

    defs = {
        "SpInfFindFirstLine": CMacro(
            "SpInfFindFirstLine", None, "(SpInfExports.SpInfFindFirstLine)", "h:1"
        ),
        "Size": CMacro("Size", ("P",), "((P)->Count * (P)->Bytes)", "h:2"),
        "Twice": CMacro("Twice", ("X",), "(Size(X) * 2)", "h:3"),
        "MAX_X": CMacro("MAX_X", None, "(BASE + 1)", "h:4"),
        "STR": CMacro("STR", ("x",), "#x", "h:5"),
    }
    src = 'SpInfFindFirstLine(a); n = Twice(e); m = MAX_X; s = STR(q); t = "Size(z)"; /* Size(z) */'
    out, used = expand(src, defs)
    # A macro is not re-expanded inside its own expansion.
    assert "(SpInfExports.SpInfFindFirstLine)(a)" in out
    # Nested, arguments substituted as whole tokens.
    assert "n = (((e)->Count * (e)->Bytes) * 2)" in out
    # Constants stay names (the seal needs them); # bodies, strings and comments untouched.
    assert "m = MAX_X" in out and "s = STR(q)" in out
    assert '"Size(z)"' in out and "/* Size(z) */" in out
    assert used == {"SpInfFindFirstLine", "Twice", "Size"}


def test_a_table_call_never_panics():
    from rustconv.local_macros import parse_reply

    reply = (
        "macro_rules! SpInfFindFirstLine {\n    ($($a:expr),*) => {\n"
        "        ((SpInfExports.SpInfFindFirstLine.unwrap())($($a),*))\n    };\n}\n"
    )
    out = parse_reply(reply, {"SpInfFindFirstLine"})
    assert ".unwrap_unchecked())($($a),*)" in out["SpInfFindFirstLine"]
    import pytest

    with pytest.raises(ValueError, match="panic path"):
        parse_reply(
            reply.replace("SpInfFindFirstLine.unwrap())", 'X.expect("y"))'), {"SpInfFindFirstLine"}
        )


def test_file_and_line_get_their_c_types():
    from rustconv.local_macros import c_source_location

    out = c_source_location("f($e, file!(), line!());")
    assert out == (
        'f($e, concat!(file!(), "\\0").as_ptr() as *const core::ffi::c_char, '
        "(line!() as core::ffi::c_int));"
    )
    assert c_source_location(out) == out  # idempotent
    # cmd's ERR!/TRACE! hand CmdTrace the C's __FUNCTION__.
    fn = c_source_location("CmdTrace(1, __FUNCTION__, $fmt)")
    assert fn == (
        'CmdTrace(1, concat!(module_path!(), "\\0").as_ptr() as *const core::ffi::c_char, $fmt)'
    )
    assert c_source_location(fn) == fn
