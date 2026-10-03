//! `startup/argv_wildcards.cpp`: `__acrt_expand_narrow_argv_wildcards` and
//! `__acrt_expand_wide_argv_wildcards` -- `*`/`?` expansion of an argv block
//! into a new single-allocation argv block. See the parent module's doc for the
//! ordering ("sorting" by slot address) and the keep-the-pattern rules.

use super::parsing::__acrt_allocate_buffer_for_argv;
use super::{calloc, errcheck, free, ArgChar, _errno, _invalid_parameter_noinfo, _recalloc, EINVAL, ENOMEM};
use crate::findfile::{file_codepage, mbs_to_wcs, wcs_to_mbs, Buf, FileTime};
use core::ffi::{c_int, c_void};
use core::ptr::{null, null_mut};

type Handle = *mut c_void;
const INVALID_HANDLE_VALUE: isize = -1;
/// `FINDEX_INFO_LEVELS::FindExInfoStandard`
const FIND_EX_INFO_STANDARD: c_int = 0;
/// `FINDEX_SEARCH_OPS::FindExSearchNameMatch`
const FIND_EX_SEARCH_NAME_MATCH: c_int = 0;

/// `WIN32_FIND_DATAW` -- the same layout `crate::findfile` pins.
#[repr(C)]
struct Win32FindDataW {
    attributes: u32,
    creation: FileTime,
    last_access: FileTime,
    last_write: FileTime,
    size_high: u32,
    size_low: u32,
    reserved0: u32,
    reserved1: u32,
    file_name: [u16; 260],
    alternate_file_name: [u16; 14],
}
const _: () = assert!(core::mem::size_of::<Win32FindDataW>() == 592);
const _: () = assert!(core::mem::offset_of!(Win32FindDataW, file_name) == 44);

extern "system" {
    fn FindFirstFileExW(
        name: *const u16,
        info_level: c_int,
        data: *mut Win32FindDataW,
        search_op: c_int,
        filter: *mut c_void,
        flags: u32,
    ) -> Handle;
    fn FindNextFileW(handle: Handle, data: *mut Win32FindDataW) -> c_int;
    fn FindClose(handle: Handle) -> c_int;
}

extern "C" {
    fn strpbrk(s: *const u8, set: *const u8) -> *mut u8;
    fn wcspbrk(s: *const u16, set: *const u16) -> *const u16;
    fn _mbsdec(string: *const u8, current: *const u8) -> *mut u8;
    fn qsort(
        base: *mut c_void,
        num: usize,
        width: usize,
        comp: Option<unsafe extern "C" fn(*const c_void, *const c_void) -> c_int>,
    );
}

/// The per-character-type half of the expansion templates.
pub(crate) trait WildChar: ArgChar {
    /// `traits::tcspbrk`
    unsafe fn tcspbrk(s: *mut Self, set: *const Self) -> *mut Self;
    /// `previous_character`
    unsafe fn previous_character(first: *mut Self, current: *mut Self) -> *mut Self;
    /// `get_wide`: the pattern as UTF-16 (NULL if it does not convert), and
    /// the buffer that owns the conversion.
    unsafe fn get_wide(source: *mut Self) -> (*const u16, Option<Buf<u16>>);
    /// `get_file_name`: a found name in this character type. `Err` is the
    /// conversion's errno (see the divergence note at the call).
    unsafe fn get_file_name(source: *mut u16) -> Result<(*mut Self, Option<Buf<u8>>), c_int>;
}

impl WildChar for u8 {
    unsafe fn tcspbrk(s: *mut u8, set: *const u8) -> *mut u8 {
        strpbrk(s, set)
    }
    unsafe fn previous_character(first: *mut u8, current: *mut u8) -> *mut u8 {
        _mbsdec(first, current)
    }
    unsafe fn get_wide(source: *mut u8) -> (*const u16, Option<Buf<u16>>) {
        match mbs_to_wcs(source, file_codepage()) {
            Ok(Some(b)) => (b.0, Some(b)),
            Ok(None) | Err(_) => (null(), None),
        }
    }
    unsafe fn get_file_name(source: *mut u16) -> Result<(*mut u8, Option<Buf<u8>>), c_int> {
        let b = wcs_to_mbs(source, file_codepage())?;
        Ok((b.0, Some(b)))
    }
}

