//! MSI-X and MSI for child devices: asking for the messages, pointing them at the
//! CPU, and turning them off when the device stops.
//!
//! Windows' rules, kept: a device gets message interrupts only when its driver opts in
//! with `MSISupported=1` under its device key's
//! `Interrupt Management\MessageSignaledInterruptProperties` (what an INF's HKR AddReg
//! writes; `MessageNumberLimit` caps the count), and the driver then connects them with
//! `IoConnectInterruptEx(CONNECT_MESSAGE_BASED)`. Everyone else keeps the line
//! interrupt, so no existing driver sees a resource list it did not ask for. Where the
//! messages go is the device's `Interrupt Management\Affinity Policy` (DevicePolicy,
//! AssignmentSetOverride), carried on each message requirement for the HAL to honour.
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
//!
//! A device without MSI-X but with MSI (capability 05h) gets a block of messages: a
//! power of two, up to what its Multiple Message Capable field says. The device ORs
//! the message number into the low bits of one data value, so the block is one
//! requirement spanning TOKEN - (N - 1) to TOKEN, as on Windows, which the HAL
//! translates to consecutive values aligned to N (vectors on an APIC, SPIs on a
//! GICv2m frame, EventIDs on an ITS) and may shrink when it cannot find N together;
//! the raw descriptor's MessageCount says what the device got.

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
    fn HalVibeosMessageTarget(Vector: ULONG, RequesterId: ULONG, Address: *mut u64, Data: *mut ULONG) -> BOOLEAN;
}

const BUS_DATA_TYPE_PCIConfiguration: BUS_DATA_TYPE = 4;
const PCI_STATUS_CAPABILITIES_LIST: u16 = 0x0010;
const PCI_CAPABILITIES_POINTER: usize = 0x34;
const PCI_CAPABILITY_ID_MSI: u8 = 0x05;
const PCI_CAPABILITY_ID_MSIX: u8 = 0x11;
// MSI capability: Message Control at +2, Message Address at +4; with 64-bit addresses
// the upper half at +8 and Message Data at +0xC, else Message Data at +8.
const MSI_CONTROL: usize = 2;
const MSI_ADDRESS: usize = 4;
const MSI_CONTROL_ENABLE: u16 = 0x0001;
const MSI_CONTROL_MULTIPLE_CAPABLE: u16 = 0x000E;
const MSI_CONTROL_MULTIPLE_ENABLE: u16 = 0x0070;
const MSI_CONTROL_64BIT: u16 = 0x0080;
const MSI_CONTROL_PER_VECTOR_MASK: u16 = 0x0100;
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

unsafe fn write_config<T: Copy>(ext: PPDO_DEVICE_EXTENSION, offset: usize, value: T) {
    let dev = (*ext).PciDevice;
    let mut v = value;
    HalSetBusDataByOffset(
        BUS_DATA_TYPE_PCIConfiguration,
        (*dev).BusNumber,
        (*dev).SlotNumber.u.AsULONG,
        &mut v as *mut T as PVOID,
        offset as ULONG,
        size_of::<T>() as ULONG,
    );
}

fn le16(cfg: &[u8; 256], off: usize) -> u16 {
    u16::from_le_bytes([cfg[off], cfg[off + 1]])
}
fn le32(cfg: &[u8; 256], off: usize) -> u32 {
    u32::from_le_bytes([cfg[off], cfg[off + 1], cfg[off + 2], cfg[off + 3]])
}

/// Walk the capability list of a type-0 header for capability `id`.
fn find_capability(cfg: &[u8; 256], id: u8) -> Option<usize> {
    if cfg[0x0E] & 0x7F != 0 || le16(cfg, 0x06) & PCI_STATUS_CAPABILITIES_LIST == 0 {
        return None;
    }
    let mut off = (cfg[PCI_CAPABILITIES_POINTER] & 0xFC) as usize;
    // 48 = the most capabilities that fit in config space; guards a looped list.
    for _ in 0..48 {
        if off < 0x40 || off > 0xF4 {
            return None;
        }
        if cfg[off] == id {
            return Some(off);
        }
        off = (cfg[off + 1] & 0xFC) as usize;
    }
    None
}

fn find_msix(cfg: &[u8; 256]) -> Option<Msix> {
    let off = find_capability(cfg, PCI_CAPABILITY_ID_MSIX)?;
    let table = le32(cfg, off + MSIX_TABLE);
    Some(Msix {
        cap: off,
        entries: (le16(cfg, off + MSIX_CONTROL) & 0x7FF) as u32 + 1,
        bir: (table & 7) as usize,
        table_offset: table & !7,
    })
}

