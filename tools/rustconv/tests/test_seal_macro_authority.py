"""A function-like macro must never be sealed as a runtime callee.

`IID_PPV_ARG(IProgressDialog, &ppv)` expands to TWO ARGUMENTS, not a call, so
"preserve this token exactly once as the called function identifier" is
unsatisfiable — and one unsatisfiable worker voids the whole TU. The macro set
comes from the real preprocessor (`macro_authority`), replacing a hand-maintained
denylist that only ever grew after someone ate a reject.
"""

import json

import pytest

from rustconv.call_seal import CallSealError, seal_calls, unseal_calls
from rustconv.macro_authority import (
    MacroAuthorityError,
    _compile_entry,
    _function_like,
    function_like_macros,
)

SOURCE = """HRESULT go() {
    ret = CoCreateInstance(CLSID_X, NULL, CTX, IID_PPV_ARG(IProgressDialog, &ppv));
    Helper(&op);
}"""


def test_function_like_macro_is_not_sealed():
    sealed = seal_calls(SOURCE, {"CoCreateInstance", "IID_PPV_ARG", "Helper"}, frozenset())
    assert "IID_PPV_ARG" in [call.original for call in sealed.calls]

    guarded = seal_calls(
        SOURCE, {"CoCreateInstance", "IID_PPV_ARG", "Helper"}, frozenset({"IID_PPV_ARG"})
    )
    assert [call.original for call in guarded.calls] == ["CoCreateInstance", "Helper"]
    # The macro survives verbatim, so the model sees the real construct.
    assert "IID_PPV_ARG(IProgressDialog, &ppv)" in guarded.text


def test_sealing_a_macro_makes_the_envelope_unsatisfiable():
    """The regression itself: a faithful lowering still fails the audit."""
    sealed = seal_calls(SOURCE, {"IID_PPV_ARG"}, frozenset())
    faithful = """unsafe fn go() {
        ret = CoCreateInstance(CLSID_X, null_mut(), CTX, &IID_IProgressDialog, &mut ppv);
        Helper(&mut op);
    }"""
    with pytest.raises(CallSealError, match="missing/non-executable"):
        unseal_calls(faithful, sealed)

    guarded = seal_calls(SOURCE, {"IID_PPV_ARG"}, frozenset({"IID_PPV_ARG"}))
    assert guarded.calls == ()
    assert unseal_calls(faithful, guarded) == faithful


def test_missing_compile_command_hard_fails_with_the_fix(tmp_path):
    """Never a silent fallback to "seal everything" — that is the bug, not the guard."""
    (tmp_path / "src").mkdir()
    source = tmp_path / "src" / "nowhere.c"
    source.write_text("int f(void) { return 0; }\n")
    with pytest.raises(MacroAuthorityError, match=r"vibeos-cli build"):
        function_like_macros(tmp_path, source)


def test_compile_entry_matches_the_whole_path_not_the_basename(tmp_path):
    """Several modules ship a `shell.c`; the wrong one gives the wrong macro set."""
    build = tmp_path / "build-i386"
    build.mkdir()
    (tmp_path / "dll" / "win32" / "a").mkdir(parents=True)
    (tmp_path / "dll" / "win32" / "b").mkdir(parents=True)
    build.joinpath("compile_commands.json").write_text(
        json.dumps(
            [
                {
                    "directory": "/tmp/vibeos/build-i386",
                    "file": "/tmp/vibeos/dll/win32/a/shell.c",
                    "command": "clang -DA_MODULE -c /tmp/vibeos/dll/win32/a/shell.c",
                }
            ]
        )
    )
    wanted = tmp_path / "dll" / "win32" / "a" / "shell.c"
    other = tmp_path / "dll" / "win32" / "b" / "shell.c"
    assert _compile_entry(tmp_path, "i386", wanted) is not None
    assert _compile_entry(tmp_path, "i386", other) is None


# ---------------------------------------------------------------------------
# A header has no compile command of its own
# ---------------------------------------------------------------------------


def _db(tmp_path, entries):
    build = tmp_path / "build-i386"
    build.mkdir(exist_ok=True)
    build.joinpath("compile_commands.json").write_text(json.dumps(entries))


