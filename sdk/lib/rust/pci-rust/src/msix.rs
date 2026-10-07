//! MSI-X for child devices: asking for the messages, and pointing them at the CPU.
//!
//! Windows' rules, kept: a device gets message interrupts only when its driver opts in
//! with `MSISupported=1` under its device key's
//! `Interrupt Management\MessageSignaledInterruptProperties` (what an INF's HKR AddReg
//! writes; `MessageNumberLimit` caps the count), and the driver then connects them with
//! `IoConnectInterruptEx(CONNECT_MESSAGE_BASED)`. Everyone else keeps the line
//! interrupt, so no existing driver sees a resource list it did not ask for.
//!
//! Where it diverges, and why: ReactOS asks for resource requirements when the device
//! is enumerated, before its driver's AddDevice could have written that opt-in, so the
//! messages are added in IRP_MN_FILTER_RESOURCE_REQUIREMENTS instead. The PnP manager
//! sends that after AddDevice, down through the driver to this PDO, which answers it
//! first.
//!
//! The pieces the requirements list cannot carry come from the HAL (see hal-rust's
//! message.rs): whether it can route messages at all, and the address/data that
//! raises each translated vector, which START writes into the device's MSI-X table.
//! MSI (capability 05h, one contiguous block of vectors) is not handled; every
//! device we drive has MSI-X.

use core::mem::size_of;
use core::ptr::addr_of_mut;

use ntkrnl_sys::ddk::*;
use ntkrnl_sys::STATUS_SUCCESS;

use crate::shared::PPDO_DEVICE_EXTENSION;

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

unsafe extern "system" {
    fn HalVibeosMessageCapacity() -> ULONG;
    fn HalVibeosMessageTarget(Vector: ULONG, Address: *mut u64, Data: *mut ULONG) -> BOOLEAN;
}

const BUS_DATA_TYPE_PCIConfiguration: BUS_DATA_TYPE = 4;
const PCI_STATUS_CAPABILITIES_LIST: u16 = 0x0010;
const PCI_CAPABILITIES_POINTER: usize = 0x34;
const PCI_CAPABILITY_ID_MSIX: u8 = 0x11;
// MSI-X capability: Message Control at +2, Table Offset/BIR at +4.
const MSIX_CONTROL: usize = 2;
const MSIX_TABLE: usize = 4;
const MSIX_CONTROL_ENABLE: u16 = 0x8000;
const MSIX_CONTROL_FUNCTION_MASK: u16 = 0x4000;
const MSIX_ENTRY_SIZE: usize = 16;
const PCI_ADDRESS_MEMORY_TYPE_64BIT: u32 = 0x4;

const CmResourceTypeInterrupt: u8 = 2;
const CmResourceTypeMemory: u8 = 3;
const CmResourceShareDeviceExclusive: u8 = 1;
const CM_RESOURCE_INTERRUPT_LATCHED: u16 = 0x0001;
pub const CM_RESOURCE_INTERRUPT_MESSAGE: u16 = 0x0002;
/// `CM_RESOURCE_INTERRUPT_MESSAGE_TOKEN`: message k is requested as this minus k.
const MESSAGE_TOKEN: u32 = 0xFFFF_FFFE;
const PLUGPLAY_REGKEY_DEVICE: ULONG = 1;
const KEY_READ: ACCESS_MASK = 0x20019;
const PagedPool: POOL_TYPE = 1;
const TAG_PCI: ULONG = u32::from_ne_bytes(*b"PCI0");

struct Msix {
    /// Config-space offset of the capability.
    cap: usize,
    entries: u32,
    bir: usize,
    table_offset: u32,
}

unsafe fn read_config(ext: PPDO_DEVICE_EXTENSION, buf: &mut [u8; 256]) -> bool {
    let dev = (*ext).PciDevice;
    HalGetBusDataByOffset(
        BUS_DATA_TYPE_PCIConfiguration,
        (*dev).BusNumber,
        (*dev).SlotNumber.u.AsULONG,
        buf.as_mut_ptr() as PVOID,
        0,
        256,
    ) == 256
}

unsafe fn write_config_u16(ext: PPDO_DEVICE_EXTENSION, offset: usize, value: u16) {
    let dev = (*ext).PciDevice;
    let mut v = value;
    HalSetBusDataByOffset(
        BUS_DATA_TYPE_PCIConfiguration,
        (*dev).BusNumber,
        (*dev).SlotNumber.u.AsULONG,
        &mut v as *mut u16 as PVOID,
        offset as ULONG,
        2,
    );
}

