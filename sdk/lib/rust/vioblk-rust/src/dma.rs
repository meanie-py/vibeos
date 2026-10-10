//! The PCI functions' DMA, through a DMA adapter that knows which device it is for.
//!
//! SCSIPORT finds our adapters by scanning the bus, not through PnP, so nobody hands
//! us a PDO to ask IoGetDmaAdapter with; the HAL gives an adapter for a requester ID
//! directly instead (HalVibeosGetDmaAdapter). Everything a function's device ever
//! reaches comes from it: its virtqueues and its transfer arena are common buffers.
//! That is what makes the driver DMA-remapping compatible, and when its service says
//! so (Parameters `DmaRemappingCompatible` = 1, as Windows reads it) it claims a
//! domain of its own for the function behind a translating SMMU (HalVibeosDmaAttach),
//! where nothing else is reachable. The bounce pool (`hal.rs`) never serves a PCI
//! function: its requests' headers and status live in the arena.

use core::ptr;
use core::sync::atomic::{AtomicBool, AtomicPtr, Ordering};

use ntkrnl_sys::ddk::{
    DEVICE_DESCRIPTION, DMA_ADAPTER, DMA_OPERATIONS, HANDLE, OBJECT_ATTRIBUTES, PHYSICAL_ADDRESS, PVOID,
    UNICODE_STRING, ZwClose, ZwOpenKey, ZwQueryValueKey, _KEY_VALUE_INFORMATION_CLASS_KeyValuePartialInformation,
};

extern "system" {
    fn HalVibeosGetDmaAdapter(RequesterId: u32, DeviceDescription: *mut DEVICE_DESCRIPTION, NumberOfMapRegisters: *mut u32) -> *mut DMA_ADAPTER;
    fn HalVibeosDmaAttach(RequesterId: u32, Strict: u8) -> u8;
}

/// HalVibeosDmaAttach: a domain of its own, claimed by a driver that found the device
/// outside PnP, so pci.sys starting the PDO later does not take it back.
const DMA_ATTACH_CLAIMED: u8 = 2;
const DEVICE_DESCRIPTION_VERSION2: u32 = 2;
/// `_INTERFACE_TYPE_PCIBus`.
const PCI_BUS: i32 = 5;

/// The service's `Parameters\DmaRemappingCompatible` = 1, read in DriverEntry.
static REMAPPING_COMPATIBLE: AtomicBool = AtomicBool::new(false);

/// The adapter HwFindAdapter is allocating for: virtio-drivers' `Hal::dma_alloc` has no
/// context, and adapters are found one at a time at PASSIVE_LEVEL.
static CURRENT: AtomicPtr<DMA_ADAPTER> = AtomicPtr::new(ptr::null_mut());

const DISK_COMMON_BUFFERS: usize = 64;
/// Every common buffer handed out, to free it by its address: (va, adapter, length).
static mut BUFFERS: [(usize, usize, u32); DISK_COMMON_BUFFERS] = [(0, 0, 0); DISK_COMMON_BUFFERS];

/// The requester ID of a segment-0 function (SCSIPORT scans no other segment).
pub fn requester(bus: u8, device: u8, function: u8) -> u32 {
    (bus as u32) << 8 | (device as u32) << 3 | function as u32
}

/// An ASCII literal as UTF-16, at compile time.
const fn wide<const N: usize>(s: &[u8; N]) -> [u16; N] {
    let mut out = [0u16; N];
    let mut i = 0;
    while i < N {
        out[i] = s[i] as u16;
        i += 1;
    }
    out
}

/// DriverEntry: read the service's Parameters\DmaRemappingCompatible.
pub unsafe fn read_parameters(registry_path: *const UNICODE_STRING) {
    const SUFFIX: [u16; 11] = wide(b"\\Parameters");
    const VALUE: [u16; 22] = wide(b"DmaRemappingCompatible");
    if registry_path.is_null() || (*registry_path).Buffer.is_null() {
        return;
    }
    let chars = (*registry_path).Length as usize / 2;
    let mut path = [0u16; 256];
    if chars + SUFFIX.len() > path.len() {
        return;
    }
    path[..chars].copy_from_slice(core::slice::from_raw_parts((*registry_path).Buffer, chars));
    path[chars..chars + SUFFIX.len()].copy_from_slice(&SUFFIX);
    let mut name: UNICODE_STRING = core::mem::zeroed();
    name.Buffer = path.as_mut_ptr();
    name.Length = ((chars + SUFFIX.len()) * 2) as u16;
    name.MaximumLength = name.Length;
    let mut oa: OBJECT_ATTRIBUTES = core::mem::zeroed();
    oa.Length = core::mem::size_of::<OBJECT_ATTRIBUTES>() as u32;
    oa.ObjectName = &mut name;
    oa.Attributes = 0x240; // OBJ_CASE_INSENSITIVE | OBJ_KERNEL_HANDLE
    let mut key: HANDLE = ptr::null_mut();
    if ZwOpenKey(&mut key, 0x20019 /* KEY_READ */, &mut oa) < 0 {
        return;
    }
    let mut value = VALUE;
    let mut value_name: UNICODE_STRING = core::mem::zeroed();
    value_name.Buffer = value.as_mut_ptr();
    value_name.Length = (VALUE.len() * 2) as u16;
    value_name.MaximumLength = value_name.Length;
    // KEY_VALUE_PARTIAL_INFORMATION: TitleIndex, Type, DataLength, then the data.
    let mut info = [0u32; 8];
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
    // REG_DWORD (4), 4 bytes.
    if status >= 0 && info[1] == 4 && info[2] == 4 {
        REMAPPING_COMPATIBLE.store(info[3] == 1, Ordering::Release);
    }
}

