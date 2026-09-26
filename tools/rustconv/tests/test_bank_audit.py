"""bank_audit resolves paid output against the tree — and must not resolve it wrongly.

Every case here is a false positive or false negative the audit actually produced while
it was being written, on the real 444-dir bank. The direction that matters is the
optimistic one: an audit that reports abandoned paid work as shipped is worse than no
audit, because it retires the question.

  - an `extern` declaration counted as a definition (a prelude promising `Foo` exists
    would have "proved" Foo shipped)
  - `debug` matched `debug_block_stamp`, `error` matched `WDML_Initialize_error` — short
    generic variadic names against a loose containment test
  - a multi-function body counted as integrated when only ONE of its functions shipped
"""

from pathlib import Path

import pytest

from rustconv.bank_audit import (
    BankedFn,
    DirReport,
    Resolution,
    audit,
    bank_module,
    banked_fns,
    declared_names,
    format_report,
    load_allowlist,
    macro_arg_names,
    member_body,
    redundancies,
    resolve,
    tchar_variants,
    truncated_fns,
    varargs_twins,
)


def _write(tmp_path: Path, name: str, text: str) -> Path:
    path = tmp_path / name
    path.parent.mkdir(parents=True, exist_ok=True)
    path.write_text(text, encoding="utf-8")
    return path


# --------------------------------------------------------------------------- definitions


def test_extern_declaration_is_not_a_definition(tmp_path):
    """The bug that made the tree index lie: a prelude's promise is not a definition."""
    src = _write(
        tmp_path,
        "prelude.rs",
        'extern "C" {\n    pub fn SetLastError(code: u32);\n    pub fn GetLastError() -> u32;\n}\n',
    )
    assert declared_names(src) == set()


def test_definition_is_found_alongside_declarations(tmp_path):
    src = _write(
        tmp_path,
        "m.rs",
        'extern "C" {\n    pub fn Declared(x: u32);\n}\n'
        "pub unsafe fn Defined(x: u32) -> u32 { x + 1 }\n",
    )
    assert declared_names(src) == {"Defined"}


def test_semicolon_inside_array_return_type_is_not_a_declaration(tmp_path):
    """`-> [u8; 4]` must not read as the `;` that ends a declaration."""
    src = _write(tmp_path, "m.rs", "fn digits() -> [u8; 4] { [0; 4] }\n")
    assert declared_names(src) == {"digits"}


def test_commented_out_c_source_is_not_indexed(tmp_path):
    """Every translated file trails the ORIGINAL C SOURCE as comments."""
    src = _write(
        tmp_path,
        "m.rs",
        "pub fn Real() {}\n// fn Ghost() {}\n/* fn AlsoGhost() {} */\n",
    )
    assert declared_names(src) == {"Real"}


# --------------------------------------------------------------------------- layouts


def test_manifest_layout_skips_bookkeeping_keys(tmp_path):
    d = tmp_path / "mod__file"
    _write(
        tmp_path,
        "mod__file/manifest.json",
        '{"Foo": {"file": "Foo.rs"}, "__preamble__": {"file": "__preamble__.rs"}}',
    )
    assert [f.name for f in banked_fns(d)] == ["Foo"]


def test_legacy_layout_strips_sequence_prefix_and_preamble(tmp_path):
    d = tmp_path / "20260101-000000-x.c"
    _write(tmp_path, "20260101-000000-x.c/00_preamble.rs", "")
    _write(tmp_path, "20260101-000000-x.c/01_Alpha.rs", "")
    _write(tmp_path, "20260101-000000-x.c/02_Beta.rs", "")
    assert sorted(f.name for f in banked_fns(d)) == ["Alpha", "Beta"]


# --------------------------------------------------------------------------- varargs twins


@pytest.mark.parametrize(
    "name,tree,expected",
    [
        ("FatalError", {"FatalError_v"}, "FatalError_v"),
        ("FormatMessageBox", {"netid_FormatMessageBox_body"}, "netid_FormatMessageBox_body"),
        ("Foo", {"Foo_impl"}, "Foo_impl"),
    ],
)
def test_varargs_twin_conventions(name, tree, expected):
    assert varargs_twins(name, tree) == [expected]


