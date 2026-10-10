use crate::prelude::*;
use core::ffi::c_void;

// ---- file-scope preamble (translated) ----
// @c2rust file-scope data (compiler-exact tables/statics)
static mut D3DKMTEmuAdapters: [D3DKMT_EMU_ADAPTER; 16] = [_D3DKMT_EMU_ADAPTER {
    InUse: 0,
    AdapterLuid: _LUID {
        LowPart: 0,
        HighPart: 0,
    },
    VidPnSourceId: 0,
}; 16];

static mut D3DKMTEmuDevices: [D3DKMT_EMU_DEVICE; 64] = [_D3DKMT_EMU_DEVICE {
    InUse: 0,
    hAdapter: 0,
}; 64];

#[allow(non_camel_case_types, non_snake_case, dead_code)]
#[repr(C)]
pub struct D3DKMT_OPENADAPTERFROMLUID {
    pub AdapterLuid: LUID,
    pub hAdapter: D3DKMT_HANDLE,
}
pub type PD3DKMT_OPENADAPTERFROMLUID = *mut D3DKMT_OPENADAPTERFROMLUID;

#[allow(non_camel_case_types, non_snake_case, dead_code)]
#[repr(C)]
pub struct D3DKMT_QUERYVIDEOMEMORYINFO {
    pub hProcess: HANDLE,
    pub hAdapter: D3DKMT_HANDLE,
    pub MemorySegmentGroup: D3DKMT_MEMORY_SEGMENT_GROUP,
    pub Budget: u64,
    pub CurrentUsage: u64,
    pub CurrentReservation: u64,
    pub AvailableForReservation: u64,
    pub PhysicalAdapterIndex: u32,
}
pub type PD3DKMT_QUERYVIDEOMEMORYINFO = *mut D3DKMT_QUERYVIDEOMEMORYINFO;

pub const D3DKMT_EMU_ADAPTER_TAG: u32 = 0x0ada0000;

pub const D3DKMT_EMU_DEVICE_TAG: u32 = 0x0de00000;

pub const D3DKMT_EMU_INDEX_MASK: u32 = 0x0000ffff;

pub const D3DKMT_EMU_MAX_ADAPTERS: usize = 16;

pub const D3DKMT_EMU_MAX_DEVICES: usize = 64;




unsafe extern "C" fn D3DKMTEmuGetAdapter(mut hAdapter: D3DKMT_HANDLE) -> *mut D3DKMT_EMU_ADAPTER {
    let mut Index: ULONG = 0;
    if hAdapter as ::core::ffi::c_uint & !D3DKMT_EMU_INDEX_MASK != D3DKMT_EMU_ADAPTER_TAG {
        return ::core::ptr::null_mut::<D3DKMT_EMU_ADAPTER>();
    }
    Index = (hAdapter as ::core::ffi::c_uint & D3DKMT_EMU_INDEX_MASK) as ULONG;
    if Index == 0 as ULONG || Index > D3DKMT_EMU_MAX_ADAPTERS as ULONG {
        return ::core::ptr::null_mut::<D3DKMT_EMU_ADAPTER>();
    }
    if (*core::ptr::addr_of_mut!(D3DKMTEmuAdapters))[Index.wrapping_sub(1 as ULONG) as usize].InUse == 0 {
        return ::core::ptr::null_mut::<D3DKMT_EMU_ADAPTER>();
    }
    return (core::ptr::addr_of_mut!(D3DKMTEmuAdapters) as *mut D3DKMT_EMU_ADAPTER)
        .offset(Index.wrapping_sub(1 as ULONG) as isize) as *mut D3DKMT_EMU_ADAPTER;
}

