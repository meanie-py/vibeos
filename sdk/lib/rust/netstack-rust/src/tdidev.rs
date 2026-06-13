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
    FILE_FULL_EA_INFORMATION, IoCreateDevice, IofCompleteRequest, NTSTATUS, PDEVICE_OBJECT,
    PDRIVER_OBJECT, PIO_STACK_LOCATION, PIRP, UNICODE_STRING,
};
// IoGetCurrentIrpStackLocation + IoMarkIrpPending are header FORCEINLINEs/macros,
// NOT ntoskrnl exports — the bindgen extern decls link undefined. Use the inlines.
use ntkrnl_sys::inline::tier1::{IoGetCurrentIrpStackLocation, IoMarkIrpPending};

use crate::serial::nsprint;
use crate::tdi::{self, TdiFileKind};
use crate::tditrans;

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
        // Clear DO_DEVICE_INITIALIZING (0x80) — IoCreateDevice sets it, and a
        // device left "initializing" cannot be opened (ZwCreateFile wedges).
        (*dev).Flags &= !0x0000_0080u32;
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

const STATUS_PENDING: NTSTATUS = 0x0000_0103;

// FsContext2 file-object kind tags.
const KIND_CONTROL: usize = 0;
const KIND_ADDR: usize = 1;
const KIND_CONN: usize = 2;

unsafe fn complete_now(irp: PIRP, status: NTSTATUS, info: usize) {
    (*irp).IoStatus.__bindgen_anon_1.Status = status;
    (*irp).IoStatus.Information = info as _;
    IofCompleteRequest(irp, 0);
}

unsafe extern "system" fn dispatch(_device: PDEVICE_OBJECT, irp: PIRP) -> NTSTATUS {
    let stack = IoGetCurrentIrpStackLocation(irp);
    match (*stack).MajorFunction {
        IRP_MJ_CREATE => {
            let s = handle_create(irp, stack);
            complete_now(irp, s, 0);
            s
        }
        IRP_MJ_CLOSE | IRP_MJ_CLEANUP => {
            handle_close(stack);
            complete_now(irp, STATUS_SUCCESS, 0);
            STATUS_SUCCESS
        }
        IRP_MJ_DEVICE_CONTROL | IRP_MJ_INTERNAL_DEVICE_CONTROL => handle_tdi(irp, stack),
        _ => {
            complete_now(irp, STATUS_NOT_IMPLEMENTED, 0);
            STATUS_NOT_IMPLEMENTED
        }
    }
}

/// Classify the CREATE by its EA buffer, stamp the file object's FsContext (id)
/// + FsContext2 (kind), and register the object with the worker.
unsafe fn handle_create(irp: PIRP, stack: PIO_STACK_LOCATION) -> NTSTATUS {
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

    let fo = (*stack).FileObject;
    if !fo.is_null() {
        let id = tditrans::alloc_id();
        let (kindtag, _) = match kind {
            TdiFileKind::Address => {
                tditrans::post(tditrans::Cmd::CreateAddr { id, port: 0 });
                (KIND_ADDR, ())
            }
            TdiFileKind::Connection => {
                // The ConnectionContext EA value is AFD's per-connection cookie. We
                // echo it back as ConnectionContext in the TDI_EVENT_RECEIVE upcall,
                // and AFD maps it to its FCB — 0 would break AFD's receive path.
                // The value sits right after the name + its NUL terminator.
                let conn_ctx = {
                    let name_len = (*ea).EaNameLength as usize;
                    let val_len = (*ea).EaValueLength as usize;
                    let val = (*ea).EaName.as_ptr().add(name_len + 1) as *const u8;
                    if val_len >= core::mem::size_of::<usize>() {
                        (val as *const usize).read_unaligned()
                    } else {
                        0
                    }
                };
                tditrans::post(tditrans::Cmd::CreateConn { id, conn_ctx });
                (KIND_CONN, ())
            }
            TdiFileKind::ControlChannel => (KIND_CONTROL, ()),
        };
        (*fo).FsContext = id as usize as *mut core::ffi::c_void;
        (*fo).FsContext2 = kindtag as *mut core::ffi::c_void;
    }
    nsprint!("[NETSTACK] TDI open: {:?} file object\n", kind);
    STATUS_SUCCESS
}

unsafe fn handle_close(stack: PIO_STACK_LOCATION) {
    let fo = (*stack).FileObject;
    if fo.is_null() {
        return;
    }
    let id = (*fo).FsContext as usize as u64;
    match (*fo).FsContext2 as usize {
        KIND_ADDR => tditrans::post(tditrans::Cmd::CloseAddr { id }),
        KIND_CONN => tditrans::post(tditrans::Cmd::CloseConn { id }),
        _ => {}
    }
}