impl WildChar for u16 {
    unsafe fn tcspbrk(s: *mut u16, set: *const u16) -> *mut u16 {
        wcspbrk(s, set) as *mut u16
    }
    unsafe fn previous_character(_first: *mut u16, current: *mut u16) -> *mut u16 {
        current.sub(1)
    }
    unsafe fn get_wide(source: *mut u16) -> (*const u16, Option<Buf<u16>>) {
        (source, None)
    }
    unsafe fn get_file_name(source: *mut u16) -> Result<(*mut u16, Option<Buf<u8>>), c_int> {
        Ok((source, None))
    }
}

/// The anonymous-namespace `argument_list<Character>`: a growable array of
/// owned strings, all freed (with the array) unless detached.
struct ArgumentList<C> {
    first: *mut *mut C,
    last: *mut *mut C,
    end: *mut *mut C,
}

impl<C> ArgumentList<C> {
    fn new() -> Self {
        ArgumentList { first: null_mut(), last: null_mut(), end: null_mut() }
    }

    fn size(&self) -> usize {
        // `_last - _first`; both NULL for an empty list.
        (self.last as usize - self.first as usize) / core::mem::size_of::<*mut C>()
    }

    unsafe fn append(&mut self, element: *mut C) -> c_int {
        let expand_status = self.expand_if_necessary();
        if expand_status != 0 {
            free(element.cast());
            return expand_status;
        }

        *self.last = element;
        self.last = self.last.add(1);
        0
    }

    unsafe fn expand_if_necessary(&mut self) -> c_int {
        // If there is already room for more elements, just return:
        if self.last != self.end {
            return 0;
        }
        // If the list has not yet had an array allocated for it, allocate one:
        if self.first.is_null() {
            const INITIAL_COUNT: usize = 4;

            self.first = calloc(INITIAL_COUNT, core::mem::size_of::<*mut C>()) as *mut *mut C;
            if self.first.is_null() {
                return ENOMEM;
            }

            self.last = self.first;
            self.end = self.first.add(INITIAL_COUNT);
            0
        }
        // Otherwise, double the size of the array:
        else {
            let old_count = self.size_of_array();
            if old_count > usize::MAX / 2 {
                return ENOMEM;
            }

            let new_count = old_count * 2;
            let new_array =
                _recalloc(self.first.cast(), new_count, core::mem::size_of::<*mut C>()) as *mut *mut C;
            if new_array.is_null() {
                return ENOMEM;
            }

            self.first = new_array;
            self.last = self.first.add(old_count);
            self.end = self.first.add(new_count);
            0
        }
    }

    /// `_end - _first`
    fn size_of_array(&self) -> usize {
        (self.end as usize - self.first as usize) / core::mem::size_of::<*mut C>()
    }
}

impl<C> Drop for ArgumentList<C> {
    fn drop(&mut self) {
        unsafe {
            let mut it = self.first;
            while it != self.last {
                free((*it).cast());
                it = it.add(1);
            }
            free(self.first.cast());
        }
    }
}