@pytest.mark.parametrize(
    "name,tree",
    [
        # the two real cross-crate mismatches a containment test produced
        ("debug", {"debug_block_stamp"}),
        ("error", {"WDML_Initialize_error"}),
        # decoration must be a known twin suffix, not any trailing word
        ("Foo", {"Foo_helper"}),
        # and a bare prefix is not a twin either
        ("Foo", {"module_Foo"}),
    ],
)
def test_varargs_twin_rejects_merely_similar_names(name, tree):
    assert varargs_twins(name, tree) == []


def test_varargs_twin_prefers_undecorated_over_module_prefixed():
    tree = {"Foo_v", "other_Foo_body"}
    assert varargs_twins("Foo", tree)[0] == "Foo_v"


def test_varargs_rule_only_applies_to_variadic_bodies(tmp_path):
    """A non-variadic `Foo` must not be satisfied by an unrelated `Foo_v`."""
    path = _write(tmp_path, "Foo.rs", "unsafe fn Foo(x: u32) -> u32 { x }\n")
    res = resolve(BankedFn("Foo", path), {"Foo_v"}, {})
    assert not res.integrated


def test_variadic_body_resolves_to_twin(tmp_path):
    path = _write(
        tmp_path,
        "Log.rs",
        "unsafe fn Log(fmt: *const u8, mut args: ...) -> i32 { va_start(args, fmt); 0 }\n",
    )
    res = resolve(BankedFn("Log", path), {"Log_v"}, {})
    assert (res.found_as, res.via) == ("Log_v", "varargs_twin")


# --------------------------------------------------------------------------- body rule


def test_body_rule_requires_every_function_to_ship(tmp_path):
    """A 3-function body with 1 shipped is partially-lost paid work, not integrated."""
    path = _write(
        tmp_path,
        "MACRO.rs",
        "unsafe fn Alpha() {}\nunsafe fn Beta() {}\nunsafe fn Gamma() {}\n",
    )
    partial = resolve(BankedFn("MACRO", path), {"Alpha"}, {})
    assert not partial.integrated

    whole = resolve(BankedFn("MACRO", path), {"Alpha", "Beta", "Gamma"}, {})
    assert whole.via == "declared_in_body"


# --------------------------------------------------------------------------- misc rules


def test_tchar_variants():
    assert tchar_variants("_tmain") == ["wmain", "main"]
    assert tchar_variants("_tWinMain") == ["wWinMain", "WinMain"]
    assert tchar_variants("plain") == []


def test_allowlist_resolves_only_with_an_entry(tmp_path):
    path = _write(tmp_path, "Foo.rs", "unsafe fn Foo() {}\n")
    assert not resolve(BankedFn("Foo", path), set(), {}).integrated
    assert resolve(BankedFn("Foo", path), set(), {"Foo": "reviewed"}).via == "allowlist"


def test_member_body_is_exact_not_containment(tmp_path):
    """`Class_Method` ships as `Class_Method_body` — but only that exact spelling.

    The near miss is real: `CDefView_Refresh` and `CDefView_RefreshGhostedState_body` are
    both in shell32, and a prefix test would report the first as shipped by the second.
    """
    tree = {"CDefView_Refresh_body", "CDefView_RefreshGhostedState_body"}
    assert member_body("CDefView_Refresh", tree) == "CDefView_Refresh_body"
    assert member_body("CDefView_RefreshGhosted", tree) is None
    assert member_body("CDefView_Refresh", set()) is None

    # The shape that actually needs this rule: a `.partial` has no complete body, so
    # `declared_in_body` cannot answer and the label is all there is to match on.
    path = _write(tmp_path, "01_CDefView_Refresh.partial", "unsafe fn CDefView_Refresh(")
    res = resolve(BankedFn("CDefView_Refresh", path, truncated=True), tree, {})
    assert (res.via, res.found_as) == ("member_body", "CDefView_Refresh_body")