fn le16(cfg: &[u8; 256], off: usize) -> u16 {
    u16::from_le_bytes([cfg[off], cfg[off + 1]])
}
fn le32(cfg: &[u8; 256], off: usize) -> u32 {
    u32::from_le_bytes([cfg[off], cfg[off + 1], cfg[off + 2], cfg[off + 3]])
}

/// Walk the capability list of a type-0 header for MSI-X.
fn find_msix(cfg: &[u8; 256]) -> Option<Msix> {
    if cfg[0x0E] & 0x7F != 0 || le16(cfg, 0x06) & PCI_STATUS_CAPABILITIES_LIST == 0 {
        return None;
    }
    let mut off = (cfg[PCI_CAPABILITIES_POINTER] & 0xFC) as usize;
    // 48 = the most capabilities that fit in config space; guards a looped list.
    for _ in 0..48 {
        if off < 0x40 || off > 0xF4 {
            return None;
        }
        if cfg[off] == PCI_CAPABILITY_ID_MSIX {
            let table = le32(cfg, off + MSIX_TABLE);
            return Some(Msix {
                cap: off,
                entries: (le16(cfg, off + MSIX_CONTROL) & 0x7FF) as u32 + 1,
                bir: (table & 7) as usize,
                table_offset: table & !7,
            });
        }
        off = (cfg[off + 1] & 0xFC) as usize;
    }
    None
}

/// The driver's opt-in: `Some(limit)` when MSISupported=1 (limit = MessageNumberLimit,
/// or u32::MAX when absent), `None` otherwise.
unsafe fn opted_in(pdo: PDEVICE_OBJECT) -> Option<u32> {
    let mut device_key: HANDLE = core::ptr::null_mut();
    if IoOpenDeviceRegistryKey(pdo, PLUGPLAY_REGKEY_DEVICE, KEY_READ, &mut device_key) < 0 {
        return None;
    }
    let path: &[u16] = utf16!("Interrupt Management\\MessageSignaledInterruptProperties");
    let mut name: UNICODE_STRING = core::mem::zeroed();
    name.Buffer = path.as_ptr() as *mut u16;
    name.Length = (path.len() * 2) as u16;
    name.MaximumLength = name.Length;
    let mut oa: OBJECT_ATTRIBUTES = core::mem::zeroed();
    oa.Length = size_of::<OBJECT_ATTRIBUTES>() as ULONG;
    oa.RootDirectory = device_key;
    oa.ObjectName = &mut name;
    oa.Attributes = 0x240; // OBJ_CASE_INSENSITIVE | OBJ_KERNEL_HANDLE
    let mut key: HANDLE = core::ptr::null_mut();
    let status = ZwOpenKey(&mut key, KEY_READ, &mut oa);
    ZwClose(device_key);
    if status < 0 {
        return None;
    }
    let supported = query_dword(key, utf16!("MSISupported"));
    let limit = query_dword(key, utf16!("MessageNumberLimit"));
    ZwClose(key);
    (supported == Some(1)).then(|| limit.filter(|&l| l > 0).unwrap_or(u32::MAX))
}

unsafe fn query_dword(key: HANDLE, value: &[u16]) -> Option<u32> {
    let mut name: UNICODE_STRING = core::mem::zeroed();
    name.Buffer = value.as_ptr() as *mut u16;
    name.Length = (value.len() * 2) as u16;
    name.MaximumLength = name.Length;
    // KEY_VALUE_PARTIAL_INFORMATION: TitleIndex, Type, DataLength, Data[].
    let mut buf = [0u32; 8];
    let mut got: ULONG = 0;
    let status = ZwQueryValueKey(
        key,
        &mut name,
        _KEY_VALUE_INFORMATION_CLASS_KeyValuePartialInformation,
        buf.as_mut_ptr() as PVOID,
        size_of::<[u32; 8]>() as ULONG,
        &mut got,
    );
    (status >= 0 && buf[1] == 4 /* REG_DWORD */ && buf[2] == 4).then_some(buf[3])
}