def test_a_header_borrows_the_macro_set_of_a_tu_that_includes_it(tmp_path):
    """`static inline` helper headers are only translatable because of this.

    wldap32.h is 62 marshalling helpers that every TU inlines a private copy of, so no
    .c defines them and nothing in the module's compile db names the header — yet the
    translated modules call them 300-odd times. A header's macro environment IS its
    includers', so borrowing an includer's flags is exact, not a guess.
    """
    mod = tmp_path / "dll" / "win32" / "widget"
    mod.mkdir(parents=True)
    (mod / "widget.h").write_text("static inline int helper(void) { return 0; }\n")
    (mod / "main.c").write_text('#include "widget.h"\nint go(void) { return helper(); }\n')
    _db(
        tmp_path,
        [
            {
                "directory": "/tmp/vibeos/build-i386",
                "file": "/tmp/vibeos/dll/win32/widget/main.c",
                "command": "clang -DWIDGET_BUILD -c /tmp/vibeos/dll/win32/widget/main.c",
            }
        ],
    )
    from rustconv.macro_authority import _includer_entry

    argv = _includer_entry(tmp_path, "i386", mod / "widget.h")
    assert argv is not None and "-DWIDGET_BUILD" in argv


def test_only_a_tu_in_the_headers_OWN_directory_counts(tmp_path):
    """A module-private header is included by its own TUs; matching tree-wide would
    hand it an arbitrary other module's flags."""
    a = tmp_path / "dll" / "win32" / "a"
    b = tmp_path / "dll" / "win32" / "b"
    a.mkdir(parents=True)
    b.mkdir(parents=True)
    (a / "shared.h").write_text("static inline int helper(void) { return 0; }\n")
    (b / "main.c").write_text('#include "shared.h"\nint go(void) { return helper(); }\n')
    _db(
        tmp_path,
        [
            {
                "directory": "/tmp/vibeos/build-i386",
                "file": "/tmp/vibeos/dll/win32/b/main.c",
                "command": "clang -DB_MODULE -c /tmp/vibeos/dll/win32/b/main.c",
            }
        ],
    )
    from rustconv.macro_authority import _includer_entry

    assert _includer_entry(tmp_path, "i386", a / "shared.h") is None


def test_a_header_nobody_includes_still_fails_loud(tmp_path):
    mod = tmp_path / "dll" / "win32" / "widget"
    mod.mkdir(parents=True)
    (mod / "orphan.h").write_text("static inline int helper(void) { return 0; }\n")
    _db(tmp_path, [])
    with pytest.raises(MacroAuthorityError, match="no compiled TU in its own directory"):
        function_like_macros(tmp_path, mod / "orphan.h")


# ---------------------------------------------------------------------------
# A FLIPPED .c has no compile command of its own either
# ---------------------------------------------------------------------------


def test_a_flipped_tu_borrows_its_own_targets_flags(tmp_path):
    """A cohab flip comments the .c out of the source list, so it leaves the compile db.

    That is exactly the module you want to re-translate or audit against its C, and the
    surviving TUs of the same target carry the identical -I/-D environment. wkssvc is fully
    flipped: all four .c are commented out and only the GENERATED MIDL `wkssvc_s.c` remains,
    under `build-<arch>/<same relative dir>` — so the build-dir spelling of the module
    directory has to count too, or a fully-flipped module resolves nothing.
    """
    from rustconv.macro_authority import _sibling_target_entry

    mod = tmp_path / "base" / "services" / "widget"
    mod.mkdir(parents=True)
    (mod / "flipped.c").write_text("int go(void) { return 0; }\n")
    gen = tmp_path / "build-i386" / "base" / "services" / "widget"
    gen.mkdir(parents=True)
    (gen / "widget_s.c").write_text("int stub(void) { return 0; }\n")
    _db(
        tmp_path,
        [
            {
                "directory": "/tmp/vibeos/build-i386",
                "file": "/tmp/vibeos/build-i386/base/services/widget/widget_s.c",
                "command": (
                    "clang -DWIDGET_BUILD -o "
                    "/tmp/vibeos/build-i386/base/services/widget/widget_s.c.obj "
                    "-c /tmp/vibeos/build-i386/base/services/widget/widget_s.c"
                ),
            }
        ],
    )
    argv = _sibling_target_entry(tmp_path, "i386", mod / "flipped.c")
    assert argv is not None
    assert "-DWIDGET_BUILD" in argv
    # The borrowed command must preprocess OUR file, not the sibling it was lent from.
    assert str(mod / "flipped.c") in argv
    assert not any(a.endswith("widget_s.c") for a in argv)
    # `-o …/widget_s.c.obj` also ends in the sibling's basename; rewriting the OUTPUT
    # instead of the input would silently derive the macro set of the wrong file.
    assert argv[argv.index("-o") + 1].endswith("widget_s.c.obj")


