use ntkrnl_sys::base::*;
use ntkrnl_sys::rtl::RtlZeroMemory;
use ntkrnl_sys::ddk::{MOUSE_INPUT_DATA, PMOUSE_INPUT_DATA};
use crate::i8042prt_h::*;

// ntddmou.h #defines (not emitted by bindgen). RawButtons/ButtonFlags bit masks.
const MOUSE_BUTTON_4_DOWN: u32 = 0x0040;
const MOUSE_BUTTON_5_DOWN: u32 = 0x0100;
const MOUSE_WHEEL: u32 = 0x0400;

// mouse.c routines — declared extern (cdecl) because mouse.rs is not yet
// integrated; during cohabitation these resolve against the C-compiled mouse.c.
// Both are plain (non-NTAPI) C functions in i8042prt.h.
extern "C" {
    fn i8042MouHandle(DeviceExtension: PI8042_MOUSE_EXTENSION, Output: UCHAR);
    fn i8042MouHandleButtons(DeviceExtension: PI8042_MOUSE_EXTENSION, Mask: USHORT);
}

// C: `VOID i8042MouHandlePs2pp(...)` — plain (non-NTAPI) cdecl in i8042prt.h,
// called from mouse.c. Export as cdecl (`extern "C"`) so the symbol matches the
// C declaration on i386 (NTAPI/stdcall would mis-decorate it `_..@8`).
#[no_mangle]
pub unsafe extern "C" fn i8042MouHandlePs2pp(
    DeviceExtension: PI8042_MOUSE_EXTENSION,
    Input: UCHAR,
) {
    let PktType: UCHAR;
    let MouseInput: PMOUSE_INPUT_DATA = (*DeviceExtension)
        .MouseBuffer
        .add((*DeviceExtension).MouseInBuffer as usize);

    match (*DeviceExtension).MouseState {
        MouseIdle | XMovement => {
            let state = (*DeviceExtension).MouseState as usize;
            (*DeviceExtension).MouseLogiBuffer[state] = Input;
            (*DeviceExtension).MouseState =
                core::mem::transmute(((*DeviceExtension).MouseState as u32) + 1);
        }

        YMovement => {
            (*DeviceExtension).MouseLogiBuffer[2] = Input;
            (*DeviceExtension).MouseState = MouseIdle;

            if ((*DeviceExtension).MouseLogiBuffer[0] & 0xC0) == 0 {
                (*DeviceExtension).MouseState = MouseIdle;
                i8042MouHandle(DeviceExtension, (*DeviceExtension).MouseLogiBuffer[0]);
                i8042MouHandle(DeviceExtension, (*DeviceExtension).MouseLogiBuffer[1]);
                i8042MouHandle(DeviceExtension, (*DeviceExtension).MouseLogiBuffer[2]);
                return;
            }

            if ((*DeviceExtension).MouseLogiBuffer[0] & 0x48) != 0x48
                || (((*DeviceExtension).MouseLogiBuffer[1] & 0x0C) >> 2)
                    != ((*DeviceExtension).MouseLogiBuffer[2] & 0x03)
            {
                return;
            }

            PktType = (((*DeviceExtension).MouseLogiBuffer[0] & 0x30) >> 2)
                | (((*DeviceExtension).MouseLogiBuffer[1] & 0x30) >> 4);

            match PktType {
                0 => {}
                1 => {
                    RtlZeroMemory(
                        MouseInput as *mut core::ffi::c_void,
                        core::mem::size_of::<MOUSE_INPUT_DATA>(),
                    );
                    // RawButtons is a direct field of MOUSE_INPUT_DATA.
                    if (*DeviceExtension).MouseLogiBuffer[2] & 0x10 != 0 {
                        let raw = core::ptr::addr_of_mut!((*MouseInput).RawButtons);
                        core::ptr::write_volatile(
                            raw,
                            core::ptr::read_volatile(raw) | MOUSE_BUTTON_4_DOWN as ULONG,
                        );
                    }
                    if (*DeviceExtension).MouseLogiBuffer[2] & 0x20 != 0 {
                        let raw = core::ptr::addr_of_mut!((*MouseInput).RawButtons);
                        core::ptr::write_volatile(
                            raw,
                            core::ptr::read_volatile(raw) | MOUSE_BUTTON_5_DOWN as ULONG,
                        );
                    }

                    // ButtonFlags/ButtonData live in the anonymous union/struct
                    // (_MOUSE_INPUT_DATA.__bindgen_anon_1.__bindgen_anon_1).
                    if (*DeviceExtension).MouseLogiBuffer[2] & 0x0F != 0 {
                        let bf = core::ptr::addr_of_mut!(
                            (*MouseInput).__bindgen_anon_1.__bindgen_anon_1.ButtonFlags
                        );
                        core::ptr::write_volatile(
                            bf,
                            core::ptr::read_volatile(bf) | MOUSE_WHEEL as USHORT,
                        );
                        let bd = core::ptr::addr_of_mut!(
                            (*MouseInput).__bindgen_anon_1.__bindgen_anon_1.ButtonData
                        );
                        if (*DeviceExtension).MouseLogiBuffer[2] & 0x08 != 0 {
                            core::ptr::write_volatile(
                                bd,
                                (((*DeviceExtension).MouseLogiBuffer[2] & 0x07) as i32 - 8)
                                    as USHORT,
                            );
                        } else {
                            core::ptr::write_volatile(
                                bd,
                                ((*DeviceExtension).MouseLogiBuffer[2] & 0x07) as USHORT,
                            );
                        }
                    }
                    i8042MouHandleButtons(
                        DeviceExtension,
                        (MOUSE_BUTTON_4_DOWN | MOUSE_BUTTON_5_DOWN) as USHORT,
                    );
                    // C: DeviceExtension->MouseHook.QueueMousePacket(CallContext).
                    // The hook field is PVOID (i8042prt_h); cast to the
                    // PI8042_QUEUE_PACKET fn ptr (ntdd8042.h: NTAPI VOID(*)(PVOID))
                    // and call if non-null.
                    let qmp = (*DeviceExtension).MouseHook.QueueMousePacket;
                    if !qmp.is_none() {
                        let f: unsafe extern "system" fn(PVOID) = core::mem::transmute(qmp);
                        f((*DeviceExtension).MouseHook.CallContext);
                    }
                }
                _ => {}
            }
        }

        _ => {}
    }
}
