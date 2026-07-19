use crate::prelude::*;
use core::ffi::c_void;
#[no_mangle]
pub unsafe extern "C" fn GetLongestChildKeyName(RegHandle: HANDLE) -> i32 {
    let mut max_adapter_name: u32 = 0;
    let status = RegQueryInfoKeyW(
        (RegHandle as *mut HKEY__),
        core::ptr::null_mut(),
        core::ptr::null_mut(),
        core::ptr::null_mut(),
        core::ptr::null_mut(),
        &mut max_adapter_name,
        core::ptr::null_mut(),
        core::ptr::null_mut(),
        core::ptr::null_mut(),
        core::ptr::null_mut(),
        core::ptr::null_mut(),
        core::ptr::null_mut(),
    );
    if status == (ERROR_SUCCESS as i32) {
        (max_adapter_name as i32) + 1
    } else {
        -1
    }
}
#[no_mangle]
pub unsafe extern "C" fn OpenChildKeyRead(RegHandle: HANDLE, ChildKeyName: PWCHAR, ReturnHandle: PHKEY) -> i32 {
    RegOpenKeyExW((RegHandle as *mut HKEY__), ChildKeyName as *const u16, 0u32, KEY_READ, ReturnHandle)
}
#[no_mangle]
pub unsafe extern "C" fn GetNthChildKeyName(RegHandle: HANDLE, n: u32) -> *mut u16 {
    let MaxAdapterName = GetLongestChildKeyName(RegHandle);
    if MaxAdapterName == -1 {
        return core::ptr::null_mut();
    }

    let mut ValueLen = MaxAdapterName as u32;
    let Value = HeapAlloc(
        GetProcessHeap(),
        0,
        ((MaxAdapterName as usize * core::mem::size_of::<u16>()) as SIZE_T),
    ) as *mut u16;
    if Value.is_null() {
        return core::ptr::null_mut();
    }

    let Status = RegEnumKeyExW(
        (RegHandle as *mut HKEY__),
        n,
        Value,
        core::ptr::addr_of_mut!(ValueLen),
        core::ptr::null_mut(),
        core::ptr::null_mut(),
        core::ptr::null_mut(),
        core::ptr::null_mut(),
    );

    if Status != (ERROR_SUCCESS as i32) {
        HeapFree(GetProcessHeap(), 0, Value as *mut core::ffi::c_void);
        return core::ptr::null_mut();
    }

    *Value.add(ValueLen as usize) = 0;
    Value
}
#[no_mangle]
pub unsafe extern "C" fn ConsumeChildKeyName(Name: *mut u16) {
    if !Name.is_null() {
        HeapFree(GetProcessHeap(), 0u32, Name as *mut core::ffi::c_void);
    }
}
#[no_mangle]
pub unsafe extern "C" fn QueryRegistryValue(reg_handle: HANDLE, value_name: PWCHAR, registry_type: LPDWORD, length: LPDWORD) -> PVOID {
    let mut read_value: PVOID = core::ptr::null_mut();
    let mut error: DWORD;

    *length = 0;
    *registry_type = REG_NONE;

    loop {
        error = (RegQueryValueExW(
            (reg_handle as *mut HKEY__),
            value_name,
            core::ptr::null_mut(),
            registry_type,
            read_value as *mut u8,
            length,
        ) as u32);

        if error == ERROR_SUCCESS {
            if !read_value.is_null() {
                break;
            }
        } else if error == ERROR_MORE_DATA {
            HeapFree(GetProcessHeap(), 0, read_value);
        } else {
            break;
        }

        read_value = HeapAlloc(GetProcessHeap(), 0, ((*length) as SIZE_T));
        if read_value.is_null() {
            return core::ptr::null_mut();
        }
    }

    if error != ERROR_SUCCESS {
        if !read_value.is_null() {
            HeapFree(GetProcessHeap(), 0, read_value);
        }
        *length = 0;
        *registry_type = REG_NONE;
        return core::ptr::null_mut();
    }

    read_value
}
#[no_mangle]
pub unsafe extern "C" fn TerminateReadString(String: *mut u16, Length: u32) -> *mut u16 {
    let heap = GetProcessHeap();
    let term = HeapAlloc(heap, 0u32, Length.wrapping_add(2) as SIZE_T);
    if term.is_null() {
        return core::ptr::null_mut();
    }
    core::ptr::copy_nonoverlapping(String as *const u8, term as *mut u8, Length as usize);
    *(term as *mut u16).add((Length / 2) as usize) = 0u16;
    (term as *mut u16)
}
#[no_mangle]
pub unsafe extern "C" fn QueryRegistryValueString(RegHandle: HANDLE, ValueName: PWCHAR) -> PWCHAR {
    let mut Type: u32 = 0;
    let mut Length: u32 = 0;

    let string = QueryRegistryValue(RegHandle, ValueName, &mut Type, &mut Length) as PWCHAR;
    if string.is_null() {
        return core::ptr::null_mut();
    }

    if Type != REG_SZ {
        DbgPrint(
            b"Type mismatch for %S (%d != %d)\n\0".as_ptr() as *const c_char,
            ValueName,
            Type,
            REG_SZ,
        );
    }

    let terminated_string = TerminateReadString(string, Length);
    HeapFree(GetProcessHeap(), 0, string.cast());
    if terminated_string.is_null() {
        return core::ptr::null_mut();
    }

    terminated_string
}
#[no_mangle]
pub unsafe extern "C" fn ConsumeRegValueString(Value: *mut u16) {
    if !Value.is_null() {
        HeapFree(GetProcessHeap(), 0, Value as *mut core::ffi::c_void);
    }
}
#[no_mangle]
pub unsafe extern "C" fn QueryRegistryValueStringMulti(
    RegHandle: HANDLE,
    ValueName: PWCHAR,
) -> *mut PWCHAR {
    let mut reg_type: u32 = 0;
    let mut length: u32 = 0;
    let string = QueryRegistryValue(RegHandle, ValueName, &mut reg_type, &mut length) as *mut u16;
    if string.is_null() {
        return core::ptr::null_mut();
    }

    if reg_type != REG_MULTI_SZ {
        DbgPrint(
            b"Type mismatch for %S (%d != %d)\n\0".as_ptr() as *const i8,
            ValueName,
            reg_type,
            REG_MULTI_SZ,
        );
        // Originally: HeapFree(GetProcessHeap(), 0, String); return NULL;
        // Commented out, so we fall through.
    }

    let terminated_string = TerminateReadString(string, length);
    HeapFree(GetProcessHeap(), 0, string as _);
    if terminated_string.is_null() {
        return core::ptr::null_mut();
    }

    // Count number of strings
    let mut tmp = terminated_string;
    let mut str_count: usize = 0;
    while *tmp != 0 {
        str_count += 1;
        while *tmp != 0 {
            tmp = tmp.add(1);
        }
        tmp = tmp.add(1);
    }

    let table_size = (str_count + 1) * core::mem::size_of::<PWCHAR>();
    let table = HeapAlloc(GetProcessHeap(), 0, ((table_size) as SIZE_T)) as *mut PWCHAR;
    if table.is_null() {
        HeapFree(GetProcessHeap(), 0, terminated_string as _);
        return core::ptr::null_mut();
    }

    let mut tmp = terminated_string;
    let mut idx: usize = 0;
    while *tmp != 0 {
        let orig = tmp;
        let mut len: usize = 0;
        while *tmp != 0 {
            len += 1;
            tmp = tmp.add(1);
        }

        let alloc_size = len * core::mem::size_of::<u16>();
        let dest = HeapAlloc(GetProcessHeap(), 0, ((alloc_size) as SIZE_T)) as *mut u16;
        core::ptr::copy_nonoverlapping(orig, dest, len);
        *table.add(idx) = dest;

        tmp = tmp.add(1); // skip null terminator
        idx += 1;
    }

    *table.add(idx) = core::ptr::null_mut();

    HeapFree(GetProcessHeap(), 0, terminated_string as _);

    table
}