def test_a_tu_whose_module_compiles_nothing_still_fails_loud(tmp_path):
    """No sibling to borrow from is still a hard failure, not an empty macro set.

    An empty set is the worst possible answer: every macro occurrence then seals as a
    runtime callee and no translation can satisfy the envelope, so the TU assembles nothing.
    """
    mod = tmp_path / "base" / "services" / "widget"
    mod.mkdir(parents=True)
    (mod / "flipped.c").write_text("int go(void) { return 0; }\n")
    _db(tmp_path, [])
    with pytest.raises(MacroAuthorityError, match=r"vibeos-cli build"):
        function_like_macros(tmp_path, mod / "flipped.c")


def test_a_sibling_from_ANOTHER_module_is_not_borrowed(tmp_path):
    """Same reasoning as the header case: another module's flags are the wrong flags."""
    from rustconv.macro_authority import _sibling_target_entry

    a = tmp_path / "base" / "services" / "a"
    b = tmp_path / "base" / "services" / "b"
    a.mkdir(parents=True)
    b.mkdir(parents=True)
    (a / "flipped.c").write_text("int go(void) { return 0; }\n")
    (b / "live.c").write_text("int other(void) { return 0; }\n")
    _db(
        tmp_path,
        [
            {
                "directory": "/tmp/vibeos/build-i386",
                "file": "/tmp/vibeos/base/services/b/live.c",
                "command": "clang -DB_MODULE -c /tmp/vibeos/base/services/b/live.c",
            }
        ],
    )
    assert _sibling_target_entry(tmp_path, "i386", a / "flipped.c") is None


def test_an_object_like_ALIAS_of_a_function_like_macro_is_function_like():
    """mingw spells three of the four va_ macros as bare aliases.

    `#define va_arg __crt_va_arg` is object-like, so the `#define NAME(` scan could not
    see it, while `#define va_copy(d,s) ...` right beside it was caught. The seal then
    tokenized `va_arg(ap, int)` as a runtime callee of arity 2, which no Rust rendering
    can match -- the type is a turbofish and the arity check blanks its commas. That is
    an envelope the model cannot satisfy at any price, so every retry re-buys the same
    reject.
    """
    dump = "\n".join(
        [
            "#define __crt_va_arg(v,l) __builtin_va_arg(v,l)",
            "#define va_arg __crt_va_arg",
            "#define va_copy(d,s) __va_copy((d),(s))",
            "#define PLAIN_OBJECT 1",
            "#define ALIAS_OF_OBJECT PLAIN_OBJECT",
        ]
    )
    names = _function_like(dump)
    assert {"__crt_va_arg", "va_arg", "va_copy"} <= names
    # An alias whose target is NOT function-like stays out: it has no call site to
    # protect, and widening the set silently relaxes the seal for an ordinary constant.
    assert "ALIAS_OF_OBJECT" not in names
    assert "PLAIN_OBJECT" not in names


def test_an_alias_CHAIN_resolves_and_a_cycle_terminates():
    chained = _function_like("#define REAL(a) a\n#define MID REAL\n#define TOP MID\n")
    assert {"REAL", "MID", "TOP"} <= chained
    # A cyclic pair defines nothing callable; it must not hang the derivation.
    assert _function_like("#define A B\n#define B A\n") == set()
