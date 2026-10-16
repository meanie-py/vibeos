//! The loader's boot options, as a driver sees them: Control\SystemStartOptions, which the
//! kernel stores without the slashes (`/NETSTACK=NDIS /XHCITRACE=300` becomes
//! `NETSTACK=NDIS XHCITRACE=300`). For debug switches a driver reads once at DriverEntry.

use crate::ddk::{
    ZwClose, ZwOpenKey, ZwQueryValueKey, HANDLE, OBJECT_ATTRIBUTES, PVOID, UNICODE_STRING,
    _KEY_VALUE_INFORMATION_CLASS_KeyValuePartialInformation,
};

const fn wide<const N: usize>(s: &[u8]) -> [u16; N] {
    let mut out = [0u16; N];
    let mut i = 0;
    while i < N {
        out[i] = s[i] as u16;
        i += 1;
    }
    out
}

/// The value of boot option `name` (`NAME=value`, or `NAME` alone, which yields an empty
/// value), copied into `out` up to the next space. `None` when the option is absent. Matching
/// is case-insensitive and whole-word. PASSIVE_LEVEL.
pub unsafe fn boot_option(name: &[u8], out: &mut [u8]) -> Option<usize> {
    const KEY: [u16; 50] = wide(b"\\Registry\\Machine\\System\\CurrentControlSet\\Control");
    const VALUE: [u16; 18] = wide(b"SystemStartOptions");
    let mut path = KEY;
    let mut key_name: UNICODE_STRING = core::mem::zeroed();
    key_name.Buffer = path.as_mut_ptr();
    key_name.Length = (KEY.len() * 2) as u16;
    key_name.MaximumLength = key_name.Length;
    let mut oa: OBJECT_ATTRIBUTES = core::mem::zeroed();
    oa.Length = core::mem::size_of::<OBJECT_ATTRIBUTES>() as u32;
    oa.ObjectName = &mut key_name;
    oa.Attributes = 0x240; // OBJ_CASE_INSENSITIVE | OBJ_KERNEL_HANDLE
    let mut key: HANDLE = core::ptr::null_mut();
    if ZwOpenKey(&mut key, 0x20019 /* KEY_READ */, &mut oa) < 0 {
        return None;
    }
    let mut value = VALUE;
    let mut value_name: UNICODE_STRING = core::mem::zeroed();
    value_name.Buffer = value.as_mut_ptr();
    value_name.Length = (VALUE.len() * 2) as u16;
    value_name.MaximumLength = value_name.Length;
    // KEY_VALUE_PARTIAL_INFORMATION: TitleIndex, Type, DataLength, then the string.
    let mut info = [0u32; 3 + 256];
    let mut got = 0u32;
    let status = ZwQueryValueKey(
        key,
        &mut value_name,
        _KEY_VALUE_INFORMATION_CLASS_KeyValuePartialInformation,
        info.as_mut_ptr() as PVOID,
        core::mem::size_of_val(&info) as u32,
        &mut got,
    );
    ZwClose(key);
    if status < 0 {
        return None;
    }
    let chars = (info[2] as usize / 2).min((info.len() - 3) * 2);
    let text = core::slice::from_raw_parts(info.as_ptr().add(3) as *const u16, chars);
    let lower = |c: u16| if (b'A' as u16..=b'Z' as u16).contains(&c) { c + 32 } else { c };
    let mut i = 0;
    while i < text.len() {
        // Word start: the beginning, or after a space.
        let at_word = i == 0 || text[i - 1] == b' ' as u16;
        if at_word && i + name.len() <= text.len()
            && text[i..i + name.len()].iter().zip(name).all(|(&c, &n)| lower(c) == lower(n as u16))
        {
            let rest = &text[i + name.len()..];
            match rest.first() {
                None | Some(&0x20) | Some(&0) => return Some(0),
                Some(&0x3D) => {
                    let mut n = 0;
                    for &c in &rest[1..] {
                        if c == b' ' as u16 || c == 0 || n == out.len() {
                            break;
                        }
                        out[n] = c as u8;
                        n += 1;
                    }
                    return Some(n);
                }
                _ => {}
            }
        }
        i += 1;
    }
    None
}

/// `boot_option` read as a decimal number; `default` when present without a value.
pub unsafe fn boot_option_u32(name: &[u8], default: u32) -> Option<u32> {
    let mut buf = [0u8; 16];
    let n = boot_option(name, &mut buf)?;
    if n == 0 {
        return Some(default);
    }
    let mut v: u32 = 0;
    for &c in &buf[..n] {
        if !c.is_ascii_digit() {
            break;
        }
        v = v.saturating_mul(10).saturating_add((c - b'0') as u32);
    }
    Some(v)
}