/// `copy_and_add_argument_to_buffer<Character>`: `directory[..directory_length]`
/// followed by `file_name`, as a fresh heap string appended to `buffer`.
unsafe fn copy_and_add_argument_to_buffer<C: WildChar>(
    file_name: *const C,
    directory: *const C,
    directory_length: usize,
    buffer: &mut ArgumentList<C>,
) -> c_int {
    let file_name_count = C::tcslen(file_name) + 1;
    if file_name_count > usize::MAX - directory_length {
        return ENOMEM;
    }

    let required_count = directory_length.wrapping_add(file_name_count).wrapping_add(1);
    // `_calloc_crt_t(Character, required_count)` -- unchecked, as in the C: a
    // NULL here fails the `_ERRCHECK`ed copy below (Watson).
    let argument_buffer = calloc(required_count, core::mem::size_of::<C>()) as *mut C;

    if directory_length > 0 {
        errcheck(C::tcsncpy_s(argument_buffer, required_count, directory, directory_length));
    }

    errcheck(C::tcsncpy_s(
        argument_buffer.wrapping_add(directory_length),
        required_count - directory_length,
        file_name,
        file_name_count,
    ));

    buffer.append(argument_buffer)
}

/// Closes the find handle at scope exit -- `__crt_findfile_handle`.
struct FindHandle(Handle);
impl Drop for FindHandle {
    fn drop(&mut self) {
        if self.0 as isize != INVALID_HANDLE_VALUE {
            unsafe { FindClose(self.0) };
        }
    }
}

/// The "sort" comparator: it compares the ELEMENT ADDRESSES `qsort` hands it,
/// never the strings they point to -- Microsoft's lambda, verbatim.
unsafe extern "C" fn compare_slot_addresses(lhs: *const c_void, rhs: *const c_void) -> c_int {
    if (lhs as usize) < (rhs as usize) {
        return -1;
    }
    if (lhs as usize) > (rhs as usize) {
        return 1;
    }
    0
}

fn is_directory_separator<C: ArgChar>(c: C) -> bool {
    c.is(b'/') || c.is(b'\\') || c.is(b':')
}

/// `expand_argument_wildcards<Character>`
unsafe fn expand_argument_wildcards<C: WildChar>(
    argument: *mut C,
    wildcard: *mut C,
    buffer: &mut ArgumentList<C>,
) -> c_int {
    // Find the first slash or colon before the wildcard:
    let mut it = wildcard;
    while it != argument && !is_directory_separator(*it) {
        it = C::previous_character(argument, it);
    }

    // If we found a colon that can't form a drive name (e.g. it can't be
    // 'D:'), then just add the argument as-is (we don't know how to expand it):
    if (*it).is(b':') && it != argument.add(1) {
        return copy_and_add_argument_to_buffer(argument, null(), 0, buffer);
    }

    let directory_length = if is_directory_separator(*it) {
        // it points to the separator, so add 1 to include it.
        (it as usize - argument as usize) / core::mem::size_of::<C>() + 1
    } else {
        0
    };

    // Try to begin the find operation:
    let mut find_data: Win32FindDataW = core::mem::zeroed();
    let (wide_file_name, _wide_owner) = C::get_wide(argument);

    let find_handle = FindHandle(FindFirstFileExW(
        wide_file_name,
        FIND_EX_INFO_STANDARD,
        &mut find_data,
        FIND_EX_SEARCH_NAME_MATCH,
        null_mut(),
        0,
    ));

    // If the find operation failed, there was no match, so just add the argument:
    if find_handle.0 as isize == INVALID_HANDLE_VALUE {
        return copy_and_add_argument_to_buffer(argument, null(), 0, buffer);
    }

    let old_argument_count = buffer.size();

    loop {
        // DIVERGENCE: the C's `get_file_name` returns NULL when the found name
        // does not convert back to the narrow codepage and the next line
        // dereferences it. Here the expansion stops with that conversion's
        // errno instead (only an allocation failure makes it fail in practice:
        // WideCharToMultiByte substitutes unmappable characters).
        let (file_name, _name_owner) = match C::get_file_name(find_data.file_name.as_mut_ptr()) {
            Ok(r) => r,
            Err(e) => return e,
        };

        // Skip . and ..:
        let skip = ((*file_name).is(b'.') && (*file_name.add(1)).is_nul())
            || ((*file_name).is(b'.') && (*file_name.add(1)).is(b'.') && (*file_name.add(2)).is_nul());

        if !skip {
            let add_status = copy_and_add_argument_to_buffer(file_name, argument, directory_length, buffer);
            if add_status != 0 {
                return add_status;
            }
        }

        if FindNextFileW(find_handle.0, &mut find_data) == 0 {
            break;
        }
    }

    // If we didn't add any arguments to the buffer, then we're done:
    let new_argument_count = buffer.size();
    if old_argument_count == new_argument_count {
        return 0;
    }

    // If we did add new arguments, let's "helpfully sort" them (by slot
    // address -- see the module doc):
    qsort(
        buffer.first.add(old_argument_count).cast(),
        new_argument_count - old_argument_count,
        core::mem::size_of::<*mut C>(),
        Some(compare_slot_addresses),
    );

    0
}