/// The function's DMA adapter, after claiming it a domain of its own when the service
/// is DMA-remapping compatible. Null when the HAL has none to give. Also whether it
/// claimed one (TRUE without an SMMU too: the HAL then has nothing to do, and says
/// what it did per requester itself).
pub unsafe fn adapter_for(requester: u32, max_transfer: u32) -> (*mut DMA_ADAPTER, bool) {
    let mut d: DEVICE_DESCRIPTION = core::mem::zeroed();
    d.Version = DEVICE_DESCRIPTION_VERSION2;
    d.Master = 1;
    d.ScatterGather = 1;
    // Its queues and arena are below 4 GiB, addressable whatever the device negotiated.
    d.Dma32BitAddresses = 1;
    d.InterfaceType = PCI_BUS as _;
    d.MaximumLength = max_transfer;
    let mut map_registers = 0u32;
    let strict = REMAPPING_COMPATIBLE.load(Ordering::Acquire);
    // Before the adapter maps anything: a domain made later would lack it.
    let claimed = strict && HalVibeosDmaAttach(requester, DMA_ATTACH_CLAIMED) != 0;
    (HalVibeosGetDmaAdapter(requester, &mut d, &mut map_registers), claimed)
}

/// `alloc` allocates from `adapter` while this lives (HwFindAdapter, for one function).
pub struct Current;

impl Current {
    pub fn set(adapter: *mut DMA_ADAPTER) -> Current {
        CURRENT.store(adapter, Ordering::Release);
        Current
    }
}

impl Drop for Current {
    fn drop(&mut self) {
        CURRENT.store(ptr::null_mut(), Ordering::Release);
    }
}

/// A zeroed common buffer of the current adapter: (va, its device address). None when
/// there is no current adapter; (null, 0) when it has no memory.
pub unsafe fn alloc(bytes: usize) -> Option<(*mut u8, u64)> {
    let adapter = CURRENT.load(Ordering::Acquire);
    if adapter.is_null() {
        return None;
    }
    let ops = (*adapter).DmaOperations as *mut DMA_OPERATIONS;
    let Some(allocate) = (*ops).AllocateCommonBuffer else {
        return Some((ptr::null_mut(), 0));
    };
    let buffers = &mut *(&raw mut BUFFERS);
    let Some(slot) = buffers.iter().position(|b| b.0 == 0) else {
        return Some((ptr::null_mut(), 0));
    };
    let mut logical: PHYSICAL_ADDRESS = core::mem::zeroed();
    let va = allocate(adapter, bytes as u32, &mut logical, 1) as *mut u8;
    if va.is_null() {
        return Some((ptr::null_mut(), 0));
    }
    ptr::write_bytes(va, 0, bytes);
    buffers[slot] = (va as usize, adapter as usize, bytes as u32);
    Some((va, logical.QuadPart as u64))
}

/// Free `va` if it is one of `alloc`'s; false when it is not.
pub unsafe fn free(va: *mut u8, device_address: u64) -> bool {
    let buffers = &mut *(&raw mut BUFFERS);
    let Some(slot) = buffers.iter().position(|b| b.0 == va as usize) else {
        return false;
    };
    let (_, adapter, bytes) = buffers[slot];
    let adapter = adapter as *mut DMA_ADAPTER;
    let ops = (*adapter).DmaOperations as *mut DMA_OPERATIONS;
    if let Some(free) = (*ops).FreeCommonBuffer {
        let mut logical: PHYSICAL_ADDRESS = core::mem::zeroed();
        logical.QuadPart = device_address as i64;
        free(adapter, bytes, logical, va as PVOID, 1);
    }
    buffers[slot] = (0, 0, 0);
    true
}
