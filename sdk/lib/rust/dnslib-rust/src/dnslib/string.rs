use ntdll_sys::nt::*;
use core::ffi::c_void;

extern "C" {
    fn strlen(s: *const i8) -> usize;
    fn wcslen(s: *const u16) -> usize;
}
extern "system" {
    fn SetLastError(dwErrCode: u32);
    fn Dns_AllocZero(cb: usize) -> *mut c_void;
    fn MultiByteToWideChar(CodePage: u32, dwFlags: u32, lpMultiByteStr: *const i8, cbMultiByte: i32, lpWideCharStr: *mut u16, cchWideChar: i32) -> i32;
    fn WideCharToMultiByte(CodePage: u32, dwFlags: u32, lpWideCharStr: *const u16, cchWideChar: i32, lpMultiByteStr: *mut i8, cbMultiByte: i32, lpDefaultChar: *const i8, lpUsedDefaultChar: *mut i32) -> i32;
}

pub const UnicodeString: u32 = 1;
pub const AnsiString: u32 = 2;
pub const Utf8String: u32 = 3;
const CP_ACP: u32 = 0;
const ERROR_MORE_DATA: u32 = 234;
const ERROR_INVALID_DATA: u32 = 13;

#[no_mangle]
pub unsafe extern "system" fn Dns_StringCopy(
    destination: *mut c_void,
    destination_size: *mut u32,
    string: *const c_void,
    string_size: u32,
    input_type: u32,
    output_type: u32,
) -> u32 {
    let mut output_size: u32 = 0;
    let mut string_size = string_size;

    if string_size == 0 {
        if input_type == UnicodeString {
            string_size = wcslen(string as *const u16) as u32;
        } else {
            string_size = strlen(string as *const i8) as u32;
        }
    }

    if !destination_size.is_null() {
        let dest_size = Dns_GetBufferLengthForStringCopy(string as *mut c_void, string_size, input_type, output_type);
        if *destination_size < dest_size {
            SetLastError(ERROR_MORE_DATA);
            core::ptr::write_volatile(destination_size, dest_size);
            return 0;
        }
        if dest_size == 0 {
            SetLastError(ERROR_INVALID_DATA);
            return 0;
        }
        core::ptr::write_volatile(destination_size, dest_size);
    }

    if input_type == UnicodeString {
        if output_type == AnsiString {
            output_size = WideCharToMultiByte(
                CP_ACP,
                0,
                string as *const u16,
                string_size as i32,
                destination as *mut i8,
                -1,
                core::ptr::null(),
                core::ptr::null_mut(),
            ) as u32 + 1;
        } else if output_type == UnicodeString {
            let byte_count = (string_size as usize) * 2;
            core::ptr::copy_nonoverlapping(
                string as *const u8,
                destination as *mut u8,
                byte_count,
            );
            output_size = byte_count as u32 + 2;
        } else if output_type == Utf8String {
            output_size = 0;
        }
    } else if input_type == AnsiString {
        if output_type == AnsiString {
            core::ptr::copy_nonoverlapping(
                string as *const u8,
                destination as *mut u8,
                string_size as usize,
            );
            output_size = string_size + 1;
        } else if output_type == UnicodeString {
            let wide_chars = MultiByteToWideChar(
                CP_ACP,
                0,
                string as *const i8,
                string_size as i32,
                destination as *mut u16,
                -1,
            );
            output_size = (wide_chars as u32).wrapping_mul(2).wrapping_add(2);
        } else if output_type == Utf8String {
            output_size = 0;
        }
    } else if input_type == Utf8String {
        output_size = 0;
    }

    output_size
}

#[no_mangle]
pub unsafe extern "system" fn Dns_CreateStringCopy_W(Name: *mut u16) -> *mut u16 {
    if Name.is_null() {
        SetLastError(ntdll_sys::nt::ERROR_INVALID_PARAMETER);
        return core::ptr::null_mut();
    }

    let string_length = (wcslen(Name as *const u16) + 1) * core::mem::size_of::<u16>();

    let name_copy = Dns_AllocZero(string_length) as *mut u16;
    if !name_copy.is_null() {
        core::ptr::copy_nonoverlapping(
            Name as *const u8,
            name_copy as *mut u8,
            string_length,
        );
    } else {
        SetLastError(ntdll_sys::nt::ERROR_NOT_ENOUGH_MEMORY);
    }

    name_copy
}

#[no_mangle]
pub unsafe extern "system" fn Dns_GetBufferLengthForStringCopy(
    String: *mut c_void,
    Size: u32,
    InputType: u32,
    OutputType: u32,
) -> u32 {
    let mut output_size: u32 = 0;

    if InputType == UnicodeString {
        let mut size_val = Size;
        if size_val == 0 {
            size_val = wcslen(String as *const u16) as u32;
        }
        if OutputType == UnicodeString {
            output_size = (size_val.wrapping_add(1u32)).wrapping_mul(core::mem::size_of::<u16>() as u32);
        } else if OutputType == Utf8String {
            output_size = 0;
        } else {
            output_size = (WideCharToMultiByte(
                CP_ACP,
                0,
                String as *const u16,
                size_val as i32,
                core::ptr::null_mut::<i8>(),
                0,
                core::ptr::null::<i8>(),
                core::ptr::null_mut::<i32>(),
            ) as u32).wrapping_add(1u32);
        }
    } else if InputType == AnsiString {
        let mut size_val = Size;
        if size_val == 0 {
            size_val = strlen(String as *const i8) as u32;
        }
        if OutputType == AnsiString {
            output_size = size_val.wrapping_add(1u32);
        } else if OutputType == UnicodeString {
            output_size = ((MultiByteToWideChar(
                CP_ACP,
                0,
                String as *const i8,
                size_val as i32,
                core::ptr::null_mut::<u16>(),
                0,
            ) as u32).wrapping_add(1u32)).wrapping_mul(core::mem::size_of::<u16>() as u32);
        } else if OutputType == Utf8String {
            output_size = 0;
        }
    } else if InputType == Utf8String {
        output_size = 0;
    }

    output_size
}
