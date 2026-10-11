"""sdk_facts resolves constants only when every architecture agrees.

The compiler-facing half needs a built tree, so these cover the parts that decide whether a
value is ACCEPTED — which is where a wrong answer would actually escape.
"""

from pathlib import Path

import pytest

from rustconv import sdk_facts


def _stub(monkeypatch, per_arch):
    def fake(repo_root, arch, tu, names, types=None):
        values = per_arch.get(arch, {})
        return values, {n: "not an integer constant expression" for n in names if n not in values}

    monkeypatch.setattr(sdk_facts, "_resolve_one_arch", fake)


def test_agreeing_architectures_produce_a_fact(monkeypatch):
    _stub(monkeypatch, {a: {"FOF_NOCONFIRMATION": 16} for a in ("i386", "amd64", "arm64")})
    res = sdk_facts.resolve(Path("/repo"), Path("x.cpp"), ["FOF_NOCONFIRMATION"])
    assert [(f.name, f.value) for f in res.facts] == [("FOF_NOCONFIRMATION", 16)]
    assert res.unresolved == {}


def test_a_value_that_differs_across_arches_is_never_returned(monkeypatch):
    """The i386-derived-width bug class: taking whichever target ran first is wrong."""
    _stub(monkeypatch, {"i386": {"SIZE_THING": 4}, "amd64": {"SIZE_THING": 8}})
    res = sdk_facts.resolve(Path("/repo"), Path("x.cpp"), ["SIZE_THING"], ("i386", "amd64"))
    assert res.facts == []
    assert "arch-dependent" in res.unresolved["SIZE_THING"]
    assert "i386=4" in res.unresolved["SIZE_THING"]


def test_a_name_no_arch_could_evaluate_is_reported_not_dropped(monkeypatch):
    _stub(monkeypatch, {"i386": {}, "amd64": {}})
    res = sdk_facts.resolve(Path("/repo"), Path("x.cpp"), ["CLSID_Thing"], ("i386", "amd64"))
    assert res.facts == []
    assert "CLSID_Thing" in res.unresolved


def test_agreement_is_over_the_arches_that_answered(monkeypatch):
    """One target missing a name must not veto the two that agree on it."""
    _stub(monkeypatch, {"i386": {"A": 1}, "amd64": {"A": 1}, "arm64": {}})
    res = sdk_facts.resolve(Path("/repo"), Path("x.cpp"), ["A"])
    assert [(f.name, f.value) for f in res.facts] == [("A", 1)]


def test_no_architecture_probed_is_a_hard_failure(monkeypatch):
    def boom(repo_root, arch, tu, names, types=None):
        raise sdk_facts.MacroAuthorityError("no compile_commands.json")

    monkeypatch.setattr(sdk_facts, "_resolve_one_arch", boom)
    with pytest.raises(sdk_facts.MacroAuthorityError, match="no architecture could be probed"):
        sdk_facts.resolve(Path("/repo"), Path("x.cpp"), ["A"])


def test_probe_puts_exactly_one_name_per_line():
    """Line attribution is what makes a failure name-able instead of batch-wide."""
    text = sdk_facts._probe_source(Path("/t/x.cpp"), ["A", "B", "C"])
    lines = text.split("\n")
    for index, name in enumerate(["A", "B", "C"]):
        assert f"({name})" in lines[sdk_facts._FIRST_NAME_LINE - 1 + index]


def test_probe_includes_the_real_tu_so_file_local_enums_resolve():
    text = sdk_facts._probe_source(Path("/t/shlexec.cpp"), ["A"])
    assert text.startswith('#include "/t/shlexec.cpp"')


@pytest.mark.parametrize(
    "value,expected", [(0, "u32"), (0xFFFFFFFF, "u32"), (-1, "i32"), (2**33, "u64")]
)
def test_rust_type_selection(value, expected):
    assert sdk_facts.rust_type(value) == expected


def test_syntax_argv_drops_output_flags_and_forces_full_diagnostics():
    argv = ["clang++", "-c", "src.cpp", "-o", "src.o", "-MF", "d.d", "-DFOO", "-Iinc"]
    out = sdk_facts._syntax_argv(argv, Path("/t/probe.cpp"))
    assert "-o" not in out and "src.o" not in out and "src.cpp" not in out
    assert "-DFOO" in out and "-Iinc" in out
    # the default 20-error cap would silently truncate a several-hundred-name batch
    assert "-ferror-limit=0" in out and "-fsyntax-only" in out


def test_c_probe_reads_value_and_typedef_from_clang_text():
    # A plain-C TU is probed as C: a duplicated case label carries the value, an
    # incompatible struct initializer the type (typedef sugar first).
    names = ["STATUS_PARTITION_FAILURE", "ERROR_LAST_ERROR_CODE"]
    src = sdk_facts._probe_source_c(Path("/t/fsutil.c"), names).splitlines()
    first = sdk_facts._C_FIRST_NAME_LINE
    assert "case 0 + (long long)(STATUS_PARTITION_FAILURE)" in src[first - 1]
    assert "struct rcv_t t0 = (STATUS_PARTITION_FAILURE)" in src[first]
    assert "(ERROR_LAST_ERROR_CODE)" in src[first + 1]
    stderr = (
        f"/x/probe.c:{first}:9: error: duplicate case value '-1073741454'\n"
        f"/x/probe.c:{first + 1}:9: error: initializing 'struct rcv_t' with an expression of "
        "incompatible type 'NTSTATUS' (aka 'long')\n"
        f"/x/probe.c:{first + 2}:9: error: duplicate case value: 'A' and 'B' both equal '44'\n"
    )
    vals = {}
    for m in sdk_facts._C_SHOW_RE.finditer(stderr):
        index, _odd = divmod(int(m.group(1)) - first, 2)
        vals[names[index]] = int(m.group(2) or m.group(3))
    assert vals == {"STATUS_PARTITION_FAILURE": -1073741454, "ERROR_LAST_ERROR_CODE": 44}
    t = next(sdk_facts._C_TYPE_RE.finditer(stderr))
    assert (t.group(2), t.group(3)) == ("NTSTATUS", "long")


def test_a_float_literal_needs_a_point_or_an_exponent():
    """Plain digits are an INTEGER macro; reading `900` as a float would retype every fact."""
    from rustconv import sdk_facts

    for text in ("3.14159265358979323846", "1e5", "(0.5f)", ".5"):
        assert sdk_facts._FLOAT_LIT.fullmatch(text), text
    for text in ("900", "0x10", "(1024)"):
        assert not sdk_facts._FLOAT_LIT.fullmatch(text), text