def test_allowlist_groups_carry_a_reason_and_bare_ones_are_dropped(tmp_path):
    """A group is a shared reason, never a way to allow a name without one."""
    path = tmp_path / "allow.toml"
    path.write_text(
        '[groups.ok]\nreason = "superseded by a rewrite"\nfunctions = ["A", "B"]\n'
        '[groups.no_reason]\nfunctions = ["C"]\n'
        '[groups.no_functions]\nreason = "orphan"\n'
        '[allow]\nD = "its own reason"\n',
        encoding="utf-8",
    )
    allow = load_allowlist(path)
    assert allow == {
        "A": "superseded by a rewrite",
        "B": "superseded by a rewrite",
        "D": "its own reason",
    }


def test_hash_keyed_memo_resolves_via_its_body_and_stays_labelled_allowlist(tmp_path):
    """The memo cache is keyed by content hash, so only its body can identify it.

    And the verdict must read `allowlist`, not `declared_in_body`: a reviewed exception
    that reports as a mechanical match is one nobody re-reads.
    """
    body = "unsafe fn CFolderOptions_ctor_body() {}\n"
    path = _write(tmp_path, "translation-memory-exact-v1/a1b2.rs", body)
    assert not resolve(BankedFn("a1b2", path), set(), {}).integrated
    res = resolve(BankedFn("a1b2", path), set(), {"CFolderOptions_ctor": "superseded"})
    assert res.via == "allowlist"


def test_memo_body_with_one_unaccounted_name_is_still_reported(tmp_path):
    """Every name must be accounted for — the optimistic direction is the dangerous one."""
    body = "unsafe fn Allowed_body() {}\nunsafe fn Missing_body() {}\n"
    path = _write(tmp_path, "translation-memory-exact-v1/c3d4.rs", body)
    assert not resolve(BankedFn("c3d4", path), set(), {"Allowed": "superseded"}).integrated


def test_allowlisted_dir_is_not_safe_to_archive(tmp_path):
    """ "Correctly absent" is not "shipping", and only the second is safe to archive.

    `member_body` and the group allowlist widen what resolves as integrated, which feeds
    `redundancies`' safe-to-archive list directly. A dir whose every body is an allowlisted
    exception is the record of a decision — shell32's `CEnumIDListBase` run archive holds
    seven bodies that must NEVER be defined in Rust — so it must not be offered.
    """
    responses = tmp_path / "responses"
    tree = tmp_path / "tree"
    _write(tmp_path, "tree/lib.rs", "pub unsafe fn Shipping() {}\n")
    for d in ("20260718-205701-file.cpp", "mod__file__factseal_v1"):
        _write(tmp_path, f"responses/{d}/01_Superseded.rs", "unsafe fn Superseded() {}\n")
    reports = audit(responses, tree, allow={"Superseded": "superseded by a rewrite"})
    dupes, safe = redundancies(reports)
    assert [d.name for d in dupes] == ["Superseded"]
    assert safe == []


# --------------------------------------------------------------------------- truncated


def test_partial_files_are_scanned_and_marked_truncated(tmp_path):
    """A billed-but-cut-off call is paid work too, and needs a different remedy."""
    d = tmp_path / "mod__file"
    _write(tmp_path, "mod__file/06_Interrupted.partial", "unsafe fn Interrupted(")
    (fn,) = truncated_fns(d)
    assert (fn.name, fn.truncated) == ("Interrupted", True)


def test_truncated_body_absent_from_tree_is_reported(tmp_path):
    responses = tmp_path / "responses"
    tree = tmp_path / "tree"
    _write(tmp_path, "responses/mod__file/06_Interrupted.partial", "unsafe fn Interrupted(")
    _write(tmp_path, "tree/lib.rs", "pub unsafe fn Unrelated() {}\n")

    (report,) = audit(responses, tree, allow={})
    (missing,) = report.absent
    assert missing.fn.truncated
    assert "RE-TRANSLATE" in format_report([report])