unsafe extern "C" fn D3DKMTEmuGetDevice(mut hDevice: D3DKMT_HANDLE) -> *mut D3DKMT_EMU_DEVICE {
    let mut Index: ULONG = 0;
    if hDevice as ::core::ffi::c_uint & !D3DKMT_EMU_INDEX_MASK != D3DKMT_EMU_DEVICE_TAG {
        return ::core::ptr::null_mut::<D3DKMT_EMU_DEVICE>();
    }
    Index = (hDevice as ::core::ffi::c_uint & D3DKMT_EMU_INDEX_MASK) as ULONG;
    if Index == 0 as ULONG || Index > D3DKMT_EMU_MAX_DEVICES as ULONG {
        return ::core::ptr::null_mut::<D3DKMT_EMU_DEVICE>();
    }
    if D3DKMTEmuDevices[Index.wrapping_sub(1 as ULONG) as usize].InUse == 0 {
        return ::core::ptr::null_mut::<D3DKMT_EMU_DEVICE>();
    }
    return (&raw mut D3DKMTEmuDevices as *mut D3DKMT_EMU_DEVICE)
        .offset(Index.wrapping_sub(1 as ULONG) as isize) as *mut D3DKMT_EMU_DEVICE;
}

unsafe extern "C" fn D3DKMTEmuOpenAdapter(
    mut AdapterLuid: *const LUID,
    mut VidPnSourceId: D3DDDI_VIDEO_PRESENT_SOURCE_ID,
) -> D3DKMT_HANDLE {
    let mut Index: ULONG = 0;
    Index = 0 as ULONG;
    while Index < D3DKMT_EMU_MAX_ADAPTERS as ULONG {
        if InterlockedCompareExchange(
            &raw mut D3DKMTEmuAdapters[Index as usize].InUse,
            1 as LONG,
            0 as LONG,
        ) == 0 as LONG
        {
            D3DKMTEmuAdapters[Index as usize].AdapterLuid = *AdapterLuid;
            D3DKMTEmuAdapters[Index as usize].VidPnSourceId = VidPnSourceId;
            return (D3DKMT_EMU_ADAPTER_TAG as ULONG
                | Index.wrapping_add(1 as ULONG))
                as D3DKMT_HANDLE;
        }
        Index = Index.wrapping_add(1);
    }

    return 0 as D3DKMT_HANDLE;
}

unsafe extern "C" fn D3DKMTEmuMakeLuid(
    mut VidPnSourceId: D3DDDI_VIDEO_PRESENT_SOURCE_ID,
    mut AdapterLuid: *mut LUID,
) {
    (*AdapterLuid).HighPart = 0x524f5300 as LONG;
    (*AdapterLuid).LowPart =
        VidPnSourceId.wrapping_add(1 as D3DDDI_VIDEO_PRESENT_SOURCE_ID) as DWORD;
}

#[no_mangle]
pub unsafe extern "system" fn D3DKMTOpenAdapterFromGdiDisplayName(
    unnamedParam1: *mut D3DKMT_OPENADAPTERFROMGDIDISPLAYNAME,
) -> NTSTATUS {
    let mut VidPnSourceId: D3DDDI_VIDEO_PRESENT_SOURCE_ID = 0;
    let mut DisplayDevice: DISPLAY_DEVICEW = core::mem::zeroed();
    let mut hAdapter: D3DKMT_HANDLE = 0;
    let mut AdapterLuid: LUID = core::mem::zeroed();
    let mut Index: DWORD = 0;

    if unnamedParam1.is_null() {
        return STATUS_INVALID_PARAMETER;
    }

    /* Locate the GDI display device with this name to obtain its index, which
       doubles as the VidPN source ID */
    DisplayDevice.cb = core::mem::size_of::<DISPLAY_DEVICEW>() as DWORD;
    while EnumDisplayDevicesW(
        core::ptr::null(),
        Index,
        core::ptr::addr_of_mut!(DisplayDevice),
        0,
    ) != 0
    {
        if wcsncmp(
            core::ptr::addr_of_mut!(DisplayDevice.DeviceName) as *mut WCHAR,
            core::ptr::addr_of_mut!((*unnamedParam1).DeviceName) as *mut WCHAR,
            (ARRAYSIZE!((*unnamedParam1).DeviceName)) as _,
        ) == 0
        {
            VidPnSourceId = Index;
            break;
        }

        DisplayDevice.cb = core::mem::size_of::<DISPLAY_DEVICEW>() as DWORD;
        Index += 1;
    }

    if DisplayDevice.DeviceName[0 as usize] == 0
        || VidPnSourceId as DWORD != Index
    {
        return STATUS_INVALID_PARAMETER;
    }

    D3DKMTEmuMakeLuid(VidPnSourceId, core::ptr::addr_of_mut!(AdapterLuid));

    hAdapter = D3DKMTEmuOpenAdapter(core::ptr::addr_of!(AdapterLuid), VidPnSourceId);
    if hAdapter == 0 {
        return STATUS_INSUFFICIENT_RESOURCES;
    }

    (*unnamedParam1).hAdapter = hAdapter;
    (*unnamedParam1).AdapterLuid = AdapterLuid;
    (*unnamedParam1).VidPnSourceId = VidPnSourceId;

    return STATUS_SUCCESS;
}

