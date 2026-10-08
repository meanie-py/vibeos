// Deterministic UTF-16 literals, generated from the C by rustconv (--seal-wide); the model never saw the bytes.
use crate::prelude::*;
pub static RUSTCONV_WSTR_075DE2B906DB: [u16; 2] = [0x000Au16, 0x0000];
pub static RUSTCONV_WSTR_DC70E3A7FF9A: [u16; 16] = [
    0x005Cu16, 0x0044, 0x006F, 0x0073, 0x0044, 0x0065, 0x0076, 0x0069, 0x0063, 0x0065,
    0x0073, 0x005C, 0x0025, 0x0063, 0x003A, 0x0000,
];

use core::ffi::c_void;

// ---- file-scope preamble (translated) ----
// @c2rust file-scope data (compiler-exact tables/statics)
pub const MOUNTMGR_DEVICE_NAME: [::core::ffi::c_ushort; 26] = unsafe {
    ::core::mem::transmute::<[u8; 52], [::core::ffi::c_ushort; 26]>(
        *b"\\\0D\0e\0v\0i\0c\0e\0\\\0M\0o\0u\0n\0t\0P\0o\0i\0n\0t\0M\0a\0n\0a\0g\0e\0r\0\0\0",
    )
};

unsafe extern "C" fn OpenMountManager(
    MountMgrHandle: PHANDLE,
    Access: ACCESS_MASK,
) -> NTSTATUS {
    let mut ObjectAttributes: _OBJECT_ATTRIBUTES = _OBJECT_ATTRIBUTES {
        Length: 0,
        RootDirectory: core::ptr::null_mut(),
        ObjectName: core::ptr::null_mut(),
        Attributes: 0,
        SecurityDescriptor: core::ptr::null_mut(),
        SecurityQualityOfService: core::ptr::null_mut(),
    };
    let mut DeviceName: _UNICODE_STRING = _UNICODE_STRING {
        Length: 0,
        MaximumLength: 0,
        Buffer: core::ptr::null_mut(),
    };
    let mut Iosb: _IO_STATUS_BLOCK = { let mut __lit: _IO_STATUS_BLOCK = core::mem::zeroed(); __lit.__bindgen_anon_1.Status = 0; __lit.Information = 0; __lit };

    RtlInitUnicodeString(&mut DeviceName, (MOUNTMGR_DEVICE_NAME).as_ptr());

    InitializeObjectAttributes!(
        &mut ObjectAttributes,
        &mut DeviceName,
        0,
        core::ptr::null_mut(),
        core::ptr::null_mut(),
    );

    return NtOpenFile(
        MountMgrHandle,
        Access | SYNCHRONIZE,
        &mut ObjectAttributes,
        &mut Iosb,
        0,
        FILE_SYNCHRONOUS_IO_NONALERT,
    );
}
#[no_mangle]
pub unsafe extern "C" fn GetAutomountState(State: PBOOL) -> BOOL {
    let mut MountMgrHandle: *mut core::ffi::c_void = core::ptr::null_mut();
    let mut AutoMount: MOUNTMGR_QUERY_AUTO_MOUNT = MOUNTMGR_QUERY_AUTO_MOUNT {
        CurrentState: Disabled,
    };
    let mut Iosb: _IO_STATUS_BLOCK = { let mut __lit: _IO_STATUS_BLOCK = core::mem::zeroed(); __lit.__bindgen_anon_1.Status = 0; __lit.Information = 0; __lit };
    let mut Status: i32 = 0;

    DPRINT!(b"ShowAutomountState()\n\0".as_ptr() as *const i8);

    Status = OpenMountManager(&mut MountMgrHandle, GENERIC_READ);
    if !NT_SUCCESS!(Status) {
        DPRINT1!(
            b"OpenMountManager() Status 0x%08lx\n\0".as_ptr() as *const i8,
            Status,
        );
        return FALSE;
    }

    Status = NtDeviceIoControlFile(
        MountMgrHandle,
        core::ptr::null_mut(),
        None,
        core::ptr::null_mut(),
        &mut Iosb,
        IOCTL_MOUNTMGR_QUERY_AUTO_MOUNT,
        core::ptr::null_mut(),
        0,
        &mut AutoMount as *mut _ as *mut core::ffi::c_void,
        core::mem::size_of::<MOUNTMGR_QUERY_AUTO_MOUNT>() as u32,
    );

    NtClose(MountMgrHandle);

    if !NT_SUCCESS!(Status) {
        DPRINT1!(
            b"NtDeviceIoControlFile() Status 0x%08lx\n\0".as_ptr() as *const i8,
            Status,
        );
        return FALSE;
    }

    if !State.is_null() {
        *State = (AutoMount.CurrentState == Enabled) as i32;
    }

    return TRUE;
}
#[no_mangle]
pub unsafe extern "C" fn SetAutomountState(bEnable: i32) -> BOOL {
    let mut MountMgrHandle: *mut core::ffi::c_void = core::ptr::null_mut();
    let mut AutoMount: MOUNTMGR_SET_AUTO_MOUNT = MOUNTMGR_SET_AUTO_MOUNT { NewState: Disabled };
    let mut Iosb: _IO_STATUS_BLOCK = { let mut __lit: _IO_STATUS_BLOCK = core::mem::zeroed(); __lit.__bindgen_anon_1.Status = 0; __lit.Information = 0; __lit };
    let mut Status: i32 = 0;

    DPRINT!(b"SetAutomountState()\n\0".as_ptr() as *const i8);

    Status = OpenMountManager(
        &mut MountMgrHandle,
        GENERIC_READ | GENERIC_WRITE,
    );
    if !NT_SUCCESS!(Status) {
        DPRINT1!(
            b"OpenMountManager() Status 0x%08lx\n\0".as_ptr() as *const i8,
            Status,
        );
        return TRUE;
    }

    if bEnable != 0 {
        AutoMount.NewState = Enabled;
    } else {
        AutoMount.NewState = Disabled;
    }

    Status = NtDeviceIoControlFile(
        MountMgrHandle,
        core::ptr::null_mut(),
        None,
        core::ptr::null_mut(),
        &mut Iosb,
        IOCTL_MOUNTMGR_SET_AUTO_MOUNT,
        &mut AutoMount as *mut MOUNTMGR_SET_AUTO_MOUNT as *mut core::ffi::c_void,
        core::mem::size_of::<MOUNTMGR_SET_AUTO_MOUNT>() as u32,
        core::ptr::null_mut(),
        0,
    );

    NtClose(MountMgrHandle);

    if !NT_SUCCESS!(Status) {
        DPRINT1!(
            b"NtDeviceIoControlFile() Status 0x%08lx\n\0".as_ptr() as *const i8,
            Status,
        );
        return TRUE;
    }

    if AutoMount.NewState == Enabled {
        ConResPuts(StdOut, IDS_AUTOMOUNT_ENABLED);
    } else {
        ConResPuts(StdOut, IDS_AUTOMOUNT_DISABLED);
    }
    ConPuts(StdOut, RUSTCONV_WSTR_075DE2B906DB.as_ptr());

    return TRUE;
}
#[no_mangle]
pub unsafe extern "C" fn ScrubAutomount() -> BOOL {
    let mut MountMgrHandle: *mut core::ffi::c_void = core::ptr::null_mut();
    let mut Iosb: _IO_STATUS_BLOCK = { let mut __lit: _IO_STATUS_BLOCK = core::mem::zeroed(); __lit.__bindgen_anon_1.Status = 0; __lit.Information = 0; __lit };
    let mut Status: i32 = 0;

    DPRINT!(b"ScrubAutomount()\n\0".as_ptr() as *const i8);

    Status = OpenMountManager(
        &mut MountMgrHandle,
        GENERIC_READ | GENERIC_WRITE,
    );
    if !NT_SUCCESS!(Status) {
        DPRINT1!(
            b"OpenMountManager() Status 0x%08lx\n\0".as_ptr() as *const i8,
            Status,
        );
        return FALSE;
    }

    Status = NtDeviceIoControlFile(
        MountMgrHandle,
        core::ptr::null_mut(),
        None,
        core::ptr::null_mut(),
        &mut Iosb,
        IOCTL_MOUNTMGR_SCRUB_REGISTRY,
        core::ptr::null_mut(),
        0,
        core::ptr::null_mut(),
        0,
    );

    NtClose(MountMgrHandle);

    if !NT_SUCCESS!(Status) {
        DPRINT1!(
            b"NtDeviceIoControlFile() Status 0x%08lx\n\0".as_ptr() as *const i8,
            Status,
        );
        return FALSE;
    }

    return TRUE;
}
#[no_mangle]
pub unsafe extern "C" fn AssignDriveLetter(
    DeviceName: PWSTR,
    DriveLetter: u16,
) -> BOOL {
    let mut DosDeviceName: [u16; 30] = [0; 30];
    let mut DosDeviceNameLength: u32 = 0;
    let mut DeviceNameLength: u32 = 0;
    let mut InputBufferLength: u32 = 0;
    let mut InputBuffer: *mut MOUNTMGR_CREATE_POINT_INPUT = core::ptr::null_mut();
    let mut MountMgrHandle: *mut core::ffi::c_void = core::ptr::null_mut();
    let mut Iosb: _IO_STATUS_BLOCK = core::mem::zeroed();
    let mut Status: i32 = 0;
    let mut Ret: i32 = TRUE;

    DPRINT1!(
        b"AssignDriveLetter(%S %c)\n\0".as_ptr() as *const core::ffi::c_char,
        DeviceName,
        DriveLetter as i32,
    );

    DeviceNameLength = (wcslen(DeviceName as *const u16) as usize)
        .wrapping_mul(core::mem::size_of::<u16>()) as u32;

    _swprintf(
        core::ptr::addr_of_mut!(DosDeviceName) as *mut u16,
        RUSTCONV_WSTR_DC70E3A7FF9A.as_ptr(),
        DriveLetter as i32,
    );
    DosDeviceNameLength = (wcslen(
        core::ptr::addr_of_mut!(DosDeviceName) as *mut u16 as *const u16,
    ) as usize)
        .wrapping_mul(core::mem::size_of::<u16>()) as u32;

    InputBufferLength = (DosDeviceNameLength as usize)
        .wrapping_add(DeviceNameLength as usize)
        .wrapping_add(core::mem::size_of::<MOUNTMGR_CREATE_POINT_INPUT>()) as u32;
    InputBuffer = RtlAllocateHeap(
        RtlGetProcessHeap!(),
        HEAP_ZERO_MEMORY,
        (InputBufferLength) as SIZE_T,
    ) as *mut MOUNTMGR_CREATE_POINT_INPUT;
    if InputBuffer.is_null() {
        DPRINT1!(
            b"InputBuffer allocation failed!\n\0".as_ptr() as *const core::ffi::c_char,
        );
        return FALSE;
    }

    (*InputBuffer).SymbolicLinkNameOffset =
        core::mem::size_of::<MOUNTMGR_CREATE_POINT_INPUT>() as u16;
    (*InputBuffer).SymbolicLinkNameLength = DosDeviceNameLength as u16;
    (*InputBuffer).DeviceNameOffset = (DosDeviceNameLength as usize)
        .wrapping_add(core::mem::size_of::<MOUNTMGR_CREATE_POINT_INPUT>()) as u16;
    (*InputBuffer).DeviceNameLength = DeviceNameLength as u16;
    RtlCopyMemory!(
        (InputBuffer as usize)
            .wrapping_add(core::mem::size_of::<MOUNTMGR_CREATE_POINT_INPUT>())
            as *mut core::ffi::c_void,
        core::ptr::addr_of_mut!(DosDeviceName) as *mut u16 as *const core::ffi::c_void,
        DosDeviceNameLength as usize,
    );
    RtlCopyMemory!(
        (InputBuffer as usize)
            .wrapping_add((*InputBuffer).DeviceNameOffset as usize)
            as *mut core::ffi::c_void,
        DeviceName as *const core::ffi::c_void,
        DeviceNameLength as usize,
    );

    'done: {
        Status = OpenMountManager(
            core::ptr::addr_of_mut!(MountMgrHandle),
            GENERIC_READ | GENERIC_WRITE,
        );
        if !NT_SUCCESS!(Status) {
            DPRINT1!(
                b"OpenMountManager() failed (Status 0x%08lx)\n\0".as_ptr()
                    as *const core::ffi::c_char,
                Status,
            );
            Ret = FALSE;
            break 'done;
        }

        Status = NtDeviceIoControlFile(
            MountMgrHandle,
            core::ptr::null_mut(),
            None,
            core::ptr::null_mut(),
            core::ptr::addr_of_mut!(Iosb),
            IOCTL_MOUNTMGR_CREATE_POINT,
            InputBuffer as *mut core::ffi::c_void,
            InputBufferLength,
            core::ptr::null_mut(),
            0,
        );
        if !NT_SUCCESS!(Status) {
            DPRINT1!(
                b"NtDeviceIoControlFile() failed (Status 0x08lx)\n\0".as_ptr()
                    as *const core::ffi::c_char,
                Status,
            );
            Ret = FALSE;
            break 'done;
        }
    }

    if !MountMgrHandle.is_null() {
        NtClose(MountMgrHandle);
    }

    RtlFreeHeap(
        RtlGetProcessHeap!(),
        0,
        InputBuffer as *mut core::ffi::c_void,
    );

    return Ret;
}
#[no_mangle]
pub unsafe extern "C" fn AssignNextDriveLetter(
    DeviceName: PWSTR,
    DriveLetter: PWCHAR,
) -> BOOL {
    let mut LetterInfo: MOUNTMGR_DRIVE_LETTER_INFORMATION = MOUNTMGR_DRIVE_LETTER_INFORMATION {
        DriveLetterWasAssigned: 0,
        CurrentDriveLetter: 0,
    };
    let mut InputBuffer: PMOUNTMGR_DRIVE_LETTER_TARGET = core::ptr::null_mut();
    let mut DeviceNameLength: ULONG = 0;
    let mut InputBufferLength: ULONG = 0;
    let mut MountMgrHandle: HANDLE = core::ptr::null_mut();
    let mut Iosb: _IO_STATUS_BLOCK = { let mut __lit: _IO_STATUS_BLOCK = core::mem::zeroed(); __lit.__bindgen_anon_1.Status = 0; __lit.Information = 0; __lit };
    let mut Status: NTSTATUS = 0;
    let mut Ret: BOOL = TRUE;

    DPRINT!("AssignNextDriveLetter(%S %p)\n", DeviceName, DriveLetter);

    DeviceNameLength = (wcslen(DeviceName) * core::mem::size_of::<WCHAR>()) as ULONG;

    InputBufferLength = DeviceNameLength + FIELD_OFFSET!(MOUNTMGR_DRIVE_LETTER_TARGET, DeviceName) as ULONG; // C: LONG offset converted to ULONG
    InputBuffer = (RtlAllocateHeap(
        RtlGetProcessHeap!(),
        HEAP_ZERO_MEMORY,
        (InputBufferLength) as SIZE_T,
    )).cast();
    if InputBuffer.is_null() {
        DPRINT1!("InputBuffer allocation failed!\n");
        return FALSE;
    }

    (*InputBuffer).DeviceNameLength = DeviceNameLength as USHORT;
    RtlCopyMemory!(
        (&raw mut (*InputBuffer).DeviceName).offset(0 as isize) as *mut core::ffi::c_void,
        DeviceName as *const core::ffi::c_void,
        DeviceNameLength as usize,
    );

    Status = OpenMountManager(
        &raw mut MountMgrHandle,
        GENERIC_READ | GENERIC_WRITE,
    );
    if !NT_SUCCESS!(Status) {
        DPRINT1!("OpenMountManager() failed (Status 0x%08lx)\n", Status);
        Ret = FALSE;
    } else {
        Status = NtDeviceIoControlFile(
            MountMgrHandle,
            core::ptr::null_mut(),
            None,
            core::ptr::null_mut(),
            &raw mut Iosb,
            IOCTL_MOUNTMGR_CREATE_POINT,
            InputBuffer as *mut core::ffi::c_void,
            InputBufferLength,
            &raw mut LetterInfo as *mut core::ffi::c_void,
            core::mem::size_of::<MOUNTMGR_DRIVE_LETTER_INFORMATION>() as ULONG,
        );
        if !NT_SUCCESS!(Status) {
            DPRINT1!("NtDeviceIoControlFile() failed (Status 0x%08lx)\n", Status);
            Ret = FALSE;
        }
    }

    if !MountMgrHandle.is_null() {
        NtClose(MountMgrHandle);
    }

    RtlFreeHeap(RtlGetProcessHeap!(), 0, (InputBuffer).cast());

    if Ret != 0 {
        if LetterInfo.DriveLetterWasAssigned != 0 {
            *DriveLetter = LetterInfo.CurrentDriveLetter as WCHAR;
        } else {
            *DriveLetter = UNICODE_NULL;
        }
    }

    return Ret;
}
#[no_mangle]
pub unsafe extern "C" fn DeleteDriveLetter(DriveLetter: u16) -> BOOL {
    let mut InputBuffer: *mut MOUNTMGR_MOUNT_POINT = core::ptr::null_mut();
    let mut OutputBuffer: *mut MOUNTMGR_MOUNT_POINTS = core::ptr::null_mut();
    let mut DosDeviceName: [u16; 30] = [0; 30];
    let mut InputBufferLength: u32;
    let mut DosDeviceNameLength: u32;
    let mut OutputBufferLength: u32 = 0x1000;
    let mut MountMgrHandle: *mut core::ffi::c_void = core::ptr::null_mut();
    let mut Iosb: _IO_STATUS_BLOCK = core::mem::zeroed();
    let mut Status: i32;
    let mut Ret: i32 = TRUE;

    DPRINT!(b"DeleteDriveLetter(%c)\n\0".as_ptr() as *const i8, DriveLetter as i32);

    _swprintf(
        DosDeviceName.as_mut_ptr(),
        RUSTCONV_WSTR_DC70E3A7FF9A.as_ptr(),
        DriveLetter as i32,
    );
    DosDeviceNameLength = (wcslen(DosDeviceName.as_ptr()) as u32)
        .wrapping_mul(core::mem::size_of::<u16>() as u32);

    InputBufferLength = DosDeviceNameLength
        .wrapping_add(core::mem::size_of::<MOUNTMGR_MOUNT_POINT>() as u32);
    InputBuffer = RtlAllocateHeap(
        RtlGetProcessHeap!(),
        HEAP_ZERO_MEMORY,
        (InputBufferLength) as SIZE_T,
    ) as *mut MOUNTMGR_MOUNT_POINT;
    if InputBuffer.is_null() {
        DPRINT1!(b"InputBuffer allocation failed!\n\0".as_ptr() as *const i8);
        return FALSE;
    }

    (*InputBuffer).SymbolicLinkNameOffset = core::mem::size_of::<MOUNTMGR_MOUNT_POINT>() as u32;
    (*InputBuffer).SymbolicLinkNameLength = DosDeviceNameLength as u16;
    RtlCopyMemory!(
        InputBuffer.add(1) as *mut core::ffi::c_void,
        DosDeviceName.as_ptr() as *const core::ffi::c_void,
        DosDeviceNameLength as usize,
    );

    OutputBuffer = RtlAllocateHeap(
        GetProcessHeap(),
        HEAP_ZERO_MEMORY,
        (OutputBufferLength) as SIZE_T,
    ) as *mut MOUNTMGR_MOUNT_POINTS;
    if OutputBuffer.is_null() {
        DPRINT1!(b"OutputBuffer allocation failed!\n\0".as_ptr() as *const i8);
        Ret = FALSE;
    } else {
        (*OutputBuffer).Size = OutputBufferLength;

        Status = OpenMountManager(
            &mut MountMgrHandle,
            GENERIC_READ | GENERIC_WRITE,
        );
        if !NT_SUCCESS!(Status) {
            DPRINT1!(
                b"OpenMountManager() failed (Status 0x%08lx)\n\0".as_ptr() as *const i8,
                Status,
            );
            Ret = FALSE;
        } else {
            Status = NtDeviceIoControlFile(
                MountMgrHandle,
                core::ptr::null_mut(),
                None,
                core::ptr::null_mut(),
                &mut Iosb,
                IOCTL_MOUNTMGR_DELETE_POINTS,
                InputBuffer as *mut core::ffi::c_void,
                InputBufferLength,
                OutputBuffer as *mut core::ffi::c_void,
                OutputBufferLength,
            );
            if !NT_SUCCESS!(Status) {
                DPRINT1!(
                    b"NtDeviceIoControlFile() failed (Status 0x%08lx)\n\0".as_ptr() as *const i8,
                    Status,
                );
                Ret = FALSE;
            }
        }
    }

    if !MountMgrHandle.is_null() {
        NtClose(MountMgrHandle);
    }

    if !InputBuffer.is_null() {
        RtlFreeHeap(
            RtlGetProcessHeap!(),
            0,
            InputBuffer as *mut core::ffi::c_void,
        );
    }

    if !OutputBuffer.is_null() {
        RtlFreeHeap(
            RtlGetProcessHeap!(),
            0,
            OutputBuffer as *mut core::ffi::c_void,
        );
    }

    return Ret;
}
