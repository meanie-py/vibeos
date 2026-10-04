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
const STATUS_INVALID_PARAMETER: NTSTATUS = 0xC000_000Du32 as NTSTATUS;

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
                // The TransportAddress EA value is a TRANSPORT_ADDRESS / TA_IP_ADDRESS
                // giving the local bind. Extract the port for the UDP datagram path
                // (0 = ephemeral, chosen by the worker on first use).
                let port = {
                    let name_len = (*ea).EaNameLength as usize;
                    let val_len = (*ea).EaValueLength as usize;
                    if val_len >= core::mem::size_of::<i32>() {
                        let val = (*ea).EaName.as_ptr().add(name_len + 1) as *const u8;
                        tdi::parse_ta(val).map(|(_, p)| p).unwrap_or(0)
                    } else {
                        0
                    }
                };
                tditrans::post(tditrans::Cmd::CreateAddr { id, port });
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
    // iphlpapi and wshtcpip open a fresh control channel for every entity query, so a
    // desktop that polls the network (the tray, rapps) logged one line per poll: 64 in a
    // single amd64 smoke. Say it once; address and connection opens stay one per socket.
    static CONTROL_LOGGED: core::sync::atomic::AtomicBool = core::sync::atomic::AtomicBool::new(false);
    if !matches!(kind, TdiFileKind::ControlChannel)
        || !CONTROL_LOGGED.swap(true, core::sync::atomic::Ordering::Relaxed)
    {
        nsprint!("[NETSTACK] TDI open: {:?} file object\n", kind);
    }
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

// ---- TDI query-information-ex (IOCTL_TCP_QUERY_INFORMATION_EX) ----
// A METHOD_NEITHER IOCTL (IRP_MJ_DEVICE_CONTROL, MinorFunction 0) — distinct from
// the TdiBuildXxx INTERNAL_DEVICE_CONTROL path. afd/wshtcpip/iphlpapi enumerate
// the transport's TDI entities through it on a control-channel handle. The one
// that gates usermode connect(): wshtcpip!WSHNotify(WSH_NOTIFY_BIND)
// (dll/win32/wshtcpip/wshtcpip.c) opens \Device\Tcp and calls tdiGetEntityIDSet;
// if that fails the helper returns WSAEINVAL and the auto-bind inside connect()
// fails with WSAEINVAL (10022) before any TDI_CONNECT is ever issued.
const IOCTL_TCP_QUERY_INFORMATION_EX: u32 = 0x0012_0003;

const GENERIC_ENTITY: u32 = 0;
const ENTITY_LIST_ID: u32 = 0;
const ENTITY_TYPE_ID: u32 = 1;
const INFO_CLASS_GENERIC: u32 = 0x100;

const IF_ENTITY: u32 = 0x200;
const AT_ENTITY: u32 = 0x280;
const CL_NL_ENTITY: u32 = 0x301;
const CO_TL_ENTITY: u32 = 0x400;
const CL_TL_ENTITY: u32 = 0x401;
const IF_GENERIC: u32 = 0x200;
const AT_ARP: u32 = 0x280;
const CL_NL_IP: u32 = 0x303;
const CO_TL_TCP: u32 = 0x404;
const CL_TL_UDP: u32 = 0x403;

// PROTOCOL-class MIB query ids (tcpioctl.h) — the iphlpapi management surface
// (ipconfig / GetAdaptersInfo / GetIpAddrTable / GetIpForwardTable / GetIpNetTable).
const INFO_CLASS_PROTOCOL: u32 = 0x200;
const IF_MIB_STATS_ID: u32 = 1; // on IF_ENTITY → IFEntry
const IP_MIB_STATS_ID: u32 = 1; // on CL_NL_ENTITY → IPSNMPInfo
const IP_MIB_ARPTABLE_ENTRY_ID: u32 = 0x101; // CL_NL → routes; AT → ARP
const IP_MIB_ADDRTABLE_ENTRY_ID: u32 = 0x102; // CL_NL → IPAddrEntry[]
const MIB_TCPSTATS_LEN: usize = 60; // MIB_TCPSTATS (tcpmib.h)
const MIB_UDPSTATS_LEN: usize = 20; // MIB_UDPSTATS (udpmib.h)

