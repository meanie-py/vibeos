//! TDI device + dispatch layer (M4 foundation).
//!
//! Creates the four transport device objects AFD opens (\Device\Tcp, \Device\Udp,
//! \Device\Ip, \Device\RawIp) and routes their IRPs. This is the *scaffolding*
//! step: opens (CREATE) succeed so winsock socket()/bind() work, and we OBSERVE
//! the TDI requests AFD issues (logged via [NETSTACK] markers). The transport
//! operations (CONNECT/SEND/RECEIVE/...) are completed cleanly with
//! STATUS_NOT_IMPLEMENTED for now — deliberately NON-blocking so an in-progress
//! winsock call fails fast instead of hanging a boot service. The real data path
//! + completion bridge into the smoltcp engine is the next M4 step.

use core::ptr::null_mut;

use ntkrnl_sys::ddk::{
    FILE_FULL_EA_INFORMATION, IoCreateDevice, NTSTATUS, PDEVICE_OBJECT, PDRIVER_OBJECT, PIRP,
    UNICODE_STRING,
};
// IoGetCurrentIrpStackLocation is a header FORCEINLINE, NOT an ntoskrnl export —
// the bindgen extern decl links undefined. Use the inline re-export.
use ntkrnl_sys::inline::tier1::IoGetCurrentIrpStackLocation;

use crate::serial::nsprint;
use crate::tdi::{self, TdiFileKind};

const STATUS_SUCCESS: NTSTATUS = 0;
const STATUS_NOT_IMPLEMENTED: NTSTATUS = 0xC000_0002u32 as NTSTATUS;
const STATUS_OBJECT_NAME_COLLISION: NTSTATUS = 0xC000_0035u32 as NTSTATUS;

const FILE_DEVICE_NETWORK: u32 = 0x0000_0012;

const IRP_MJ_CREATE: u8 = 0x00;
const IRP_MJ_CLOSE: u8 = 0x02;
const IRP_MJ_DEVICE_CONTROL: u8 = 0x0E;
const IRP_MJ_INTERNAL_DEVICE_CONTROL: u8 = 0x0F;
const IRP_MJ_CLEANUP: u8 = 0x12;
const IRP_MJ_MAXIMUM_FUNCTION: usize = 0x1B;

/// Compile-time ASCII -> UTF-16 for the device-name literals.
const fn wname<const N: usize>(s: &[u8]) -> [u16; N] {
    let mut a = [0u16; N];
    let mut i = 0;
    while i < s.len() {
        a[i] = s[i] as u16;
        i += 1;
    }
    a
}

static TCP_NAME: [u16; 11] = wname(b"\\Device\\Tcp");
static UDP_NAME: [u16; 11] = wname(b"\\Device\\Udp");
static IP_NAME: [u16; 10] = wname(b"\\Device\\Ip");
static RAWIP_NAME: [u16; 13] = wname(b"\\Device\\RawIp");

fn unicode(name: &'static [u16]) -> UNICODE_STRING {
    let bytes = (name.len() * 2) as u16;
    UNICODE_STRING {
        Length: bytes,
        MaximumLength: bytes,
        Buffer: name.as_ptr() as *mut u16,
    }
}

unsafe fn create_one(driver: PDRIVER_OBJECT, name: &'static [u16], label: &str) -> PDEVICE_OBJECT {
    let mut us = unicode(name);
    let mut dev: PDEVICE_OBJECT = null_mut();
    let status = IoCreateDevice(
        driver,
        0,
        &mut us,
        FILE_DEVICE_NETWORK,
        0,
        0, // not exclusive
        &mut dev,
    );
    if status == STATUS_SUCCESS {
        nsprint!("[NETSTACK] TDI device \\Device\\{} created\n", label);
        dev
    } else if status == STATUS_OBJECT_NAME_COLLISION {
        // tcpip.sys still owns the name. Expected until tcpip is disabled.
        nsprint!("[NETSTACK] TDI \\Device\\{} name collision (tcpip owns it)\n", label);
        null_mut()
    } else {
        nsprint!(
            "[NETSTACK] TDI \\Device\\{} create failed {:#010x}\n",
            label, status as u32
        );
        null_mut()
    }
}

