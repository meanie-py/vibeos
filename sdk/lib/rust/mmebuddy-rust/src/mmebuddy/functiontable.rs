use crate::mmebuddy::devicelist::IsValidSoundDevice;
use crate::prelude::*;
use core::ffi::c_void;
#[no_mangle]
pub unsafe extern "C" fn SetSoundDeviceFunctionTable(
        SoundDevice: PSOUND_DEVICE,
        FunctionTable: PMMFUNCTION_TABLE) -> MMRESULT {
    VALIDATE_MMSYS_PARAMETER!(IsValidSoundDevice(SoundDevice));
    VALIDATE_MMSYS_PARAMETER!(!FunctionTable.is_null());

    /* Zero out the existing function table (if present) */
    ZeroMemory!(
        core::ptr::addr_of_mut!((*SoundDevice).FunctionTable) as *mut core::ffi::c_void,
        core::mem::size_of::<MMFUNCTION_TABLE>() as usize,
    );

    if !FunctionTable.is_null() {
    } else {
        return MMSYSERR_INVALPARAM;
    }

    /* Fill in the client-supplied functions */
    CopyMemory!(
        core::ptr::addr_of_mut!((*SoundDevice).FunctionTable) as *mut core::ffi::c_void,
        FunctionTable as *const core::ffi::c_void,
        core::mem::size_of::<MMFUNCTION_TABLE>() as usize,
    );

    return MMSYSERR_NOERROR;
}
#[no_mangle]
pub unsafe extern "C" fn GetSoundDeviceFunctionTable(mut SoundDevice: PSOUND_DEVICE, mut FunctionTable: *mut PMMFUNCTION_TABLE) -> MMRESULT {
    if IsValidSoundDevice(SoundDevice) == 0 {
        return MMSYSERR_INVALPARAM as MMRESULT;
    }
    if FunctionTable.is_null() {
        return MMSYSERR_INVALPARAM as MMRESULT;
    }
    *FunctionTable = core::ptr::addr_of_mut!((*SoundDevice).FunctionTable) as PMMFUNCTION_TABLE;
    return MMSYSERR_NOERROR;
}