#[repr(C)]
#[derive(Clone, Copy)]
struct TdiEntityId {
    tei_entity: u32,
    tei_instance: u32,
}

#[repr(C)]
struct TdiObjectId {
    toi_entity: TdiEntityId,
    toi_class: u32,
    toi_type: u32,
    toi_id: u32,
}

/// The TDI entities netstack advertises: TCP (connection-oriented transport),
/// UDP (connectionless transport), IP (connectionless network) and one
/// interface — all instance 0 (single smoltcp interface).
const ENTITIES: [TdiEntityId; 5] = [
    TdiEntityId { tei_entity: CO_TL_ENTITY, tei_instance: 0 },
    TdiEntityId { tei_entity: CL_TL_ENTITY, tei_instance: 0 },
    TdiEntityId { tei_entity: CL_NL_ENTITY, tei_instance: 0 },
    TdiEntityId { tei_entity: IF_ENTITY, tei_instance: 0 },
    // ARP/address-translation entity — lets iphlpapi's hasArp() detect the
    // adapter supports ARP (GetAdaptersInfo enumeration) + GetIpNetTable.
    TdiEntityId { tei_entity: AT_ENTITY, tei_instance: 0 },
];

/// Handle IOCTL_TCP_QUERY_INFORMATION_EX (METHOD_NEITHER): input is a
/// TCP_REQUEST_QUERY_INFORMATION_EX in Type3InputBuffer, output goes to
/// Irp->UserBuffer, and the byte count rides back in IoStatus.Information.
/// Runs synchronously in the caller's thread context, so the usermode buffers
/// are valid here.
unsafe fn handle_query_info_ex(irp: PIRP, stack: PIO_STACK_LOCATION) -> NTSTATUS {
    let dic = &(*stack).Parameters.DeviceIoControl;
    let in_buf = dic.Type3InputBuffer as *const TdiObjectId;
    let in_len = dic.InputBufferLength as usize;
    let out_buf = (*irp).UserBuffer as *mut u8;
    let out_len = dic.OutputBufferLength as usize;

    // Only the query selector (TDIObjectID) is required as input. The output may
    // legitimately be NULL/0: sdk/lib/tdilib/enum.c queries with a zero-length
    // output FIRST purely to learn the size (Iosb.Information), then allocates
    // and re-queries. So a NULL output is a size-probe, NOT an error.
    if in_buf.is_null() || in_len < core::mem::size_of::<TdiObjectId>() {
        complete_now(irp, STATUS_INVALID_PARAMETER, 0);
        return STATUS_INVALID_PARAMETER;
    }

    let id = core::ptr::read_unaligned(in_buf);
    let ent = id.toi_entity.tei_entity;

    // ENTITY_LIST — the full entity-id array (what wshtcpip's bind path needs).
    if id.toi_class == INFO_CLASS_GENERIC && id.toi_id == ENTITY_LIST_ID && ent == GENERIC_ENTITY {
        let bytes = core::slice::from_raw_parts(
            ENTITIES.as_ptr() as *const u8,
            core::mem::size_of::<TdiEntityId>() * ENTITIES.len(),
        );
        return complete_query(irp, out_buf, out_len, bytes);
    }

    // ENTITY_TYPE — the concrete protocol type for a given entity.
    if id.toi_class == INFO_CLASS_GENERIC && id.toi_id == ENTITY_TYPE_ID {
        let ty: u32 = match ent {
            CO_TL_ENTITY => CO_TL_TCP,
            CL_TL_ENTITY => CL_TL_UDP,
            CL_NL_ENTITY => CL_NL_IP,
            IF_ENTITY => IF_GENERIC,
            AT_ENTITY => AT_ARP,
            _ => 0,
        };
        return complete_query(irp, out_buf, out_len, &ty.to_ne_bytes());
    }

    // PROTOCOL-class MIB queries — the iphlpapi management plane. Each returns a
    // fixed struct (or a size-probe count for the table queries) via complete_query.
    if id.toi_class == INFO_CLASS_PROTOCOL {
        match (ent, id.toi_id) {
            (IF_ENTITY, IF_MIB_STATS_ID) => return mib_if_entry(irp, out_buf, out_len),
            (CL_NL_ENTITY, IP_MIB_STATS_ID) => return mib_ip_stats(irp, out_buf, out_len),
            (CL_NL_ENTITY, IP_MIB_ADDRTABLE_ENTRY_ID) => {
                return mib_ip_addrtable(irp, out_buf, out_len)
            }
            (CL_NL_ENTITY, IP_MIB_ARPTABLE_ENTRY_ID) => {
                return mib_ip_routetable(irp, out_buf, out_len)
            }
            (AT_ENTITY, IP_MIB_ARPTABLE_ENTRY_ID) => {
                // ARP table: empty (smoltcp owns its neighbour cache internally and
                // doesn't expose it; an empty GetIpNetTable is valid).
                return complete_query(irp, out_buf, out_len, &[]);
            }
            (CO_TL_ENTITY, 1) => return complete_query(irp, out_buf, out_len, &[0u8; MIB_TCPSTATS_LEN]),
            (CL_TL_ENTITY, 1) => return complete_query(irp, out_buf, out_len, &[0u8; MIB_UDPSTATS_LEN]),
            _ => {}
        }
    }

    // IP MIB stats / address table / per-protocol classes — not yet implemented.
    // Logged so the next blocker (if any) surfaces during boot.
    nsprint!(
        "[NETSTACK] QUERY_INFO_EX unhandled ent={:#x} class={:#x} type={:#x} id={:#x}\n",
        ent, id.toi_class, id.toi_type, id.toi_id
    );
    complete_now(irp, STATUS_NOT_IMPLEMENTED, 0);
    STATUS_NOT_IMPLEMENTED
}

