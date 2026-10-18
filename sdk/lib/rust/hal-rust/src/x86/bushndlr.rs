//! Generic HAL bus-handler database — translated from
//! `hal/halx86/legacy/bus/bushndlr.c`.
//!
//! i386-only (legacy.cmake is only included for the i386 HAL).  Maintains a
//! 2-level `PARRAY`-keyed lookup of `HAL_BUS_HANDLER` records, one tree
//! indexed by `InterfaceType + BusNumber` (HalpBusTable) and one by
//! `BUS_DATA_TYPE + BusNumber` (HalpConfigTable).  Plus a flat
//! HalpAllBusHandlers list and a database event/spinlock pair.
//!
//! Exports:
//!  Globals: HalpBusDatabaseSpinLock, HalpBusDatabaseEvent, HalpAllBusHandlers,
//!           HalpBusTable, HalpConfigTable.
//!  Private (NTAPI): HalpAllocateArray, HalpGrowArray, HalpNoBusData,
//!           HalpNoAdjustResourceList, HalpNoAssignSlotResources,
//!           HalpContextToBusHandler, HalpInitBusHandler, HaliRegisterBusHandler.
//!  Private (FASTCALL): HalpLookupHandler.
//!  Dispatched-through-HALPRIVATEDISPATCH (FASTCALL):
//!           HaliReferenceBusHandler, HaliDereferenceBusHandler,
//!           HaliHandlerForBus, HaliHandlerForConfigSpace,
//!           HaliReferenceHandlerForBus, HaliReferenceHandlerForConfigSpace.

#![cfg(target_arch = "x86")]

use ntkrnl_sys::hal_ddk::{
    BOOLEAN, BUS_DATA_TYPE, INTERFACE_TYPE, KEVENT, KSPIN_LOCK, LIST_ENTRY, NTSTATUS,
    PARRAY, PBUS_HANDLER, PCM_RESOURCE_LIST, PDEVICE_OBJECT, PDRIVER_OBJECT, PHAL_BUS_HANDLER,
    PIO_RESOURCE_REQUIREMENTS_LIST, PLIST_ENTRY, PUNICODE_STRING, PVOID,
    ULONG, ULONG_PTR, _ARRAY, _HAL_BUS_HANDLER,
    _BUS_DATA_TYPE_ConfigurationSpaceUndefined as CONFIG_SPACE_UNDEF,
    _INTERFACE_TYPE_InterfaceTypeUndefined as IFT_UNDEF,
};

// KIRQL / PINSTALL_BUS_HANDLER are only referenced by the spinlock externs
// and HaliRegisterBusHandler, both `#[cfg(not(feature = "minihal"))]`.
// Gate the imports too so the minihal build stays unused-import clean.
#[cfg(not(feature = "minihal"))]
use ntkrnl_sys::hal_ddk::{KIRQL, PINSTALL_BUS_HANDLER};

// HalDispatchTable / HalPrivateDispatchTable:
//
// In the full HAL build these come from `hal_ddk`, where bindgen decorates
// them with `#[link(name = "ntoskrnl.exe", kind = "raw-dylib", modifiers = "+verbatim")]` — correct
// for the real HAL (the kernel exports them).
//
// In the FreeLDR `--features minihal` build there is NO ntoskrnl.exe; the
// loader provides these globals from its own libs as PLAIN externs. Using the
// raw-dylib decl here would force rosload.exe to spuriously import
// ntoskrnl.exe (breaks FreeLDR — feedback_freeldr_clean_build_regression).
// So under minihal we declare HalPrivateDispatchTable as a plain extern and
// never reference HalDispatchTable (its only uses are in the non-minihal
// dispatcher-wiring block of HalpInitBusHandler).
#[cfg(not(feature = "minihal"))]
use ntkrnl_sys::hal_ddk::{HalDispatchTable, HalPrivateDispatchTable};

#[cfg(feature = "minihal")]
unsafe extern "C" {
    static mut HalPrivateDispatchTable: ntkrnl_sys::hal_ddk::_HAL_PRIVATE_DISPATCH;
}

// --- C-side constants the bindgen catalog doesn't surface ----------------

