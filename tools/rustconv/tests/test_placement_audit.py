"""placement_audit: a permutation of sealed constants is caught, a faithful body is not."""

from rustconv.placement_audit import placement_findings


def test_swapped_assignments_are_a_ring():
    c = "void f(void) { a = 10; b = 20; }"
    rust = "unsafe fn f() { a = 20; b = 10; }"
    [finding] = placement_findings(c, rust)
    assert finding.ring == ("10", "20")
    assert "sits where the C has" in finding.message()


def test_faithful_body_is_silent():
    c = "void f(void) { a = 10; b = 20; }"
    assert placement_findings(c, "unsafe fn f() { a = 10; b = 20; }") == []


def test_integer_matched_by_value():
    c = "void f(void) { a = 0x10; b = 3; }"
    assert placement_findings(c, "fn f() { a = 16u32; b = 3_u32; }") == []
    assert placement_findings(c, "fn f() { a = 3; b = 0x10; }") != []


def test_error_code_moved_between_calls():
    # MSVCRT_exp's shape: the domain-error code went to the overflow call.
    c = """double f(double x) {
        if (x < 0) return math_error(_DOMAIN, "exp", x, 0, ret);
        if (x > MAXV) return math_error(_OVERFLOW, "exp", x, 0, ret);
        return y; }"""
    rust = """fn f(x: f64) -> f64 {
        if x < 0.0 { return math_error(_OVERFLOW, "exp", x, 0.0, ret); }
        if x > MAXV { return math_error(_DOMAIN, "exp", x, 0.0, ret); }
        y }"""
    # Both calls look alike apart from the guard, which `if` splits off in Rust,
    # so the statement contexts are identical: the check must stay SILENT rather
    # than guess.
    assert placement_findings(c, rust) == []


def test_three_way_ring():
    c = "void f(void) { a = 1; b = 2; c = 3; }"
    [finding] = placement_findings(c, "fn f() { a = 2; b = 3; c = 1; }")
    assert finding.ring == ("1", "2", "3")


def test_one_way_move_is_not_enough():
    # A single constant in an unexpected place (a hoisted temporary, a restructured
    # expression) is weak evidence; only a cycle is reported.
    c = "void f(void) { a = 10; b = 20; }"
    assert placement_findings(c, "fn f() { let t = 10; a = t; b = 20; }") == []


def test_symbols_and_shared_spellings():
    c = "void f(void) { x = FLAG_A; y = FLAG_B; z = FLAG_A; }"
    assert placement_findings(c, "fn f() { x = FLAG_A; y = FLAG_B; z = FLAG_A; }") == []
    [finding] = placement_findings(c, "fn f() { x = FLAG_B; y = FLAG_A; z = FLAG_A; }")
    assert finding.ring == ("FLAG_A", "FLAG_B")


def test_rust_syntax_constants_are_not_the_cs():
    # C truthiness and ++ have no constant; Rust's `!= 0` / `+= 1` must not be
    # mistaken for the C's own 0 and 1 trading places.
    c = "void f(void) { n = 0; while (*p) { if (x & F) n++; p++; } total = 1; }"
    rust = (
        "fn f() { let mut n = 0; while *p != 0 { if (x & F) != 0 { n += 1; } p = p.add(1); }"
        " total = 1; }"
    )
    assert placement_findings(c, rust) == []


def test_type_positions_are_not_values():
    c = "void f(void) { a = 3; b = 4; memset(&v, 0, sizeof(VIEW)); }"
    rust = "fn f() { let s: [u8; 4] = [0; 3]; a = 3; b = 4; write_bytes(&mut v, 0, size_of::<VIEW>()); }"
    assert placement_findings(c, rust) == []


def test_restored_integers_are_rust_with_the_c_value():
    from rustconv.fact_seal import rust_integer_spelling as r

    assert r("8192ULL") == "8192"
    assert r("0x10UL") == "0x10"
    assert r("0644") == "0o644"  # C octal; verbatim it would be decimal 644
    assert r("0") == "0" and r("10") == "10"
    assert r("0o17") == "0o17"


def test_crate_repair_c_suffix_and_null_fn_rules_are_registered():
    from rustconv import crate_repair as cr

    assert cr.RULE_C_INT_SUFFIX in cr.DIAGNOSTIC_RULES
    assert cr.RULE_NULL_FN_POINTER in cr.DIAGNOSTIC_RULES
    assert cr._NULL_EXPR.match("core::ptr::null_mut()")
    assert cr._NULL_EXPR.match("ptr::null::<u8>()")
    assert cr._NULL_EXPR.match("0 as *mut c_void")
    assert not cr._NULL_EXPR.match("some_ptr")
