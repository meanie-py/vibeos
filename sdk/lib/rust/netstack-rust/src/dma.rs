//! The virtio-net function's DMA, through a DMA adapter that knows which device it is.
//!
//! netstack finds its NIC by scanning PCI itself (it is not the device's PnP driver),
//! so there is no PDO to ask IoGetDmaAdapter with; the HAL gives an adapter for a
//! requester ID directly instead (HalVibeosGetDmaAdapter). Everything the device ever
//! reaches comes from it: the virtqueues are common buffers, and every frame buffer
//! virtio-drivers shares is bounced through a pool of 2 KiB slots in one more, since
//! smoltcp's buffers live in pool memory no adapter mapped. That is what makes the
//! driver DMA-remapping compatible, and when its service says so (Parameters
//! `DmaRemappingCompatible` = 1, as Windows reads it) it claims a domain of its own for
//! the NIC behind a translating SMMU (HalVibeosDmaAttach), where nothing else is
//! reachable.

use core::ptr::{self, NonNull};
use core::sync::atomic::{AtomicBool, AtomicPtr, Ordering};

use ntkrnl_sys::ddk::{
    DEVICE_DESCRIPTION, DMA_ADAPTER, DMA_OPERATIONS, HANDLE, OBJECT_ATTRIBUTES, PHYSICAL_ADDRESS, PVOID,
    UNICODE_STRING, ZwClose, ZwOpenKey, ZwQueryValueKey, _KEY_VALUE_INFORMATION_CLASS_KeyValuePartialInformation,
};
use ntkrnl_sys::guards::SpinLockGuard;
use ntkrnl_sys::ke::KSPIN_LOCK;
use virtio_drivers::transport::pci::bus::DeviceFunction;

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
/// The NIC's adapter, once `attach` has it (the self-test and the engine bring the
/// device up in turn; both use the one adapter).
static ADAPTER: AtomicPtr<DMA_ADAPTER> = AtomicPtr::new(ptr::null_mut());

/// A frame bounce slot: a virtio-net buffer (crate::device::BUF_LEN) fits one.
pub const FRAME_SLOT: usize = 2048;
/// Every receive buffer stays shared while it is queued (device::QUEUE_SIZE of them),
/// with room for as many frames in flight the other way.
const FRAME_SLOTS: usize = 64;
static mut POOL_VA: usize = 0;
static mut POOL_DEVICE: u64 = 0;
/// Which buffer each slot bounces (its address; 0 free), and how many slots it spans.
static mut OWNER: [(usize, u16); FRAME_SLOTS] = [(0, 0); FRAME_SLOTS];
static mut POOL_LOCK: KSPIN_LOCK = 0;

const NIC_COMMON_BUFFERS: usize = 16;
/// The common buffers `alloc` handed out, to free one by its address: (va, length).
static mut BUFFERS: [(usize, u32); NIC_COMMON_BUFFERS] = [(0, 0); NIC_COMMON_BUFFERS];

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

/// Bring-up, at PASSIVE_LEVEL, before any queue is made: the NIC's adapter and its
/// bounce pool, after claiming it a domain of its own when the service is
/// DMA-remapping compatible. False when the HAL has no adapter or memory to give;
/// the device must then not be brought up, or its DMA would go nowhere it could reach.
pub unsafe fn attach(df: DeviceFunction) -> bool {
    if !ADAPTER.load(Ordering::Acquire).is_null() {
        // A bring-up again (the engine's, after the self-test's): it resets the
        // device before making new queues, so whatever the last one left shared (its
        // receive buffers, which nothing unshares when the queues go) is dead.
        let _lock = SpinLockGuard::acquire(&raw mut POOL_LOCK);
        *(&raw mut OWNER) = [(0, 0); FRAME_SLOTS];
        return true;
    }
    let requester = (df.bus as u32) << 8 | (df.device as u32) << 3 | df.function as u32;
    let claimed = REMAPPING_COMPATIBLE.load(Ordering::Acquire) && HalVibeosDmaAttach(requester, DMA_ATTACH_CLAIMED) != 0;
    let mut d: DEVICE_DESCRIPTION = core::mem::zeroed();
    d.Version = DEVICE_DESCRIPTION_VERSION2;
    d.Master = 1;
    d.ScatterGather = 1;
    // Below 4 GiB: addressable whether or not the device negotiated 64-bit.
    d.Dma32BitAddresses = 1;
    d.InterfaceType = PCI_BUS as _;
    d.MaximumLength = FRAME_SLOT as u32;
    let mut map_registers = 0u32;
    let adapter = HalVibeosGetDmaAdapter(requester, &mut d, &mut map_registers);
    if adapter.is_null() {
        crate::serial::emit(format_args!("[NETSTACK] no DMA adapter from the HAL for the NIC\n"));
        return false;
    }
    ADAPTER.store(adapter, Ordering::Release);
    let Some((va, device)) = alloc(FRAME_SLOTS * FRAME_SLOT) else {
        crate::serial::emit(format_args!("[NETSTACK] no memory for the NIC's bounce pool\n"));
        ADAPTER.store(ptr::null_mut(), Ordering::Release);
        return false;
    };
    POOL_VA = va as usize;
    POOL_DEVICE = device;
    crate::serial::emit(format_args!(
        "[NETSTACK] NIC {:02x}:{:02x}.{}: DMA through its adapter{}\n",
        df.bus,
        df.device,
        df.function,
        if claimed { ", DMA-remapping compatible" } else { "" }
    ));
    true
}