const TAG_BUS_HANDLER: u32 = u32::from_le_bytes(*b"BusH"); // 'BusH'
const HAL_MEMORY_ALLOCATION: u32 = 0xAC; // BUGCHECK code (see bugcodes.h)
const STATUS_INSUFFICIENT_RESOURCES: NTSTATUS = 0xC000009A_u32 as NTSTATUS;
const STATUS_SUCCESS: NTSTATUS = 0;
const STATUS_UNSUCCESSFUL: NTSTATUS = 0xC0000001_u32 as NTSTATUS;
const STATUS_NOT_SUPPORTED: NTSTATUS = 0xC00000BB_u32 as NTSTATUS;
const NON_PAGED_POOL: i32 = 0; // POOL_TYPE::NonPagedPool
const SYNCHRONIZATION_EVENT: i32 = 1; // EVENT_TYPE::SynchronizationEvent; nolint: shadowed-c-define (video.h's is VIDEO_EVENT's 0)
const KERNEL_MODE: i8 = 0; // MODE::KernelMode
const WR_EXECUTIVE: i32 = 0; // KWAIT_REASON::WrExecutive

// --- Kernel-side ABIs ------------------------------------------------------
// Pool + bugcheck are NTAPI (stdcall on i386). These three are in the
// minihal allowed-imports set (ExAllocatePoolWithTag, KeBugCheckEx,
// KeInitializeEvent), so the block is unconditional.
unsafe extern "system" {
    fn ExAllocatePoolWithTag(PoolType: i32, NumberOfBytes: usize, Tag: u32) -> PVOID;
    fn KeBugCheckEx(
        BugCheckCode: u32,
        BugCheckParameter1: ULONG_PTR,
        BugCheckParameter2: ULONG_PTR,
        BugCheckParameter3: ULONG_PTR,
        BugCheckParameter4: ULONG_PTR,
    ) -> !;
    fn KeInitializeEvent(Event: *mut KEVENT, Type: i32, State: BOOLEAN);
}

// ExFreePoolWithTag / KeWaitForSingleObject are NOT in the minihal
// allowed-imports set — only HaliRegisterBusHandler (itself
// `#ifndef _MINIHAL_`) uses them. Gate the decls so the minihal build
// never even declares them.
#[cfg(not(feature = "minihal"))]
unsafe extern "system" {
    fn ExFreePoolWithTag(P: PVOID, Tag: u32);
    fn KeWaitForSingleObject(
        Object: PVOID,
        WaitReason: i32,
        WaitMode: i8,
        Alertable: BOOLEAN,
        Timeout: *mut i64,
    ) -> NTSTATUS;
}

// KeInitializeSpinLock is the kernel macro `*sl = 0` (UP + DUMMY_PSEH),
// NOT a linkable symbol in the FreeLDR mini-HAL build. Inline per
// system_prompt_hal.md (matches dma.rs). feedback_freeldr_clean_build_regression
#[allow(non_snake_case)]
#[inline]
unsafe fn KeInitializeSpinLock(SpinLock: *mut KSPIN_LOCK) {
    unsafe { *SpinLock = 0 }
}

// KeSetEvent is NTAPI (stdcall) on i386. NOT in the minihal allowed-imports
// set — only HaliRegisterBusHandler (`#ifndef _MINIHAL_`) uses it.
#[cfg(not(feature = "minihal"))]
unsafe extern "system" {
    fn KeSetEvent(Event: *mut KEVENT, Increment: i32, Wait: BOOLEAN) -> i32;
}

// KeAcquireSpinLock / KeReleaseSpinLock are macros around the FASTCALL
// Kf* primitives — go straight to those. NOT in the minihal allowed-imports
// set — only HaliRegisterBusHandler (`#ifndef _MINIHAL_`) uses them.
#[cfg(not(feature = "minihal"))]
unsafe extern "fastcall" {
    fn KfAcquireSpinLock(SpinLock: *mut KSPIN_LOCK) -> KIRQL;
    fn KfReleaseSpinLock(SpinLock: *mut KSPIN_LOCK, OldIrql: KIRQL);
}