/// The MSI capability's offset, for a device without MSI-X (which wins when a device
/// has both, as on Windows).
fn find_msi(cfg: &[u8; 256]) -> Option<usize> {
    find_capability(cfg, PCI_CAPABILITY_ID_MSIX).is_none().then(|| find_capability(cfg, PCI_CAPABILITY_ID_MSI)).flatten()
}

/// Open `path` under the device's hardware key.
unsafe fn open_device_subkey(pdo: PDEVICE_OBJECT, path: &[u16]) -> Option<HANDLE> {
    let mut device_key: HANDLE = core::ptr::null_mut();
    if IoOpenDeviceRegistryKey(pdo, PLUGPLAY_REGKEY_DEVICE, KEY_READ, &mut device_key) < 0 {
        return None;
    }
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
    (status >= 0).then_some(key)
}

/// The driver's opt-in: `Some(limit)` when MSISupported=1 (limit = MessageNumberLimit,
/// or u32::MAX when absent), `None` otherwise.
unsafe fn opted_in(pdo: PDEVICE_OBJECT) -> Option<u32> {
    let key = open_device_subkey(pdo, utf16!("Interrupt Management\\MessageSignaledInterruptProperties"))?;
    let supported = query_dword(key, utf16!("MSISupported"));
    let limit = query_dword(key, utf16!("MessageNumberLimit"));
    ZwClose(key);
    (supported == Some(1)).then(|| limit.filter(|&l| l > 0).unwrap_or(u32::MAX))
}

/// The device's interrupt affinity policy: (IRQ_DEVICE_POLICY, AssignmentSetOverride).
/// No key is IrqPolicyMachineDefault.
unsafe fn affinity_policy(pdo: PDEVICE_OBJECT) -> (i32, KAFFINITY) {
    let Some(key) = open_device_subkey(pdo, utf16!("Interrupt Management\\Affinity Policy")) else {
        return (0, 0);
    };
    let policy = query_dword(key, utf16!("DevicePolicy")).unwrap_or(0) as i32;
    // REG_BINARY on Windows (a KAFFINITY's bytes); a DWORD or QWORD says the same.
    let set = query_value(key, utf16!("AssignmentSetOverride"))
        .filter(|&(ty, len, _)| matches!(ty, 3 | 4 | 11) && (len == 4 || len == 8))
        .map(|(_, _, v)| v as KAFFINITY)
        .unwrap_or(0);
    ZwClose(key);
    (policy, set)
}

unsafe fn query_dword(key: HANDLE, value: &[u16]) -> Option<u32> {
    query_value(key, value).and_then(|(ty, len, v)| (ty == 4 /* REG_DWORD */ && len == 4).then_some(v as u32))
}

