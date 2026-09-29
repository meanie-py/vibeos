//! `startup/argv_parsing.cpp`: `_configure_narrow_argv`, `_configure_wide_argv`,
//! `__acrt_allocate_buffer_for_argv`, and the `parse_command_line` template.
//! See the parent module's doc for the parse rules and the block layout.

use super::{calloc, free, ArgChar, _errno, _invalid_parameter_noinfo, _ismbblead, EINVAL, ENOMEM, MAX_PATH};
use crate::argv_data::{__argc, __argv, __wargv, _acmdln, _pgmptr, _wcmdln, _wpgmptr};
use core::ffi::{c_char, c_int, c_void};
use core::ptr::{addr_of_mut, null_mut};

/// `_crt_argv_mode` (corecrt_startup.h).
const CRT_ARGV_NO_ARGUMENTS: c_int = 0;
const CRT_ARGV_UNEXPANDED_ARGUMENTS: c_int = 1;
const CRT_ARGV_EXPANDED_ARGUMENTS: c_int = 2;

extern "C" {
    /// corecrt_internal.h -- the narrow `traits::get_module_file_name` (cdecl).
    fn __acrt_GetModuleFileNameA(module: *mut c_void, filename: *mut c_char, size: u32) -> u32;
}

extern "system" {
    /// The wide `traits::get_module_file_name`.
    fn GetModuleFileNameW(module: *mut c_void, filename: *mut u16, size: u32) -> u32;
}

/// The per-character-type half of `common_configure_argv` and `parse_command_line`.
pub(crate) trait ParseChar: ArgChar {
    /// `should_copy_another_character`: a DBCS lead byte (narrow only).
    unsafe fn should_copy_another_character(self) -> bool;
    /// `do_locale_initialization`
    unsafe fn do_locale_initialization();
    /// `common_configure_argv`'s `static Character program_name[MAX_PATH + 1]`.
    unsafe fn program_name() -> *mut Self;
    /// `traits::get_module_file_name(nullptr, buffer, size)`
    unsafe fn get_module_file_name(buffer: *mut Self, size: u32) -> u32;
    /// `traits::set_program_name` (`_set_pgmptr`/`_set_wpgmptr`)
    unsafe fn set_program_name(name: *mut Self);
    /// `get_command_line`
    unsafe fn get_command_line() -> *mut Self;
    /// `get_argv(Character()) = argv`
    unsafe fn set_argv(argv: *mut *mut Self);
    /// `expand_argv_wildcards`
    unsafe fn expand_argv_wildcards(argv: *mut *mut Self, expanded: *mut *mut *mut Self) -> c_int;
}

static mut NARROW_PROGRAM_NAME: [u8; MAX_PATH + 1] = [0; MAX_PATH + 1];
static mut WIDE_PROGRAM_NAME: [u16; MAX_PATH + 1] = [0; MAX_PATH + 1];

impl ParseChar for u8 {
    unsafe fn should_copy_another_character(self) -> bool {
        // The narrow template's parameter is plain (signed) `char`, promoted to
        // `unsigned int` by sign extension; `_ismbblead` truncates it back.
        _ismbblead(self as i8 as i32 as u32) != 0
    }
    unsafe fn do_locale_initialization() {
        crate::locale_abi::__acrt_initialize_multibyte();
    }
    unsafe fn program_name() -> *mut u8 {
        addr_of_mut!(NARROW_PROGRAM_NAME).cast()
    }
    unsafe fn get_module_file_name(buffer: *mut u8, size: u32) -> u32 {
        __acrt_GetModuleFileNameA(null_mut(), buffer.cast(), size)
    }
    unsafe fn set_program_name(name: *mut u8) {
        _pgmptr = name;
    }
    unsafe fn get_command_line() -> *mut u8 {
        _acmdln
    }
    unsafe fn set_argv(argv: *mut *mut u8) {
        __argv = argv;
    }
    unsafe fn expand_argv_wildcards(argv: *mut *mut u8, expanded: *mut *mut *mut u8) -> c_int {
        super::wildcards::__acrt_expand_narrow_argv_wildcards(argv, expanded)
    }
}

impl ParseChar for u16 {
    unsafe fn should_copy_another_character(self) -> bool {
        // "This is OK for UTF-16 as a quote is never part of a surrogate pair."
        false
    }
    unsafe fn do_locale_initialization() {}
    unsafe fn program_name() -> *mut u16 {
        addr_of_mut!(WIDE_PROGRAM_NAME).cast()
    }
    unsafe fn get_module_file_name(buffer: *mut u16, size: u32) -> u32 {
        GetModuleFileNameW(null_mut(), buffer, size)
    }
    unsafe fn set_program_name(name: *mut u16) {
        _wpgmptr = name;
    }
    unsafe fn get_command_line() -> *mut u16 {
        _wcmdln
    }
    unsafe fn set_argv(argv: *mut *mut u16) {
        __wargv = argv;
    }
    unsafe fn expand_argv_wildcards(argv: *mut *mut u16, expanded: *mut *mut *mut u16) -> c_int {
        super::wildcards::__acrt_expand_wide_argv_wildcards(argv, expanded)
    }
}