// HalpAssignSlotResources / HaliTranslateBusAddress / HaliFindBusAddressTranslation
// are still in C (legacy/bussupp.c + pcibus.c).  Need full NTAPI signatures so
// the linker emits the right `_Name@N` stdcall decoration.
unsafe extern "system" {
    fn HalpAssignSlotResources(
        BusHandler: PBUS_HANDLER,
        RootHandler: PBUS_HANDLER,
        RegistryPath: PUNICODE_STRING,
        DriverClassName: PUNICODE_STRING,
        DriverObject: PDRIVER_OBJECT,
        DeviceObject: PDEVICE_OBJECT,
        SlotNumber: ULONG,
        AllocatedResources: *mut PCM_RESOURCE_LIST,
    ) -> NTSTATUS;
    fn HaliTranslateBusAddress(
        InterfaceType: INTERFACE_TYPE,
        BusNumber: ULONG,
        BusAddress: ntkrnl_sys::hal_ddk::PHYSICAL_ADDRESS,
        AddressSpace: *mut ULONG,
        TranslatedAddress: *mut ntkrnl_sys::hal_ddk::PHYSICAL_ADDRESS,
    ) -> BOOLEAN;
    fn HaliFindBusAddressTranslation(
        BusAddress: ntkrnl_sys::hal_ddk::PHYSICAL_ADDRESS,
        AddressSpace: *mut ULONG,
        TranslatedAddress: *mut ntkrnl_sys::hal_ddk::PHYSICAL_ADDRESS,
        Context: *mut ULONG_PTR,
        NextBus: BOOLEAN,
    ) -> BOOLEAN;
}

// LIST_ENTRY ops — InitializeListHead / IsListEmpty / InsertTailList /
// RemoveEntryList are macros; inline them.
#[inline(always)]
unsafe fn initialize_list_head(head: PLIST_ENTRY) {
    unsafe {
        (*head).Flink = head;
        (*head).Blink = head;
    }
}

#[inline(always)]
unsafe fn is_list_empty(head: PLIST_ENTRY) -> bool {
    unsafe { (*head).Flink == head }
}

#[inline(always)]
unsafe fn insert_tail_list(head: PLIST_ENTRY, entry: PLIST_ENTRY) {
    unsafe {
        let prev = (*head).Blink;
        (*entry).Flink = head;
        (*entry).Blink = prev;
        (*prev).Flink = entry;
        (*head).Blink = entry;
    }
}

#[inline(always)]
unsafe fn remove_entry_list(entry: PLIST_ENTRY) {
    unsafe {
        let prev = (*entry).Blink;
        let next = (*entry).Flink;
        (*prev).Flink = next;
        (*next).Blink = prev;
    }
}

// CONTAINING_RECORD — given a pointer to a field, recover the enclosing struct.
#[inline(always)]
fn containing_record_bus_from_handler(handler: PBUS_HANDLER) -> PHAL_BUS_HANDLER {
    let off = core::mem::offset_of!(_HAL_BUS_HANDLER, Handler);
    ((handler as usize).wrapping_sub(off)) as PHAL_BUS_HANDLER
}

#[inline(always)]
fn containing_record_bus_from_all_handlers(all_handlers: PLIST_ENTRY) -> PHAL_BUS_HANDLER {
    let off = core::mem::offset_of!(_HAL_BUS_HANDLER, AllHandlers);
    ((all_handlers as usize).wrapping_sub(off)) as PHAL_BUS_HANDLER
}

// ---------------------------------------------------------------------------
// Globals — defined here, exported as #[no_mangle] static muts so the C HAL
// (still linked) can read them through the existing `extern KSPIN_LOCK
// HalpBusDatabaseSpinLock;` declarations.
// ---------------------------------------------------------------------------

#[unsafe(no_mangle)]
pub static mut HalpBusDatabaseSpinLock: KSPIN_LOCK = 0;

#[unsafe(no_mangle)]
pub static mut HalpBusDatabaseEvent: KEVENT = unsafe { core::mem::zeroed() };

#[unsafe(no_mangle)]
pub static mut HalpAllBusHandlers: LIST_ENTRY = unsafe { core::mem::zeroed() };

#[unsafe(no_mangle)]
pub static mut HalpBusTable: PARRAY = core::ptr::null_mut();

#[unsafe(no_mangle)]
pub static mut HalpConfigTable: PARRAY = core::ptr::null_mut();

// ---------------------------------------------------------------------------
// HalpAllocateArray — allocate a PARRAY with `ArraySize` element slots
// (zeroed).  Caller may pass MAXULONG, which the original C treats as a
// "no slots, just the header" request by zeroing the size.  KeBugCheckEx
// on failure — pool exhaustion at HAL init is unrecoverable.
// ---------------------------------------------------------------------------