/// Wire the TDI major-function dispatch table and create the transport devices.
/// Returns the number of devices netstack actually owns.
pub unsafe fn init(driver: PDRIVER_OBJECT) -> u32 {
    // Route every major function through our single dispatcher. PDRIVER_DISPATCH
    // is extern "C" on amd64/arm64 but extern "stdcall" on i386; `dispatch` is
    // extern "system" (right ABI per arch) — bridge the nominal type gap with a
    // transmute (same ABI, distinct Rust types).
    let mf = &mut (*driver).MajorFunction;
    let entry = Some(core::mem::transmute::<_, _>(dispatch as *const ()));
    for i in 0..=IRP_MJ_MAXIMUM_FUNCTION {
        mf[i] = entry;
    }

    let mut owned = 0u32;
    for &(name, label) in &[
        (&TCP_NAME[..], "Tcp"),
        (&UDP_NAME[..], "Udp"),
        (&IP_NAME[..], "Ip"),
        (&RAWIP_NAME[..], "RawIp"),
    ] {
        if !create_one(driver, name, label).is_null() {
            owned += 1;
        }
    }
    nsprint!("[NETSTACK] TDI dispatch ready ({} of 4 devices owned)\n", owned);
    owned
}

unsafe extern "system" fn dispatch(_device: PDEVICE_OBJECT, irp: PIRP) -> NTSTATUS {
    let stack = IoGetCurrentIrpStackLocation(irp);
    let major = (*stack).MajorFunction;

    let status = match major {
        IRP_MJ_CREATE => handle_create(irp),
        IRP_MJ_CLOSE | IRP_MJ_CLEANUP => STATUS_SUCCESS,
        IRP_MJ_DEVICE_CONTROL | IRP_MJ_INTERNAL_DEVICE_CONTROL => {
            let minor = (*stack).MinorFunction;
            // Observe what AFD drives. NOT_IMPLEMENTED keeps it non-blocking
            // until the engine data path lands.
            nsprint!("[NETSTACK] TDI {} (minor {:#04x})\n", tdi::minor_name(minor), minor);
            STATUS_NOT_IMPLEMENTED
        }
        _ => STATUS_NOT_IMPLEMENTED,
    };

    (*irp).IoStatus.__bindgen_anon_1.Status = status;
    (*irp).IoStatus.Information = 0;
    complete_request(irp);
    status
}

/// Classify the CREATE by its EA buffer (TransportAddress / ConnectionContext /
/// none) and accept it so winsock socket()/bind() succeed.
unsafe fn handle_create(irp: PIRP) -> NTSTATUS {
    let ea = (*irp).AssociatedIrp.SystemBuffer as *const FILE_FULL_EA_INFORMATION;
    let kind = if ea.is_null() {
        TdiFileKind::ControlChannel
    } else {
        let name_len = (*ea).EaNameLength as usize;
        let name = core::slice::from_raw_parts((*ea).EaName.as_ptr() as *const u8, name_len);
        if name == tdi::EA_TRANSPORT_ADDRESS {
            TdiFileKind::Address
        } else if name == tdi::EA_CONNECTION_CONTEXT {
            TdiFileKind::Connection
        } else {
            TdiFileKind::ControlChannel
        }
    };
    nsprint!("[NETSTACK] TDI open: {:?} file object\n", kind);
    STATUS_SUCCESS
}

/// IoCompleteRequest(irp, IO_NO_INCREMENT) — the C macro aliases IofCompleteRequest.
unsafe fn complete_request(irp: PIRP) {
    use ntkrnl_sys::ddk::IofCompleteRequest;
    IofCompleteRequest(irp, 0);
}