/// Whether the NIC's DMA goes through its adapter (`attach` ran).
pub fn attached() -> bool {
    !ADAPTER.load(Ordering::Acquire).is_null()
}

/// A zeroed common buffer of the NIC's adapter: (va, its device address). PASSIVE_LEVEL.
pub unsafe fn alloc(bytes: usize) -> Option<(*mut u8, u64)> {
    let adapter = ADAPTER.load(Ordering::Acquire);
    if adapter.is_null() {
        return None;
    }
    let ops = (*adapter).DmaOperations as *mut DMA_OPERATIONS;
    let allocate = (*ops).AllocateCommonBuffer?;
    let buffers = &mut *(&raw mut BUFFERS);
    let slot = buffers.iter().position(|b| b.0 == 0)?;
    let mut logical: PHYSICAL_ADDRESS = core::mem::zeroed();
    let va = allocate(adapter, bytes as u32, &mut logical, 1) as *mut u8;
    if va.is_null() {
        return None;
    }
    ptr::write_bytes(va, 0, bytes);
    buffers[slot] = (va as usize, bytes as u32);
    Some((va, logical.QuadPart as u64))
}

/// Free `va` if it is one of `alloc`'s; false when it is not.
pub unsafe fn free(va: *mut u8, device_address: u64) -> bool {
    let adapter = ADAPTER.load(Ordering::Acquire);
    let buffers = &mut *(&raw mut BUFFERS);
    let Some(slot) = buffers.iter().position(|b| b.0 == va as usize) else {
        return false;
    };
    let ops = (*adapter).DmaOperations as *mut DMA_OPERATIONS;
    if let Some(free) = (*ops).FreeCommonBuffer {
        let mut logical: PHYSICAL_ADDRESS = core::mem::zeroed();
        logical.QuadPart = device_address as i64;
        free(adapter, buffers[slot].1, logical, va as PVOID, 1);
    }
    buffers[slot] = (0, 0);
    true
}

/// Bounce `buffer` through the pool: its device address, the bytes copied in when the
/// device is to read them. None when the pool is full or the buffer too big.
pub unsafe fn share(buffer: NonNull<[u8]>, to_device: bool) -> Option<u64> {
    let addr = buffer.as_ptr() as *mut u8 as usize;
    let len = buffer.len();
    let need = len.div_ceil(FRAME_SLOT).max(1);
    let first = {
        let _lock = SpinLockGuard::acquire(&raw mut POOL_LOCK);
        let owners = &mut *(&raw mut OWNER);
        let first = (0..=FRAME_SLOTS.checked_sub(need)?).find(|&i| owners[i..i + need].iter().all(|o| o.0 == 0))?;
        owners[first] = (addr, need as u16);
        for o in &mut owners[first + 1..first + need] {
            *o = (addr, 0);
        }
        first
    };
    let bounce = (POOL_VA + first * FRAME_SLOT) as *mut u8;
    if to_device {
        ptr::copy_nonoverlapping(addr as *const u8, bounce, len);
    }
    Some(POOL_DEVICE + (first * FRAME_SLOT) as u64)
}

/// Undo `share`: the bytes copied back when the device wrote them. False when
/// `device_address` is not a pool slot.
pub unsafe fn unshare(device_address: u64, buffer: NonNull<[u8]>, from_device: bool) -> bool {
    if device_address < POOL_DEVICE || device_address >= POOL_DEVICE + (FRAME_SLOTS * FRAME_SLOT) as u64 {
        return false;
    }
    let first = ((device_address - POOL_DEVICE) as usize) / FRAME_SLOT;
    if from_device {
        ptr::copy_nonoverlapping((POOL_VA + first * FRAME_SLOT) as *const u8, buffer.as_ptr() as *mut u8, buffer.len());
    }
    let _lock = SpinLockGuard::acquire(&raw mut POOL_LOCK);
    let owners = &mut *(&raw mut OWNER);
    let span = owners[first].1.max(1) as usize;
    for o in &mut owners[first..(first + span).min(FRAME_SLOTS)] {
        *o = (0, 0);
    }
    true
}
