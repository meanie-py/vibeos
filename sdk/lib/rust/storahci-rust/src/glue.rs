//! AHCI helper-macro implementations + bindgen-union register accessors.
//!
//! storahci's C source accesses AHCI MMIO registers (`_AHCI_PORT_CMD`, `_AHCI_GHC`,
//! `_AHCI_INTERRUPT_*`, `_AHCI_SERIAL_ATA_*`, `_AHCI_TASK_FILE_DATA`) as a flat union of a
//! raw ULONG (`reg.Status`) overlapping a bitfield struct (`reg.FR`, `reg.ST`, ...). bindgen
//! emits these as a `__BindgenUnionField` union whose bitfield getters/setters live on the
//! INNER `__bindgen_ty_1` struct, reached via `reg.__bindgen_anon_1`. The translation calls
//! the C-flat forms (`reg.FR()`, `reg.set_FRE(1)`, `reg.__bindgen_anon_1.Status`).
//!
//! These inherent impls (legal here because the bindgen types are defined in this same crate)
//! forward each flat accessor to the real union member, keeping the call sites byte-faithful
//! to the C while reading/writing the exact same storage. Semantics are identical.

use crate::types::*;
use ntkrnl_sys::ddk::{ULONG, PVOID};
use ntkrnl_sys::base::BOOLEAN;

// ---------------------------------------------------------------------------
// Bare aliases for bindgen-prefixed enum values / types the translation uses unqualified.
// (bindgen emits enum values as `_ENUM_Value` module consts; c2rust output names them bare.)
// ---------------------------------------------------------------------------
pub use ntkrnl_sys::ddk::_INTERFACE_TYPE_PCIBus as PCIBus;
pub use ntkrnl_sys::ddk::_BUS_DATA_TYPE_PCIConfiguration as PCIConfiguration;
pub use ntkrnl_sys::ddk::_DMA_WIDTH_Width32Bits as Width32Bits;
// storport-private enums live in the driver's own bindgen (crate::types):
pub use crate::types::_SCSI_NOTIFICATION_TYPE_RequestComplete as RequestComplete;
pub use crate::types::_STOR_SPINLOCK_InterruptLock as InterruptLock;
pub use crate::types::_STOR_SYNCHRONIZATION_MODEL_StorSynchronizeFullDuplex as StorSynchronizeFullDuplex;
pub use crate::types::_STOR_DEVICE_POWER_STATE_StorPowerDeviceD0 as StorPowerDeviceD0;

// ---------------------------------------------------------------------------
// Register-union accessor forwarding
// ---------------------------------------------------------------------------

/// Generate an inherent impl on a bindgen register union that forwards:
///   - the raw value: `status()` / `set_status(v)`  (the union's `Status: __BindgenUnionField<ULONG>`)
///   - each bitfield getter/setter to the inner `__bindgen_anon_1` view.
macro_rules! ahci_reg {
    ($outer:ty, $inner:ident, $( $get:ident / $set:ident ),* $(,)?) => {
        impl $outer {
            #[inline]
            pub fn status(&self) -> ULONG {
                unsafe { *self.Status.as_ref() }
            }
            #[inline]
            pub fn set_status(&mut self, v: ULONG) {
                unsafe { *self.Status.as_mut() = v; }
            }
            $(
                #[inline]
                pub fn $get(&self) -> ULONG {
                    unsafe { self.__bindgen_anon_1.as_ref().$get() }
                }
                #[inline]
                pub fn $set(&mut self, val: ULONG) {
                    unsafe { self.__bindgen_anon_1.as_mut().$set(val) }
                }
            )*
        }
    };
}

ahci_reg!(_AHCI_PORT_CMD, _AHCI_PORT_CMD__bindgen_ty_1,
    ST/set_ST, SUD/set_SUD, POD/set_POD, CLO/set_CLO, FRE/set_FRE, RSV0/set_RSV0,
    CCS/set_CCS, MPSS/set_MPSS, FR/set_FR, CR/set_CR, CPS/set_CPS, PMA/set_PMA,
    HPCP/set_HPCP, MPSP/set_MPSP, CPD/set_CPD, ESP/set_ESP, FBSCP/set_FBSCP,
    APSTE/set_APSTE, ATAPI/set_ATAPI, DLAE/set_DLAE, ALPE/set_ALPE, ASP/set_ASP, ICC/set_ICC,
);

ahci_reg!(_AHCI_GHC, _AHCI_GHC__bindgen_ty_1,
    HR/set_HR, IE/set_IE, MRSM/set_MRSM, RSV0/set_RSV0, AE/set_AE,
);