def test_completed_body_supersedes_its_leftover_partial(tmp_path):
    """A retried call leaves the `.partial` behind; the function is still one function."""
    responses = tmp_path / "responses"
    tree = tmp_path / "tree"
    _write(tmp_path, "responses/mod__file/manifest.json", '{"Foo": {"file": "Foo.rs"}}')
    _write(tmp_path, "responses/mod__file/Foo.rs", "unsafe fn Foo() {}\n")
    _write(tmp_path, "responses/mod__file/06_Foo.partial", "unsafe fn Foo(")
    _write(tmp_path, "tree/lib.rs", "pub unsafe fn Foo() {}\n")

    (report,) = audit(responses, tree, allow={})
    assert report.total == 1


def test_truncated_body_that_later_landed_is_not_reported(tmp_path):
    """Every `.partial` in the real bank belongs to a call that was retried."""
    responses = tmp_path / "responses"
    tree = tmp_path / "tree"
    _write(tmp_path, "responses/mod__file/06_Retried.partial", "unsafe fn Retried(")
    _write(tmp_path, "tree/lib.rs", "pub unsafe fn Retried() {}\n")

    (report,) = audit(responses, tree, allow={})
    assert report.absent == []


# --------------------------------------------------------------------------- end to end


def test_audit_reports_a_genuine_absence(tmp_path):
    responses = tmp_path / "responses"
    tree = tmp_path / "tree"
    _write(tmp_path, "responses/mod__file/manifest.json", '{"Shipped": {}, "Dropped": {}}')
    _write(tmp_path, "responses/mod__file/Shipped.rs", "unsafe fn Shipped() {}\n")
    _write(tmp_path, "responses/mod__file/Dropped.rs", "unsafe fn Dropped() {}\n")
    _write(tmp_path, "tree/lib.rs", "pub unsafe fn Shipped() {}\n")

    (report,) = audit(responses, tree, allow={})
    assert report.status == "PARTIAL"
    assert [r.fn.name for r in report.absent] == ["Dropped"]


# --- the two rules added after the bank read 60 unintegrated and 0 of them were real ---


def test_va_list_twin_puts_the_v_after_leading_underscores():
    """C stdio's convention: fprintf's va_list twin is vfprintf, _fwprintf_p_l's is
    _vfwprintf_p_l. Putting the `v` at the front instead matches nothing."""
    tree = {"vfprintf", "_vfwprintf_p_l", "_sopen_dispatch"}
    assert varargs_twins("fprintf", tree) == ["vfprintf"]
    assert varargs_twins("_fwprintf_p_l", tree) == ["_vfwprintf_p_l"]
    # `_dispatch` is msvcrt's shim convention for the non-printf variadics.
    assert varargs_twins("_sopen", tree) == ["_sopen_dispatch"]
    # And a name with no twin present still resolves to nothing.
    assert varargs_twins("fputs", tree) == []


def test_macro_arg_index_finds_generated_symbols(tmp_path):
    """msvcrt ships 24 ctype predicates as `isc!(_isalnum_l, isalnum, ...)` in lib.rs.
    There is no `fn _isalnum_l` anywhere, and every one read as unintegrated paid work."""
    f = tmp_path / "lib.rs"
    f.write_text(
        "isc!(_isalnum_l, isalnum, (sys::_ALPHA | sys::_DIGIT) as i32);\n"
        "isc!(\n    _isprint_l,\n    isprint,\n    sys::_BLANK as i32,\n);\n"
    )
    names = macro_arg_names(f)
    assert {"_isalnum_l", "isalnum", "_isprint_l", "isprint"} <= names


def test_macro_arg_index_refuses_anything_but_a_bare_identifier(tmp_path):
    """The dangerous direction is a false name in the TREE index — it reports paid work
    as shipped when it is not. So only a whole-argument identifier counts."""
    f = tmp_path / "lib.rs"
    f.write_text(
        'println!("calling _isalnum_l now");\n'  # inside a string literal
        "other!(foo(_bar), sys::_BAZ, 1 + qux, a as i32);\n"  # expressions, paths, casts
        "// isc!(_commented_out, x);\n"  # inside a comment
    )
    names = macro_arg_names(f)
    for bad in ("_isalnum_l", "_bar", "_BAZ", "qux", "_commented_out", "a"):
        assert bad not in names, bad


