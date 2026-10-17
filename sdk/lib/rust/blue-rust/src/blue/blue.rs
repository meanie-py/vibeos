// Deterministic UTF-16 literals, generated from the C by rustconv (--seal-wide); the model never saw the bytes.
pub static RUSTCONV_WSTR_38821E379CB2: [u16; 15] = [
    0x005Cu16, 0x003F, 0x003F, 0x005C, 0x0042, 0x006C, 0x0075, 0x0065, 0x0053, 0x0063,
    0x0072, 0x0065, 0x0065, 0x006E, 0x0000,
];
pub static RUSTCONV_WSTR_3C4AFF199191: [u16; 19] = [
    0x005Cu16, 0x0044, 0x0065, 0x0076, 0x0069, 0x0063, 0x0065, 0x005C, 0x0042, 0x006C,
    0x0075, 0x0065, 0x0053, 0x0063, 0x0072, 0x0065, 0x0065, 0x006E, 0x0000,
];

use crate::prelude::*;
use ntkrnl_sys::ddk::{InbvCheckDisplayOwnership, InbvNotifyDisplayOwnershipLost, IoCreateDevice, IoCreateSymbolicLink, IoDeleteDevice, KeDelayExecutionThread, KeSetPriorityThread, LARGE_INTEGER, MmMapIoSpace, MmUnmapIoSpace, _OBJECT_ATTRIBUTES, PDEVICE_OBJECT, PDRIVER_OBJECT, PHYSICAL_ADDRESS, PIO_STACK_LOCATION, PIRP, PUNICODE_STRING, PsCreateSystemThread, UINT32};
use core::ffi::c_void;

// ---- file-scope preamble (translated) ----
// @c2rust file-scope data (compiler-exact tables/statics)
static mut VidpMode3Regs: VGA_REGISTERS = _VGA_REGISTERS {
    CRT: [
        0x5f as ::core::ffi::c_int as UCHAR,
        0x4f as ::core::ffi::c_int as UCHAR,
        0x50 as ::core::ffi::c_int as UCHAR,
        0x82 as ::core::ffi::c_int as UCHAR,
        0x55 as ::core::ffi::c_int as UCHAR,
        0x81 as ::core::ffi::c_int as UCHAR,
        0xbf as ::core::ffi::c_int as UCHAR,
        0x1f as ::core::ffi::c_int as UCHAR,
        0 as ::core::ffi::c_int as UCHAR,
        0x47 as ::core::ffi::c_int as UCHAR,
        0x1e as ::core::ffi::c_int as UCHAR,
        0 as ::core::ffi::c_int as UCHAR,
        0 as ::core::ffi::c_int as UCHAR,
        0 as ::core::ffi::c_int as UCHAR,
        0x5 as ::core::ffi::c_int as UCHAR,
        0xf0 as ::core::ffi::c_int as UCHAR,
        0x9c as ::core::ffi::c_int as UCHAR,
        0x8e as ::core::ffi::c_int as UCHAR,
        0x8f as ::core::ffi::c_int as UCHAR,
        0x28 as ::core::ffi::c_int as UCHAR,
        0x1f as ::core::ffi::c_int as UCHAR,
        0x96 as ::core::ffi::c_int as UCHAR,
        0xb9 as ::core::ffi::c_int as UCHAR,
        0xa3 as ::core::ffi::c_int as UCHAR,
    ],
    Attribute: [
        0 as ::core::ffi::c_int as UCHAR,
        0x1 as ::core::ffi::c_int as UCHAR,
        0x2 as ::core::ffi::c_int as UCHAR,
        0x3 as ::core::ffi::c_int as UCHAR,
        0x4 as ::core::ffi::c_int as UCHAR,
        0x5 as ::core::ffi::c_int as UCHAR,
        0x14 as ::core::ffi::c_int as UCHAR,
        0x7 as ::core::ffi::c_int as UCHAR,
        0x38 as ::core::ffi::c_int as UCHAR,
        0x39 as ::core::ffi::c_int as UCHAR,
        0x3a as ::core::ffi::c_int as UCHAR,
        0x3b as ::core::ffi::c_int as UCHAR,
        0x3c as ::core::ffi::c_int as UCHAR,
        0x3d as ::core::ffi::c_int as UCHAR,
        0x3e as ::core::ffi::c_int as UCHAR,
        0x3f as ::core::ffi::c_int as UCHAR,
        0xc as ::core::ffi::c_int as UCHAR,
        0 as ::core::ffi::c_int as UCHAR,
        0xf as ::core::ffi::c_int as UCHAR,
        0x8 as ::core::ffi::c_int as UCHAR,
        0 as ::core::ffi::c_int as UCHAR,
    ],
    Graphics: [
        0 as ::core::ffi::c_int as UCHAR,
        0 as ::core::ffi::c_int as UCHAR,
        0 as ::core::ffi::c_int as UCHAR,
        0 as ::core::ffi::c_int as UCHAR,
        0 as ::core::ffi::c_int as UCHAR,
        0x10 as ::core::ffi::c_int as UCHAR,
        0xe as ::core::ffi::c_int as UCHAR,
        0 as ::core::ffi::c_int as UCHAR,
        0xff as ::core::ffi::c_int as UCHAR,
    ],
    Sequencer: [
        0x3 as ::core::ffi::c_int as UCHAR,
        0 as ::core::ffi::c_int as UCHAR,
        0x3 as ::core::ffi::c_int as UCHAR,
        0 as ::core::ffi::c_int as UCHAR,
        0x2 as ::core::ffi::c_int as UCHAR,
    ],
    Misc: 0x67 as UCHAR,
};

static mut DefaultPalette: [UCHAR; 48] = [
    0 as ::core::ffi::c_int as UCHAR,
    0 as ::core::ffi::c_int as UCHAR,
    0 as ::core::ffi::c_int as UCHAR,
    0 as ::core::ffi::c_int as UCHAR,
    0 as ::core::ffi::c_int as UCHAR,
    0xc0 as ::core::ffi::c_int as UCHAR,
    0 as ::core::ffi::c_int as UCHAR,
    0xc0 as ::core::ffi::c_int as UCHAR,
    0 as ::core::ffi::c_int as UCHAR,
    0 as ::core::ffi::c_int as UCHAR,
    0xc0 as ::core::ffi::c_int as UCHAR,
    0xc0 as ::core::ffi::c_int as UCHAR,
    0xc0 as ::core::ffi::c_int as UCHAR,
    0 as ::core::ffi::c_int as UCHAR,
    0 as ::core::ffi::c_int as UCHAR,
    0xc0 as ::core::ffi::c_int as UCHAR,
    0 as ::core::ffi::c_int as UCHAR,
    0xc0 as ::core::ffi::c_int as UCHAR,
    0xc0 as ::core::ffi::c_int as UCHAR,
    0xc0 as ::core::ffi::c_int as UCHAR,
    0 as ::core::ffi::c_int as UCHAR,
    0xc0 as ::core::ffi::c_int as UCHAR,
    0xc0 as ::core::ffi::c_int as UCHAR,
    0xc0 as ::core::ffi::c_int as UCHAR,
    0x80 as ::core::ffi::c_int as UCHAR,
    0x80 as ::core::ffi::c_int as UCHAR,
    0x80 as ::core::ffi::c_int as UCHAR,
    0 as ::core::ffi::c_int as UCHAR,
    0 as ::core::ffi::c_int as UCHAR,
    0xff as ::core::ffi::c_int as UCHAR,
    0 as ::core::ffi::c_int as UCHAR,
    0xff as ::core::ffi::c_int as UCHAR,
    0 as ::core::ffi::c_int as UCHAR,
    0 as ::core::ffi::c_int as UCHAR,
    0xff as ::core::ffi::c_int as UCHAR,
    0xff as ::core::ffi::c_int as UCHAR,
    0xff as ::core::ffi::c_int as UCHAR,
    0 as ::core::ffi::c_int as UCHAR,
    0 as ::core::ffi::c_int as UCHAR,
    0xff as ::core::ffi::c_int as UCHAR,
    0 as ::core::ffi::c_int as UCHAR,
    0xff as ::core::ffi::c_int as UCHAR,
    0xff as ::core::ffi::c_int as UCHAR,
    0xff as ::core::ffi::c_int as UCHAR,
    0 as ::core::ffi::c_int as UCHAR,
    0xff as ::core::ffi::c_int as UCHAR,
    0xff as ::core::ffi::c_int as UCHAR,
    0xff as ::core::ffi::c_int as UCHAR,
];

static mut ResetDisplayParametersDeviceExtension: PDEVICE_EXTENSION =
    ::core::ptr::null::<_DEVICE_EXTENSION>() as *mut _DEVICE_EXTENSION;

static mut InbvThreadHandle: HANDLE = NULL;

static mut InbvMonitoring: BOOLEAN = FALSE as BOOLEAN;

pub type PDEVICE_EXTENSION = *mut DEVICE_EXTENSION;

pub type PVGA_REGISTERS = *mut VGA_REGISTERS;

unsafe extern "system" fn ScrResetDisplayParametersEx(
    Columns: ULONG,
    Rows: ULONG,
    CalledByInbv: BOOLEAN,
) -> BOOLEAN {
    let mut DeviceExtension: PDEVICE_EXTENSION;

    /* Bail out early if we don't have any resettable adapter */
    if ResetDisplayParametersDeviceExtension.is_null() {
        return FALSE; // No adapter found: request HAL to perform a full reset.
    }

    /*
     * If we have been unexpectedly called via a callback from
     * InbvAcquireDisplayOwnership(), start monitoring INBV.
     */
    if CalledByInbv != 0 {
        InbvMonitoring = TRUE;
    }

    DeviceExtension = ResetDisplayParametersDeviceExtension;
    ASSERT!(!DeviceExtension.is_null());

    /* Disable the screen but don't reset all screen settings (OK at high IRQL) */
    return ScrResetScreen(DeviceExtension, FALSE, FALSE);
}
unsafe extern "system" fn ScrResetDisplayParameters(mut Columns: ULONG, mut Rows: ULONG) -> BOOLEAN {
    return ScrResetDisplayParametersEx(Columns, Rows, TRUE as BOOLEAN);
}

