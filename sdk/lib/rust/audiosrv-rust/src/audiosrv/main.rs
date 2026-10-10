// Deterministic UTF-16 literals, generated from the C by rustconv (--seal-wide); the model never saw the bytes.
use crate::prelude::*;
pub static RUSTCONV_WSTR_840CCC4E2E04: [u16; 29] = [
    0x0054u16, 0x0068, 0x0069, 0x0073, 0x0044, 0x0065, 0x0076, 0x0069, 0x0063, 0x0065,
    0x0044, 0x006F, 0x0065, 0x0073, 0x004E, 0x006F, 0x0074, 0x0052, 0x0065, 0x0061,
    0x006C, 0x006C, 0x0079, 0x0045, 0x0078, 0x0069, 0x0073, 0x0074, 0x0000,
];

use core::ffi::c_void;

// ---- file-scope preamble (translated) ----
// @c2rust file-scope data (compiler-exact tables/statics)
pub const SERVICE_NAME: [::core::ffi::c_ushort; 9] = unsafe {
    ::core::mem::transmute::<[u8; 18], [::core::ffi::c_ushort; 9]>(*b"A\0u\0d\0i\0o\0S\0r\0v\0\0\0")
};

#[no_mangle]
pub static mut service_status_handle: SERVICE_STATUS_HANDLE =
    ::core::ptr::null::<SERVICE_STATUS_HANDLE__>() as *mut SERVICE_STATUS_HANDLE__;

#[no_mangle]
pub static mut service_status: SERVICE_STATUS = _SERVICE_STATUS {
    dwServiceType: 0,
    dwCurrentState: 0,
    dwControlsAccepted: 0,
    dwWin32ExitCode: 0,
    dwServiceSpecificExitCode: 0,
    dwCheckPoint: 0,
    dwWaitHint: 0,
};
#[no_mangle]
pub unsafe extern "C" fn InitializeFakeDevice() {
    let mut list_node: *mut PnP_AudioDevice;
    list_node = CreateDeviceDescriptor(RUSTCONV_WSTR_840CCC4E2E04.as_ptr() as *mut WCHAR, TRUE as BOOL) as *mut PnP_AudioDevice;
    AppendAudioDeviceToList(list_node);
    DestroyDeviceDescriptor!(list_node);
}