/// `should_copy_another_character(c)`, but never across the terminating NUL.
///
/// DIVERGENCE: the C copies the byte after a lead byte unconditionally and
/// steps past it, so a lead byte right before the NUL makes it copy the NUL as
/// a "trail byte" and keep scanning PAST the end of the command line. Here a NUL
/// is never a trail byte. On a line whose memory past the NUL is another NUL
/// the resulting argv strings are identical; only the over-read is gone.
#[inline(always)]
unsafe fn has_trail<C: ParseChar>(c: C, next: *const C) -> bool {
    c.should_copy_another_character() && !(*next).is_nul()
}

/// `parse_command_line<Character>`. With `argv`/`args` NULL it only counts:
/// `argument_count` includes the final NULL slot, `character_count` every
/// character stored including each argument's terminator.
unsafe fn parse_command_line<C: ParseChar>(
    cmdstart: *mut C,
    mut argv: *mut *mut C,
    mut args: *mut C,
    argument_count: &mut usize,
    character_count: &mut usize,
) {
    *character_count = 0;
    *argument_count = 1; // We'll have at least the program name

    let mut c: C;

    // first scan the program name, copy it, and count the bytes
    let mut p = cmdstart;
    if !argv.is_null() {
        *argv = args;
        argv = argv.add(1);
    }

    // A quoted program name is handled here: whatever lies between the leading
    // double-quote and the next one, or a terminal NUL, is accepted. The quote
    // characters are not copied and do not count.
    let mut in_quotes = false;
    loop {
        if (*p).is(b'"') {
            in_quotes = !in_quotes;
            c = *p;
            p = p.add(1);
            // `continue` in the C's do-while: straight to the condition, with
            // c == '"', which always continues.
        } else {
            *character_count += 1;
            if !args.is_null() {
                *args = *p;
                args = args.add(1);
            }

            c = *p;
            p = p.add(1);

            if has_trail(c, p) {
                *character_count += 1;
                if !args.is_null() {
                    *args = *p; // Copy 2nd byte too
                    args = args.add(1);
                }
                p = p.add(1); // skip over trail byte
            }
        }
        if !(!c.is_nul() && (in_quotes || (!c.is(b' ') && !c.is(b'\t')))) {
            break;
        }
    }

    if c.is_nul() {
        p = p.sub(1);
    } else if !args.is_null() {
        *args.sub(1) = C::NUL;
    }

    in_quotes = false;

    // Loop on each argument
    loop {
        if !(*p).is_nul() {
            while (*p).is(b' ') || (*p).is(b'\t') {
                p = p.add(1);
            }
        }

        if (*p).is_nul() {
            break; // End of arguments
        }

        // Scan an argument:
        if !argv.is_null() {
            *argv = args;
            argv = argv.add(1);
        }

        *argument_count += 1;

        // Loop through scanning one argument:
        loop {
            let mut copy_character = true;

            // 2N     backslashes + " ==> N backslashes and begin/end quote
            // 2N + 1 backslashes + " ==> N backslashes + literal "
            // N      backslashes     ==> N backslashes
            let mut numslash: u32 = 0;

            while (*p).is(b'\\') {
                p = p.add(1);
                numslash += 1;
            }

            if (*p).is(b'"') {
                // if 2N backslashes before, start/end quote, otherwise copy
                // literally:
                if numslash % 2 == 0 {
                    if in_quotes && (*p.add(1)).is(b'"') {
                        // The `""`-inside-quotes quirk: one literal quote,
                        // and the quoted run CONTINUES.
                        p = p.add(1);
                    } else {
                        // Skip first quote char and copy second:
                        copy_character = false; // Don't copy quote
                        in_quotes = !in_quotes;
                    }
                }

                numslash /= 2;
            }

            // Copy slashes:
            while numslash > 0 {
                numslash -= 1;
                if !args.is_null() {
                    *args = C::ascii(b'\\');
                    args = args.add(1);
                }
                *character_count += 1;
            }

            // If at end of arg, break loop:
            if (*p).is_nul() || (!in_quotes && ((*p).is(b' ') || (*p).is(b'\t'))) {
                break;
            }

            // Copy character into argument:
            if copy_character {
                if !args.is_null() {
                    *args = *p;
                    args = args.add(1);
                }

                if has_trail(*p, p.add(1)) {
                    p = p.add(1);
                    *character_count += 1;

                    if !args.is_null() {
                        *args = *p;
                        args = args.add(1);
                    }
                }

                *character_count += 1;
            }

            p = p.add(1);
        }

        // Null-terminate the argument:
        if !args.is_null() {
            *args = C::NUL;
            args = args.add(1);
        }

        *character_count += 1;
    }

    // We put one last argument in -- a null pointer:
    if !argv.is_null() {
        *argv = null_mut();
    }

    *argument_count += 1;
}