/// IRP_MN_FILTER_RESOURCE_REQUIREMENTS at the PDO: append one message requirement per
/// MSI-X entry the device may use. Returns the status to complete the IRP with; on any
/// reason not to (no opt-in, no capability, no HAL support) the list passes through
/// untouched.
pub unsafe fn filter_requirements(DeviceObject: PDEVICE_OBJECT, Irp: PIRP, IrpSp: PIO_STACK_LOCATION) -> NTSTATUS {
    let ext = (*DeviceObject).DeviceExtension as PPDO_DEVICE_EXTENSION;
    let incoming = if (*Irp).IoStatus.Information != 0 {
        (*Irp).IoStatus.Information as PIO_RESOURCE_REQUIREMENTS_LIST
    } else {
        (*IrpSp).Parameters.FilterResourceRequirements.IoResourceRequirementList
    };
    let status = (*Irp).IoStatus.__bindgen_anon_1.Status;
    let Some(limit) = opted_in(DeviceObject) else {
        return status;
    };
    let dev = (*ext).PciDevice;
    let slot = (*dev).SlotNumber.u.AsULONG;
    if incoming.is_null() || (*incoming).AlternativeLists != 1 {
        DPRINT1!("MSI-X: slot %#x: no single requirements list to extend\n", slot);
        return status;
    }
    let list = addr_of_mut!((*incoming).List[0]);
    let descs = addr_of_mut!((*list).Descriptors[0]);
    for i in 0..(*list).Count as usize {
        if (*descs.add(i)).Type == CmResourceTypeInterrupt
            && (*descs.add(i)).Flags & CM_RESOURCE_INTERRUPT_MESSAGE != 0
        {
            return status; // already filtered (a rebalance re-sends the IRP)
        }
    }
    let mut cfg = [0u8; 256];
    if !read_config(ext, &mut cfg) {
        DPRINT1!("MSI-X: slot %#x: config space unreadable\n", slot);
        return status;
    }
    let Some(msix) = find_msix(&cfg) else {
        DPRINT1!("MSI-X: slot %#x: opted in, but the device has no MSI-X capability\n", slot);
        return status;
    };
    let capacity = HalVibeosMessageCapacity();
    let count = msix.entries.min(limit).min(capacity);
    if count == 0 {
        DPRINT1!("MSI-X: slot %#x: the HAL cannot route messages (capacity %u)\n", slot, capacity);
        return status;
    }
    DPRINT1!("MSI-X: slot %#x: requesting %u of %u table entries\n", slot, count, msix.entries);

    let old_size = (*incoming).ListSize as usize;
    let new_size = old_size + count as usize * size_of::<IO_RESOURCE_DESCRIPTOR>();
    let grown = ExAllocatePoolWithTag(PagedPool, new_size as SIZE_T, TAG_PCI) as PIO_RESOURCE_REQUIREMENTS_LIST;
    if grown.is_null() {
        return status;
    }
    core::ptr::copy_nonoverlapping(incoming as *const u8, grown as *mut u8, old_size);
    (*grown).ListSize = new_size as ULONG;
    let list = addr_of_mut!((*grown).List[0]);
    let first = addr_of_mut!((*list).Descriptors[0]).add((*list).Count as usize);
    for k in 0..count as usize {
        let d = first.add(k);
        core::ptr::write_bytes(d, 0, 1);
        (*d).Type = CmResourceTypeInterrupt;
        (*d).ShareDisposition = CmResourceShareDeviceExclusive;
        (*d).Flags = CM_RESOURCE_INTERRUPT_LATCHED | CM_RESOURCE_INTERRUPT_MESSAGE;
        (*d).u.Interrupt.MinimumVector = MESSAGE_TOKEN - k as u32;
        (*d).u.Interrupt.MaximumVector = MESSAGE_TOKEN - k as u32;
    }
    (*list).Count += count;
    // The list we replace is ours (QUERY_RESOURCE_REQUIREMENTS allocated it) or an
    // upper filter's that already handed it on; either way nothing else holds it.
    if (*Irp).IoStatus.Information != 0 {
        ExFreePool(incoming as PVOID);
    }
    (*Irp).IoStatus.Information = grown as ULONG_PTR;
    STATUS_SUCCESS
}

