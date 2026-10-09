//! Message-signaled interrupts for xHCI controllers.
//!
//! An xHCI controller raises MSI-X or MSI on every platform that has them (on an arm64
//! server, often the only interrupt it can raise: PCIe INTx routing there is optional).
//! Windows' usbxhci.inf asks for them with `MSISupported`; here there is no INF, so
//! AddDevice writes the same opt-in under the device key before the PnP manager
//! assigns resources (nvme.sys does the same). pci.sys then adds message requirements,
//! and USBPORT_ParseResources takes the message over the line.
//!
//! One message: usbport has one interrupt object and the miniport one interrupter.
//! The tripwire token `XHCI_MSI_MESSAGES=<hex>` asks for more, so a rig can exercise
//! a multiple-message MSI block; the rest are assigned but left unconnected, as a
//! Windows driver leaves the messages it does not use.

use core::mem::size_of;
use core::ptr::null_mut;

use ntkrnl_sys::ddk::{
    IoOpenDeviceRegistryKey, ZwClose, ZwCreateKey, ZwOpenKey, ZwQueryValueKey, ZwSetValueKey,
    _KEY_VALUE_INFORMATION_CLASS_KeyValuePartialInformation, HANDLE, OBJECT_ATTRIBUTES, PDEVICE_OBJECT, ULONG,
    UNICODE_STRING,
};

/// A UTF-16 literal without a terminator: UNICODE_STRING lengths exclude it.
macro_rules! utf16 {
    ($s:literal) => {{
        const W: &[u16] = &{
            const B: &[u8] = $s.as_bytes();
            let mut out = [0u16; B.len()];
            let mut i = 0;
            while i < B.len() {
                out[i] = B[i] as u16;
                i += 1;
            }
            out
        };
        W
    }};
}

extern "C" {
    fn DbgPrint(Format: *const u8, ...) -> u32;
}

const PLUGPLAY_REGKEY_DEVICE: ULONG = 1;
const KEY_ALL_ACCESS: ULONG = 0xF003F;
const KEY_READ: ULONG = 0x20019;
const OBJ_CASE_INSENSITIVE_KERNEL_HANDLE: ULONG = 0x40 | 0x200;
const REG_SZ: u32 = 1;
const REG_DWORD: ULONG = 4;

unsafe fn unicode(s: &[u16]) -> UNICODE_STRING {
    let mut u: UNICODE_STRING = core::mem::zeroed();
    u.Buffer = s.as_ptr() as *mut u16;
    u.Length = (s.len() * 2) as u16;
    u.MaximumLength = u.Length;
    u
}

unsafe fn attributes(parent: HANDLE, name: &mut UNICODE_STRING) -> OBJECT_ATTRIBUTES {
    let mut oa: OBJECT_ATTRIBUTES = core::mem::zeroed();
    oa.Length = size_of::<OBJECT_ATTRIBUTES>() as ULONG;
    oa.RootDirectory = parent;
    oa.ObjectName = name;
    oa.Attributes = OBJ_CASE_INSENSITIVE_KERNEL_HANDLE;
    oa
}

unsafe fn create_key(parent: HANDLE, name: &[u16]) -> Option<HANDLE> {
    let mut name = unicode(name);
    let mut oa = attributes(parent, &mut name);
    let mut key: HANDLE = null_mut();
    (ZwCreateKey(&mut key, KEY_ALL_ACCESS, &mut oa, 0, null_mut(), 0, null_mut()) >= 0).then_some(key)
}

unsafe fn set_dword(key: HANDLE, name: &[u16], value: u32) {
    let mut name = unicode(name);
    let mut v = value;
    ZwSetValueKey(key, &mut name, 0, REG_DWORD, (&raw mut v).cast(), 4);
}

/// The hex value of tripwire token `<key>=<hex>` in `VibeTripwire` under
/// CurrentControlSet\Control, where the kernel publishes a rig's tripwire.
unsafe fn tripwire_value(key: &[u16]) -> Option<u32> {
    let mut name = unicode(utf16!("\\Registry\\Machine\\SYSTEM\\CurrentControlSet\\Control"));
    let mut oa = attributes(null_mut(), &mut name);
    let mut control: HANDLE = null_mut();
    if ZwOpenKey(&mut control, KEY_READ, &mut oa) < 0 {
        return None;
    }
    let mut value = unicode(utf16!("VibeTripwire"));
    // KEY_VALUE_PARTIAL_INFORMATION: TitleIndex, Type, DataLength, then the string.
    let mut buf = [0u32; 3 + 128];
    let mut got: ULONG = 0;
    let status = ZwQueryValueKey(
        control,
        &mut value,
        _KEY_VALUE_INFORMATION_CLASS_KeyValuePartialInformation,
        buf.as_mut_ptr().cast(),
        size_of::<[u32; 3 + 128]>() as ULONG,
        &mut got,
    );
    ZwClose(control);
    if status < 0 || buf[1] != REG_SZ {
        return None;
    }
    let chars = core::slice::from_raw_parts(buf.as_ptr().add(3) as *const u16, (buf[2] as usize / 2).min(256));
    let mut tokens = chars.split(|&c| c == b' ' as u16 || c == b';' as u16 || c == 0);
    let token = tokens.find(|t| t.len() > key.len() && t.starts_with(key) && t[key.len()] == b'=' as u16)?;
    let digits = &token[key.len() + 1..];
    let digits = digits.strip_prefix(utf16!("0x")).unwrap_or(digits);
    digits.iter().try_fold(0u32, |acc, &c| Some(acc << 4 | char::from_u32(c as u32)?.to_digit(16)?)).filter(|&m| m != 0)
}

/// AddDevice, for an xHCI controller: ask pci.sys for message interrupts.
pub unsafe fn opt_in(pdo: PDEVICE_OBJECT) {
    let mut device_key: HANDLE = null_mut();
    if IoOpenDeviceRegistryKey(pdo, PLUGPLAY_REGKEY_DEVICE, KEY_ALL_ACCESS, &mut device_key) < 0 {
        DbgPrint(b"[USBPORT] could not open the xHCI device key; line interrupt only\n\0".as_ptr());
        return;
    }
    let messages = tripwire_value(utf16!("XHCI_MSI_MESSAGES")).unwrap_or(1);
    if let Some(management) = create_key(device_key, utf16!("Interrupt Management")) {
        if let Some(props) = create_key(management, utf16!("MessageSignaledInterruptProperties")) {
            set_dword(props, utf16!("MSISupported"), 1);
            set_dword(props, utf16!("MessageNumberLimit"), messages);
            ZwClose(props);
        }
        ZwClose(management);
    }
    ZwClose(device_key);
    if messages > 1 {
        DbgPrint(b"[USBPORT] xHCI: asking for %u messages (tripwire)\n\0".as_ptr(), messages);
    }
}