def test_macro_generated_is_a_last_resort_and_says_so(tmp_path):
    banked = tmp_path / "m.rs"
    banked.write_text("pub unsafe fn _isalnum_l() {}\n")
    fn = BankedFn("_isalnum_l", banked)
    # Not in the fn index...
    assert not resolve(fn, set(), {}, set()).integrated
    # ...but resolvable through the macro-argument index, and labelled as such.
    r = resolve(fn, set(), {}, {"_isalnum_l"})
    assert r.integrated and r.via == "macro_generated"


# --- redundancy: one purchase written twice is NOT two purchases ------------
#
# The first draft of `redundancies` called every cross-dir duplicate "bought more than
# once" and was wrong about all 235 of msvcrt's: `translate` writes each body to BOTH a
# timestamped RunArchive and the namespaced ResponseCache, so a single run leaves two
# byte-identical copies. Hashing the bodies is what separates that from a real re-buy.


def _rep(name: str, fns: dict[str, Path], integrated: bool = True) -> DirReport:
    res = [
        Resolution(BankedFn(fn, path), fn if integrated else None, "direct")
        for fn, path in fns.items()
    ]
    return DirReport(name, len(res), res)


def _body(tmp_path: Path, rel: str, text: str) -> Path:
    p = tmp_path / rel
    p.parent.mkdir(parents=True, exist_ok=True)
    p.write_text(text, encoding="utf-8")
    return p


def test_identical_copies_are_one_purchase_not_two(tmp_path):
    a = _body(tmp_path, "20260919-022222-file.c/01_alpha.rs", "fn alpha() {}")
    b = _body(tmp_path, "msvcrt__file__factseal_v1/alpha.rs", "fn alpha() {}")
    dupes, _ = redundancies(
        [
            _rep("20260919-022222-file.c", {"alpha": a}),
            _rep("msvcrt__file__factseal_v1", {"alpha": b}),
        ]
    )
    assert [d.identical for d in dupes] == [True]


def test_divergent_copies_are_a_real_rebuy(tmp_path):
    a = _body(tmp_path, "20260801-010101-file.c/01_alpha.rs", "fn alpha() { /* v1 */ }")
    b = _body(tmp_path, "20260919-022222-file.c/01_alpha.rs", "fn alpha() { /* v2 */ }")
    dupes, _ = redundancies(
        [_rep("20260801-010101-file.c", {"alpha": a}), _rep("20260919-022222-file.c", {"alpha": b})]
    )
    assert [d.identical for d in dupes] == [False]


def test_the_replay_CACHE_dir_is_never_offered_for_removal(tmp_path):
    """Deleting the namespaced dir makes the next run re-buy the file. It is the cache,
    not a spare copy, and no amount of mirroring makes it disposable."""
    a = _body(tmp_path, "20260919-022222-file.c/01_alpha.rs", "fn alpha() {}")
    b = _body(tmp_path, "msvcrt__file__factseal_v1/alpha.rs", "fn alpha() {}")
    _, safe = redundancies(
        [
            _rep("20260919-022222-file.c", {"alpha": a}),
            _rep("msvcrt__file__factseal_v1", {"alpha": b}),
        ]
    )
    assert safe == ["20260919-022222-file.c"]


def test_an_archive_with_one_unmirrored_member_is_not_safe(tmp_path):
    a = _body(tmp_path, "20260919-022222-file.c/01_alpha.rs", "fn alpha() {}")
    only = _body(tmp_path, "20260919-022222-file.c/02_beta.rs", "fn beta() {}")
    b = _body(tmp_path, "msvcrt__file__factseal_v1/alpha.rs", "fn alpha() {}")
    _, safe = redundancies(
        [
            _rep("20260919-022222-file.c", {"alpha": a, "beta": only}),
            _rep("msvcrt__file__factseal_v1", {"alpha": b}),
        ]
    )
    assert safe == []