/// `__acrt_allocate_buffer_for_argv`: one zeroed block for `argument_count`
/// pointers followed by `character_count` characters, or NULL on overflow or
/// allocation failure.
#[no_mangle]
pub unsafe extern "C" fn __acrt_allocate_buffer_for_argv(
    argument_count: usize,
    character_count: usize,
    character_size: usize,
) -> *mut u8 {
    if argument_count >= usize::MAX / core::mem::size_of::<*mut c_void>() {
        return null_mut();
    }

    // DIVERGENCE: the C divides by `character_size` unchecked (a zero is a
    // divide fault); both callers pass sizeof(char)/sizeof(wchar_t), and a zero
    // here refuses instead of faulting.
    match usize::MAX.checked_div(character_size) {
        Some(limit) if character_count < limit => {}
        _ => return null_mut(),
    }

    let argument_array_size = argument_count * core::mem::size_of::<*mut c_void>();
    let character_array_size = character_count * character_size;

    if usize::MAX - argument_array_size <= character_array_size {
        return null_mut();
    }

    let total_size = argument_array_size + character_array_size;
    // `_calloc_crt_t(unsigned char, total_size)`
    calloc(total_size, core::mem::size_of::<u8>()) as *mut u8
}

/// Frees its block on drop unless detached -- `__crt_unique_heap_ptr`.
struct HeapPtr(*mut c_void);
impl HeapPtr {
    fn detach(&mut self) -> *mut c_void {
        core::mem::replace(&mut self.0, null_mut())
    }
}
impl Drop for HeapPtr {
    fn drop(&mut self) {
        unsafe { free(self.0) };
    }
}

/// `common_configure_argv<Character>`
unsafe fn common_configure_argv<C: ParseChar>(mode: c_int) -> c_int {
    if mode == CRT_ARGV_NO_ARGUMENTS {
        return 0;
    }

    // _VALIDATE_RETURN_ERRCODE(mode == expanded || mode == unexpanded, EINVAL)
    if !(mode == CRT_ARGV_EXPANDED_ARGUMENTS || mode == CRT_ARGV_UNEXPANDED_ARGUMENTS) {
        *_errno() = EINVAL;
        _invalid_parameter_noinfo();
        return EINVAL;
    }

    C::do_locale_initialization();

    let program_name = C::program_name();
    C::get_module_file_name(program_name, MAX_PATH as u32);
    C::set_program_name(program_name);

    // If there's no command line at all, parse the program name instead, so
    // that argv[0] is initialized with it.
    let raw_command_line = C::get_command_line();
    let command_line = if raw_command_line.is_null() || (*raw_command_line).is_nul() {
        program_name
    } else {
        raw_command_line
    };

    let mut argument_count: usize = 0;
    let mut character_count: usize = 0;
    parse_command_line::<C>(command_line, null_mut(), null_mut(), &mut argument_count, &mut character_count);

    let mut buffer = HeapPtr(
        __acrt_allocate_buffer_for_argv(argument_count, character_count, core::mem::size_of::<C>()).cast(),
    );

    // _VALIDATE_RETURN_ERRCODE_NOEXC(buffer, ENOMEM): errno only, no handler.
    if buffer.0.is_null() {
        *_errno() = ENOMEM;
        return ENOMEM;
    }

    let first_argument = buffer.0 as *mut *mut C;
    let first_string = (buffer.0 as *mut u8).add(argument_count * core::mem::size_of::<*mut C>()) as *mut C;

    parse_command_line::<C>(command_line, first_argument, first_string, &mut argument_count, &mut character_count);

    // If we are not expanding wildcards, then we are done...
    if mode == CRT_ARGV_UNEXPANDED_ARGUMENTS {
        __argc = (argument_count - 1) as c_int;
        C::set_argv(buffer.detach() as *mut *mut C);
        return 0;
    }

    // ... otherwise, we try to do the wildcard expansion:
    let mut expanded: *mut *mut C = null_mut();
    let argv_expansion_status = C::expand_argv_wildcards(first_argument, &mut expanded);
    let mut expanded_argv = HeapPtr(expanded.cast());
    if argv_expansion_status != 0 {
        return argv_expansion_status;
    }

    let mut n: usize = 0;
    let mut it = expanded_argv.0 as *mut *mut C;
    while !(*it).is_null() {
        it = it.add(1);
        n += 1;
    }
    __argc = n as c_int;

    C::set_argv(expanded_argv.detach() as *mut *mut C);
    0
}

#[no_mangle]
pub unsafe extern "C" fn _configure_narrow_argv(mode: c_int) -> c_int {
    common_configure_argv::<u8>(mode)
}

#[no_mangle]
pub unsafe extern "C" fn _configure_wide_argv(mode: c_int) -> c_int {
    common_configure_argv::<u16>(mode)
}