/// Complete an IOCTL_TCP_QUERY_INFORMATION_EX query. On a size-probe (output
/// too small or NULL) report the required byte count via IoStatus.Information
/// with STATUS_SUCCESS — tdilib's enum.c needs NT_SUCCESS to read the size and
/// re-query. When the buffer fits, copy the payload. Both the probe and the
/// real call report the same `need`, which is how tdilib's stabilization loop
/// detects it got everything.
unsafe fn complete_query(irp: PIRP, out_buf: *mut u8, out_len: usize, data: &[u8]) -> NTSTATUS {
    let need = data.len();
    if !out_buf.is_null() && out_len >= need {
        core::ptr::copy_nonoverlapping(data.as_ptr(), out_buf, need);
    }
    complete_now(irp, STATUS_SUCCESS, need);
    STATUS_SUCCESS
}

// ---- MIB response structs (tcpioctl.h). All natural-aligned ULONG/UCHAR, so
// repr(C) matches the C layout exactly; fields are serialized whole via as_bytes,
// hence #[allow(dead_code)] (rustc can't see the byte-cast read).

unsafe fn as_bytes<T>(v: &T) -> &[u8] {
    core::slice::from_raw_parts(v as *const T as *const u8, core::mem::size_of::<T>())
}

#[repr(C)]
#[allow(dead_code)]
struct IfEntryFixed {
    if_index: u32,
    if_type: u32,
    if_mtu: u32,
    if_speed: u32,
    if_physaddrlen: u32,
    if_physaddr: [u8; 8],
    if_adminstatus: u32,
    if_operstatus: u32,
    if_lastchange: u32,
    if_inoctets: u32,
    if_inucastpkts: u32,
    if_innucastpkts: u32,
    if_indiscards: u32,
    if_inerrors: u32,
    if_inunknownprotos: u32,
    if_outoctets: u32,
    if_outucastpkts: u32,
    if_outnucastpkts: u32,
    if_outdiscards: u32,
    if_outerrors: u32,
    if_outqlen: u32,
    if_descrlen: u32,
    // if_descr[if_descrlen] is appended after this fixed part (92 bytes).
}