/// A value of at most 8 bytes: (type, length, the bytes as a little-endian integer).
unsafe fn query_value(key: HANDLE, value: &[u16]) -> Option<(u32, u32, u64)> {
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
    if status < 0 || buf[2] > 8 {
        return None;
    }
    let high = if buf[2] > 4 { buf[4] as u64 } else { 0 };
    Some((buf[1], buf[2], buf[3] as u64 | high << 32))
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
    let entries = match (find_msix(&cfg), find_msi(&cfg)) {
        (Some(msix), _) => msix.entries,
        // Multiple Message Capable: log2 of the block, 5 (32) at most.
        (None, Some(cap)) => 1 << ((le16(&cfg, cap + MSI_CONTROL) & MSI_CONTROL_MULTIPLE_CAPABLE) >> 1).min(5),
        (None, None) => {
            DPRINT1!("MSI-X: slot %#x: opted in, but the device has neither MSI-X nor MSI\n", slot);
            return status;
        }
    };
    let capacity = HalVibeosMessageCapacity();
    let msix = find_msix(&cfg).is_some();
    let mut count = entries.min(limit).min(capacity);
    if !msix && count > 1 {
        count = 1 << (31 - count.leading_zeros()); // an MSI block is a power of two
    }
    if count == 0 {
        DPRINT1!("MSI-X: slot %#x: the HAL cannot route messages (capacity %u)\n", slot, capacity);
        return status;
    }
    let (policy, targeted) = affinity_policy(DeviceObject);
    if msix {
        DPRINT1!("MSI-X: slot %#x: requesting %u of %u table entries\n", slot, count, entries);
    } else {
        DPRINT1!("MSI: slot %#x: requesting a block of %u of the %u it can take\n", slot, count, entries);
    }
    // MSI-X: a requirement per table entry. MSI: one for the whole block.
    let (descriptors, span) = if msix { (count, 1) } else { (1, count) };
    if policy != 0 {
        DPRINT1!("MSI-X: slot %#x: affinity policy %u, processors %p\n", slot, policy as u32, targeted as PVOID);
    }

    let old_size = (*incoming).ListSize as usize;
    let new_size = old_size + descriptors as usize * size_of::<IO_RESOURCE_DESCRIPTOR>();
    let grown = ExAllocatePoolWithTag(PagedPool, new_size as SIZE_T, TAG_PCI) as PIO_RESOURCE_REQUIREMENTS_LIST;
    if grown.is_null() {
        return status;
    }
    core::ptr::copy_nonoverlapping(incoming as *const u8, grown as *mut u8, old_size);
    (*grown).ListSize = new_size as ULONG;
    let list = addr_of_mut!((*grown).List[0]);
    let first = addr_of_mut!((*list).Descriptors[0]).add((*list).Count as usize);
    for k in 0..descriptors as usize {
        let d = first.add(k);
        core::ptr::write_bytes(d, 0, 1);
        (*d).Type = CmResourceTypeInterrupt;
        (*d).ShareDisposition = CmResourceShareDeviceExclusive;
        (*d).Flags = CM_RESOURCE_INTERRUPT_LATCHED | CM_RESOURCE_INTERRUPT_MESSAGE;
        (*d).u.Interrupt.MinimumVector = MESSAGE_TOKEN - k as u32 - (span - 1);
        (*d).u.Interrupt.MaximumVector = MESSAGE_TOKEN - k as u32;
        (*d).u.Interrupt.AffinityPolicy = policy as _;
        (*d).u.Interrupt.TargetedProcessors = targeted;
    }
    (*list).Count += descriptors;
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
    // The requester ID the device's writes carry (a GICv3 ITS translates by it), with
    // its PCI segment above it: BusNumber is a PCI_SEGMENT_BUS_NUMBER.
    let dev = (*ext).PciDevice;
    let slot = (*dev).SlotNumber.u.AsULONG;
    let requester = ((*dev).BusNumber & 0xFF_FFFF) << 8 | (slot & 0x1F) << 3 | (slot >> 5) & 7;
    let Some(msix) = find_msix(&cfg) else {
        if let Some(cap) = find_msi(&cfg) {
            program_msi(ext, &cfg, cap, requester, (0..n).map(|i| (rd.add(i), td.add(i))));
        }
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
        if entry >= msix.entries || HalVibeosMessageTarget((*td.add(i)).u.Interrupt.Vector, requester, &mut address, &mut data) == 0 {
            continue;
        }
        let e = table.add(entry as usize * MSIX_ENTRY_SIZE / 4);
        core::ptr::write_volatile(e, address as u32);
        core::ptr::write_volatile(e.add(1), (address >> 32) as u32);
        core::ptr::write_volatile(e.add(2), data);
        core::ptr::write_volatile(e.add(3), 0); // vector control: unmasked
        // Read it back: a table the device does not answer for (its BAR not decoding,
        // or the wrong BAR) takes nothing, and its entries stay masked forever.
        if core::ptr::read_volatile(e.add(2)) != data || core::ptr::read_volatile(e.add(3)) & 1 != 0 {
            DPRINT1!(
                "MSI-X table entry %u at %p did not take the message (data reads %#x) - not enabled\n",
                entry,
                e as PVOID,
                core::ptr::read_volatile(e.add(2))
            );
            continue;
        }
        programmed += 1;
    }
    MmUnmapIoSpace(table as PVOID, table_len as SIZE_T);
    if programmed == 0 {
        return;
    }
    let control = le16(&cfg, msix.cap + MSIX_CONTROL);
    write_config(ext, msix.cap + MSIX_CONTROL, (control | MSIX_CONTROL_ENABLE) & !MSIX_CONTROL_FUNCTION_MASK);
    DPRINT1!(
        "MSI-X enabled for %04x:%04x: %u of %u table entries\n",
        le16(&cfg, 0) as u32,
        le16(&cfg, 2) as u32,
        programmed,
        msix.entries
    );
}