ahci_reg!(_AHCI_INTERRUPT_ENABLE, _AHCI_INTERRUPT_ENABLE__bindgen_ty_1,
    DHRE/set_DHRE, PSE/set_PSE, DSE/set_DSE, SDBE/set_SDBE, UFE/set_UFE, DPE/set_DPE,
    PCE/set_PCE, DMPE/set_DMPE, DW5_Reserved/set_DW5_Reserved, PRCE/set_PRCE, IPME/set_IPME,
    OFE/set_OFE, DW5_Reserved2/set_DW5_Reserved2, INFE/set_INFE, IFE/set_IFE, HBDE/set_HBDE,
    HBFE/set_HBFE, TFEE/set_TFEE, CPDE/set_CPDE,
);

ahci_reg!(_AHCI_INTERRUPT_STATUS, _AHCI_INTERRUPT_STATUS__bindgen_ty_1,
    DHRS/set_DHRS, PSS/set_PSS, DSS/set_DSS, SDBS/set_SDBS, UFS/set_UFS, DPS/set_DPS,
    PCS/set_PCS, DMPS/set_DMPS, Reserved/set_Reserved, PRCS/set_PRCS, IPMS/set_IPMS,
    OFS/set_OFS, Reserved2/set_Reserved2, INFS/set_INFS, IFS/set_IFS, HBDS/set_HBDS,
    HBFS/set_HBFS, TFES/set_TFES, CPDS/set_CPDS,
);

ahci_reg!(_AHCI_SERIAL_ATA_STATUS, _AHCI_SERIAL_ATA_STATUS__bindgen_ty_1,
    DET/set_DET, SPD/set_SPD, IPM/set_IPM, RSV0/set_RSV0,
);

ahci_reg!(_AHCI_SERIAL_ATA_CONTROL, _AHCI_SERIAL_ATA_CONTROL__bindgen_ty_1,
    DET/set_DET, SPD/set_SPD, IPM/set_IPM, SPM/set_SPM, PMP/set_PMP,
    DW11_Reserved/set_DW11_Reserved,
);

ahci_reg!(_AHCI_COMMAND_HEADER_DESCRIPTION, _AHCI_COMMAND_HEADER_DESCRIPTION__bindgen_ty_1,
    CFL/set_CFL, A/set_A, W/set_W, P/set_P, R/set_R, B/set_B, C/set_C,
    RSV/set_RSV, PMP/set_PMP, PRDTL/set_PRDTL,
);

// _AHCI_TASK_FILE_DATA: source only touches the raw value (`tfd.Status`); no bitfield forwards.
impl _AHCI_TASK_FILE_DATA {
    #[inline]
    pub fn status(&self) -> ULONG {
        unsafe { *self.Status.as_ref() }
    }
    #[inline]
    pub fn set_status(&mut self, v: ULONG) {
        unsafe { *self.Status.as_mut() = v; }
    }
}

// ---------------------------------------------------------------------------
// PCI_COMMON_CONFIG — faithful layout from xdk/iotypes.h (PCI_COMMON_HEADER_LAYOUT + u +
// DeviceSpecific[192]). ntkrnl_sys's ddk binding is a stub that only exposes DeviceSpecific,
// so this driver (which reads VendorID/DeviceID/RevisionID and u.type0.BaseAddresses) needs
// the real shape. Casts in storahci.rs target this type.
// ---------------------------------------------------------------------------
use ntkrnl_sys::ddk::{USHORT, UCHAR};

#[repr(C)]
#[derive(Copy, Clone)]
pub struct _PCI_HEADER_TYPE_0 {
    pub BaseAddresses: [ULONG; 6usize], // PCI_TYPE0_ADDRESSES
    pub CIS: ULONG,
    pub SubVendorID: USHORT,
    pub SubSystemID: USHORT,
    pub ROMBaseAddress: ULONG,
    pub CapabilitiesPtr: UCHAR,
    pub Reserved1: [UCHAR; 3usize],
    pub Reserved2: ULONG,
    pub InterruptLine: UCHAR,
    pub InterruptPin: UCHAR,
    pub MinimumGrant: UCHAR,
    pub MaximumLatency: UCHAR,
}

#[repr(C)]
#[derive(Copy, Clone)]
pub union _PCI_COMMON_CONFIG_u {
    pub type0: _PCI_HEADER_TYPE_0,
    // type1/type2 omitted: same overall union size is dominated by DeviceSpecific tail;
    // the driver only reads u.type0, and the leading PCI header fields are what it casts at.
    pub _raw: [UCHAR; 48usize],
}