#[unsafe(no_mangle)]
pub unsafe extern "system" fn HalpAllocateArray(mut ArraySize: ULONG) -> PARRAY {
    unsafe {
        if ArraySize == u32::MAX {
            ArraySize = 0;
        }
        // sizeof(_ARRAY) already accounts for one PVOID; multiply by ArraySize
        // (not ArraySize-1) to mirror the C: ArraySize * sizeof(PARRAY) + sizeof(ARRAY).
        // sizeof(PARRAY) == sizeof(PVOID) == 4 on i386.
        let size = (ArraySize as usize)
            .wrapping_mul(core::mem::size_of::<PVOID>())
            .wrapping_add(core::mem::size_of::<_ARRAY>());

        let raw = ExAllocatePoolWithTag(NON_PAGED_POOL, size, TAG_BUS_HANDLER);
        if raw.is_null() {
            // BUGCHECK code, allocation size, 0 (file), 0 (line) — match C
            // semantics (file/line are not portable so we pass 0).
            KeBugCheckEx(HAL_MEMORY_ALLOCATION, size as ULONG_PTR, 0, 0, 0);
        }

        let array = raw as PARRAY;
        (*array).ArraySize = ArraySize;
        // Zero (ArraySize + 1) PVOID slots — the +1 covers the inline first
        // Element slot, which is already inside sizeof(_ARRAY) but the C
        // ZeroMemory call still touches it via &Element[0].
        core::ptr::write_bytes(
            (*array).Element.as_mut_ptr() as *mut u8,
            0,
            core::mem::size_of::<PVOID>() * (ArraySize as usize + 1),
        );
        array
    }
}

// ---------------------------------------------------------------------------
// HalpGrowArray — if `CurrentArray` is null OR smaller than `NewArray`,
// copy the current elements into the new one and swap them via the
// in/out pointers.  Caller frees the displaced (in `*NewArray` after swap)
// allocation.
// ---------------------------------------------------------------------------

#[unsafe(no_mangle)]
pub unsafe extern "system" fn HalpGrowArray(CurrentArray: *mut PARRAY, NewArray: *mut PARRAY) {
    unsafe {
        let cur = *CurrentArray;
        let new_ = *NewArray;
        if cur.is_null() || (*new_).ArraySize > (*cur).ArraySize {
            if !cur.is_null() {
                // Copy (CurrentArray->ArraySize + 1) PVOID slots.
                core::ptr::copy_nonoverlapping(
                    (*cur).Element.as_ptr() as *const u8,
                    (*new_).Element.as_mut_ptr() as *mut u8,
                    core::mem::size_of::<PVOID>() * ((*cur).ArraySize as usize + 1),
                );
            }
            // Swap.
            *CurrentArray = new_;
            *NewArray = cur;
        }
    }
}

// ---------------------------------------------------------------------------
// HalpLookupHandler — index `Array[Type]->Element[Number]` to get the
// _HAL_BUS_HANDLER and return &handler.Handler.  FASTCALL on i386 (the
// HAL macros call this hot path through edx/ecx).
// ---------------------------------------------------------------------------

#[unsafe(no_mangle)]
pub unsafe extern "fastcall" fn HalpLookupHandler(
    mut Array: PARRAY,
    Type: ULONG,
    Number: ULONG,
    AddReference: BOOLEAN,
) -> PBUS_HANDLER {
    unsafe {
        let mut handler: PBUS_HANDLER = core::ptr::null_mut();
        if (*Array).ArraySize >= Type {
            // Element[Type] — the bindgen Element is [PVOID; 1] (flexible array),
            // so use .add(Type) on the base pointer to dodge the bounds check.
            let outer = *((*Array).Element.as_ptr().add(Type as usize)) as PARRAY;
            if !outer.is_null() && (*outer).ArraySize >= Number {
                let bus = *((*outer).Element.as_ptr().add(Number as usize)) as PHAL_BUS_HANDLER;
                handler = &raw mut (*bus).Handler;
                if AddReference != 0 {
                    (*bus).ReferenceCount += 1;
                }
            }
        }
        handler
    }
}

// ---------------------------------------------------------------------------
// Stub handlers used for buses without a real configuration space — these
// are NTAPI and get plugged into BUS_HANDLER's function-pointer fields.
// ---------------------------------------------------------------------------