unsafe extern "system" fn InbvMonitorThread(Context: PVOID) {
    let mut Delay: LARGE_INTEGER = core::mem::zeroed();
    let mut i: USHORT;

    KeSetPriorityThread((KeGetCurrentThread()).cast(), LOW_REALTIME_PRIORITY as KPRIORITY);

    loop {
        /*
         * During one second, check the INBV status each 100 milliseconds,
         * then revert to 1 second delay.
         */
        i = 10 as USHORT;
        Delay.QuadPart = -(100 as LONGLONG)
            * (1000 as LONGLONG)
            * (10 as LONGLONG); // 100 millisecond delay
        while InbvMonitoring == 0 {
            KeDelayExecutionThread(
                0i32 as KPROCESSOR_MODE,
                FALSE as BOOLEAN,
                &mut Delay,
            );

            if (i > (0 as USHORT)) && {
                i = i.wrapping_sub(1);
                i == (0 as USHORT)
            } {
                Delay.QuadPart = -(1 as LONGLONG)
                    * (1000 as LONGLONG)
                    * (1000 as LONGLONG)
                    * (10 as LONGLONG); // 1 second delay
            }
        }

        /*
         * Loop while the display is owned by INBV. We cannot do anything else
         * than polling since INBV does not offer a proper notification system.
         *
         * During one second, check the INBV status each 100 milliseconds,
         * then revert to 1 second delay.
         */
        i = 10 as USHORT;
        Delay.QuadPart = -(100 as LONGLONG)
            * (1000 as LONGLONG)
            * (10 as LONGLONG); // 100 millisecond delay
        while InbvCheckDisplayOwnership() != 0 {
            KeDelayExecutionThread(
                0i32 as KPROCESSOR_MODE,
                FALSE as BOOLEAN,
                &mut Delay,
            );

            if (i > (0 as USHORT)) && {
                i = i.wrapping_sub(1);
                i == (0 as USHORT)
            } {
                Delay.QuadPart = -(1 as LONGLONG)
                    * (1000 as LONGLONG)
                    * (1000 as LONGLONG)
                    * (10 as LONGLONG); // 1 second delay
            }
        }

        /* Reset the monitoring */
        InbvMonitoring = FALSE as BOOLEAN;

        /*
         * Somebody released INBV display ownership, usually by invoking
         * InbvNotifyDisplayOwnershipLost(). However the caller of this
         * function certainly specified a different callback than ours.
         * As we are going to be the only owner of the active display,
         * we need to re-register our own display reset callback.
         */
        InbvNotifyDisplayOwnershipLost(Some(
            ScrResetDisplayParameters,
        ));

        /* Re-enable the screen, keeping the original screen settings */
        if !ResetDisplayParametersDeviceExtension.is_null() {
            ScrResetScreen(
                ResetDisplayParametersDeviceExtension,
                FALSE as BOOLEAN,
                TRUE as BOOLEAN,
            );
        }
    }

    // FIXME: See ScrInbvCleanup().
    // PsTerminateSystemThread(STATUS_SUCCESS);
}

unsafe extern "C" fn ScrInbvInitialize() -> NTSTATUS {
    /* Create the INBV monitoring thread if needed */
    if InbvThreadHandle.is_null() {
        let mut Status: NTSTATUS = 0;
        let mut ObjectAttributes: _OBJECT_ATTRIBUTES = RTL_CONSTANT_OBJECT_ATTRIBUTES!(core::ptr::null_mut(), OBJ_KERNEL_HANDLE);

        Status = PsCreateSystemThread(
            &raw mut InbvThreadHandle,
            0,
            &raw mut ObjectAttributes,
            core::ptr::null_mut(),
            core::ptr::null_mut(),
            Some(InbvMonitorThread),
            core::ptr::null_mut(),
        );
        if !NT_SUCCESS!(Status) {
            InbvThreadHandle = core::ptr::null_mut() as HANDLE;
        }
    }

    /* Re-register the display reset callback with INBV */
    InbvNotifyDisplayOwnershipLost(Some(
        ScrResetDisplayParameters,
    ));

    return STATUS_SUCCESS;
}

unsafe extern "C" fn ScrInbvCleanup() -> NTSTATUS {
    // HANDLE ThreadHandle;

    // ResetDisplayParametersDeviceExtension = NULL;
    if !ResetDisplayParametersDeviceExtension.is_null() {
        InbvNotifyDisplayOwnershipLost(None);
        ScrResetDisplayParametersEx(80, 50, FALSE);
        // or InbvAcquireDisplayOwnership(); ?
    }

    return STATUS_SUCCESS;
}
#[cfg(target_arch = "x86")]
unsafe extern "fastcall" fn ScrSetRegisters(Registers: *const VGA_REGISTERS) {
    let mut i: UINT32;

    /* Update misc output register */
    WRITE_PORT_UCHAR(MISC, (*Registers).Misc);

    /* Synchronous reset on */
    WRITE_PORT_UCHAR(SEQ, 0x00);
    WRITE_PORT_UCHAR(SEQDATA, 0x01);

    /* Write sequencer registers */
    i = 1;
    while (i as usize) < core::mem::size_of::<[UCHAR; 5]>() {
        WRITE_PORT_UCHAR(SEQ, (i) as UCHAR);
        WRITE_PORT_UCHAR(SEQDATA, (*Registers).Sequencer[i as usize]);
        i = i.wrapping_add(1);
    }

    /* Synchronous reset off */
    WRITE_PORT_UCHAR(SEQ, 0x00);
    WRITE_PORT_UCHAR(SEQDATA, 0x03);

    /* Deprotect CRT registers 0-7 */
    WRITE_PORT_UCHAR(CRTC, 0x11);
    WRITE_PORT_UCHAR(
        CRTCDATA,
        (*Registers).CRT[0x11 as usize] & 0x7f,
    );

    /* Write CRT registers */
    i = 0;
    while (i as usize) < core::mem::size_of_val(&(*Registers).CRT) {
        WRITE_PORT_UCHAR(CRTC, (i) as UCHAR);
        WRITE_PORT_UCHAR(CRTCDATA, (*Registers).CRT[i as usize]);
        i = i.wrapping_add(1);
    }

    /* Write graphics controller registers */
    i = 0;
    while (i as usize) < core::mem::size_of::<[UCHAR; 9]>() {
        WRITE_PORT_UCHAR(GRAPHICS, (i) as UCHAR);
        WRITE_PORT_UCHAR(GRAPHICSDATA, (*Registers).Graphics[i as usize]);
        i = i.wrapping_add(1);
    }

    /* Write attribute controller registers */
    i = 0;
    while (i as usize) < core::mem::size_of::<[UCHAR; 21]>() {
        READ_PORT_UCHAR(STATUS);
        WRITE_PORT_UCHAR(ATTRIB, (i) as UCHAR);
        WRITE_PORT_UCHAR(ATTRIB, (*Registers).Attribute[i as usize]);
        i = i.wrapping_add(1);
    }

    /* Set the PEL mask */
    WRITE_PORT_UCHAR(PELMASK, 0xff);
}

#[cfg(not(target_arch = "x86"))]
unsafe extern "C" fn ScrSetRegisters(Registers: *const VGA_REGISTERS) {
    let mut i: UINT32;

    /* Update misc output register */
    WRITE_PORT_UCHAR(MISC, (*Registers).Misc);

    /* Synchronous reset on */
    WRITE_PORT_UCHAR(SEQ, 0x00);
    WRITE_PORT_UCHAR(SEQDATA, 0x01);

    /* Write sequencer registers */
    i = 1;
    while (i as usize) < core::mem::size_of::<[UCHAR; 5]>() {
        WRITE_PORT_UCHAR(SEQ, (i) as UCHAR);
        WRITE_PORT_UCHAR(SEQDATA, (*Registers).Sequencer[i as usize]);
        i = i.wrapping_add(1);
    }

    /* Synchronous reset off */
    WRITE_PORT_UCHAR(SEQ, 0x00);
    WRITE_PORT_UCHAR(SEQDATA, 0x03);

    /* Deprotect CRT registers 0-7 */
    WRITE_PORT_UCHAR(CRTC, 0x11);
    WRITE_PORT_UCHAR(
        CRTCDATA,
        (*Registers).CRT[0x11 as usize] & 0x7f,
    );

    /* Write CRT registers */
    i = 0;
    while (i as usize) < core::mem::size_of_val(&(*Registers).CRT) {
        WRITE_PORT_UCHAR(CRTC, (i) as UCHAR);
        WRITE_PORT_UCHAR(CRTCDATA, (*Registers).CRT[i as usize]);
        i = i.wrapping_add(1);
    }

    /* Write graphics controller registers */
    i = 0;
    while (i as usize) < core::mem::size_of::<[UCHAR; 9]>() {
        WRITE_PORT_UCHAR(GRAPHICS, (i) as UCHAR);
        WRITE_PORT_UCHAR(GRAPHICSDATA, (*Registers).Graphics[i as usize]);
        i = i.wrapping_add(1);
    }

    /* Write attribute controller registers */
    i = 0;
    while (i as usize) < core::mem::size_of::<[UCHAR; 21]>() {
        READ_PORT_UCHAR(STATUS);
        WRITE_PORT_UCHAR(ATTRIB, (i) as UCHAR);
        WRITE_PORT_UCHAR(ATTRIB, (*Registers).Attribute[i as usize]);
        i = i.wrapping_add(1);
    }

    /* Set the PEL mask */
    WRITE_PORT_UCHAR(PELMASK, 0xff);
}
#[cfg(target_arch = "x86")]
unsafe extern "fastcall" fn ScrSetCursor(DeviceExtension: PDEVICE_EXTENSION) {
    let mut Offset: ULONG = 0;
    if (*DeviceExtension).VideoMemory.is_null() {
        return;
    }
    Offset = ((*DeviceExtension).CursorY as ULONG)
        .wrapping_mul((*DeviceExtension).Columns as ULONG)
        .wrapping_add((*DeviceExtension).CursorX as ULONG);
    _disable();
    WRITE_PORT_UCHAR(CRTC_COMMAND, CRTC_CURSORPOSLO as UCHAR);
    WRITE_PORT_UCHAR(CRTC_DATA, Offset as UCHAR);
    WRITE_PORT_UCHAR(CRTC_COMMAND, CRTC_CURSORPOSHI as UCHAR);
    WRITE_PORT_UCHAR(CRTC_DATA, (Offset >> 8) as UCHAR);
    _enable();
}

#[cfg(not(target_arch = "x86"))]
unsafe extern "C" fn ScrSetCursor(DeviceExtension: PDEVICE_EXTENSION) {
    let mut Offset: ULONG = 0;
    if (*DeviceExtension).VideoMemory.is_null() {
        return;
    }
    Offset = ((*DeviceExtension).CursorY as ULONG)
        .wrapping_mul((*DeviceExtension).Columns as ULONG)
        .wrapping_add((*DeviceExtension).CursorX as ULONG);
    _disable();
    WRITE_PORT_UCHAR(CRTC_COMMAND, CRTC_CURSORPOSLO as UCHAR);
    WRITE_PORT_UCHAR(CRTC_DATA, Offset as UCHAR);
    WRITE_PORT_UCHAR(CRTC_COMMAND, CRTC_CURSORPOSHI as UCHAR);
    WRITE_PORT_UCHAR(CRTC_DATA, (Offset >> 8) as UCHAR);
    _enable();
}
#[cfg(target_arch = "x86")]
unsafe extern "fastcall" fn ScrSetCursorShape(mut DeviceExtension: PDEVICE_EXTENSION) {
    let mut size: ULONG = 0;
    let mut height: ULONG = 0;
    let mut data: UCHAR = 0;
    let mut value: UCHAR = 0;
    if (*DeviceExtension).VideoMemory.is_null() {
        return;
    }
    height = (*DeviceExtension).ScanLines as ULONG;
    data = (if (*DeviceExtension).CursorVisible != 0 {
        0x00 as ::core::ffi::c_int
    } else {
        0x20 as ::core::ffi::c_int
    }) as UCHAR;
    size = (*DeviceExtension)
        .CursorSize
        .wrapping_mul(height)
        .wrapping_div(100 as ULONG);
    if size < 1 as ULONG {
        size = 1 as ULONG;
    }
    data = (data as ::core::ffi::c_int
        | height.wrapping_sub(size) as UCHAR as ::core::ffi::c_int) as UCHAR;
    _disable();
    WRITE_PORT_UCHAR(
        CRTC_COMMAND,
        CRTC_CURSORSTART as UCHAR,
    );
    WRITE_PORT_UCHAR(CRTC_DATA, data);
    WRITE_PORT_UCHAR(
        CRTC_COMMAND,
        CRTC_CURSOREND as UCHAR,
    );
    value = (READ_PORT_UCHAR(CRTC_DATA) as ::core::ffi::c_int
        & 0xE0 as ::core::ffi::c_int) as UCHAR;
    WRITE_PORT_UCHAR(
        CRTC_DATA,
        (value as ULONG | height.wrapping_sub(1 as ULONG)) as UCHAR,
    );
    _enable();
}