#[repr(C)]
pub struct _PCI_COMMON_CONFIG {
    pub VendorID: USHORT,
    pub DeviceID: USHORT,
    pub Command: USHORT,
    pub Status: USHORT,
    pub RevisionID: UCHAR,
    pub ProgIf: UCHAR,
    pub SubClass: UCHAR,
    pub BaseClass: UCHAR,
    pub CacheLineSize: UCHAR,
    pub LatencyTimer: UCHAR,
    pub HeaderType: UCHAR,
    pub BIST: UCHAR,
    pub u: _PCI_COMMON_CONFIG_u,
    pub DeviceSpecific: [UCHAR; 192usize],
}
pub type PCI_COMMON_CONFIG = _PCI_COMMON_CONFIG;
pub type PPCI_COMMON_CONFIG = *mut _PCI_COMMON_CONFIG;

// ---------------------------------------------------------------------------
// c2rust "phantom extern" macros — faithful Rust from drivers/storage/port/storahci/storahci.h
// ---------------------------------------------------------------------------

// storahci.h:62/67  ATA function bits
const ATA_FUNCTION_ATA_COMMAND: ULONG = 0x100;
const ATA_FUNCTION_ATAPI_COMMAND: ULONG = 0x200;
// storahci.h:70/71  data-transfer direction flags
const ATA_FLAGS_DATA_IN: ULONG = 1 << 1;
const ATA_FLAGS_DATA_OUT: ULONG = 1 << 2;
// storahci.h:35  HBA CAP S64A
const AHCI_Global_HBA_CAP_S64A: ULONG = 1 << 31;

/// storahci.h:75  `#define IsAtaCommand(AtaFunction) (AtaFunction & ATA_FUNCTION_ATA_COMMAND)`
#[inline]
pub unsafe fn IsAtaCommand(AtaFunction: ULONG) -> i32 {
    (AtaFunction & ATA_FUNCTION_ATA_COMMAND) as i32
}

/// storahci.h:76  `#define IsAtapiCommand(AtaFunction) (AtaFunction & ATA_FUNCTION_ATAPI_COMMAND)`
#[inline]
pub unsafe fn IsAtapiCommand(AtaFunction: ULONG) -> i32 {
    (AtaFunction & ATA_FUNCTION_ATAPI_COMMAND) as i32
}

/// storahci.h:77  `#define IsDataTransferNeeded(SrbExtension) (SrbExtension->Flags & (ATA_FLAGS_DATA_IN | ATA_FLAGS_DATA_OUT))`
#[inline]
pub unsafe fn IsDataTransferNeeded(SrbExtension: PAHCI_SRB_EXTENSION) -> i32 {
    ((*SrbExtension).Flags & (ATA_FLAGS_DATA_IN | ATA_FLAGS_DATA_OUT)) as i32
}

/// storahci.h:78  `#define IsAdapterCAPS64(CAP) (CAP & AHCI_Global_HBA_CAP_S64A)`
#[inline]
pub unsafe fn IsAdapterCAPS64(CAP: ULONG) -> BOOLEAN {
    (CAP & AHCI_Global_HBA_CAP_S64A) as BOOLEAN
}

/// storahci.h:81  `#define AHCI_Global_Port_CAP_NCS(x) (((x) & 0xF00) >> 8)`
#[inline]
pub unsafe fn AHCI_Global_Port_CAP_NCS(x: ULONG) -> ULONG {
    ((x) & 0xF00) >> 8
}

/// scsi.h:3498 `REVERSE_BYTES_QUAD` — reverse 8 bytes (big-endian → little-endian quad).
#[inline]
pub unsafe fn REVERSE_BYTES_QUAD(dest: *mut core::ffi::c_void, src: *const core::ffi::c_void) {
    let s = src as *const u8;
    let d = dest as *mut u8;
    *d.add(7) = *s.add(0);
    *d.add(6) = *s.add(1);
    *d.add(5) = *s.add(2);
    *d.add(4) = *s.add(3);
    *d.add(3) = *s.add(4);
    *d.add(2) = *s.add(5);
    *d.add(1) = *s.add(6);
    *d.add(0) = *s.add(7);
}

/// storahci.h:83  `#define ROUND_UP(N, S) ((((N) + (S) - 1) / (S)) * (S))`
/// Generic over the integer the callers use (ULONG); keeps the exact C arithmetic.
#[inline]
pub fn ROUND_UP<T>(n: T, s: T) -> T
where
    T: Copy
        + core::ops::Add<Output = T>
        + core::ops::Sub<Output = T>
        + core::ops::Div<Output = T>
        + core::ops::Mul<Output = T>
        + From<u8>,
{
    (((n + s) - T::from(1u8)) / s) * s
}