#[unsafe(no_mangle)]
pub unsafe extern "system" fn HalpNoBusData(
    _BusHandler: PBUS_HANDLER,
    _RootHandler: PBUS_HANDLER,
    _SlotNumber: ULONG,
    _Buffer: PVOID,
    _Offset: ULONG,
    _Length: ULONG,
) -> ULONG {
    0
}

#[unsafe(no_mangle)]
pub unsafe extern "system" fn HalpNoAdjustResourceList(
    _BusHandler: PBUS_HANDLER,
    _RootHandler: PBUS_HANDLER,
    _pResourceList: *mut PIO_RESOURCE_REQUIREMENTS_LIST,
) -> NTSTATUS {
    STATUS_UNSUCCESSFUL
}

#[unsafe(no_mangle)]
pub unsafe extern "system" fn HalpNoAssignSlotResources(
    _BusHandler: PBUS_HANDLER,
    _RootHandler: PBUS_HANDLER,
    _RegistryPath: PUNICODE_STRING,
    _DriverClassName: PUNICODE_STRING,
    _DriverObject: PDRIVER_OBJECT,
    _DeviceObject: PDEVICE_OBJECT,
    _SlotNumber: ULONG,
    _AllocatedResources: *mut PCM_RESOURCE_LIST,
) -> NTSTATUS {
    STATUS_NOT_SUPPORTED
}

// ---------------------------------------------------------------------------
// HaliReferenceBusHandler / HaliDereferenceBusHandler — FASTCALL primitives
// the kernel calls through HALPRIVATEDISPATCH.
// ---------------------------------------------------------------------------

#[unsafe(no_mangle)]
pub unsafe extern "fastcall" fn HaliReferenceBusHandler(Handler: PBUS_HANDLER) {
    unsafe {
        let bus = containing_record_bus_from_handler(Handler);
        (*bus).ReferenceCount += 1;
    }
}

#[unsafe(no_mangle)]
pub unsafe extern "fastcall" fn HaliDereferenceBusHandler(Handler: PBUS_HANDLER) {
    unsafe {
        let bus = containing_record_bus_from_handler(Handler);
        (*bus).ReferenceCount -= 1;
        // C has `ASSERT(Bus->ReferenceCount != 0)` after the decrement;
        // intentionally a post-decrement assert (catches underflow).  Skip
        // the assert in release builds — matches C+NDEBUG behavior.
    }
}

// ---------------------------------------------------------------------------
// Lookup wrappers — turn (InterfaceType, BusNumber) into a PBUS_HANDLER by
// indexing HalpBusTable (interfaces) or HalpConfigTable (config spaces),
// optionally bumping the refcount.
// ---------------------------------------------------------------------------

#[unsafe(no_mangle)]
pub unsafe extern "fastcall" fn HaliHandlerForBus(
    InterfaceType: INTERFACE_TYPE,
    BusNumber: ULONG,
) -> PBUS_HANDLER {
    unsafe { HalpLookupHandler(HalpBusTable, InterfaceType as ULONG, BusNumber, 0) }
}

#[unsafe(no_mangle)]
pub unsafe extern "fastcall" fn HaliHandlerForConfigSpace(
    ConfigType: BUS_DATA_TYPE,
    BusNumber: ULONG,
) -> PBUS_HANDLER {
    unsafe { HalpLookupHandler(HalpConfigTable, ConfigType as ULONG, BusNumber, 0) }
}

#[unsafe(no_mangle)]
pub unsafe extern "fastcall" fn HaliReferenceHandlerForBus(
    InterfaceType: INTERFACE_TYPE,
    BusNumber: ULONG,
) -> PBUS_HANDLER {
    unsafe { HalpLookupHandler(HalpBusTable, InterfaceType as ULONG, BusNumber, 1) }
}

#[unsafe(no_mangle)]
pub unsafe extern "fastcall" fn HaliReferenceHandlerForConfigSpace(
    ConfigType: BUS_DATA_TYPE,
    BusNumber: ULONG,
) -> PBUS_HANDLER {
    unsafe { HalpLookupHandler(HalpConfigTable, ConfigType as ULONG, BusNumber, 1) }
}

// ---------------------------------------------------------------------------
// HalpContextToBusHandler — walk HalpAllBusHandlers looking for the
// _HAL_BUS_HANDLER whose .Handler matches ContextValue.  If ContextValue
// is 0, return the first entry.  Returns null if not found.
// ---------------------------------------------------------------------------