#[cfg(not(target_arch = "x86"))]
unsafe extern "C" fn ScrSetCursorShape(mut DeviceExtension: PDEVICE_EXTENSION) {
    let mut size: ULONG = 0;
    let mut height: ULONG = 0;
    let mut data: UCHAR = 0;
    let mut value: UCHAR = 0;
    if (*DeviceExtension).VideoMemory.is_null() {
        return;
    }
    height = (*DeviceExtension).ScanLines as ULONG;
    data = (if (*DeviceExtension).CursorVisible != 0 {
        0x00 as ::core::ffi::c_int
    } else {
        0x20 as ::core::ffi::c_int
    }) as UCHAR;
    size = (*DeviceExtension)
        .CursorSize
        .wrapping_mul(height)
        .wrapping_div(100 as ULONG);
    if size < 1 as ULONG {
        size = 1 as ULONG;
    }
    data = (data as ::core::ffi::c_int
        | height.wrapping_sub(size) as UCHAR as ::core::ffi::c_int) as UCHAR;
    _disable();
    WRITE_PORT_UCHAR(
        CRTC_COMMAND,
        CRTC_CURSORSTART as UCHAR,
    );
    WRITE_PORT_UCHAR(CRTC_DATA, data);
    WRITE_PORT_UCHAR(
        CRTC_COMMAND,
        CRTC_CURSOREND as UCHAR,
    );
    value = (READ_PORT_UCHAR(CRTC_DATA) as ::core::ffi::c_int
        & 0xE0 as ::core::ffi::c_int) as UCHAR;
    WRITE_PORT_UCHAR(
        CRTC_DATA,
        (value as ULONG | height.wrapping_sub(1 as ULONG)) as UCHAR,
    );
    _enable();
}
#[cfg(target_arch = "x86")]
unsafe extern "fastcall" fn ScrAcquireOwnership(DeviceExtension: PDEVICE_EXTENSION) {
    let mut data: UCHAR = 0;
    let mut value: UCHAR = 0;
    let mut offset: ULONG = 0;
    let mut Index: ULONG = 0;

    _disable();

    ScrSetRegisters(core::ptr::addr_of!(VidpMode3Regs));

    /* Disable screen and enable palette access */
    READ_PORT_UCHAR(STATUS);
    WRITE_PORT_UCHAR(ATTRIB, 0x00);

    Index = 0;
    while (Index as usize) < (core::mem::size_of::<[UCHAR; 48]>() / (3 as usize)) {
        WRITE_PORT_UCHAR(PELINDEX, Index as UCHAR);
        WRITE_PORT_UCHAR(
            PELDATA,
            DefaultPalette[(Index * 3) as usize] >> 2,
        );
        WRITE_PORT_UCHAR(
            PELDATA,
            DefaultPalette[(Index * 3 + 1) as usize]
                >> 2,
        );
        WRITE_PORT_UCHAR(
            PELDATA,
            DefaultPalette[(Index * 3 + 2) as usize]
                >> 2,
        );
        Index += 1;
    }

    /* Enable screen and disable palette access */
    READ_PORT_UCHAR(STATUS);
    WRITE_PORT_UCHAR(ATTRIB, 0x20);

    /* Switch blinking characters off */
    READ_PORT_UCHAR(ATTRC_INPST1);
    value = READ_PORT_UCHAR(ATTRC_WRITEREG);
    WRITE_PORT_UCHAR(ATTRC_WRITEREG, 0x10);
    data = READ_PORT_UCHAR(ATTRC_READREG);
    data = data & !(0x08);
    WRITE_PORT_UCHAR(ATTRC_WRITEREG, data);
    WRITE_PORT_UCHAR(ATTRC_WRITEREG, value);
    READ_PORT_UCHAR(ATTRC_INPST1);

    /* Read screen information from CRT controller */
    WRITE_PORT_UCHAR(CRTC_COMMAND, (CRTC_COLUMNS) as UCHAR);
    (*DeviceExtension).Columns = ((READ_PORT_UCHAR(CRTC_DATA) as ULONG)
        + 1) as USHORT;
    WRITE_PORT_UCHAR(CRTC_COMMAND, (CRTC_ROWS) as UCHAR);
    (*DeviceExtension).Rows = READ_PORT_UCHAR(CRTC_DATA) as USHORT;
    WRITE_PORT_UCHAR(CRTC_COMMAND, (CRTC_OVERFLOW) as UCHAR);
    data = READ_PORT_UCHAR(CRTC_DATA);
    (*DeviceExtension).Rows = (((*DeviceExtension).Rows as ULONG)
        | ((((data as ULONG) & 0x02) << 7)
            | (((data as ULONG) & 0x40) << 3))) as USHORT;
    (*DeviceExtension).Rows = (*DeviceExtension).Rows.wrapping_add(1);
    WRITE_PORT_UCHAR(CRTC_COMMAND, (CRTC_SCANLINES) as UCHAR);
    (*DeviceExtension).ScanLines = ((((READ_PORT_UCHAR(CRTC_DATA) as ULONG)
        & 0x1F)
        + 1) as UCHAR);

    /* Retrieve the current output cursor position */
    WRITE_PORT_UCHAR(CRTC_COMMAND, (CRTC_CURSORPOSLO) as UCHAR);
    offset = READ_PORT_UCHAR(CRTC_DATA) as ULONG;
    WRITE_PORT_UCHAR(CRTC_COMMAND, (CRTC_CURSORPOSHI) as UCHAR);
    offset = offset.wrapping_add((READ_PORT_UCHAR(CRTC_DATA) as ULONG)
        << 8);

    /* Show blinking cursor */
    // FIXME: cursor block? Call ScrSetCursorShape() instead?
    WRITE_PORT_UCHAR(CRTC_COMMAND, (CRTC_CURSORSTART) as UCHAR);
    WRITE_PORT_UCHAR(
        CRTC_DATA,
        ((((*DeviceExtension).ScanLines as i32) - 1) & 0x1F)
            as UCHAR,
    );
    WRITE_PORT_UCHAR(CRTC_COMMAND, (CRTC_CURSOREND) as UCHAR);
    data = READ_PORT_UCHAR(CRTC_DATA) & 0xE0;
    WRITE_PORT_UCHAR(
        CRTC_DATA,
        ((data as i32)
            | ((((*DeviceExtension).ScanLines as i32) - 1)
                & 0x1F)) as UCHAR,
    );

    _enable();

    /* Calculate number of text rows */
    (*DeviceExtension).Rows = (((*DeviceExtension).Rows as i32)
        / ((*DeviceExtension).ScanLines as i32)) as USHORT;

    /* Set the cursor position, clipping it to the screen */
    (*DeviceExtension).CursorX = (offset % ((*DeviceExtension).Columns as ULONG)) as USHORT;
    (*DeviceExtension).CursorY = (offset / ((*DeviceExtension).Columns as ULONG)) as USHORT;
    // DeviceExtension->CursorX = min(max(DeviceExtension->CursorX, 0), DeviceExtension->Columns - 1);
    (*DeviceExtension).CursorY = min(
        max((*DeviceExtension).CursorY as i32, 0),
        ((*DeviceExtension).Rows as i32) - 1,
    ) as USHORT;

    /* Set the font */
    if !(*DeviceExtension).FontBitfield.is_null() {
        ScrSetFont((*DeviceExtension).FontBitfield);
    }

    DPRINT!(
        "%d Columns  %d Rows %d Scanlines\n",
        (*DeviceExtension).Columns,
        (*DeviceExtension).Rows,
        (*DeviceExtension).ScanLines
    );
}