/// scsi.h:3511 `REVERSE_BYTES` — reverse 4 bytes (big-endian → little-endian dword).
/// Generic over the typed pointers the translation passes (cast to bytes, identical semantics).
#[inline]
pub unsafe fn REVERSE_BYTES<D, S>(dest: *mut D, src: *const S) {
    let s = src as *const u8;
    let d = dest as *mut u8;
    *d.add(3) = *s.add(0);
    *d.add(2) = *s.add(1);
    *d.add(1) = *s.add(2);
    *d.add(0) = *s.add(3);
}

/// `NT_ASSERT(c)` — debug-build assertion; a no-op in free builds (DUMMY_PSEH / NDEBUG).
/// Generic so the translation's `BOOLEAN`/`i32` argument forms both type-check.
#[inline]
pub unsafe fn NT_ASSERT<T>(_condition: T) {}

// ---------------------------------------------------------------------------
// StorPort FORCEINLINE wrappers and macro stubs
// (storahci.h defines these as C macros / FORCEINLINE — no symbol in storport.sys)
// ---------------------------------------------------------------------------

extern "C" {
    fn StorPortNotification(NotificationType: u32, HwDeviceExtension: PVOID, ...) -> ULONG;
}

// AhciDebugPrint is now a macro (see lib.rs) — stable Rust cannot define a C-variadic fn.

/// `StorPortCopyMemory(Dest, Src, Len)` — C macro wrapper around `memcpy`.
#[inline]
pub unsafe fn StorPortCopyMemory(Destination: PVOID, Source: PVOID, Length: usize) {
    core::ptr::copy_nonoverlapping(Source as *const u8, Destination as *mut u8, Length);
}

/// `StorPortEnablePassiveInitialization` — FORCEINLINE wrapper around `StorPortNotification`.
#[inline]
pub unsafe fn StorPortEnablePassiveInitialization(
    DeviceExtension: PVOID,
    Routine: PVOID,
) -> BOOLEAN {
    let mut succ: core::ffi::c_long = 0;
    StorPortNotification(
        _SCSI_NOTIFICATION_TYPE_EnablePassiveInitialization,
        DeviceExtension,
        Routine,
        core::ptr::addr_of_mut!(succ),
    );
    succ as BOOLEAN
}

/// `StorPortInitializeDpc` — FORCEINLINE wrapper around `StorPortNotification`.
#[inline]
pub unsafe fn StorPortInitializeDpc(
    DeviceExtension: PVOID,
    Dpc: PSTOR_DPC,
    HwDpcRoutine: PHW_DPC_ROUTINE,
) {
    StorPortNotification(
        _SCSI_NOTIFICATION_TYPE_InitializeDpc,
        DeviceExtension,
        Dpc,
        HwDpcRoutine,
    );
}

/// `StorPortIssueDpc` — FORCEINLINE wrapper around `StorPortNotification`.
#[inline]
pub unsafe fn StorPortIssueDpc(
    HwDeviceExtension: PVOID,
    Dpc: PVOID,
    SystemArgument1: PVOID,
    SystemArgument2: PVOID,
) -> BOOLEAN {
    let mut succ: core::ffi::c_long = 0;
    StorPortNotification(
        _SCSI_NOTIFICATION_TYPE_IssueDpc,
        HwDeviceExtension,
        Dpc,
        SystemArgument1,
        SystemArgument2,
        core::ptr::addr_of_mut!(succ),
    );
    succ as BOOLEAN
}

/// `StorPortAcquireSpinLock` — FORCEINLINE wrapper around `StorPortNotification`.
#[inline]
pub unsafe fn StorPortAcquireSpinLock(
    HwDeviceExtension: PVOID,
    LockType: STOR_SPINLOCK,
    LockContext: PVOID,
    LockHandle: PSTOR_LOCK_HANDLE,
) {
    StorPortNotification(
        _SCSI_NOTIFICATION_TYPE_AcquireSpinLock,
        HwDeviceExtension,
        LockType,
        LockContext,
        LockHandle,
    );
}

/// `StorPortReleaseSpinLock` — FORCEINLINE wrapper around `StorPortNotification`.
#[inline]
pub unsafe fn StorPortReleaseSpinLock(
    HwDeviceExtension: PVOID,
    LockHandle: PSTOR_LOCK_HANDLE,
) {
    StorPortNotification(
        _SCSI_NOTIFICATION_TYPE_ReleaseSpinLock,
        HwDeviceExtension,
        LockHandle,
    );
}