#[unsafe(no_mangle)]
pub unsafe extern "system" fn HalpContextToBusHandler(ContextValue: ULONG_PTR) -> PBUS_HANDLER {
    unsafe {
        let head: PLIST_ENTRY = &raw mut HalpAllBusHandlers;
        let mut next_entry: PLIST_ENTRY = (*head).Flink;
        let mut this_handler: PHAL_BUS_HANDLER =
            containing_record_bus_from_all_handlers(next_entry);

        if ContextValue != 0 {
            if is_list_empty(head) {
                return core::ptr::null_mut();
            }
            // ContextValue is a PBUS_HANDLER — recover the enclosing _HAL_BUS_HANDLER.
            let target = containing_record_bus_from_handler(ContextValue as PBUS_HANDLER);
            loop {
                this_handler = containing_record_bus_from_all_handlers(next_entry);
                if this_handler == target {
                    break;
                }
                next_entry = (*next_entry).Flink;
                if next_entry == head {
                    break;
                }
            }
        }

        if next_entry == head {
            return core::ptr::null_mut();
        }
        &raw mut (*this_handler).Handler
    }
}

// ---------------------------------------------------------------------------
// HaliRegisterBusHandler — allocate a _HAL_BUS_HANDLER, splice it into
// HalpBusTable[InterfaceType][BusNumber] and HalpConfigTable[ConfigType]
// [BusNumber], link onto HalpAllBusHandlers, inherit parent dispatchers
// if there's a parent.  Caller-provided InstallCallback isn't supported
// (the C ASSERTs it null).  Returns STATUS_INSUFFICIENT_RESOURCES on any
// allocation failure; STATUS_SUCCESS otherwise.
// ---------------------------------------------------------------------------

