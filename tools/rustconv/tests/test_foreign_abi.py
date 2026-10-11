from rustconv.foreign_abi import split_variadic_system_blocks


def test_varargs_does_not_turn_rtlassert_into_cdecl():
    code = """#[cfg(target_arch = "x86")]
#[link(name = "ntoskrnl")]
unsafe extern "system" {
    fn RtlAssert(a: *mut u8, b: *mut u8, line: u32, msg: *mut i8);
    #[link_name = "DbgPrint"]
    fn print(fmt: *const i8, ...);
}"""
    fixed = split_variadic_system_blocks(code)
    system, cdecl = fixed.split('unsafe extern "C"')
    assert "fn RtlAssert" in system and "fn print" not in system
    assert "fn print" in cdecl and "fn RtlAssert" not in cdecl
    assert '#[link_name = "DbgPrint"]' in cdecl
    assert fixed.count('#[cfg(target_arch = "x86")]') == 2
    assert fixed.count('#[link(name = "ntoskrnl")]') == 2
    assert split_variadic_system_blocks(fixed) == fixed


def test_variadic_callback_does_not_change_containing_function_abi():
    code = 'extern "system" { fn Fixed(cb: unsafe extern "C" fn(i32, ...)); }'
    assert split_variadic_system_blocks(code) == code