#[repr(C)]
#[allow(dead_code)]
struct IpSnmpInfo {
    ipsi_forwarding: u32,
    ipsi_defaultttl: u32,
    ipsi_inreceives: u32,
    ipsi_inhdrerrors: u32,
    ipsi_inaddrerrors: u32,
    ipsi_forwdatagrams: u32,
    ipsi_inunknownprotos: u32,
    ipsi_indiscards: u32,
    ipsi_indelivers: u32,
    ipsi_outrequests: u32,
    ipsi_routingdiscards: u32,
    ipsi_outdiscards: u32,
    ipsi_outnoroutes: u32,
    ipsi_reasmtimeout: u32,
    ipsi_reasmreqds: u32,
    ipsi_reasmoks: u32,
    ipsi_reasmfails: u32,
    ipsi_fragoks: u32,
    ipsi_fragfails: u32,
    ipsi_fragcreates: u32,
    ipsi_numif: u32,
    ipsi_numaddr: u32,
    ipsi_numroutes: u32,
}

#[repr(C)]
#[allow(dead_code)]
struct IpAddrEntry {
    iae_addr: u32,
    iae_index: u32,
    iae_mask: u32,
    iae_bcastaddr: u32,
    iae_reasmsize: u32,
    iae_context: u16,
    iae_pad: u16,
}

#[repr(C)]
#[allow(dead_code)]
struct IpRouteEntry {
    ire_dest: u32,
    ire_index: u32,
    ire_metric1: u32,
    ire_metric2: u32,
    ire_metric3: u32,
    ire_metric4: u32,
    ire_gw: u32,
    ire_type: u32,
    ire_proto: u32,
    ire_age: u32,
    ire_mask: u32,
    ire_metric5: u32,
    ire_info: u32,
}

// Lock the MIB struct layouts at compile time (all 3 arches) — iphlpapi reads
// these by exact field offset, so a layout drift would mangle ipconfig output.
const _: () = assert!(core::mem::size_of::<IfEntryFixed>() == 92);
const _: () = assert!(core::mem::size_of::<IpSnmpInfo>() == 92);
const _: () = assert!(core::mem::size_of::<IpAddrEntry>() == 24);
const _: () = assert!(core::mem::size_of::<IpRouteEntry>() == 52);

/// IFEntry — the adapter's interface MIB (ifenum reads index/type/mtu/physaddr/descr).
unsafe fn mib_if_entry(irp: PIRP, out_buf: *mut u8, out_len: usize) -> NTSTATUS {
    let mac = crate::netcfg::mac();
    const DESCR: &[u8] = b"netstack smoltcp adapter";
    let e = IfEntryFixed {
        if_index: 1,
        if_type: 6, // MIB_IF_TYPE_ETHERNET
        if_mtu: 1500,
        if_speed: 100_000_000,
        if_physaddrlen: 6,
        if_physaddr: [mac[0], mac[1], mac[2], mac[3], mac[4], mac[5], 0, 0],
        if_adminstatus: 1,
        if_operstatus: 5, // IF_OPER_STATUS_OPERATIONAL
        if_lastchange: 0,
        if_inoctets: 0,
        if_inucastpkts: 0,
        if_innucastpkts: 0,
        if_indiscards: 0,
        if_inerrors: 0,
        if_inunknownprotos: 0,
        if_outoctets: 0,
        if_outucastpkts: 0,
        if_outnucastpkts: 0,
        if_outdiscards: 0,
        if_outerrors: 0,
        if_outqlen: 0,
        if_descrlen: DESCR.len() as u32,
    };
    let fixed = core::mem::size_of::<IfEntryFixed>();
    let mut buf = [0u8; core::mem::size_of::<IfEntryFixed>() + 32];
    core::ptr::copy_nonoverlapping(&e as *const _ as *const u8, buf.as_mut_ptr(), fixed);
    let dl = DESCR.len().min(buf.len() - fixed);
    core::ptr::copy_nonoverlapping(DESCR.as_ptr(), buf.as_mut_ptr().add(fixed), dl);
    complete_query(irp, out_buf, out_len, &buf[..fixed + dl])
}