#[cfg(not(feature = "minihal"))]
#[unsafe(no_mangle)]
pub unsafe extern "system" fn HaliRegisterBusHandler(
    InterfaceType: INTERFACE_TYPE,
    ConfigType: BUS_DATA_TYPE,
    mut BusNumber: ULONG,
    ParentBusType: INTERFACE_TYPE,
    ParentBusNumber: ULONG,
    ExtraData: ULONG,
    _InstallCallback: PINSTALL_BUS_HANDLER,
    ReturnedBusHandler: *mut PBUS_HANDLER,
) -> NTSTATUS {
    unsafe {
        // Allocate the bus handler — sizeof(HAL_BUS_HANDLER) + extra trailing data.
        let mut bus = ExAllocatePoolWithTag(
            NON_PAGED_POOL,
            core::mem::size_of::<_HAL_BUS_HANDLER>() + ExtraData as usize,
            TAG_BUS_HANDLER,
        ) as PHAL_BUS_HANDLER;
        if bus.is_null() {
            return STATUS_INSUFFICIENT_RESOURCES;
        }
        *ReturnedBusHandler = &raw mut (*bus).Handler;

        // Synchronize.
        KeWaitForSingleObject(
            &raw mut HalpBusDatabaseEvent as PVOID,
            WR_EXECUTIVE,
            KERNEL_MODE,
            0,
            core::ptr::null_mut(),
        );

        // BusNumber == -1 means "auto-pick the next free slot".
        if BusNumber == u32::MAX {
            BusNumber = 0;
            while !HaliHandlerForBus(InterfaceType, BusNumber).is_null() {
                BusNumber = BusNumber.wrapping_add(1);
            }
        }

        // Allocate all four arrays up front so we can fail atomically.
        let mut interface_array = HalpAllocateArray(InterfaceType as ULONG);
        let mut interface_bus_array = HalpAllocateArray(BusNumber);
        let mut config_array = HalpAllocateArray(ConfigType as ULONG);
        let mut config_bus_array = HalpAllocateArray(BusNumber);

        let mut old_handler: PHAL_BUS_HANDLER = core::ptr::null_mut();
        let status: NTSTATUS;

        if !interface_array.is_null()
            && !interface_bus_array.is_null()
            && !config_array.is_null()
            && !config_bus_array.is_null()
        {
            let parent = HaliReferenceHandlerForBus(ParentBusType, ParentBusNumber);

            // Zero the entire bus record (incl. ExtraData) before populating.
            core::ptr::write_bytes(
                bus as *mut u8,
                0,
                core::mem::size_of::<_HAL_BUS_HANDLER>() + ExtraData as usize,
            );
            (*bus).ReferenceCount = 1;
            (*bus).Handler.BusNumber = BusNumber;
            (*bus).Handler.InterfaceType = InterfaceType;
            (*bus).Handler.ConfigurationType = ConfigType;
            (*bus).Handler.ParentHandler = parent;

            // Default stubs.  extern "system" and "stdcall" share ABI on i386
            // but differ at the Rust type level; transmute around it.
            (*bus).Handler.GetBusData = Some(core::mem::transmute(HalpNoBusData as usize));
            (*bus).Handler.SetBusData = Some(core::mem::transmute(HalpNoBusData as usize));
            (*bus).Handler.AdjustResourceList =
                Some(core::mem::transmute(HalpNoAdjustResourceList as usize));
            (*bus).Handler.AssignSlotResources =
                Some(core::mem::transmute(HalpNoAssignSlotResources as usize));

            if ExtraData != 0 {
                (*bus).Handler.BusData = bus.add(1) as PVOID;
            }

            // Inherit parent dispatchers if there is a parent.
            if !parent.is_null() {
                (*bus).Handler.GetBusData = (*parent).GetBusData;
                (*bus).Handler.SetBusData = (*parent).SetBusData;
                (*bus).Handler.AdjustResourceList = (*parent).AdjustResourceList;
                (*bus).Handler.AssignSlotResources = (*parent).AssignSlotResources;
                (*bus).Handler.TranslateBusAddress = (*parent).TranslateBusAddress;
                (*bus).Handler.GetInterruptVector = (*parent).GetInterruptVector;
            }

            // Lock the database.
            let old_irql = KfAcquireSpinLock(&raw mut HalpBusDatabaseSpinLock);

            // Make room for the interface row.
            HalpGrowArray(&raw mut HalpBusTable, &raw mut interface_array);

            if InterfaceType != IFT_UNDEF {
                // Row-level array — &HalpBusTable->Element[InterfaceType] is a slot
                // holding a PARRAY pointer.  PARRAY** here is a `*mut PARRAY`.
                let row_slot =
                    (*HalpBusTable).Element.as_mut_ptr().add(InterfaceType as usize) as *mut PARRAY;
                HalpGrowArray(row_slot, &raw mut interface_bus_array);

                // BusEntry — PHAL_BUS_HANDLER* into the column slot.
                let column_array = *row_slot;
                let bus_slot = (*column_array)
                    .Element
                    .as_mut_ptr()
                    .add(BusNumber as usize) as *mut PHAL_BUS_HANDLER;
                if !(*bus_slot).is_null() {
                    old_handler = *bus_slot;
                }
                *bus_slot = bus;
            }

            HalpGrowArray(&raw mut HalpConfigTable, &raw mut config_array);

            if ConfigType != CONFIG_SPACE_UNDEF {
                let row_slot = (*HalpConfigTable)
                    .Element
                    .as_mut_ptr()
                    .add(ConfigType as usize) as *mut PARRAY;
                HalpGrowArray(row_slot, &raw mut config_bus_array);

                let column_array = *row_slot;
                let bus_slot = (*column_array)
                    .Element
                    .as_mut_ptr()
                    .add(BusNumber as usize) as *mut PHAL_BUS_HANDLER;
                if !(*bus_slot).is_null() {
                    // C ASSERTs old_handler matches if both rows are populated.
                    old_handler = *bus_slot;
                }
                *bus_slot = bus;
            }

            insert_tail_list(&raw mut HalpAllBusHandlers, &raw mut (*bus).AllHandlers);

            // Now unlink the old (displaced) entry, if any.
            bus = old_handler;
            if !bus.is_null() {
                remove_entry_list(&raw mut (*bus).AllHandlers);
            }

            KfReleaseSpinLock(&raw mut HalpBusDatabaseSpinLock, old_irql);
            status = STATUS_SUCCESS;
        } else {
            status = STATUS_INSUFFICIENT_RESOURCES;
        }

        // Signal the event.
        KeSetEvent(&raw mut HalpBusDatabaseEvent, 0, 0);

        // Free anything that was either displaced (bus = old_handler) or
        // never used.  All four array allocations are freed unconditionally
        // — HalpGrowArray swaps the "old" allocation into the local on
        // success, so freeing the local always reclaims either the displaced
        // table or the newly-allocated one on the failure path.
        if !bus.is_null() {
            ExFreePoolWithTag(bus as PVOID, TAG_BUS_HANDLER);
        }
        if !interface_array.is_null() {
            ExFreePoolWithTag(interface_array as PVOID, TAG_BUS_HANDLER);
        }
        if !interface_bus_array.is_null() {
            ExFreePoolWithTag(interface_bus_array as PVOID, TAG_BUS_HANDLER);
        }
        if !config_array.is_null() {
            ExFreePoolWithTag(config_array as PVOID, TAG_BUS_HANDLER);
        }
        if !config_bus_array.is_null() {
            ExFreePoolWithTag(config_bus_array as PVOID, TAG_BUS_HANDLER);
        }

        status
    }
}