#[no_mangle]
pub unsafe extern "system" fn D3DKMTOpenAdapterFromLuid(
    unnamedParam1: *const D3DKMT_OPENADAPTERFROMLUID,
) -> NTSTATUS {
    let mut Desc: *mut D3DKMT_OPENADAPTERFROMLUID = unnamedParam1 as *mut D3DKMT_OPENADAPTERFROMLUID;
    let mut VidPnSourceId: D3DDDI_VIDEO_PRESENT_SOURCE_ID = 0;
    let mut hAdapter: D3DKMT_HANDLE = 0;

    if unnamedParam1.is_null() {
        return STATUS_INVALID_PARAMETER;
    }

    if (*unnamedParam1).AdapterLuid.HighPart == 0x524f5300 as LONG
        && (*unnamedParam1).AdapterLuid.LowPart > 0 as DWORD
    {
        VidPnSourceId = (*unnamedParam1)
            .AdapterLuid
            .LowPart
            .wrapping_sub(1 as DWORD) as D3DDDI_VIDEO_PRESENT_SOURCE_ID;
    } else {
        VidPnSourceId = 0 as D3DDDI_VIDEO_PRESENT_SOURCE_ID;
    }

    hAdapter = D3DKMTEmuOpenAdapter(
        core::ptr::addr_of!((*unnamedParam1).AdapterLuid),
        VidPnSourceId,
    );
    if hAdapter == 0 {
        return STATUS_INSUFFICIENT_RESOURCES;
    }

    (*Desc).hAdapter = hAdapter;

    return STATUS_SUCCESS;
}

#[no_mangle]
pub unsafe extern "system" fn D3DKMTCloseAdapter(unnamedParam1: *const D3DKMT_CLOSEADAPTER) -> NTSTATUS {
    let mut Adapter: *mut D3DKMT_EMU_ADAPTER;
    let mut Status: NTSTATUS;

    if unnamedParam1.is_null() {
        return STATUS_INVALID_PARAMETER;
    }

    Status = NtGdiDdDDICloseAdapter(unnamedParam1);
    if Status != STATUS_PROCEDURE_NOT_FOUND {
        return Status;
    }

    Adapter = D3DKMTEmuGetAdapter((*unnamedParam1).hAdapter);
    if Adapter.is_null() {
        return STATUS_INVALID_PARAMETER;
    }

    InterlockedExchange(
        core::ptr::addr_of_mut!((*Adapter).InUse) as *mut ::core::ffi::c_long,
        0 as ::core::ffi::c_long,
    );

    return STATUS_SUCCESS;
}