/// IPSNMPInfo — IP protocol stats; iphlpapi reads numif/numaddr/numroutes to size
/// the address + route table queries.
unsafe fn mib_ip_stats(irp: PIRP, out_buf: *mut u8, out_len: usize) -> NTSTATUS {
    let n = if crate::netcfg::configured() { 1 } else { 0 };
    let mut info: IpSnmpInfo = core::mem::zeroed();
    info.ipsi_forwarding = 2; // not forwarding
    info.ipsi_defaultttl = 128;
    info.ipsi_numif = 1;
    info.ipsi_numaddr = n;
    info.ipsi_numroutes = n;
    complete_query(irp, out_buf, out_len, as_bytes(&info))
}

/// IPAddrEntry[] — the IP address table (one entry: our DHCP address on if 1).
unsafe fn mib_ip_addrtable(irp: PIRP, out_buf: *mut u8, out_len: usize) -> NTSTATUS {
    if !crate::netcfg::configured() {
        return complete_query(irp, out_buf, out_len, &[]);
    }
    let e = IpAddrEntry {
        iae_addr: crate::netcfg::ipv4_naddr(),
        iae_index: 1,
        iae_mask: crate::netcfg::mask_naddr(),
        iae_bcastaddr: crate::netcfg::broadcast_naddr(),
        iae_reasmsize: 0xffff,
        iae_context: 0,
        iae_pad: 0,
    };
    complete_query(irp, out_buf, out_len, as_bytes(&e))
}

/// IPRouteEntry[] — the route table (one default route via the DHCP gateway).
unsafe fn mib_ip_routetable(irp: PIRP, out_buf: *mut u8, out_len: usize) -> NTSTATUS {
    if !crate::netcfg::configured() || crate::netcfg::gateway_naddr() == 0 {
        return complete_query(irp, out_buf, out_len, &[]);
    }
    let mut e: IpRouteEntry = core::mem::zeroed();
    e.ire_dest = 0; // 0.0.0.0 default
    e.ire_index = 1;
    e.ire_metric1 = 1;
    e.ire_gw = crate::netcfg::gateway_naddr();
    e.ire_type = 4; // MIB_IPROUTE_TYPE_INDIRECT (remote)
    e.ire_proto = 3; // MIB_IPPROTO_NETMGMT
    e.ire_mask = 0; // 0.0.0.0 mask for the default route
    complete_query(irp, out_buf, out_len, as_bytes(&e))
}