// ---------------------------------------------------------------------------
// HalpInitBusHandler — one-shot init from HAL setup.  Initializes the lock
// + event + empty tables and plugs the FASTCALL dispatchers into
// HalPrivateDispatchTable.  HalpAssignSlotResources, HaliTranslateBusAddress,
// HaliFindBusAddressTranslation still live in C (legacy/bussupp.c +
// pcibus.c) and resolve via libhal at link time.
// ---------------------------------------------------------------------------

// C legacy/bushndlr.c common region (after `#endif`): compiled in the
// -D_MINIHAL_ build (FreeLDR links it). The Hali* dispatcher wiring is
// itself `#ifndef _MINIHAL_` in C, so it is cfg'd-out under minihal —
// FreeLDR's loader HAL has no bus-handler dispatch. See memory
// feedback_freeldr_clean_build_regression.
#[unsafe(no_mangle)]
pub unsafe extern "system" fn HalpInitBusHandler() {
    unsafe {
        KeInitializeSpinLock(&raw mut HalpBusDatabaseSpinLock);
        KeInitializeEvent(&raw mut HalpBusDatabaseEvent, SYNCHRONIZATION_EVENT, 1);

        HalpBusTable = HalpAllocateArray(0);
        HalpConfigTable = HalpAllocateArray(0);

        initialize_list_head(&raw mut HalpAllBusHandlers);

        // C: `#ifndef _MINIHAL_` — full-HAL dispatcher wiring only.
        #[cfg(not(feature = "minihal"))]
        {
            // HAL_PRIVATE_DISPATCH wiring — register/handler/config dispatchers.
            HalPrivateDispatchTable.HalRegisterBusHandler =
                Some(core::mem::transmute(HaliRegisterBusHandler as usize));
            HalPrivateDispatchTable.HalHandlerForBus =
                Some(core::mem::transmute(HaliHandlerForBus as usize));
            HalPrivateDispatchTable.HalHandlerForConfigSpace =
                Some(core::mem::transmute(HaliHandlerForConfigSpace as usize));

            // HAL_DISPATCH wiring — public reference/dereference paths (these
            // are FASTCALL on i386 since `pHalReferenceBusHandler` is FASTCALL).
            HalDispatchTable.HalReferenceHandlerForBus =
                Some(core::mem::transmute(HaliReferenceHandlerForBus as usize));
            HalDispatchTable.HalReferenceBusHandler =
                Some(core::mem::transmute(HaliReferenceBusHandler as usize));
            HalDispatchTable.HalDereferenceBusHandler =
                Some(core::mem::transmute(HaliDereferenceBusHandler as usize));
        }

        // Three more dispatchers — common region in C (always compiled).
        HalPrivateDispatchTable.HalPciAssignSlotResources =
            Some(core::mem::transmute(HalpAssignSlotResources as usize));
        HalPrivateDispatchTable.HalPciTranslateBusAddress =
            Some(core::mem::transmute(HaliTranslateBusAddress as usize));
        // C: `if (!HalFindBusAddressTranslation) HalFindBusAddressTranslation = ...`
        // Read the static field once into a local so we don't take a shared ref
        // to the mutable static (clippy/Rust-2024 footgun).
        let cur = core::ptr::addr_of!(HalPrivateDispatchTable.HalFindBusAddressTranslation)
            .read_unaligned();
        if cur.is_none() {
            HalPrivateDispatchTable.HalFindBusAddressTranslation =
                Some(core::mem::transmute(HaliFindBusAddressTranslation as usize));
        }
    }
}