def test_an_archive_of_UNINTEGRATED_work_is_not_safe(tmp_path):
    a = _body(tmp_path, "20260919-022222-file.c/01_alpha.rs", "fn alpha() {}")
    b = _body(tmp_path, "msvcrt__file__factseal_v1/alpha.rs", "fn alpha() {}")
    _, safe = redundancies(
        [
            _rep("20260919-022222-file.c", {"alpha": a}, integrated=False),
            _rep("msvcrt__file__factseal_v1", {"alpha": b}, integrated=False),
        ]
    )
    assert safe == []


def test_bank_module_normalizes_both_layouts():
    """Both spellings of one module must agree, or duplicate detection splits or merges
    the wrong things. `parts[0]` buckets by GROUP and collapsed 37 shell32 modules into
    one; keeping the `.cpp` makes `utils.cpp` and `utils` look like different modules."""
    assert bank_module("20260919-022222-file.c") == "file"
    assert bank_module("20260913-012131-utils.cpp") == "utils"
    assert bank_module("msvcrt__file__factseal_v1__callseal_v1") == "file"
    assert bank_module("shell32__CDefView__factseal_v1__callseal_v1") == "CDefView"
    assert bank_module("plain-name") == "plain-name"
    assert bank_module("20260919-022222-file.c-run-012345abcdef") == "file"


def test_a_retry_is_not_a_rebuy(tmp_path):
    """A `.partial` among the copies means the divergence is a retry after a reject —
    legitimate, and the second purchase is the one you keep. Conflating the two
    overstated shell32's re-buys roughly fourfold."""
    a = _body(tmp_path, "20260718-111034-CFSFolder.cpp/18_alpha.partial", "fn alpha() { /*cut*/")
    b = _body(tmp_path, "20260718-111247-CFSFolder.cpp/18_alpha.rs", "fn alpha() {}")
    dupes, _ = redundancies(
        [
            _rep("20260718-111034-CFSFolder.cpp", {"alpha": a}),
            _rep("20260718-111247-CFSFolder.cpp", {"alpha": b}),
        ]
    )
    assert [(d.identical, d.retry) for d in dupes] == [(False, True)]


def test_same_name_in_two_modules_is_two_functions_not_one_rebuy(tmp_path):
    """shell32 has an `IsRegItem` in three folder classes. They share a name and nothing
    else."""
    a = _body(tmp_path, "20260716-234707-CDrivesFolder.cpp/01_IsRegItem.rs", "fn a() {}")
    b = _body(tmp_path, "20260718-113355-CDesktopFolder.cpp/01_IsRegItem.rs", "fn b() {}")
    dupes, _ = redundancies(
        [
            _rep("20260716-234707-CDrivesFolder.cpp", {"IsRegItem": a}),
            _rep("20260718-113355-CDesktopFolder.cpp", {"IsRegItem": b}),
        ]
    )
    assert dupes == []


def test_a_collision_does_not_hide_a_rebuy_inside_one_module(tmp_path):
    """The 60-vs-65 case. `IsRegItem` collides across modules AND CDrivesFolder bought
    its own twice. Discarding the whole name because it collides loses the real re-buy —
    which is what my first ad-hoc pass did, undercounting by exactly these five."""
    a1 = _body(tmp_path, "20260716-234707-CDrivesFolder.cpp/01_IsRegItem.rs", "fn v1() {}")
    a2 = _body(tmp_path, "20260803-231933-CDrivesFolder.cpp/01_IsRegItem.rs", "fn v2() {}")
    b = _body(tmp_path, "20260718-113355-CDesktopFolder.cpp/01_IsRegItem.rs", "fn other() {}")
    dupes, _ = redundancies(
        [
            _rep("20260716-234707-CDrivesFolder.cpp", {"IsRegItem": a1}),
            _rep("20260803-231933-CDrivesFolder.cpp", {"IsRegItem": a2}),
            _rep("20260718-113355-CDesktopFolder.cpp", {"IsRegItem": b}),
        ]
    )
    assert [(d.module, d.identical, d.retry) for d in dupes] == [("CDrivesFolder", False, False)]