/// MSI: the block's first message into the capability, and enable it with Multiple
/// Message Enable = log2 of the block the PnP manager got for it (the raw
/// descriptor's MessageCount).
unsafe fn program_msi(
    ext: PPDO_DEVICE_EXTENSION,
    cfg: &[u8; 256],
    cap: usize,
    requester: u32,
    mut descriptors: impl Iterator<Item = (PCM_PARTIAL_RESOURCE_DESCRIPTOR, PCM_PARTIAL_RESOURCE_DESCRIPTOR)>,
) {
    let Some((r, t)) = descriptors.find(|&(r, _)| {
        (*r).Type == CmResourceTypeInterrupt
            && (*r).Flags & CM_RESOURCE_INTERRUPT_MESSAGE != 0
            && (*r).u.Interrupt.Vector == MESSAGE_TOKEN
    }) else {
        return;
    };
    let control = le16(cfg, cap + MSI_CONTROL);
    let count = ((*r).u.Interrupt.Level >> 16).max(1);
    let capable = 1 << ((control & MSI_CONTROL_MULTIPLE_CAPABLE) >> 1).min(5);
    if !count.is_power_of_two() || count > capable {
        DPRINT1!("MSI: %04x:%04x was given %u messages, and takes a block of at most %u - not enabled\n", le16(cfg, 0) as u32, le16(cfg, 2) as u32, count, capable);
        return;
    }
    let mut address = 0u64;
    let mut data: ULONG = 0;
    if HalVibeosMessageTarget((*t).u.Interrupt.Vector, requester, &mut address, &mut data) == 0 {
        return;
    }
    if data & (count - 1) != 0 {
        // The device would OR message numbers into bits that are not free.
        DPRINT1!("MSI: data %#x for a block of %u is not aligned - not enabled\n", data, count);
        return;
    }
    let wide = control & MSI_CONTROL_64BIT != 0;
    if !wide && address >> 32 != 0 {
        DPRINT1!(
            "MSI: %04x:%04x takes 32-bit addresses only, and its message goes to %p - not enabled\n",
            le16(cfg, 0) as u32,
            le16(cfg, 2) as u32,
            address as PVOID
        );
        return;
    }
    write_config(ext, cap + MSI_ADDRESS, address as u32);
    let data_at = if wide {
        write_config(ext, cap + MSI_ADDRESS + 4, (address >> 32) as u32);
        cap + 0x0C
    } else {
        cap + 0x08
    };
    write_config(ext, data_at, data as u16);
    if control & MSI_CONTROL_PER_VECTOR_MASK != 0 {
        write_config(ext, data_at + 4, 0u32); // Mask Bits: every message unmasked
    }
    let enable = (count.trailing_zeros() as u16) << 4;
    write_config(ext, cap + MSI_CONTROL, (control & !MSI_CONTROL_MULTIPLE_ENABLE) | enable | MSI_CONTROL_ENABLE);
    DPRINT1!(
        "MSI enabled for %04x:%04x: %u message(s), data %#x\n",
        le16(cfg, 0) as u32,
        le16(cfg, 2) as u32,
        count,
        data
    );
}

/// STOP, REMOVE, SURPRISE_REMOVAL: messages off, before the PnP manager hands the
/// vectors back to the HAL for another device. A device left enabled keeps writing
/// its old address/data, which then raise someone else's interrupt.
pub unsafe fn disable(ext: PPDO_DEVICE_EXTENSION) {
    let mut cfg = [0u8; 256];
    if !read_config(ext, &mut cfg) {
        return;
    }
    if let Some(msix) = find_msix(&cfg) {
        let control = le16(&cfg, msix.cap + MSIX_CONTROL);
        if control & MSIX_CONTROL_ENABLE != 0 {
            write_config(ext, msix.cap + MSIX_CONTROL, control & !MSIX_CONTROL_ENABLE);
            DPRINT1!("MSI-X disabled for %04x:%04x\n", le16(&cfg, 0) as u32, le16(&cfg, 2) as u32);
        }
    } else if let Some(cap) = find_msi(&cfg) {
        let control = le16(&cfg, cap + MSI_CONTROL);
        if control & MSI_CONTROL_ENABLE != 0 {
            write_config(ext, cap + MSI_CONTROL, control & !MSI_CONTROL_ENABLE);
            DPRINT1!("MSI disabled for %04x:%04x\n", le16(&cfg, 0) as u32, le16(&cfg, 2) as u32);
        }
    }
}