/// The bus-relative base of BAR `bir`.
fn bar_base(cfg: &[u8; 256], bir: usize) -> Option<u64> {
    if bir > 5 {
        return None;
    }
    let lo = le32(cfg, 0x10 + bir * 4);
    if lo & 1 != 0 {
        return None; // an I/O BAR cannot hold an MSI-X table
    }
    let mut base = (lo & !0xF) as u64;
    if lo & 0x6 == PCI_ADDRESS_MEMORY_TYPE_64BIT && bir < 5 {
        base |= (le32(cfg, 0x10 + (bir + 1) * 4) as u64) << 32;
    }
    Some(base)
}

/// IRP_MN_START_DEVICE: write each assigned message's address/data into the MSI-X table
/// and enable MSI-X. The raw descriptor's token says which table entry it is.
pub unsafe fn program(ext: PPDO_DEVICE_EXTENSION, raw: PCM_RESOURCE_LIST, translated: PCM_RESOURCE_LIST) {
    if raw.is_null() || translated.is_null() || (*raw).Count != 1 || (*translated).Count != 1 {
        return;
    }
    let rl = addr_of_mut!((*raw).List[0].PartialResourceList);
    let tl = addr_of_mut!((*translated).List[0].PartialResourceList);
    let n = (*rl).Count.min((*tl).Count) as usize;
    let rd = addr_of_mut!((*rl).PartialDescriptors[0]);
    let td = addr_of_mut!((*tl).PartialDescriptors[0]);
    if !(0..n).any(|i| {
        (*rd.add(i)).Type == CmResourceTypeInterrupt && (*rd.add(i)).Flags & CM_RESOURCE_INTERRUPT_MESSAGE != 0
    }) {
        return;
    }
    let mut cfg = [0u8; 256];
    if !read_config(ext, &mut cfg) {
        return;
    }
    let Some(msix) = find_msix(&cfg) else {
        return;
    };
    let Some(bar) = bar_base(&cfg, msix.bir) else {
        return;
    };
    // The table BAR's CPU address: the translated twin of the raw memory resource
    // that starts at the BAR.
    let Some(cpu) = (0..n).find_map(|i| {
        let r = rd.add(i);
        ((*r).Type == CmResourceTypeMemory && (*r).u.Memory.Start.QuadPart as u64 == bar)
            .then(|| (*td.add(i)).u.Memory.Start.QuadPart as u64)
    }) else {
        DPRINT1!("MSI-X table BAR %u has no assigned memory resource\n", msix.bir as u32);
        return;
    };
    let table_len = msix.entries as usize * MSIX_ENTRY_SIZE;
    let mut pa: PHYSICAL_ADDRESS = core::mem::zeroed();
    pa.QuadPart = (cpu + msix.table_offset as u64) as i64;
    let table = MmMapIoSpace(pa, table_len as SIZE_T, _MEMORY_CACHING_TYPE_MmNonCached) as *mut u32;
    if table.is_null() {
        return;
    }
    let mut programmed = 0u32;
    for i in 0..n {
        let r = rd.add(i);
        if (*r).Type != CmResourceTypeInterrupt || (*r).Flags & CM_RESOURCE_INTERRUPT_MESSAGE == 0 {
            continue;
        }
        let entry = MESSAGE_TOKEN.wrapping_sub((*r).u.Interrupt.Vector);
        let mut address = 0u64;
        let mut data: ULONG = 0;
        if entry >= msix.entries || HalVibeosMessageTarget((*td.add(i)).u.Interrupt.Vector, &mut address, &mut data) == 0 {
            continue;
        }
        let e = table.add(entry as usize * MSIX_ENTRY_SIZE / 4);
        core::ptr::write_volatile(e, address as u32);
        core::ptr::write_volatile(e.add(1), (address >> 32) as u32);
        core::ptr::write_volatile(e.add(2), data);
        core::ptr::write_volatile(e.add(3), 0); // vector control: unmasked
        programmed += 1;
    }
    MmUnmapIoSpace(table as PVOID, table_len as SIZE_T);
    if programmed == 0 {
        return;
    }
    let control = le16(&cfg, msix.cap + MSIX_CONTROL);
    write_config_u16(ext, msix.cap + MSIX_CONTROL, (control | MSIX_CONTROL_ENABLE) & !MSIX_CONTROL_FUNCTION_MASK);
    DPRINT1!(
        "MSI-X enabled for %04x:%04x: %u of %u table entries\n",
        le16(&cfg, 0) as u32,
        le16(&cfg, 2) as u32,
        programmed,
        msix.entries
    );
}