/// Handle IOCTL_TCP_SET_INFORMATION_EX (METHOD_BUFFERED). Input is a
/// TCP_REQUEST_SET_INFORMATION_EX (TDIObjectID + BufferSize + Buffer[]) in
/// SystemBuffer. The transport's management writes ride here: iphlpapi route
/// add/del (an IPRouteEntry on the IP entity) and wshtcpip sockopts. netstack's
/// engine is authoritative for its own L2/L3 config (it DHCPs + self-routes), so
/// we reprogram the default route when asked and acknowledge the rest cleanly —
/// NOT failing, because AFD/wshtcpip flush these best-effort and a hard error
/// would surface spuriously to setsockopt()/route callers.
unsafe fn handle_set_info_ex(irp: PIRP, stack: PIO_STACK_LOCATION) -> NTSTATUS {
    let dic = &(*stack).Parameters.DeviceIoControl;
    let in_len = dic.InputBufferLength as usize;
    let sysbuf = (*irp).AssociatedIrp.SystemBuffer as *const u8;

    // TCP_REQUEST_SET_INFORMATION_EX header = TDIObjectID(20) + BufferSize(4).
    const HDR: usize = core::mem::size_of::<TdiObjectId>() + core::mem::size_of::<u32>();
    if sysbuf.is_null() || in_len < HDR {
        complete_now(irp, STATUS_INVALID_PARAMETER, 0);
        return STATUS_INVALID_PARAMETER;
    }
    let id = core::ptr::read_unaligned(sysbuf as *const TdiObjectId);
    let buf_size =
        core::ptr::read_unaligned(sysbuf.add(core::mem::size_of::<TdiObjectId>()) as *const u32)
            as usize;
    let buf = sysbuf.add(HDR);
    let avail = in_len - HDR;

    // Route programming: an IPRouteEntry on the IP network-layer entity.
    if id.toi_class == INFO_CLASS_PROTOCOL
        && id.toi_entity.tei_entity == CL_NL_ENTITY
        && id.toi_id == IP_MIB_ARPTABLE_ENTRY_ID
        && buf_size >= core::mem::size_of::<IpRouteEntry>()
        && avail >= core::mem::size_of::<IpRouteEntry>()
    {
        let rte = core::ptr::read_unaligned(buf as *const IpRouteEntry);
        // Only the default route (0.0.0.0/0) maps cleanly onto smoltcp's single
        // default route; a more specific route is acked but not installed (slirp
        // is a single-gateway mini-internet, so the default route is all the
        // engine needs). gw octets come straight from the network-order in_addr.
        if rte.ire_dest == 0 && rte.ire_mask == 0 {
            let gw = rte.ire_gw.to_ne_bytes();
            match rte.ire_type {
                tdi::IP_FORWARD_ADD => {
                    tditrans::post(tditrans::Cmd::SetDefaultRoute { gw, add: true })
                }
                tdi::IP_FORWARD_DEL => {
                    tditrans::post(tditrans::Cmd::SetDefaultRoute { gw, add: false })
                }
                _ => {}
            }
        }
        complete_now(irp, STATUS_SUCCESS, 0);
        return STATUS_SUCCESS;
    }

    // Sockopts + any other well-formed SET: netstack doesn't plumb transport
    // sockopts down to smoltcp yet (roadmap M12+), but acknowledge them so
    // setsockopt(TCP_NODELAY / IP_TTL / ...) succeeds rather than erroring.
    nsprint!(
        "[NETSTACK] SET_INFO_EX ent={:#x} class={:#x} type={:#x} id={:#x} ({} bytes) acked\n",
        id.toi_entity.tei_entity, id.toi_class, id.toi_type, id.toi_id, buf_size
    );
    complete_now(irp, STATUS_SUCCESS, 0);
    STATUS_SUCCESS
}

/// Handle IOCTL_ICMP_ECHO_REQUEST (METHOD_BUFFERED) on \Device\Ip. Input and
/// output share `AssociatedIrp.SystemBuffer`; the request header is parsed here,
/// the actual echo + reply is plumbed cross-thread to the smoltcp worker (which
/// alone owns the ICMP socket), and the IRP pends until a reply or the timeout.
/// `UserBuffer` is captured so the worker can compute the reply's `Data`
/// back-pointer into the caller's address space (a value, never dereferenced).
unsafe fn handle_icmp_echo(irp: PIRP, stack: PIO_STACK_LOCATION) -> NTSTATUS {
    let dic = &(*stack).Parameters.DeviceIoControl;
    let in_len = dic.InputBufferLength as usize;
    let out_len = dic.OutputBufferLength as usize;
    let sysbuf = (*irp).AssociatedIrp.SystemBuffer as *const u8;
    let user_buffer = (*irp).UserBuffer as usize;

    // Match the C transport's DispEchoRequest validation exactly.
    if sysbuf.is_null()
        || out_len < core::mem::size_of::<tdi::IcmpEchoReply>()
        || in_len < core::mem::size_of::<tdi::IcmpEchoRequest>()
    {
        complete_now(irp, STATUS_INVALID_PARAMETER, 0);
        return STATUS_INVALID_PARAMETER;
    }

    let req = &*(sysbuf as *const tdi::IcmpEchoRequest);
    let data_off = req.data_offset as usize;
    let data_size = req.data_size as usize;
    let dst = req.address.to_ne_bytes(); // IPAddr network-order octets a.b.c.d
    let timeout_ms = req.timeout;
    // DataOffset + DataSize must stay inside the (copied-in) input buffer.
    if data_off.checked_add(data_size).map(|e| e > in_len).unwrap_or(true) {
        complete_now(irp, STATUS_INVALID_PARAMETER, 0);
        return STATUS_INVALID_PARAMETER;
    }

    // Copy the request payload out of SystemBuffer now: the worker emits it on a
    // later poll iteration, and SystemBuffer is the *output* the worker overwrites.
    let mut data = alloc::vec::Vec::with_capacity(data_size);
    data.extend_from_slice(core::slice::from_raw_parts(sysbuf.add(data_off), data_size));

    nsprint!(
        "[NETSTACK] ICMP echo -> {}.{}.{}.{} ({} bytes, timeout {}ms)\n",
        dst[0], dst[1], dst[2], dst[3], data_size, timeout_ms
    );
    IoMarkIrpPending(irp);
    tditrans::post(tditrans::Cmd::IcmpEcho {
        irp: irp as usize,
        sysbuf: sysbuf as usize,
        out_len,
        user_buffer,
        dst,
        timeout_ms,
        data,
    });
    STATUS_PENDING
}