/// `common_expand_argv_wildcards<Character>`
unsafe fn common_expand_argv_wildcards<C: WildChar>(argv: *mut *mut C, result: *mut *mut *mut C) -> c_int {
    // _VALIDATE_RETURN_ERRCODE(result != nullptr, EINVAL)
    if result.is_null() {
        *_errno() = EINVAL;
        _invalid_parameter_noinfo();
        return EINVAL;
    }
    *result = null_mut();

    let mut expansion_buffer = ArgumentList::<C>::new();
    let mut it = argv;
    while !(*it).is_null() {
        let wildcard_characters: [C; 3] = [C::ascii(b'*'), C::ascii(b'?'), C::NUL];
        let wildcard = C::tcspbrk(*it, wildcard_characters.as_ptr());

        // If no wildcard characters were found in the argument string, just
        // append it to the list and continue on. Otherwise, do the expansion:
        if wildcard.is_null() {
            let append_status = copy_and_add_argument_to_buffer(*it, null(), 0, &mut expansion_buffer);
            if append_status != 0 {
                return append_status;
            }
        } else {
            let expand_status = expand_argument_wildcards(*it, wildcard, &mut expansion_buffer);
            if expand_status != 0 {
                return expand_status;
            }
        }
        it = it.add(1);
    }

    // Re-pack the accumulated arguments in the argv parser's single-block form.
    let argument_count = expansion_buffer.size() + 1;
    let mut character_count: usize = 0;
    let mut e = expansion_buffer.first;
    while e != expansion_buffer.last {
        character_count += C::tcslen(*e) + 1;
        e = e.add(1);
    }

    let expanded_argv = __acrt_allocate_buffer_for_argv(argument_count, character_count, core::mem::size_of::<C>());

    if expanded_argv.is_null() {
        return -1; // sic: not ENOMEM
    }

    let argument_first = expanded_argv as *mut *mut C;
    let character_first = expanded_argv.add(argument_count * core::mem::size_of::<*mut C>()) as *mut C;

    let mut argument_it = argument_first;
    let mut character_it = character_first;
    let mut e = expansion_buffer.first;
    while e != expansion_buffer.last {
        let count = C::tcslen(*e) + 1;

        errcheck(C::tcsncpy_s(
            character_it,
            character_count - (character_it as usize - character_first as usize) / core::mem::size_of::<C>(),
            *e,
            count,
        ));

        *argument_it = character_it;
        argument_it = argument_it.add(1);
        character_it = character_it.add(count);
        e = e.add(1);
    }

    *result = argument_first;
    0
}

#[no_mangle]
#[inline(never)]
pub unsafe extern "C" fn __acrt_expand_narrow_argv_wildcards(argv: *mut *mut u8, result: *mut *mut *mut u8) -> c_int {
    common_expand_argv_wildcards(argv, result)
}

#[no_mangle]
#[inline(never)]
pub unsafe extern "C" fn __acrt_expand_wide_argv_wildcards(
    argv: *mut *mut u16,
    result: *mut *mut *mut u16,
) -> c_int {
    common_expand_argv_wildcards(argv, result)
}