#[cfg(not(target_arch = "x86"))]
unsafe extern "C" fn ScrAcquireOwnership(DeviceExtension: PDEVICE_EXTENSION) {
    let mut data: UCHAR = 0;
    let mut value: UCHAR = 0;
    let mut offset: ULONG = 0;
    let mut Index: ULONG = 0;

    _disable();

    ScrSetRegisters(core::ptr::addr_of!(VidpMode3Regs));

    /* Disable screen and enable palette access */
    READ_PORT_UCHAR(STATUS);
    WRITE_PORT_UCHAR(ATTRIB, 0x00);

    Index = 0;
    while (Index as usize) < (core::mem::size_of::<[UCHAR; 48]>() / (3 as usize)) {
        WRITE_PORT_UCHAR(PELINDEX, Index as UCHAR);
        WRITE_PORT_UCHAR(
            PELDATA,
            DefaultPalette[(Index * 3) as usize] >> 2,
        );
        WRITE_PORT_UCHAR(
            PELDATA,
            DefaultPalette[(Index * 3 + 1) as usize]
                >> 2,
        );
        WRITE_PORT_UCHAR(
            PELDATA,
            DefaultPalette[(Index * 3 + 2) as usize]
                >> 2,
        );
        Index += 1;
    }

    /* Enable screen and disable palette access */
    READ_PORT_UCHAR(STATUS);
    WRITE_PORT_UCHAR(ATTRIB, 0x20);

    /* Switch blinking characters off */
    READ_PORT_UCHAR(ATTRC_INPST1);
    value = READ_PORT_UCHAR(ATTRC_WRITEREG);
    WRITE_PORT_UCHAR(ATTRC_WRITEREG, 0x10);
    data = READ_PORT_UCHAR(ATTRC_READREG);
    data = data & !(0x08);
    WRITE_PORT_UCHAR(ATTRC_WRITEREG, data);
    WRITE_PORT_UCHAR(ATTRC_WRITEREG, value);
    READ_PORT_UCHAR(ATTRC_INPST1);

    /* Read screen information from CRT controller */
    WRITE_PORT_UCHAR(CRTC_COMMAND, (CRTC_COLUMNS) as UCHAR);
    (*DeviceExtension).Columns = ((READ_PORT_UCHAR(CRTC_DATA) as ULONG)
        + 1) as USHORT;
    WRITE_PORT_UCHAR(CRTC_COMMAND, (CRTC_ROWS) as UCHAR);
    (*DeviceExtension).Rows = READ_PORT_UCHAR(CRTC_DATA) as USHORT;
    WRITE_PORT_UCHAR(CRTC_COMMAND, (CRTC_OVERFLOW) as UCHAR);
    data = READ_PORT_UCHAR(CRTC_DATA);
    (*DeviceExtension).Rows = (((*DeviceExtension).Rows as ULONG)
        | ((((data as ULONG) & 0x02) << 7)
            | (((data as ULONG) & 0x40) << 3))) as USHORT;
    (*DeviceExtension).Rows = (*DeviceExtension).Rows.wrapping_add(1);
    WRITE_PORT_UCHAR(CRTC_COMMAND, (CRTC_SCANLINES) as UCHAR);
    (*DeviceExtension).ScanLines = ((((READ_PORT_UCHAR(CRTC_DATA) as ULONG)
        & 0x1F)
        + 1) as UCHAR);

    /* Retrieve the current output cursor position */
    WRITE_PORT_UCHAR(CRTC_COMMAND, (CRTC_CURSORPOSLO) as UCHAR);
    offset = READ_PORT_UCHAR(CRTC_DATA) as ULONG;
    WRITE_PORT_UCHAR(CRTC_COMMAND, (CRTC_CURSORPOSHI) as UCHAR);
    offset = offset.wrapping_add((READ_PORT_UCHAR(CRTC_DATA) as ULONG)
        << 8);

    /* Show blinking cursor */
    // FIXME: cursor block? Call ScrSetCursorShape() instead?
    WRITE_PORT_UCHAR(CRTC_COMMAND, (CRTC_CURSORSTART) as UCHAR);
    WRITE_PORT_UCHAR(
        CRTC_DATA,
        ((((*DeviceExtension).ScanLines as i32) - 1) & 0x1F)
            as UCHAR,
    );
    WRITE_PORT_UCHAR(CRTC_COMMAND, (CRTC_CURSOREND) as UCHAR);
    data = READ_PORT_UCHAR(CRTC_DATA) & 0xE0;
    WRITE_PORT_UCHAR(
        CRTC_DATA,
        ((data as i32)
            | ((((*DeviceExtension).ScanLines as i32) - 1)
                & 0x1F)) as UCHAR,
    );

    _enable();

    /* Calculate number of text rows */
    (*DeviceExtension).Rows = (((*DeviceExtension).Rows as i32)
        / ((*DeviceExtension).ScanLines as i32)) as USHORT;

    /* Set the cursor position, clipping it to the screen */
    (*DeviceExtension).CursorX = (offset % ((*DeviceExtension).Columns as ULONG)) as USHORT;
    (*DeviceExtension).CursorY = (offset / ((*DeviceExtension).Columns as ULONG)) as USHORT;
    // DeviceExtension->CursorX = min(max(DeviceExtension->CursorX, 0), DeviceExtension->Columns - 1);
    (*DeviceExtension).CursorY = min(
        max((*DeviceExtension).CursorY as i32, 0),
        ((*DeviceExtension).Rows as i32) - 1,
    ) as USHORT;

    /* Set the font */
    if !(*DeviceExtension).FontBitfield.is_null() {
        ScrSetFont((*DeviceExtension).FontBitfield);
    }

    DPRINT!(
        "%d Columns  %d Rows %d Scanlines\n",
        (*DeviceExtension).Columns,
        (*DeviceExtension).Rows,
        (*DeviceExtension).ScanLines
    );
}

unsafe extern "C" fn ScrResetScreen(
    DeviceExtension: PDEVICE_EXTENSION,
    FullReset: BOOLEAN,
    Enable: BOOLEAN,
) -> BOOLEAN {
    let mut BaseAddress: PHYSICAL_ADDRESS = core::mem::zeroed();

    /* Allow resets to the same state only for full resets */
    if FullReset == 0 && Enable == (*DeviceExtension).Enabled {
        return FALSE; // STATUS_INVALID_PARAMETER; STATUS_INVALID_DEVICE_REQUEST;
    }

    if FullReset != 0 {
        (*DeviceExtension).CursorSize = 5; /* FIXME: value correct?? */
        (*DeviceExtension).CursorVisible = (TRUE as i32);

        if !(*DeviceExtension).FontBitfield.is_null() {
            ExFreePoolWithTag((*DeviceExtension).FontBitfield as PVOID, TAG_BLUE);
            (*DeviceExtension).FontBitfield = core::ptr::null_mut();
        }

        /* More initialization */
        (*DeviceExtension).CharAttribute = (BACKGROUND_BLUE | FOREGROUND_LIGHTGRAY) as USHORT;
        (*DeviceExtension).Mode = (ENABLE_PROCESSED_OUTPUT |
                                   ENABLE_WRAP_AT_EOL_OUTPUT) as ULONG;
    }

    if Enable != 0 {
        ScrAcquireOwnership(DeviceExtension);

        if FullReset != 0 {
            /*
             * Fully reset the screen and all its settings.
             */

            /* Unmap any previously mapped video memory */
            if !(*DeviceExtension).VideoMemory.is_null() {
                ASSERT!((*DeviceExtension).VideoMemorySize != 0);
                MmUnmapIoSpace((*DeviceExtension).VideoMemory as PVOID, (*DeviceExtension).VideoMemorySize);
            }
            (*DeviceExtension).VideoMemory = core::ptr::null_mut();
            (*DeviceExtension).VideoMemorySize = 0;

            /* Free any previously allocated backup screenbuffer */
            if !(*DeviceExtension).ScreenBuffer.is_null() {
                ASSERT!((*DeviceExtension).ScreenBufferSize != 0);
                ExFreePoolWithTag((*DeviceExtension).ScreenBuffer as PVOID, TAG_BLUE);
            }
            (*DeviceExtension).ScreenBuffer = core::ptr::null_mut();
            (*DeviceExtension).ScreenBufferSize = 0;

            /* Get a pointer to the video memory */
            (*DeviceExtension).VideoMemorySize = ((*DeviceExtension).Rows as SIZE_T
                * (*DeviceExtension).Columns as SIZE_T
                * 2) as SIZE_T;
            if (*DeviceExtension).VideoMemorySize == 0 {
                return FALSE; // STATUS_INVALID_VIEW_SIZE; STATUS_MAPPED_FILE_SIZE_ZERO;
            }

            /* Map the video memory */
            BaseAddress.QuadPart = (VIDMEM_BASE as _);
            (*DeviceExtension).VideoMemory =
                MmMapIoSpace(BaseAddress, (*DeviceExtension).VideoMemorySize, MmNonCached) as PUCHAR;
            if (*DeviceExtension).VideoMemory.is_null() {
                (*DeviceExtension).VideoMemorySize = 0;
                return FALSE; // STATUS_NONE_MAPPED; STATUS_NOT_MAPPED_VIEW; STATUS_CONFLICTING_ADDRESSES;
            }

            /* Initialize the backup screenbuffer in non-paged pool (must be accessible at high IRQL) */
            (*DeviceExtension).ScreenBufferSize = (*DeviceExtension).VideoMemorySize;
            (*DeviceExtension).ScreenBuffer =
                ExAllocatePoolWithTag(NonPagedPool, (*DeviceExtension).ScreenBufferSize, TAG_BLUE) as PUCHAR;
            if (*DeviceExtension).ScreenBuffer.is_null() {
                DPRINT1!("Could not allocate screenbuffer, ignore...\n");
                (*DeviceExtension).ScreenBufferSize = 0;
            }

            /* (Re-)initialize INBV */
            ScrInbvInitialize();
        } else {
            /*
             * Restore the previously disabled screen.
             */

            /* Restore the snapshot of the video memory from the backup screenbuffer */
            if !(*DeviceExtension).ScreenBuffer.is_null() {
                ASSERT!(!(*DeviceExtension).VideoMemory.is_null());
                ASSERT!(!(*DeviceExtension).ScreenBuffer.is_null());
                ASSERT!((*DeviceExtension).ScreenBufferSize != 0);
                ASSERT!((*DeviceExtension).VideoMemorySize == (*DeviceExtension).ScreenBufferSize);

                RtlCopyMemory!(
                    (*DeviceExtension).VideoMemory as PVOID,
                    (*DeviceExtension).ScreenBuffer as PVOID,
                    (*DeviceExtension).VideoMemorySize,
                );
            }

            /* Restore the cursor state */
            ScrSetCursor(DeviceExtension);
            ScrSetCursorShape(DeviceExtension);
        }
        (*DeviceExtension).Enabled = TRUE;
    } else {
        (*DeviceExtension).Enabled = FALSE;
        if FullReset != 0 {
            /*
             * Fully disable the screen and reset all its settings.
             */

            /* Clean INBV up */
            ScrInbvCleanup();

            /* Unmap any previously mapped video memory */
            if !(*DeviceExtension).VideoMemory.is_null() {
                ASSERT!((*DeviceExtension).VideoMemorySize != 0);
                MmUnmapIoSpace((*DeviceExtension).VideoMemory as PVOID, (*DeviceExtension).VideoMemorySize);
            }
            (*DeviceExtension).VideoMemory = core::ptr::null_mut();
            (*DeviceExtension).VideoMemorySize = 0;

            /* Free any previously allocated backup screenbuffer */
            if !(*DeviceExtension).ScreenBuffer.is_null() {
                ASSERT!((*DeviceExtension).ScreenBufferSize != 0);
                ExFreePoolWithTag((*DeviceExtension).ScreenBuffer as PVOID, TAG_BLUE);
            }
            (*DeviceExtension).ScreenBuffer = core::ptr::null_mut();
            (*DeviceExtension).ScreenBufferSize = 0;

            /* Store dummy values */
            (*DeviceExtension).Columns = 1;
            (*DeviceExtension).Rows = 1;
            (*DeviceExtension).ScanLines = 1;
        } else {
            /*
             * Partially disable the screen such that it can be restored later.
             */

            /* Take a snapshot of the video memory into the backup screenbuffer */
            if !(*DeviceExtension).ScreenBuffer.is_null() {
                ASSERT!(!(*DeviceExtension).VideoMemory.is_null());
                ASSERT!(!(*DeviceExtension).ScreenBuffer.is_null());
                ASSERT!((*DeviceExtension).ScreenBufferSize != 0);
                ASSERT!((*DeviceExtension).VideoMemorySize == (*DeviceExtension).ScreenBufferSize);

                RtlCopyMemory!(
                    (*DeviceExtension).ScreenBuffer as PVOID,
                    (*DeviceExtension).VideoMemory as PVOID,
                    (*DeviceExtension).VideoMemorySize,
                );
            }
        }
    }

    return TRUE; // STATUS_SUCCESS;
}
unsafe extern "system" fn ScrCreateClose(DeviceObject: PDEVICE_OBJECT, Irp: PIRP) -> NTSTATUS {
    let stk: PIO_STACK_LOCATION = IoGetCurrentIrpStackLocation(Irp);

    UNREFERENCED_PARAMETER!(DeviceObject);

    if (*stk).MajorFunction == (IRP_MJ_CREATE as u8) {
        (*Irp).IoStatus.Information = (FILE_OPENED as _);
    }
    // else: IRP_MJ_CLOSE

    (*Irp).IoStatus.__bindgen_anon_1.Status = STATUS_SUCCESS;
    IoCompleteRequest(Irp, (IO_NO_INCREMENT as i8));
    return STATUS_SUCCESS;
}