#[no_mangle]
pub unsafe extern "system" fn ServiceControlHandler(
    dwControl: DWORD,
    dwEventType: DWORD,
    lpEventData: LPVOID,
    lpContext: LPVOID,
) -> DWORD {
    if dwControl == SERVICE_CONTROL_INTERROGATE {
        DPRINT!("* Interrogation\n");
        return NO_ERROR;
    } else if dwControl == SERVICE_CONTROL_STOP || dwControl == SERVICE_CONTROL_SHUTDOWN {
        DPRINT!("* Service Stop/Shutdown request received\n");

        DPRINT!("Unregistering device notifications\n");
        UnregisterDeviceNotifications();

        DPRINT!("Destroying audio device list\n");
        DestroyAudioDeviceList();

        service_status.dwCurrentState = SERVICE_STOP_PENDING;
        SetServiceStatus(service_status_handle, core::ptr::addr_of_mut!(service_status));

        service_status.dwWin32ExitCode = 0;
        service_status.dwCurrentState = SERVICE_STOPPED;

        SetServiceStatus(service_status_handle, core::ptr::addr_of_mut!(service_status));

        DPRINT!("* Service stopped\n");

        return NO_ERROR;
    } else if dwControl == SERVICE_CONTROL_DEVICEEVENT {
        DPRINT!("* Device Event\n");
        return HandleDeviceEvent(dwEventType, lpEventData);
    } else {
        return ERROR_CALL_NOT_IMPLEMENTED;
    }
}
#[no_mangle]
pub unsafe extern "system" fn ServiceMain(argc: DWORD, argv: LPWSTR) {
    DPRINT!("* Service starting\n");
    DPRINT!("Registering service control handler\n");
    service_status_handle = RegisterServiceCtrlHandlerExW(
        SERVICE_NAME.as_ptr(),
        Some(ServiceControlHandler as unsafe extern "system" fn(DWORD, DWORD, LPVOID, LPVOID) -> DWORD),
        core::ptr::null_mut(),
    );

    DPRINT!("Service status handle %d\n", service_status_handle);
    if service_status_handle.is_null() {
        DPRINT!("Failed to register service control handler\n");
        /* FIXME - we should fail */
    }

    /* Set these to defaults */
    service_status.dwServiceType = SERVICE_WIN32_OWN_PROCESS;
    service_status.dwServiceSpecificExitCode = 0;
    service_status.dwWin32ExitCode = NO_ERROR;
    service_status.dwWaitHint = 0;
    service_status.dwControlsAccepted = 0;
    service_status.dwCheckPoint = 0;

    /* Tell SCM we're starting */
    service_status.dwCurrentState = SERVICE_START_PENDING;
    SetServiceStatus(service_status_handle, core::ptr::addr_of_mut!(service_status));

    DPRINT!("Creating audio device list\n");
    /* This creates the audio device list and mutex */
    if CreateAudioDeviceList(AUDIO_LIST_MAX_SIZE) == 0 {
        DPRINT!("Failed to create audio device list\n");
        service_status.dwCurrentState = SERVICE_STOPPED;
        service_status.dwWin32ExitCode = (-(1 as i32)) as DWORD;
        SetServiceStatus(service_status_handle, core::ptr::addr_of_mut!(service_status));
        return;
    }

    DPRINT!("Registering for device notifications\n");
    /* We want to know when devices are added/removed */
    if RegisterForDeviceNotifications() == 0 {
        /* FIXME: This is not fatal at present as ROS does not support this */
        DPRINT!("Failed to register for device notifications\n");
/*
        DestroyAudioDeviceList();

        service_status.dwCurrentState = SERVICE_STOPPED;
        service_status.dwWin32ExitCode = -1;
        SetServiceStatus(service_status_handle, &service_status);
        return;
*/
    }
    /* start system audio services */
    StartSystemAudioServices();

    InitializeFakeDevice();

    DPRINT!("Processing existing devices\n");
    /* Now find any devices that already exist on the system */
    if ProcessExistingDevices() == 0 {
        DPRINT!("Could not process existing devices\n");
        UnregisterDeviceNotifications();
        DestroyAudioDeviceList();

        service_status.dwCurrentState = SERVICE_STOPPED;
        service_status.dwWin32ExitCode = (-(1 as i32)) as DWORD;
        SetServiceStatus(service_status_handle, core::ptr::addr_of_mut!(service_status));
        return;
    }

    DPRINT!("* Service started\n");
    /* Tell SCM we are now running, and we may be stopped */
    service_status.dwCurrentState = SERVICE_RUNNING;
    service_status.dwControlsAccepted = SERVICE_ACCEPT_STOP;
    SetServiceStatus(service_status_handle, core::ptr::addr_of_mut!(service_status));
}
#[no_mangle]
pub unsafe extern "C" fn wmain() -> ::core::ffi::c_int {
    let mut service_table: [SERVICE_TABLE_ENTRYW; 2] = [
        SERVICE_TABLE_ENTRYW {
            lpServiceName: SERVICE_NAME.as_ptr() as LPWSTR,
            lpServiceProc: ::core::mem::transmute::<
                Option<unsafe extern "C" fn(DWORD, LPWSTR) -> ()>,
                LPSERVICE_MAIN_FUNCTIONW,
            >(Some(::core::mem::transmute::<
                unsafe extern "system" fn(DWORD, LPWSTR),
                unsafe extern "C" fn(DWORD, LPWSTR) -> (),
            >(ServiceMain as unsafe extern "system" fn(DWORD, LPWSTR)))),
        },
        SERVICE_TABLE_ENTRYW {
            lpServiceName: core::ptr::null_mut(),
            lpServiceProc: None,
        },
    ];

    DPRINT!("Audio Service main()\n");
    if StartServiceCtrlDispatcherW(service_table.as_mut_ptr()) == 0 {
        DPRINT!("StartServiceCtrlDispatcher failed\n");
    }

    return 0;
}