#[no_mangle]
pub unsafe extern "system" fn D3DKMTCreateDevice(
    unnamedParam1: *mut D3DKMT_CREATEDEVICE,
) -> NTSTATUS {
    let mut Status: NTSTATUS;
    let mut Index: ULONG;

    if unnamedParam1.is_null() {
        return STATUS_INVALID_PARAMETER;
    }

    Status = NtGdiDdDDICreateDevice(unnamedParam1);
    if Status != STATUS_PROCEDURE_NOT_FOUND {
        return Status;
    }

    if D3DKMTEmuGetAdapter((*unnamedParam1).__bindgen_anon_1.hAdapter).is_null() {
        return STATUS_INVALID_PARAMETER;
    }

    Index = 0;
    while Index < (D3DKMT_EMU_MAX_DEVICES as u32) {
        if InterlockedCompareExchange(
            core::ptr::addr_of_mut!(D3DKMTEmuDevices[Index as usize].InUse),
            1,
            0,
        ) == 0
        {
            D3DKMTEmuDevices[Index as usize].hAdapter = (*unnamedParam1).__bindgen_anon_1.hAdapter;

            (*unnamedParam1).hDevice = (D3DKMT_EMU_DEVICE_TAG
                | (Index.wrapping_add(1)))
                as D3DKMT_HANDLE;
            (*unnamedParam1).pCommandBuffer = core::ptr::null_mut();
            (*unnamedParam1).CommandBufferSize = 0;
            (*unnamedParam1).pAllocationList = core::ptr::null_mut();
            (*unnamedParam1).AllocationListSize = 0;
            (*unnamedParam1).pPatchLocationList = core::ptr::null_mut();
            (*unnamedParam1).PatchLocationListSize = 0;

            return STATUS_SUCCESS;
        }
        Index = Index.wrapping_add(1);
    }

    return STATUS_INSUFFICIENT_RESOURCES;
}

#[no_mangle]
pub unsafe extern "system" fn D3DKMTDestroyDevice(
    unnamedParam1: *const D3DKMT_DESTROYDEVICE,
) -> NTSTATUS {
    let mut Device: *mut D3DKMT_EMU_DEVICE = core::ptr::null_mut();
    let mut Status: NTSTATUS;

    if unnamedParam1.is_null() {
        return STATUS_INVALID_PARAMETER;
    }

    Status = NtGdiDdDDIDestroyDevice(unnamedParam1);
    if Status != STATUS_PROCEDURE_NOT_FOUND {
        return Status;
    }

    Device = D3DKMTEmuGetDevice((*unnamedParam1).hDevice);
    if Device.is_null() {
        return STATUS_INVALID_PARAMETER;
    }

    InterlockedExchange(
        core::ptr::addr_of_mut!((*Device).InUse),
        0,
    );

    return STATUS_SUCCESS;
}

#[no_mangle]
pub unsafe extern "system" fn D3DKMTSetVidPnSourceOwner(
    unnamedParam1: *const D3DKMT_SETVIDPNSOURCEOWNER,
) -> NTSTATUS {
    let mut Status: NTSTATUS;

    if unnamedParam1.is_null() {
        return STATUS_INVALID_PARAMETER;
    }

    Status = NtGdiDdDDISetVidPnSourceOwner(unnamedParam1);
    if Status != STATUS_PROCEDURE_NOT_FOUND {
        return Status;
    }

    /* A zero VidPnSourceCount releases ownership, which always succeeds. */
    if (*unnamedParam1).VidPnSourceCount == 0 {
        return STATUS_SUCCESS;
    }

    if D3DKMTEmuGetDevice((*unnamedParam1).hDevice).is_null() {
        return STATUS_INVALID_PARAMETER;
    }

    return STATUS_PROCEDURE_NOT_FOUND;
}

#[no_mangle]
pub unsafe extern "system" fn D3DKMTCheckVidPnExclusiveOwnership(
    unnamedParam1: *const D3DKMT_CHECKVIDPNEXCLUSIVEOWNERSHIP,
) -> NTSTATUS {
    if unnamedParam1.is_null() {
        return STATUS_INVALID_PARAMETER;
    }

    return STATUS_PROCEDURE_NOT_FOUND;
}

#[no_mangle]
pub unsafe extern "system" fn D3DKMTQueryVideoMemoryInfo(
    unnamedParam1: *mut D3DKMT_QUERYVIDEOMEMORYINFO,
) -> NTSTATUS {
    if unnamedParam1.is_null() {
        return STATUS_INVALID_PARAMETER;
    }

    if D3DKMTEmuGetAdapter((*unnamedParam1).hAdapter).is_null() {
        return STATUS_INVALID_PARAMETER;
    }

    return STATUS_PROCEDURE_NOT_FOUND;
}