/// Route a TDI request (IRP_MJ_INTERNAL_DEVICE_CONTROL, MinorFunction) into the
/// data path. Connect/Send/Receive/Disconnect pend (worker completes); the rest
/// complete synchronously.
unsafe fn handle_tdi(irp: PIRP, stack: PIO_STACK_LOCATION) -> NTSTATUS {
    let fo = (*stack).FileObject;
    let id = if fo.is_null() { 0 } else { (*fo).FsContext as usize as u64 };
    let minor = (*stack).MinorFunction;
    let params = &(*stack).Parameters as *const _ as *const u8;

    match minor {
        tdi::TDI_SET_EVENT_HANDLER => {
            let p = params as *const tdi::TDI_REQUEST_KERNEL_SET_EVENT;
            tditrans::post(tditrans::Cmd::SetEvent {
                addr_id: id,
                ev_type: (*p).EventType,
                handler: (*p).EventHandler as usize,
                context: (*p).EventContext as usize,
            });
            complete_now(irp, STATUS_SUCCESS, 0);
            STATUS_SUCCESS
        }
        tdi::TDI_ASSOCIATE_ADDRESS => {
            let p = params as *const tdi::TDI_REQUEST_KERNEL_ASSOCIATE;
            let addr_id = resolve_addr_handle((*p).AddressHandle);
            tditrans::post(tditrans::Cmd::Associate { conn_id: id, addr_id });
            complete_now(irp, STATUS_SUCCESS, 0);
            STATUS_SUCCESS
        }
        tdi::TDI_CONNECT => {
            let p = params as *const tdi::TDI_REQUEST_KERNEL;
            if let Some((ip, port)) = tdi::parse_remote_ip((*p).RequestConnectionInformation) {
                nsprint!(
                    "[NETSTACK] TDI CONNECT conn {} -> {}.{}.{}.{}:{}\n",
                    id, ip[0], ip[1], ip[2], ip[3], port
                );
                IoMarkIrpPending(irp);
                tditrans::post(tditrans::Cmd::Connect { conn_id: id, ip, port, irp: irp as usize });
                STATUS_PENDING
            } else {
                complete_now(irp, STATUS_NOT_IMPLEMENTED, 0);
                STATUS_NOT_IMPLEMENTED
            }
        }
        tdi::TDI_SEND => {
            let p = params as *const tdi::TDI_REQUEST_KERNEL_SEND;
            let len = (*p).SendLength as usize;
            let va = mdl_va(irp);
            if va == 0 {
                complete_now(irp, STATUS_NOT_IMPLEMENTED, 0);
                return STATUS_NOT_IMPLEMENTED;
            }
            IoMarkIrpPending(irp);
            tditrans::post(tditrans::Cmd::Send { conn_id: id, irp: irp as usize, data: va, len });
            STATUS_PENDING
        }
        tdi::TDI_RECEIVE => {
            let p = params as *const tdi::TDI_REQUEST_KERNEL_RECEIVE;
            let len = (*p).ReceiveLength as usize;
            let va = mdl_va(irp);
            if va == 0 {
                complete_now(irp, STATUS_NOT_IMPLEMENTED, 0);
                return STATUS_NOT_IMPLEMENTED;
            }
            IoMarkIrpPending(irp);
            tditrans::post(tditrans::Cmd::Receive { conn_id: id, irp: irp as usize, buf: va, len });
            STATUS_PENDING
        }
        tdi::TDI_DISCONNECT => {
            IoMarkIrpPending(irp);
            tditrans::post(tditrans::Cmd::Disconnect { conn_id: id, irp: irp as usize });
            STATUS_PENDING
        }
        _ => {
            nsprint!("[NETSTACK] TDI {} (minor {:#04x}) unhandled\n", tdi::minor_name(minor), minor);
            complete_now(irp, STATUS_NOT_IMPLEMENTED, 0);
            STATUS_NOT_IMPLEMENTED
        }
    }
}

/// Resolve a TDI address-object HANDLE to the worker's addr id (stored in the
/// referenced FILE_OBJECT's FsContext).
unsafe fn resolve_addr_handle(handle: *mut core::ffi::c_void) -> u64 {
    use ntkrnl_sys::ddk::{ObReferenceObjectByHandle, ObfDereferenceObject};
    let mut obj: *mut core::ffi::c_void = core::ptr::null_mut();
    // DesiredAccess 0, ObjectType null (any), KernelMode (0).
    let status = ObReferenceObjectByHandle(handle, 0, core::ptr::null_mut(), 0, &mut obj, core::ptr::null_mut());
    if status != 0 || obj.is_null() {
        return 0;
    }
    let fo = obj as *mut ntkrnl_sys::ddk::_FILE_OBJECT;
    let addr_id = (*fo).FsContext as usize as u64;
    ObfDereferenceObject(obj);
    addr_id
}

/// System VA of the IRP's data MDL (SEND/RECEIVE buffers), or 0.
unsafe fn mdl_va(irp: PIRP) -> usize {
    let mdl = (*irp).MdlAddress;
    if mdl.is_null() {
        return 0;
    }
    // NormalPagePriority = 16.
    ntkrnl_sys::inline::mm::MmGetSystemAddressForMdlSafe(mdl, 16) as usize
}
