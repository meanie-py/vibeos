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
from rustconv.macro_authority import MacroAuthorityError, _compile_entry, function_like_macros

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