unsafe extern "system" fn ScrWrite(DeviceObject: PDEVICE_OBJECT, Irp: PIRP) -> NTSTATUS {
    let mut Status: NTSTATUS = 0;
    let stk: PIO_STACK_LOCATION = IoGetCurrentIrpStackLocation(Irp);
    let DeviceExtension: PDEVICE_EXTENSION = (*DeviceObject).DeviceExtension as PDEVICE_EXTENSION;
    let mut pch: PCHAR = (*Irp).UserBuffer as PCHAR;
    let mut vidmem: PUCHAR = core::ptr::null_mut();
    let mut i: ULONG = 0;
    let mut j: ULONG = 0;
    let mut offset: ULONG = 0;
    let mut cursorx: USHORT = 0;
    let mut cursory: USHORT = 0;
    let mut rows: USHORT = 0;
    let mut columns: USHORT = 0;
    let processed: BOOLEAN = (((*DeviceExtension).Mode & ENABLE_PROCESSED_OUTPUT) != 0) as BOOLEAN;

    if (*DeviceExtension).Enabled == 0 || (*DeviceExtension).VideoMemory.is_null() {
        /* Display is not enabled, we're not allowed to touch it */
        Status = STATUS_SUCCESS;

        (*Irp).IoStatus.__bindgen_anon_1.Status = Status;
        IoCompleteRequest(Irp, (IO_NO_INCREMENT as i8));

        return Status;
    }

    vidmem = (*DeviceExtension).VideoMemory;
    rows = (*DeviceExtension).Rows;
    columns = (*DeviceExtension).Columns;
    cursorx = (*DeviceExtension).CursorX;
    cursory = (*DeviceExtension).CursorY;

    if processed == 0 {
        /* Raw output mode */

        /* Calculate the offset from the cursor position */
        offset = (cursorx as ULONG + cursory as ULONG * columns as ULONG) as ULONG;

        // FIXME: Does the buffer only contains chars? or chars + attributes?
        // FIXME2: Fix buffer overflow.
        RtlCopyMemory!(
            vidmem.add((offset * 2) as usize) as PVOID,
            pch as PVOID,
            (*stk).Parameters.Write.Length as SIZE_T,
        );
        offset += ((*stk).Parameters.Write.Length / 2) as ULONG;

        /* Set the cursor position, clipping it to the screen */
        cursorx = (offset % columns as ULONG) as USHORT;
        cursory = (offset / columns as ULONG) as USHORT;
        // cursorx = min(max(cursorx, 0), columns - 1);
        cursory = min(max(cursory, 0), rows - 1);
    } else {
        /* Cooked output mode */
        i = 0;
        while i < (*stk).Parameters.Write.Length {
            match *pch as u8 {
                b'\x08' => {
                    if cursorx > 0 {
                        cursorx -= 1;
                    } else if cursory > 0 {
                        cursory -= 1;
                        cursorx = columns - 1;
                    }
                    offset = (cursorx as ULONG + cursory as ULONG * columns as ULONG) as ULONG;
                    *vidmem.add((offset * 2) as usize) = b' ';
                    *vidmem.add((offset * 2 + 1) as usize) =
                        (*DeviceExtension).CharAttribute as u8;
                }

                b'\n' => {
                    cursory += 1;
                    /* Fall back */
                    cursorx = 0;
                }
                b'\r' => {
                    cursorx = 0;
                }

                b'\t' => {
                    offset = (TAB_WIDTH - (cursorx % (TAB_WIDTH as u16)) as ULONG) as ULONG;
                    while {
                        let fresh = offset;
                        offset = offset.wrapping_sub(1);
                        fresh != 0
                    } {
                        *vidmem.add(((cursorx as ULONG + cursory as ULONG * columns as ULONG)
                            * 2) as usize) = b' ';
                        cursorx += 1;
                        if cursorx >= columns {
                            cursorx = 0;
                            cursory += 1;
                            /* We jumped to the next line, stop there */
                            break;
                        }
                    }
                }

                _ => {
                    offset = (cursorx as ULONG + cursory as ULONG * columns as ULONG) as ULONG;
                    *vidmem.add((offset * 2) as usize) = *pch as u8;
                    *vidmem.add((offset * 2 + 1) as usize) =
                        (*DeviceExtension).CharAttribute as u8;
                    cursorx += 1;
                    if cursorx >= columns {
                        cursorx = 0;
                        cursory += 1;
                    }
                }
            }

            /* Scroll up the contents of the screen if we are at the end */
            if cursory >= rows {
                let mut LinePtr: PUSHORT = core::ptr::null_mut();

                RtlCopyMemory!(
                    vidmem as PVOID,
                    vidmem.add((columns * 2) as usize) as PVOID,
                    (columns * (rows - 1) * 2) as SIZE_T,
                );

                LinePtr = vidmem.add((columns * (rows - 1) * 2) as usize) as PUSHORT;

                j = 0;
                while j < columns as ULONG {
                    *LinePtr.add(j as usize) = ((*DeviceExtension).CharAttribute << 8) as USHORT;
                    j += 1;
                }
                cursory = rows - 1;
                j = 0;
                while j < columns as ULONG {
                    offset = j + (cursory as ULONG * columns as ULONG);
                    *vidmem.add((offset * 2) as usize) = b' ';
                    *vidmem.add((offset * 2 + 1) as usize) =
                        (*DeviceExtension).CharAttribute as u8;
                    j += 1;
                }
            }
            i += 1;
            pch = pch.add(1);
        }
    }

    /* Set the cursor position */
    ASSERT!((0 <= cursorx) && (cursorx < (*DeviceExtension).Columns));
    ASSERT!((0 <= cursory) && (cursory < (*DeviceExtension).Rows));
    (*DeviceExtension).CursorX = cursorx;
    (*DeviceExtension).CursorY = cursory;
    ScrSetCursor(DeviceExtension);

    Status = STATUS_SUCCESS;

    (*Irp).IoStatus.__bindgen_anon_1.Status = Status;
    IoCompleteRequest(Irp, (IO_VIDEO_INCREMENT as i8));

    return Status;
}
unsafe extern "system" fn ScrIoControl(DeviceObject: PDEVICE_OBJECT, Irp: PIRP) -> NTSTATUS {
    let mut Status: NTSTATUS = 0;
    let stk: PIO_STACK_LOCATION = IoGetCurrentIrpStackLocation(Irp);
    let DeviceExtension: PDEVICE_EXTENSION = (*DeviceObject).DeviceExtension as PDEVICE_EXTENSION;

    'switch: {
        match (*stk).Parameters.DeviceIoControl.IoControlCode {
            IOCTL_CONSOLE_RESET_SCREEN => {
                let mut Enable: BOOLEAN = 0;

                /* Validate input buffer */
                if ((*stk).Parameters.DeviceIoControl.InputBufferLength as usize)
                    < core::mem::size_of::<ULONG>()
                {
                    Status = STATUS_INVALID_PARAMETER;
                    break 'switch;
                }
                ASSERT!((*Irp).AssociatedIrp.SystemBuffer);

                Enable = (*((*Irp).AssociatedIrp.SystemBuffer as PULONG) != 0) as BOOLEAN;

                /* Fully enable or disable the screen */
                Status = if ScrResetScreen(DeviceExtension, TRUE, Enable) != 0 {
                    STATUS_SUCCESS
                } else {
                    STATUS_UNSUCCESSFUL
                };
                (*Irp).IoStatus.Information = 0;
                break 'switch;
            }

            IOCTL_CONSOLE_GET_SCREEN_BUFFER_INFO => {
                let mut pcsbi: PCONSOLE_SCREEN_BUFFER_INFO = core::ptr::null_mut();
                let mut rows: USHORT = (*DeviceExtension).Rows;
                let mut columns: USHORT = (*DeviceExtension).Columns;

                /* Validate output buffer */
                if ((*stk).Parameters.DeviceIoControl.OutputBufferLength as usize)
                    < core::mem::size_of::<CONSOLE_SCREEN_BUFFER_INFO>()
                {
                    Status = STATUS_INVALID_PARAMETER;
                    break 'switch;
                }
                ASSERT!((*Irp).AssociatedIrp.SystemBuffer);

                pcsbi = (*Irp).AssociatedIrp.SystemBuffer as PCONSOLE_SCREEN_BUFFER_INFO;
                RtlZeroMemory!(
                    pcsbi as PVOID,
                    core::mem::size_of::<CONSOLE_SCREEN_BUFFER_INFO>() as SIZE_T,
                );

                (*pcsbi).dwSize.X = columns as SHORT;
                (*pcsbi).dwSize.Y = rows as SHORT;

                (*pcsbi).dwCursorPosition.X = (*DeviceExtension).CursorX as SHORT;
                (*pcsbi).dwCursorPosition.Y = (*DeviceExtension).CursorY as SHORT;

                (*pcsbi).wAttributes = (*DeviceExtension).CharAttribute;

                (*pcsbi).srWindow.Left = 0;
                (*pcsbi).srWindow.Right = (columns as i32 - 1) as SHORT;
                (*pcsbi).srWindow.Top = 0;
                (*pcsbi).srWindow.Bottom = (rows as i32 - 1) as SHORT;

                (*pcsbi).dwMaximumWindowSize.X = columns as SHORT;
                (*pcsbi).dwMaximumWindowSize.Y = rows as SHORT;

                (*Irp).IoStatus.Information =
                    core::mem::size_of::<CONSOLE_SCREEN_BUFFER_INFO>() as ULONG_PTR;
                Status = STATUS_SUCCESS;
                break 'switch;
            }

            IOCTL_CONSOLE_SET_SCREEN_BUFFER_INFO => {
                let mut pcsbi: PCONSOLE_SCREEN_BUFFER_INFO = core::ptr::null_mut();

                /* Validate input buffer */
                if ((*stk).Parameters.DeviceIoControl.InputBufferLength as usize)
                    < core::mem::size_of::<CONSOLE_SCREEN_BUFFER_INFO>()
                {
                    Status = STATUS_INVALID_PARAMETER;
                    break 'switch;
                }
                ASSERT!((*Irp).AssociatedIrp.SystemBuffer);

                pcsbi = (*Irp).AssociatedIrp.SystemBuffer as PCONSOLE_SCREEN_BUFFER_INFO;

                if ((*pcsbi).dwCursorPosition.X as i32) < 0
                    || ((*pcsbi).dwCursorPosition.X as i32) >= ((*DeviceExtension).Columns as i32)
                    || ((*pcsbi).dwCursorPosition.Y as i32) < 0
                    || ((*pcsbi).dwCursorPosition.Y as i32) >= ((*DeviceExtension).Rows as i32)
                {
                    (*Irp).IoStatus.Information = 0;
                    Status = STATUS_INVALID_PARAMETER;
                    break 'switch;
                }

                (*DeviceExtension).CharAttribute = (*pcsbi).wAttributes;

                /* Set the cursor position */
                ASSERT!(
                    (0 <= ((*pcsbi).dwCursorPosition.X as i32))
                        && (((*pcsbi).dwCursorPosition.X as i32) < ((*DeviceExtension).Columns as i32))
                );
                ASSERT!(
                    (0 <= ((*pcsbi).dwCursorPosition.Y as i32))
                        && (((*pcsbi).dwCursorPosition.Y as i32) < ((*DeviceExtension).Rows as i32))
                );
                (*DeviceExtension).CursorX = (*pcsbi).dwCursorPosition.X as USHORT;
                (*DeviceExtension).CursorY = (*pcsbi).dwCursorPosition.Y as USHORT;
                if (*DeviceExtension).Enabled != 0 {
                    ScrSetCursor(DeviceExtension);
                }

                (*Irp).IoStatus.Information = 0;
                Status = STATUS_SUCCESS;
                break 'switch;
            }

            IOCTL_CONSOLE_GET_CURSOR_INFO => {
                let mut pcci: PCONSOLE_CURSOR_INFO = core::ptr::null_mut();

                /* Validate output buffer */
                if ((*stk).Parameters.DeviceIoControl.OutputBufferLength as usize)
                    < core::mem::size_of::<CONSOLE_CURSOR_INFO>()
                {
                    Status = STATUS_INVALID_PARAMETER;
                    break 'switch;
                }
                ASSERT!((*Irp).AssociatedIrp.SystemBuffer);

                pcci = (*Irp).AssociatedIrp.SystemBuffer as PCONSOLE_CURSOR_INFO;
                RtlZeroMemory!(
                    pcci as PVOID,
                    core::mem::size_of::<CONSOLE_CURSOR_INFO>() as SIZE_T,
                );

                (*pcci).dwSize = (*DeviceExtension).CursorSize;
                (*pcci).bVisible = (*DeviceExtension).CursorVisible;

                (*Irp).IoStatus.Information =
                    core::mem::size_of::<CONSOLE_CURSOR_INFO>() as ULONG_PTR;
                Status = STATUS_SUCCESS;
                break 'switch;
            }

            IOCTL_CONSOLE_SET_CURSOR_INFO => {
                let mut pcci: PCONSOLE_CURSOR_INFO = core::ptr::null_mut();

                /* Validate input buffer */
                if ((*stk).Parameters.DeviceIoControl.InputBufferLength as usize)
                    < core::mem::size_of::<CONSOLE_CURSOR_INFO>()
                {
                    Status = STATUS_INVALID_PARAMETER;
                    break 'switch;
                }
                ASSERT!((*Irp).AssociatedIrp.SystemBuffer);

                pcci = (*Irp).AssociatedIrp.SystemBuffer as PCONSOLE_CURSOR_INFO;

                (*DeviceExtension).CursorSize = (*pcci).dwSize;
                (*DeviceExtension).CursorVisible = (*pcci).bVisible;
                if (*DeviceExtension).Enabled != 0 {
                    ScrSetCursorShape(DeviceExtension);
                }

                (*Irp).IoStatus.Information = 0;
                Status = STATUS_SUCCESS;
                break 'switch;
            }

            IOCTL_CONSOLE_GET_MODE => {
                let mut pcm: PCONSOLE_MODE = core::ptr::null_mut();

                /* Validate output buffer */
                if ((*stk).Parameters.DeviceIoControl.OutputBufferLength as usize)
                    < core::mem::size_of::<CONSOLE_MODE>()
                {
                    Status = STATUS_INVALID_PARAMETER;
                    break 'switch;
                }
                ASSERT!((*Irp).AssociatedIrp.SystemBuffer);

                pcm = (*Irp).AssociatedIrp.SystemBuffer as PCONSOLE_MODE;
                RtlZeroMemory!(
                    pcm as PVOID,
                    core::mem::size_of::<CONSOLE_MODE>() as SIZE_T,
                );

                (*pcm).dwMode = (*DeviceExtension).Mode;

                (*Irp).IoStatus.Information =
                    core::mem::size_of::<CONSOLE_MODE>() as ULONG_PTR;
                Status = STATUS_SUCCESS;
                break 'switch;
            }

            IOCTL_CONSOLE_SET_MODE => {
                let mut pcm: PCONSOLE_MODE = core::ptr::null_mut();

                /* Validate input buffer */
                if ((*stk).Parameters.DeviceIoControl.InputBufferLength as usize)
                    < core::mem::size_of::<CONSOLE_MODE>()
                {
                    Status = STATUS_INVALID_PARAMETER;
                    break 'switch;
                }
                ASSERT!((*Irp).AssociatedIrp.SystemBuffer);

                pcm = (*Irp).AssociatedIrp.SystemBuffer as PCONSOLE_MODE;
                (*DeviceExtension).Mode = (*pcm).dwMode;

                (*Irp).IoStatus.Information = 0;
                Status = STATUS_SUCCESS;
                break 'switch;
            }

            IOCTL_CONSOLE_FILL_OUTPUT_ATTRIBUTE => {
                let mut Buf: POUTPUT_ATTRIBUTE = core::ptr::null_mut();
                let mut vidmem: PUCHAR = core::ptr::null_mut();
                let mut offset: ULONG = 0;
                let mut dwCount: ULONG = 0;
                let mut nMaxLength: ULONG = 0;

                /* Validate input and output buffers */
                if ((*stk).Parameters.DeviceIoControl.InputBufferLength as usize)
                    < core::mem::size_of::<OUTPUT_ATTRIBUTE>()
                    || ((*stk).Parameters.DeviceIoControl.OutputBufferLength as usize)
                        < core::mem::size_of::<OUTPUT_ATTRIBUTE>()
                {
                    Status = STATUS_INVALID_PARAMETER;
                    break 'switch;
                }
                ASSERT!((*Irp).AssociatedIrp.SystemBuffer);

                Buf = (*Irp).AssociatedIrp.SystemBuffer as POUTPUT_ATTRIBUTE;
                nMaxLength = (*Buf).nLength;

                (*Buf).dwTransfered = 0;
                (*Irp).IoStatus.Information =
                    core::mem::size_of::<OUTPUT_ATTRIBUTE>() as ULONG_PTR;

                if ((*Buf).dwCoord.X as i32) < 0
                    || ((*Buf).dwCoord.X as i32) >= ((*DeviceExtension).Columns as i32)
                    || ((*Buf).dwCoord.Y as i32) < 0
                    || ((*Buf).dwCoord.Y as i32) >= ((*DeviceExtension).Rows as i32)
                    || nMaxLength == 0
                {
                    Status = STATUS_SUCCESS;
                    break 'switch;
                }

                if (*DeviceExtension).Enabled != 0 && !(*DeviceExtension).VideoMemory.is_null() {
                    let mut attr: UCHAR = ((*Buf).wAttribute) as u8;

                    vidmem = (*DeviceExtension).VideoMemory;
                    offset = (((*Buf).dwCoord.X as i32
                        + (*Buf).dwCoord.Y as i32 * (*DeviceExtension).Columns as i32)
                        * 2
                        + 1) as ULONG;

                    nMaxLength = min(
                        nMaxLength,
                        (((*DeviceExtension).Rows as i32 - (*Buf).dwCoord.Y as i32)
                            * (*DeviceExtension).Columns as i32
                            - (*Buf).dwCoord.X as i32) as ULONG,
                    );

                    dwCount = 0;
                    while dwCount < nMaxLength {
                        *vidmem.offset(
                            offset.wrapping_add(dwCount.wrapping_mul(2 as ULONG))
                                as isize,
                        ) = attr;
                        dwCount = dwCount.wrapping_add(1);
                    }
                    (*Buf).dwTransfered = dwCount;
                }

                Status = STATUS_SUCCESS;
                break 'switch;
            }

            IOCTL_CONSOLE_READ_OUTPUT_ATTRIBUTE => {
                let mut Buf: POUTPUT_ATTRIBUTE = core::ptr::null_mut();
                let mut pAttr: PUSHORT = core::ptr::null_mut();
                let mut vidmem: PUCHAR = core::ptr::null_mut();
                let mut offset: ULONG = 0;
                let mut dwCount: ULONG = 0;
                let mut nMaxLength: ULONG = 0;

                /* Validate input buffer */
                if ((*stk).Parameters.DeviceIoControl.InputBufferLength as usize)
                    < core::mem::size_of::<OUTPUT_ATTRIBUTE>()
                {
                    Status = STATUS_INVALID_PARAMETER;
                    break 'switch;
                }
                ASSERT!((*Irp).AssociatedIrp.SystemBuffer);

                Buf = (*Irp).AssociatedIrp.SystemBuffer as POUTPUT_ATTRIBUTE;
                (*Irp).IoStatus.Information = 0;

                /* Validate output buffer */
                if (*stk).Parameters.DeviceIoControl.OutputBufferLength == 0
                {
                    Status = STATUS_SUCCESS;
                    break 'switch;
                }
                ASSERT!((*Irp).MdlAddress);
                pAttr = MmGetSystemAddressForMdlSafe!((*Irp).MdlAddress, NormalPagePriority)
                    as PUSHORT;
                if pAttr.is_null() {
                    Status = STATUS_INSUFFICIENT_RESOURCES;
                    break 'switch;
                }

                if ((*Buf).dwCoord.X as i32) < 0
                    || ((*Buf).dwCoord.X as i32) >= ((*DeviceExtension).Columns as i32)
                    || ((*Buf).dwCoord.Y as i32) < 0
                    || ((*Buf).dwCoord.Y as i32) >= ((*DeviceExtension).Rows as i32)
                {
                    Status = STATUS_SUCCESS;
                    break 'switch;
                }

                nMaxLength = (*stk).Parameters.DeviceIoControl.OutputBufferLength;
                nMaxLength /= core::mem::size_of::<USHORT>() as ULONG;

                if (*DeviceExtension).Enabled != 0 && !(*DeviceExtension).VideoMemory.is_null() {
                    vidmem = (*DeviceExtension).VideoMemory;
                    offset = (((*Buf).dwCoord.X as i32
                        + (*Buf).dwCoord.Y as i32 * (*DeviceExtension).Columns as i32)
                        * 2
                        + 1) as ULONG;

                    nMaxLength = min(
                        nMaxLength,
                        (((*DeviceExtension).Rows as i32 - (*Buf).dwCoord.Y as i32)
                            * (*DeviceExtension).Columns as i32
                            - (*Buf).dwCoord.X as i32) as ULONG,
                    );

                    dwCount = 0;
                    while dwCount < nMaxLength {
                        *(pAttr as PCHAR) = (*vidmem.offset(
                            offset.wrapping_add(dwCount.wrapping_mul(2 as ULONG))
                                as isize,
                        )) as i8;
                        dwCount = dwCount.wrapping_add(1);
                        pAttr = pAttr.offset(1);
                    }
                    (*Irp).IoStatus.Information = (dwCount as usize)
                        .wrapping_mul(core::mem::size_of::<USHORT>())
                        as ULONG_PTR;
                }

                Status = STATUS_SUCCESS;
                break 'switch;
            }

            IOCTL_CONSOLE_WRITE_OUTPUT_ATTRIBUTE => {
                let mut dwCoord: COORD;
                let mut pCoord: PCOORD = core::ptr::null_mut();
                let mut pAttr: PUSHORT = core::ptr::null_mut();
                let mut vidmem: PUCHAR = core::ptr::null_mut();
                let mut offset: ULONG = 0;
                let mut dwCount: ULONG = 0;
                let mut nMaxLength: ULONG = 0;

                //
                // NOTE: For whatever reason no OUTPUT_ATTRIBUTE structure
                // is used for this IOCTL.
                //

                /* Validate output buffer */
                if ((*stk).Parameters.DeviceIoControl.OutputBufferLength as usize)
                    < core::mem::size_of::<COORD>()
                {
                    Status = STATUS_INVALID_PARAMETER;
                    break 'switch;
                }
                ASSERT!((*Irp).MdlAddress);
                pCoord = MmGetSystemAddressForMdlSafe!((*Irp).MdlAddress, NormalPagePriority)
                    as PCOORD;
                if pCoord.is_null() {
                    Status = STATUS_INSUFFICIENT_RESOURCES;
                    break 'switch;
                }
                /* Capture the input info data */
                dwCoord = *pCoord;

                nMaxLength = ((*stk).Parameters.DeviceIoControl.OutputBufferLength as usize)
                    .wrapping_sub(core::mem::size_of::<COORD>())
                    as ULONG;
                nMaxLength /= core::mem::size_of::<USHORT>() as ULONG;

                (*Irp).IoStatus.Information = 0;

                if (dwCoord.X as i32) < 0
                    || (dwCoord.X as i32) >= ((*DeviceExtension).Columns as i32)
                    || (dwCoord.Y as i32) < 0
                    || (dwCoord.Y as i32) >= ((*DeviceExtension).Rows as i32)
                    || nMaxLength == 0
                {
                    Status = STATUS_SUCCESS;
                    break 'switch;
                }

                pAttr = pCoord.offset(1 as isize) as PUSHORT;

                if (*DeviceExtension).Enabled != 0 && !(*DeviceExtension).VideoMemory.is_null() {
                    vidmem = (*DeviceExtension).VideoMemory;
                    offset = ((dwCoord.X as i32
                        + dwCoord.Y as i32 * (*DeviceExtension).Columns as i32)
                        * 2
                        + 1) as ULONG;

                    nMaxLength = min(
                        nMaxLength,
                        (((*DeviceExtension).Rows as i32 - dwCoord.Y as i32)
                            * (*DeviceExtension).Columns as i32
                            - dwCoord.X as i32) as ULONG,
                    );

                    dwCount = 0;
                    while dwCount < nMaxLength {
                        *vidmem.offset(
                            offset.wrapping_add(dwCount.wrapping_mul(2 as ULONG))
                                as isize,
                        ) = (*(pAttr as PCHAR)) as u8;
                        dwCount = dwCount.wrapping_add(1);
                        pAttr = pAttr.offset(1);
                    }
                    (*Irp).IoStatus.Information = (dwCount as usize)
                        .wrapping_mul(core::mem::size_of::<USHORT>())
                        as ULONG_PTR;
                }

                Status = STATUS_SUCCESS;
                break 'switch;
            }

            IOCTL_CONSOLE_SET_TEXT_ATTRIBUTE => {
                /* Validate input buffer */
                if ((*stk).Parameters.DeviceIoControl.InputBufferLength as usize)
                    < core::mem::size_of::<USHORT>()
                {
                    Status = STATUS_INVALID_PARAMETER;
                    break 'switch;
                }
                ASSERT!((*Irp).AssociatedIrp.SystemBuffer);

                (*DeviceExtension).CharAttribute =
                    *((*Irp).AssociatedIrp.SystemBuffer as PUSHORT);

                (*Irp).IoStatus.Information = 0;
                Status = STATUS_SUCCESS;
                break 'switch;
            }

            IOCTL_CONSOLE_FILL_OUTPUT_CHARACTER => {
                let mut Buf: POUTPUT_CHARACTER = core::ptr::null_mut();
                let mut vidmem: PUCHAR = core::ptr::null_mut();
                let mut offset: ULONG = 0;
                let mut dwCount: ULONG = 0;
                let mut nMaxLength: ULONG = 0;

                /* Validate input and output buffers */
                if ((*stk).Parameters.DeviceIoControl.InputBufferLength as usize)
                    < core::mem::size_of::<OUTPUT_CHARACTER>()
                    || ((*stk).Parameters.DeviceIoControl.OutputBufferLength as usize)
                        < core::mem::size_of::<OUTPUT_CHARACTER>()
                {
                    Status = STATUS_INVALID_PARAMETER;
                    break 'switch;
                }
                ASSERT!((*Irp).AssociatedIrp.SystemBuffer);

                Buf = (*Irp).AssociatedIrp.SystemBuffer as POUTPUT_CHARACTER;
                nMaxLength = (*Buf).nLength;

                (*Buf).dwTransfered = 0;
                (*Irp).IoStatus.Information =
                    core::mem::size_of::<OUTPUT_CHARACTER>() as ULONG_PTR;

                if ((*Buf).dwCoord.X as i32) < 0
                    || ((*Buf).dwCoord.X as i32) >= ((*DeviceExtension).Columns as i32)
                    || ((*Buf).dwCoord.Y as i32) < 0
                    || ((*Buf).dwCoord.Y as i32) >= ((*DeviceExtension).Rows as i32)
                    || nMaxLength == 0
                {
                    Status = STATUS_SUCCESS;
                    break 'switch;
                }

                if (*DeviceExtension).Enabled != 0 && !(*DeviceExtension).VideoMemory.is_null() {
                    let mut ch: UCHAR = ((*Buf).cCharacter) as u8;

                    vidmem = (*DeviceExtension).VideoMemory;
                    offset = (((*Buf).dwCoord.X as i32
                        + (*Buf).dwCoord.Y as i32 * (*DeviceExtension).Columns as i32)
                        * 2) as ULONG;

                    nMaxLength = min(
                        nMaxLength,
                        (((*DeviceExtension).Rows as i32 - (*Buf).dwCoord.Y as i32)
                            * (*DeviceExtension).Columns as i32
                            - (*Buf).dwCoord.X as i32) as ULONG,
                    );

                    dwCount = 0;
                    while dwCount < nMaxLength {
                        *vidmem.offset(
                            offset.wrapping_add(dwCount.wrapping_mul(2 as ULONG))
                                as isize,
                        ) = ch;
                        dwCount = dwCount.wrapping_add(1);
                    }
                    (*Buf).dwTransfered = dwCount;
                }

                Status = STATUS_SUCCESS;
                break 'switch;
            }

            IOCTL_CONSOLE_READ_OUTPUT_CHARACTER => {
                let mut Buf: POUTPUT_CHARACTER = core::ptr::null_mut();
                let mut pChar: PCHAR = core::ptr::null_mut();
                let mut vidmem: PUCHAR = core::ptr::null_mut();
                let mut offset: ULONG = 0;
                let mut dwCount: ULONG = 0;
                let mut nMaxLength: ULONG = 0;

                /* Validate input buffer */
                if ((*stk).Parameters.DeviceIoControl.InputBufferLength as usize)
                    < core::mem::size_of::<OUTPUT_CHARACTER>()
                {
                    Status = STATUS_INVALID_PARAMETER;
                    break 'switch;
                }
                ASSERT!((*Irp).AssociatedIrp.SystemBuffer);

                Buf = (*Irp).AssociatedIrp.SystemBuffer as POUTPUT_CHARACTER;
                (*Irp).IoStatus.Information = 0;

                /* Validate output buffer */
                if (*stk).Parameters.DeviceIoControl.OutputBufferLength == 0
                {
                    Status = STATUS_SUCCESS;
                    break 'switch;
                }
                ASSERT!((*Irp).MdlAddress);
                pChar = MmGetSystemAddressForMdlSafe!((*Irp).MdlAddress, NormalPagePriority)
                    as PCHAR;
                if pChar.is_null() {
                    Status = STATUS_INSUFFICIENT_RESOURCES;
                    break 'switch;
                }

                if ((*Buf).dwCoord.X as i32) < 0
                    || ((*Buf).dwCoord.X as i32) >= ((*DeviceExtension).Columns as i32)
                    || ((*Buf).dwCoord.Y as i32) < 0
                    || ((*Buf).dwCoord.Y as i32) >= ((*DeviceExtension).Rows as i32)
                {
                    Status = STATUS_SUCCESS;
                    break 'switch;
                }

                nMaxLength = (*stk).Parameters.DeviceIoControl.OutputBufferLength;

                if (*DeviceExtension).Enabled != 0 && !(*DeviceExtension).VideoMemory.is_null() {
                    vidmem = (*DeviceExtension).VideoMemory;
                    offset = (((*Buf).dwCoord.X as i32
                        + (*Buf).dwCoord.Y as i32 * (*DeviceExtension).Columns as i32)
                        * 2) as ULONG;

                    nMaxLength = min(
                        nMaxLength,
                        (((*DeviceExtension).Rows as i32 - (*Buf).dwCoord.Y as i32)
                            * (*DeviceExtension).Columns as i32
                            - (*Buf).dwCoord.X as i32) as ULONG,
                    );

                    dwCount = 0;
                    while dwCount < nMaxLength {
                        *pChar = *vidmem.offset(
                            offset.wrapping_add(dwCount.wrapping_mul(2 as ULONG))
                                as isize,
                        ) as CHAR;
                        dwCount = dwCount.wrapping_add(1);
                        pChar = pChar.offset(1);
                    }
                    (*Irp).IoStatus.Information = (dwCount as usize)
                        .wrapping_mul(core::mem::size_of::<CHAR>())
                        as ULONG_PTR;
                }

                Status = STATUS_SUCCESS;
                break 'switch;
            }

            IOCTL_CONSOLE_WRITE_OUTPUT_CHARACTER => {
                let mut dwCoord: COORD;
                let mut pCoord: PCOORD = core::ptr::null_mut();
                let mut pChar: PCHAR = core::ptr::null_mut();
                let mut vidmem: PUCHAR = core::ptr::null_mut();
                let mut offset: ULONG = 0;
                let mut dwCount: ULONG = 0;
                let mut nMaxLength: ULONG = 0;

                //
                // NOTE: For whatever reason no OUTPUT_CHARACTER structure
                // is used for this IOCTL.
                //

                /* Validate output buffer */
                if ((*stk).Parameters.DeviceIoControl.OutputBufferLength as usize)
                    < core::mem::size_of::<COORD>()
                {
                    Status = STATUS_INVALID_PARAMETER;
                    break 'switch;
                }
                ASSERT!((*Irp).MdlAddress);
                pCoord = MmGetSystemAddressForMdlSafe!((*Irp).MdlAddress, NormalPagePriority)
                    as PCOORD;
                if pCoord.is_null() {
                    Status = STATUS_INSUFFICIENT_RESOURCES;
                    break 'switch;
                }
                /* Capture the input info data */
                dwCoord = *pCoord;

                nMaxLength = ((*stk).Parameters.DeviceIoControl.OutputBufferLength as usize)
                    .wrapping_sub(core::mem::size_of::<COORD>())
                    as ULONG;
                (*Irp).IoStatus.Information = 0;

                if (dwCoord.X as i32) < 0
                    || (dwCoord.X as i32) >= ((*DeviceExtension).Columns as i32)
                    || (dwCoord.Y as i32) < 0
                    || (dwCoord.Y as i32) >= ((*DeviceExtension).Rows as i32)
                    || nMaxLength == 0
                {
                    Status = STATUS_SUCCESS;
                    break 'switch;
                }

                pChar = pCoord.offset(1 as isize) as PCHAR;

                if (*DeviceExtension).Enabled != 0 && !(*DeviceExtension).VideoMemory.is_null() {
                    vidmem = (*DeviceExtension).VideoMemory;
                    offset = ((dwCoord.X as i32
                        + dwCoord.Y as i32 * (*DeviceExtension).Columns as i32)
                        * 2) as ULONG;

                    nMaxLength = min(
                        nMaxLength,
                        (((*DeviceExtension).Rows as i32 - dwCoord.Y as i32)
                            * (*DeviceExtension).Columns as i32
                            - dwCoord.X as i32) as ULONG,
                    );

                    dwCount = 0;
                    while dwCount < nMaxLength {
                        *vidmem.offset(
                            offset.wrapping_add(dwCount.wrapping_mul(2 as ULONG))
                                as isize,
                        ) = *pChar as UCHAR;
                        dwCount = dwCount.wrapping_add(1);
                        pChar = pChar.offset(1);
                    }
                    (*Irp).IoStatus.Information = (dwCount as usize)
                        .wrapping_mul(core::mem::size_of::<CHAR>())
                        as ULONG_PTR;
                }

                Status = STATUS_SUCCESS;
                break 'switch;
            }

            IOCTL_CONSOLE_DRAW => {
                let mut ConsoleDraw: CONSOLE_DRAW;
                let mut pConsoleDraw: PCONSOLE_DRAW = core::ptr::null_mut();
                let mut Src: PUCHAR = core::ptr::null_mut();
                let mut Dest: PUCHAR = core::ptr::null_mut();
                let mut SrcDelta: UINT32 = 0;
                let mut DestDelta: UINT32 = 0;
                let mut i: UINT32 = 0;

                /* Validate output buffer */
                if ((*stk).Parameters.DeviceIoControl.OutputBufferLength as usize)
                    < core::mem::size_of::<CONSOLE_DRAW>()
                {
                    Status = STATUS_INVALID_PARAMETER;
                    break 'switch;
                }
                ASSERT!((*Irp).MdlAddress);
                pConsoleDraw = MmGetSystemAddressForMdlSafe!((*Irp).MdlAddress, NormalPagePriority)
                    as PCONSOLE_DRAW;
                if pConsoleDraw.is_null() {
                    Status = STATUS_INSUFFICIENT_RESOURCES;
                    break 'switch;
                }
                /* Capture the input info data */
                ConsoleDraw = *pConsoleDraw;

                /* Check whether we have the size for the header plus the data area */
                if (((*stk).Parameters.DeviceIoControl.OutputBufferLength as usize)
                    .wrapping_sub(core::mem::size_of::<CONSOLE_DRAW>())
                    / (2 as usize))
                    < ((ConsoleDraw.SizeX as ULONG).wrapping_mul(ConsoleDraw.SizeY as ULONG) as usize)
                {
                    Status = STATUS_INVALID_BUFFER_SIZE;
                    break 'switch;
                }

                (*Irp).IoStatus.Information = 0;

                /* Set the cursor position, clipping it to the screen */
                (*DeviceExtension).CursorX = min(
                    max(ConsoleDraw.CursorX as i32, 0),
                    (*DeviceExtension).Columns as i32 - 1,
                ) as USHORT;
                (*DeviceExtension).CursorY = min(
                    max(ConsoleDraw.CursorY as i32, 0),
                    (*DeviceExtension).Rows as i32 - 1,
                ) as USHORT;
                if (*DeviceExtension).Enabled != 0 {
                    ScrSetCursor(DeviceExtension);
                }

                // TODO: For the moment if the ConsoleDraw rectangle has borders
                // out of the screen-buffer we just bail out. Would it be better
                // to actually clip the rectangle within its borders instead?
                if (ConsoleDraw.X as i32) < 0
                    || (ConsoleDraw.X as i32) >= ((*DeviceExtension).Columns as i32)
                    || (ConsoleDraw.Y as i32) < 0
                    || (ConsoleDraw.Y as i32) >= ((*DeviceExtension).Rows as i32)
                {
                    Status = STATUS_SUCCESS;
                    break 'switch;
                }
                if (ConsoleDraw.SizeX as i32)
                    > ((*DeviceExtension).Columns as i32 - ConsoleDraw.X as i32)
                    || (ConsoleDraw.SizeY as i32)
                        > ((*DeviceExtension).Rows as i32 - ConsoleDraw.Y as i32)
                {
                    Status = STATUS_SUCCESS;
                    break 'switch;
                }

                if (*DeviceExtension).Enabled != 0 && !(*DeviceExtension).VideoMemory.is_null() {
                    Src = pConsoleDraw.offset(1 as isize) as PUCHAR;
                    SrcDelta = (ConsoleDraw.SizeX as UINT32).wrapping_mul(2 as UINT32);
                    Dest = (*DeviceExtension).VideoMemory.offset(
                        ((ConsoleDraw.X as i32 + ConsoleDraw.Y as i32 * (*DeviceExtension).Columns as i32)
                            * 2) as isize,
                    );
                    DestDelta = ((*DeviceExtension).Columns as UINT32).wrapping_mul(2 as UINT32);
                    /* 2 == sizeof(CHAR) + sizeof(BYTE) */

                    /* Copy each line */
                    i = 0 as UINT32;
                    while i < ConsoleDraw.SizeY as UINT32 {
                        RtlCopyMemory!(Dest as PVOID, Src as PVOID, SrcDelta as SIZE_T);
                        Src = Src.offset(SrcDelta as isize);
                        Dest = Dest.offset(DestDelta as isize);
                        i = i.wrapping_add(1);
                    }
                }

                Status = STATUS_SUCCESS;
                break 'switch;
            }

            IOCTL_CONSOLE_LOADFONT => {
                //
                // FIXME: For the moment we support only a fixed 256-char 8-bit font.
                //

                /* Validate input buffer */
                if ((*stk).Parameters.DeviceIoControl.InputBufferLength as usize)
                    < (256 * 8) as usize
                {
                    Status = STATUS_INVALID_PARAMETER;
                    break 'switch;
                }
                ASSERT!((*Irp).AssociatedIrp.SystemBuffer);

                if !(*DeviceExtension).FontBitfield.is_null() {
                    ExFreePoolWithTag((*DeviceExtension).FontBitfield as PVOID, TAG_BLUE);
                }
                (*DeviceExtension).FontBitfield = ExAllocatePoolWithTag(
                    NonPagedPool,
                    (256 * 8) as SIZE_T,
                    TAG_BLUE,
                ) as PUCHAR;
                if (*DeviceExtension).FontBitfield.is_null() {
                    Status = STATUS_NO_MEMORY;
                    break 'switch;
                }
                RtlCopyMemory!(
                    (*DeviceExtension).FontBitfield as PVOID,
                    (*Irp).AssociatedIrp.SystemBuffer as PVOID,
                    (256 * 8) as SIZE_T,
                );

                /* Set the font if needed */
                if (*DeviceExtension).Enabled != 0 && !(*DeviceExtension).VideoMemory.is_null() {
                    ScrSetFont((*DeviceExtension).FontBitfield);
                }

                (*Irp).IoStatus.Information = 0;
                Status = STATUS_SUCCESS;
                break 'switch;
            }

            _ => {
                Status = STATUS_NOT_IMPLEMENTED;
            }
        }
    }

    (*Irp).IoStatus.__bindgen_anon_1.Status = Status;
    IoCompleteRequest(Irp, (IO_VIDEO_INCREMENT as i8));

    return Status;
}