// ============================================================================
// ORIGINAL C SOURCE
// ============================================================================
//
// #include "iphlpapi_private.h"
//
// int GetLongestChildKeyName( HANDLE RegHandle ) {
//   LONG Status;
//   DWORD MaxAdapterName;
//
//   Status = RegQueryInfoKeyW(RegHandle,
// 			    NULL,
// 			    NULL,
// 			    NULL,
// 			    NULL,
// 			    &MaxAdapterName,
// 			    NULL,
// 			    NULL,
// 			    NULL,
// 			    NULL,
// 			    NULL,
// 			    NULL);
//   if (Status == ERROR_SUCCESS)
//     return MaxAdapterName + 1;
//   else
//     return -1;
// }
//
// LONG OpenChildKeyRead( HANDLE RegHandle,
// 		       PWCHAR ChildKeyName,
// 		       PHKEY ReturnHandle ) {
//   return RegOpenKeyExW( RegHandle,
// 			ChildKeyName,
// 			0,
// 			KEY_READ,
// 			ReturnHandle );
// }
//
// /*
//  * Yields a malloced value that must be freed.
//  */
//
// PWCHAR GetNthChildKeyName( HANDLE RegHandle, DWORD n ) {
//   LONG Status;
//   int MaxAdapterName = GetLongestChildKeyName( RegHandle );
//   PWCHAR Value;
//   DWORD ValueLen;
//
//   if (MaxAdapterName == -1)
//     return 0;
//
//   ValueLen = MaxAdapterName;
//   Value = (PWCHAR)HeapAlloc( GetProcessHeap(), 0, MaxAdapterName * sizeof(WCHAR) );
//   if (!Value) return 0;
//
//   Status = RegEnumKeyExW( RegHandle, n, Value, &ValueLen,
// 			  NULL, NULL, NULL, NULL );
//   if (Status != ERROR_SUCCESS) {
//     HeapFree(GetProcessHeap(), 0, Value);
//     return 0;
//   } else {
//     Value[ValueLen] = 0;
//     return Value;
//   }
// }
//
// void ConsumeChildKeyName( PWCHAR Name ) {
//   if (Name) HeapFree( GetProcessHeap(), 0, Name );
// }
//
// PVOID QueryRegistryValue(HANDLE RegHandle, PWCHAR ValueName, LPDWORD RegistryType, LPDWORD Length)
// {
//     PVOID ReadValue = NULL;
//     DWORD Error;
//
//     *Length = 0;
//     *RegistryType = REG_NONE;
//
//     while (TRUE)
//     {
//         Error = RegQueryValueExW(RegHandle, ValueName, NULL, RegistryType, ReadValue, Length);
//         if (Error == ERROR_SUCCESS)
//         {
//             if (ReadValue) break;
//         }
//         else if (Error == ERROR_MORE_DATA)
//         {
//             HeapFree(GetProcessHeap(), 0, ReadValue);
//         }
//         else break;
//
//         ReadValue = HeapAlloc(GetProcessHeap(), 0, *Length);
//         if (!ReadValue) return NULL;
//     }
//
//     if (Error != ERROR_SUCCESS)
//     {
//         if (ReadValue) HeapFree(GetProcessHeap(), 0, ReadValue);
//
//         *Length = 0;
//         *RegistryType = REG_NONE;
//         ReadValue = NULL;
//     }
//
//     return ReadValue;
// }
//
// PWCHAR TerminateReadString(PWCHAR String, DWORD Length)
// {
//     PWCHAR TerminatedString;
//
//     TerminatedString = HeapAlloc(GetProcessHeap(), 0, Length + sizeof(WCHAR));
//     if (TerminatedString == NULL)
//         return NULL;
//
//     memcpy(TerminatedString, String, Length);
//
//     TerminatedString[Length / sizeof(WCHAR)] = UNICODE_NULL;
//
//     return TerminatedString;
// }
//
// PWCHAR QueryRegistryValueString( HANDLE RegHandle, PWCHAR ValueName )
// {
//     PWCHAR String, TerminatedString;
//     DWORD Type, Length;
//
//     String = QueryRegistryValue(RegHandle, ValueName, &Type, &Length);
//     if (!String) return NULL;
//     if (Type != REG_SZ)
//     {
//         DbgPrint("Type mismatch for %S (%d != %d)\n", ValueName, Type, REG_SZ);
//         //HeapFree(GetProcessHeap(), 0, String);
//         //return NULL;
//     }
//
//     TerminatedString = TerminateReadString(String, Length);
//     HeapFree(GetProcessHeap(), 0, String);
//     if (!TerminatedString) return NULL;
//
//     return TerminatedString;
// }
//
// void ConsumeRegValueString( PWCHAR Value ) {
//   if (Value) HeapFree(GetProcessHeap(), 0, Value);
// }
//
// PWCHAR *QueryRegistryValueStringMulti( HANDLE RegHandle, PWCHAR ValueName ) {
//     PWCHAR String, TerminatedString, Tmp;
//     PWCHAR *Table;
//     DWORD Type, Length, i, j;
//
//     String = QueryRegistryValue(RegHandle, ValueName, &Type, &Length);
//     if (!String) return NULL;
//     if (Type != REG_MULTI_SZ)
//     {
//         DbgPrint("Type mismatch for %S (%d != %d)\n", ValueName, Type, REG_MULTI_SZ);
//         //HeapFree(GetProcessHeap(), 0, String);
//         //return NULL;
//     }
//
//     TerminatedString = TerminateReadString(String, Length);
//     HeapFree(GetProcessHeap(), 0, String);
//     if (!TerminatedString) return NULL;
//
//     for (Tmp = TerminatedString, i = 0; *Tmp; Tmp++, i++) while (*Tmp) Tmp++;
//
//     Table = HeapAlloc(GetProcessHeap(), 0, (i + 1) * sizeof(PWCHAR));
//     if (!Table)
//     {
//         HeapFree(GetProcessHeap(), 0, TerminatedString);
//         return NULL;
//     }
//
//     for (Tmp = TerminatedString, j = 0; *Tmp; Tmp++, j++)
//     {
//         PWCHAR Orig = Tmp;
//
//         for (i = 0; *Tmp; i++, Tmp++);
//
//         Table[j] = HeapAlloc(GetProcessHeap(), 0, i * sizeof(WCHAR));
//         memcpy(Table[j], Orig, i * sizeof(WCHAR));
//     }
//
//     Table[j] = NULL;
//
//     HeapFree(GetProcessHeap(), 0, TerminatedString);
//
//     return Table;
// }
//
// ============================================================================
// END ORIGINAL C SOURCE
// ============================================================================
