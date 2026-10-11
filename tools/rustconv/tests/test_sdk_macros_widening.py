from pathlib import Path

from rustconv import sdk_macros


def test_one_names_probe_error_does_not_hide_another(monkeypatch):
    # cmd: `__WINE_IS_DEBUG_ON("rcv")` is an arity error; a shared probe failed every TU,
    # so `_T` (725 call sites) was never widened. `_T` is defined only in the second TU.
    monkeypatch.setattr(sdk_macros, "compile_argv", lambda repo, arch, tu: ["cc"])

    def pre(argv, tu, probe):
        if "__WINE_IS_DEBUG_ON" in probe:
            raise sdk_macros.LowerError("macro expects 2 arguments")
        return probe.replace('_T("rcv")', 'L"rcv"') if tu.name == "b.c" else probe

    monkeypatch.setattr(sdk_macros, "_preprocess", pre)
    tus = [Path("a.c"), Path("b.c")]
    assert sdk_macros.widening_macros(Path("."), tus, {"_T", "__WINE_IS_DEBUG_ON"}) == {"_T"}


def test_a_widened_literal_holding_a_comment_close_stays_in_its_comment():
    # cmd's set.c `_T("*/%+-&^|")`: the trailing note must not end at the literal's `*/`.
    out, n = sdk_macros.widen_call_sites('f(_T("*/%+-&^|"), p);\n', {"_T"})
    assert n == 1
    note = out[out.index("/*") + 2 :]
    assert note.index("*/") == len(note) - len("*/, p);\n")