unsafe extern "system" fn ScrDispatch(DeviceObject: PDEVICE_OBJECT, Irp: PIRP) -> NTSTATUS {
    let stk: PIO_STACK_LOCATION = IoGetCurrentIrpStackLocation(Irp);

    UNREFERENCED_PARAMETER!(DeviceObject);

    DPRINT1!("ScrDispatch(0x%p): stk->MajorFunction = %lu UNIMPLEMENTED\n",
            DeviceObject, (*stk).MajorFunction as core::ffi::c_uint);

    (*Irp).IoStatus.__bindgen_anon_1.Status = STATUS_NOT_IMPLEMENTED;
    IoCompleteRequest(Irp, IO_NO_INCREMENT as CCHAR);
    return STATUS_NOT_IMPLEMENTED;
}

#[no_mangle]
pub unsafe extern "system" fn DriverEntry(
    DriverObject: PDRIVER_OBJECT,
    RegistryPath: PUNICODE_STRING,
) -> NTSTATUS {
    let mut Status: NTSTATUS;
    let mut DeviceObject: PDEVICE_OBJECT = core::mem::zeroed();
    let mut DeviceName: _UNICODE_STRING = RTL_CONSTANT_STRING!(RUSTCONV_WSTR_3C4AFF199191.as_ptr());
    let mut SymlinkName: _UNICODE_STRING = RTL_CONSTANT_STRING!(RUSTCONV_WSTR_38821E379CB2.as_ptr());

    DPRINT!("Screen Driver 0.0.6\n");

    (*DriverObject).MajorFunction[IRP_MJ_CREATE as usize] =
        Some(ScrCreateClose);
    (*DriverObject).MajorFunction[IRP_MJ_CLOSE as usize] =
        Some(ScrCreateClose);
    (*DriverObject).MajorFunction[IRP_MJ_READ as usize] =
        Some(ScrDispatch);
    (*DriverObject).MajorFunction[IRP_MJ_WRITE as usize] =
        Some(ScrWrite);
    (*DriverObject).MajorFunction[IRP_MJ_DEVICE_CONTROL as usize] =
        Some(ScrIoControl);

    Status = IoCreateDevice(
        DriverObject,
        core::mem::size_of::<DEVICE_EXTENSION>() as ULONG,
        core::ptr::addr_of_mut!(DeviceName),
        FILE_DEVICE_SCREEN as ULONG,
        FILE_DEVICE_SECURE_OPEN as ULONG,
        TRUE as BOOLEAN,
        core::ptr::addr_of_mut!(DeviceObject),
    );
    if !NT_SUCCESS!(Status) {
        return Status;
    }

    Status = IoCreateSymbolicLink(
        core::ptr::addr_of_mut!(SymlinkName),
        core::ptr::addr_of_mut!(DeviceName),
    );
    if NT_SUCCESS!(Status) {
        /* By default disable the screen (but don't touch INBV: ResetDisplayParametersDeviceExtension is still NULL) */
        ScrResetScreen(
            (*DeviceObject).DeviceExtension as PDEVICE_EXTENSION,
            TRUE as BOOLEAN,
            FALSE as BOOLEAN,
        );
        /* Now set ResetDisplayParametersDeviceExtension to enable synchronizing with INBV */
        ResetDisplayParametersDeviceExtension =
            (*DeviceObject).DeviceExtension as PDEVICE_EXTENSION;
        (*DeviceObject).Flags &= !(DO_DEVICE_INITIALIZING as ULONG);
    } else {
        IoDeleteDevice(DeviceObject);
    }
    return Status;
}
