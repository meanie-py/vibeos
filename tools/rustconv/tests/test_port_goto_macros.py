"""port translate: a fresh TU that calls a label-jumping macro is bought with it expanded."""

from rustconv import port


def test_goto_macro_uses_are_found_per_tu_but_not_at_the_definition(tmp_path, monkeypatch):
    mod = tmp_path / "base" / "m"
    mod.mkdir(parents=True)
    (mod / "m.h").write_text(
        "#define CHK_HR(stmt) do { hr=(stmt); if (FAILED(hr)) goto CleanUp; } while(0)\n"
        "#define SAFE_RELEASE(p) do { if (p) (p)->Release(); } while(0)\n"
    )
    # cmd.c's shape: the macro is defined and used in the one file.
    (mod / "a.c").write_text(
        "#define APPEND(f) \\\n do { if (f) goto too_long; } while (0)\n"
        "void f(void) { APPEND(x); too_long: ; }\n"
    )
    (mod / "b.c").write_text('#include "m.h"\nvoid g(void) { CHK_HR(h()); CleanUp: ; }\n')
    (mod / "c.c").write_text('void k(void) { SAFE_RELEASE(p); puts("CHK_HR"); }\n')
    (mod / "d.c").write_text("#define APPEND(f) do { if (f) goto too_long; } while (0)\n")
    monkeypatch.setattr(port, "REPO", tmp_path)
    state = {"module_dir": "base/m"}
    tus = [f"base/m/{n}" for n in ("a.c", "b.c", "c.c", "d.c")]
    assert port._goto_macro_calls(state, tus) == {"a.c": ["APPEND"], "b.c": ["CHK_HR"]}
    assert port._goto_macro_calls(state, []) == {}