/// Route a TDI request (IRP_MJ_INTERNAL_DEVICE_CONTROL, MinorFunction) into the
/// data path. Connect/Send/Receive/Disconnect pend (worker completes); the rest
/// complete synchronously.
unsafe fn handle_tdi(irp: PIRP, stack: PIO_STACK_LOCATION) -> NTSTATUS {
    let fo = (*stack).FileObject;
    let id = if fo.is_null() { 0 } else { (*fo).FsContext as usize as u64 };
    let minor = (*stack).MinorFunction;
    let params = &(*stack).Parameters as *const _ as *const u8;

    // IRP_MJ_DEVICE_CONTROL carries TDI IOCTLs (METHOD_NEITHER), where the
    // operation rides in IoControlCode and MinorFunction is 0 — not a TDI minor.
    if (*stack).MajorFunction == IRP_MJ_DEVICE_CONTROL
        && (*stack).Parameters.DeviceIoControl.IoControlCode == IOCTL_TCP_QUERY_INFORMATION_EX
    {
        return handle_query_info_ex(irp, stack);
    }

    // ICMP echo (ping) — \Device\Ip, METHOD_BUFFERED. iphlpapi's IcmpSendEcho2.
    if (*stack).MajorFunction == IRP_MJ_DEVICE_CONTROL
        && (*stack).Parameters.DeviceIoControl.IoControlCode == tdi::IOCTL_ICMP_ECHO_REQUEST
    {
        return handle_icmp_echo(irp, stack);
    }

    // Management-plane SET (route/sockopt programming) — METHOD_BUFFERED.
    // iphlpapi's createIpForwardEntry/deleteIpForwardEntry + wshtcpip sockopts.
    if (*stack).MajorFunction == IRP_MJ_DEVICE_CONTROL
        && (*stack).Parameters.DeviceIoControl.IoControlCode == tdi::IOCTL_TCP_SET_INFORMATION_EX
    {
        return handle_set_info_ex(irp, stack);
    }

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
        tdi::TDI_DISASSOCIATE_ADDRESS => {
            // AFD issues this when tearing a connection down; the connection
            // object's own CLOSE (CloseConn) already releases worker state, so
            // just acknowledge it cleanly rather than failing the cleanup path.
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
        tdi::TDI_LISTEN => {
            // Server-side: AFD pre-posts this (after associate+bind) and it pends
            // until an inbound connection is established. ReturnConnectionInformation
            // receives the peer address on completion.
            let p = params as *const tdi::TDI_REQUEST_KERNEL;
            let ret_info = (*p).ReturnConnectionInformation as usize;
            nsprint!("[NETSTACK] TDI LISTEN conn {}\n", id);
            IoMarkIrpPending(irp);
            tditrans::post(tditrans::Cmd::Listen { conn_id: id, irp: irp as usize, ret_info });
            STATUS_PENDING
        }
        tdi::TDI_ACCEPT => {
            // smoltcp completes the handshake during listen, so by the time AFD
            // accepts, the connection is already established + bound to this conn
            // object. Acknowledge synchronously.
            nsprint!("[NETSTACK] TDI ACCEPT conn {}\n", id);
            complete_now(irp, STATUS_SUCCESS, 0);
            STATUS_SUCCESS
        }
        tdi::TDI_QUERY_INFORMATION => {
            // getsockname: AfdGetSockName issues TDI_QUERY_ADDRESS_INFO and reads
            // the reply (a TDI_ADDRESS_INFO) from the IRP's MDL.
            let p = params as *const tdi::TDI_REQUEST_KERNEL_QUERY;
            if (*p).QueryType == tdi::TDI_QUERY_ADDRESS_INFO {
                let va = mdl_va(irp);
                if va == 0 {
                    complete_now(irp, STATUS_INVALID_PARAMETER, 0);
                    return STATUS_INVALID_PARAMETER;
                }
                IoMarkIrpPending(irp);
                tditrans::post(tditrans::Cmd::QueryAddress {
                    id, irp: irp as usize, out: va, out_len: mdl_len(irp),
                });
                STATUS_PENDING
            } else {
                nsprint!("[NETSTACK] TDI QUERY_INFORMATION type {} unhandled\n", (*p).QueryType);
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
        tdi::TDI_SEND_DATAGRAM => {
            // Connectionless send — rides on the ADDRESS object (id = addr id). The
            // destination is in SendDatagramInformation->RemoteAddress.
            let p = params as *const tdi::TDI_REQUEST_KERNEL_SENDDG;
            let len = (*p).SendLength as usize;
            let va = mdl_va(irp);
            if va == 0 {
                complete_now(irp, STATUS_NOT_IMPLEMENTED, 0);
                return STATUS_NOT_IMPLEMENTED;
            }
            if let Some((ip, port)) = tdi::parse_remote_ip((*p).SendDatagramInformation) {
                nsprint!(
                    "[NETSTACK] TDI SEND_DATAGRAM addr {} -> {}.{}.{}.{}:{} ({} bytes)\n",
                    id, ip[0], ip[1], ip[2], ip[3], port, len
                );
                IoMarkIrpPending(irp);
                tditrans::post(tditrans::Cmd::SendDatagram {
                    addr_id: id, irp: irp as usize, data: va, len, ip, port,
                });
                STATUS_PENDING
            } else {
                complete_now(irp, STATUS_NOT_IMPLEMENTED, 0);
                STATUS_NOT_IMPLEMENTED
            }
        }
        tdi::TDI_RECEIVE_DATAGRAM => {
            let p = params as *const tdi::TDI_REQUEST_KERNEL_RECEIVEDG;
            let len = (*p).ReceiveLength as usize;
            let va = mdl_va(irp);
            if va == 0 {
                complete_now(irp, STATUS_NOT_IMPLEMENTED, 0);
                return STATUS_NOT_IMPLEMENTED;
            }
            IoMarkIrpPending(irp);
            tditrans::post(tditrans::Cmd::ReceiveDatagram {
                addr_id: id, irp: irp as usize, buf: va, len,
                ret_info: (*p).ReturnDatagramInformation as usize,
            });
            STATUS_PENDING
        }
        _ => {
            // For an unhandled IRP_MJ_DEVICE_CONTROL, surface the IOCTL code so the
            // next missing TDI IOCTL (if any) is visible in the boot log.
            let major = (*stack).MajorFunction;
            if major == IRP_MJ_DEVICE_CONTROL {
                let ioctl = (*stack).Parameters.DeviceIoControl.IoControlCode;
                nsprint!("[NETSTACK] TDI device-control ioctl {:#010x} unhandled\n", ioctl);
            } else {
                nsprint!("[NETSTACK] TDI {} (minor {:#04x}) unhandled\n", tdi::minor_name(minor), minor);
            }
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

/// Byte length of the IRP's data MDL (TDI_QUERY output buffer), or 0.
unsafe fn mdl_len(irp: PIRP) -> usize {
    let mdl = (*irp).MdlAddress;
    if mdl.is_null() {
        return 0;
    }
    (*(mdl as *const ntkrnl_sys::ddk::_MDL)).ByteCount as usize
}
