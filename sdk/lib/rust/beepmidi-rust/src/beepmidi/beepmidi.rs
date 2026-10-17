// Deterministic UTF-16 literals, generated from the C by rustconv (--seal-wide); the model never saw the bytes.
use crate::prelude::*;
pub static RUSTCONV_WSTR_261985D313CC: [u16; 13] = [
    0x005Cu16, 0x0044, 0x0065, 0x0076, 0x0069, 0x0063, 0x0065, 0x005C, 0x0042, 0x0065,
    0x0065, 0x0070, 0x0000,
];
pub static RUSTCONV_WSTR_7217F8813428: [u16; 11] = [
    0x0050u16, 0x0043, 0x0020, 0x0073, 0x0070, 0x0065, 0x0061, 0x006B, 0x0065, 0x0072,
    0x0000,
];

use core::ffi::c_void;

// ---- file-scope preamble (translated) ----
// @c2rust file-scope data (compiler-exact tables/statics)
#[no_mangle]
pub static mut the_device: *mut DeviceInfo = ::core::ptr::null::<DeviceInfo>() as *mut DeviceInfo;

#[no_mangle]
pub static mut device_lock: CRITICAL_SECTION = _RTL_CRITICAL_SECTION {
    DebugInfo: ::core::ptr::null::<_RTL_CRITICAL_SECTION_DEBUG>()
        as *mut _RTL_CRITICAL_SECTION_DEBUG,
    LockCount: 0,
    RecursionCount: 0,
    OwningThread: ::core::ptr::null::<::core::ffi::c_void>() as *mut ::core::ffi::c_void,
    LockSemaphore: ::core::ptr::null::<::core::ffi::c_void>() as *mut ::core::ffi::c_void,
    SpinCount: 0,
};

pub const TIMESLICE_SIZE: i32 = 60;

pub const POLYPHONY: i32 = 3;

pub const MIDI_NOTE_OFF: u8 = 0x80;
pub const MIDI_NOTE_ON: u8 = 0x90;
pub const MIDI_CONTROL_CHANGE: u8 = 0xB0;
pub const MIDI_PROGRAM: u8 = 0xC0;
pub const MIDI_PITCH_BEND: u8 = 0xE0;
pub const MIDI_SYSTEM: u8 = 0xFF;
pub const MIDI_RESET: u8 = 0xFF;

// #define DPRINT FakePrintf  (macro alias to a function name; handled at call sites)

#[repr(C)]
pub struct NoteNode {
    pub next: *mut NoteNode,
    pub previous: *mut NoteNode,

    pub note: UCHAR,
    pub velocity: UCHAR, // 0 is note-off
}

#[repr(C)]
pub struct DeviceInfo {
    pub mme_handle: HDRVR,
    pub kernel_device: HANDLE,

    pub callback: DWORD_PTR,
    pub instance: DWORD,
    pub flags: DWORD,

    pub running_status: UCHAR,

    pub playing_notes_count: DWORD,
    pub note_list: *mut NoteNode,
    pub refresh_notes: BOOL,

    pub thread_handle: HANDLE,
    pub terminate_thread: BOOL,
    pub work_available: HANDLE,
}
#[no_mangle]
pub unsafe extern "C" fn FakePrintf_v(str: *mut i8, ap_list: *mut ::core::ffi::c_void) {
    /* Just to shut the compiler up */
}


#[no_mangle]
pub unsafe extern "system" fn ProcessPlayingNotes(parameter: LPVOID) -> DWORD {
    let mut device_info: *mut DeviceInfo = parameter as *mut DeviceInfo;
    let mut status: NTSTATUS;
    let mut io_status_block: _IO_STATUS_BLOCK = core::mem::zeroed();
    let mut arp_notes: DWORD;

    DPRINT!(b"Note processing started\n\0".as_ptr() as *const ::core::ffi::c_char);

    /* We lock the note list only while accessing it */

    loop {
        WaitForSingleObject((*the_device).work_available, INFINITE as DWORD);
        if (*device_info).terminate_thread != 0 {
            break;
        }
        let mut node: *mut NoteNode;

        /* Number of notes being arpeggiated */
        arp_notes = 1 as DWORD;

        EnterCriticalSection(core::ptr::addr_of_mut!(device_lock));

        /* Calculate how much time to allocate to each playing note */

        DPRINT!(
            b"%d notes active\n\0".as_ptr() as *const ::core::ffi::c_char,
            (*device_info).playing_notes_count as ::core::ffi::c_int,
        );

        node = (*device_info).note_list;

        while !node.is_null() && arp_notes <= POLYPHONY as DWORD {
            let mut beep_data: BEEP_SET_PARAMETERS = core::mem::zeroed();
            let mut actually_playing: DWORD = 0 as DWORD;

            let mut frequency: f64 = (*node).note as f64;

            DPRINT!(b"playing..\n\0".as_ptr() as *const ::core::ffi::c_char);

            frequency = frequency / (12 as f64);
            frequency = pow(2 as f64, frequency);
            frequency = 8.1758f64 * frequency;

            if (*device_info).playing_notes_count > POLYPHONY as DWORD {
                actually_playing = POLYPHONY as DWORD;
            } else {
                actually_playing = (*device_info).playing_notes_count;
            }

            DPRINT!(
                b"Frequency %f\n\0".as_ptr() as *const ::core::ffi::c_char,
                frequency,
            );

            // TODO
            beep_data.Frequency = frequency as DWORD;
            beep_data.Duration = (TIMESLICE_SIZE as DWORD) / actually_playing; /* device_info->playing_notes_count; */

            status = NtDeviceIoControlFile(
                (*device_info).kernel_device,
                core::ptr::null_mut(),
                None,
                core::ptr::null_mut(),
                core::ptr::addr_of_mut!(io_status_block),
                IOCTL_BEEP_SET as ULONG,
                core::ptr::addr_of_mut!(beep_data) as PVOID,
                core::mem::size_of::<BEEP_SET_PARAMETERS>() as ULONG,
                core::ptr::null_mut(),
                0 as ULONG,
            );

            if !NT_SUCCESS!(status) {
                DPRINT!(
                    b"ERROR %d\n\0".as_ptr() as *const ::core::ffi::c_char,
                    GetLastError() as ::core::ffi::c_int,
                );
            }

            SleepEx(beep_data.Duration as DWORD, TRUE as BOOL);

            if (*device_info).refresh_notes != 0 {
                (*device_info).refresh_notes = FALSE as BOOL;
                break;
            }

            arp_notes = arp_notes.wrapping_add(1);
            node = (*node).next;
        }

        LeaveCriticalSection(core::ptr::addr_of_mut!(device_lock));
    }

    return 0 as DWORD;
}
#[no_mangle]
pub unsafe extern "C" fn GetDeviceCapabilities(caps: *mut MIDIOUTCAPS) -> MMRESULT {
    /* These are ignored for now */
    (*caps).wMid = 0 as WORD;
    (*caps).wPid = 0 as WORD;

    (*caps).vDriverVersion = 0x0100 as MMVERSION;

    memset(
        core::ptr::addr_of_mut!((*caps).szPname) as *mut core::ffi::c_void,
        0 as core::ffi::c_int,
        core::mem::size_of::<[WCHAR; 32]>() as size_t,
    );
    wcscpy(
        core::ptr::addr_of_mut!((*caps).szPname) as *mut WCHAR,
        RUSTCONV_WSTR_7217F8813428.as_ptr(),
    );

    (*caps).wTechnology = MOD_SQSYNTH as WORD;

    (*caps).wVoices = 1 as WORD; /* We only have one voice */
    (*caps).wNotes = POLYPHONY as WORD;
    (*caps).wChannelMask = 0xFFBF as WORD; /* Ignore channel 10 */

    (*caps).dwSupport = 0 as DWORD;

    return MMSYSERR_NOERROR as MMRESULT;
}
#[no_mangle]
pub unsafe extern "C" fn CallClient(
    mut device_info: *mut DeviceInfo,
    mut message: DWORD_PTR,
    mut parameter1: DWORD_PTR,
    mut parameter2: DWORD_PTR,
) -> BOOL {
    DPRINT!(
        b"Calling client - callback 0x%x mmhandle 0x%x\n\0".as_ptr() as *const i8,
        (*device_info).callback,
        (*device_info).mme_handle,
    );
    return DriverCallback(
        (*device_info).callback,
        HIWORD!((*device_info).flags) as DWORD,
        (*device_info).mme_handle,
        message as DWORD,
        (*device_info).instance as DWORD_PTR,
        parameter1,
        parameter2,
    );
}
#[no_mangle]
pub unsafe extern "C" fn OpenDevice(
    private_data: *mut *mut DeviceInfo,
    open_desc: *mut MIDIOPENDESC,
    flags: DWORD,
) -> MMRESULT {
    let mut status: NTSTATUS = 0;
    let mut heap: HANDLE = core::ptr::null_mut::<core::ffi::c_void>();
    let mut kernel_device: HANDLE = core::ptr::null_mut::<core::ffi::c_void>();
    let mut beep_device_name: _UNICODE_STRING = _UNICODE_STRING {
        Length: 0,
        MaximumLength: 0,
        Buffer: core::ptr::null_mut::<WCHAR>(),
    };
    let mut attribs: _OBJECT_ATTRIBUTES = _OBJECT_ATTRIBUTES {
        Length: 0,
        RootDirectory: core::ptr::null_mut::<core::ffi::c_void>(),
        ObjectName: core::ptr::null_mut::<_UNICODE_STRING>(),
        Attributes: 0,
        SecurityDescriptor: core::ptr::null_mut::<core::ffi::c_void>(),
        SecurityQualityOfService: core::ptr::null_mut::<core::ffi::c_void>(),
    };
    let mut status_block: _IO_STATUS_BLOCK = core::mem::zeroed();

    /* One at a time.. */
    if !the_device.is_null() {
        DPRINT!(
            b"Already allocated\n\0" as *const u8 as *const core::ffi::c_char
                as *mut core::ffi::c_char,
        );
        return MMSYSERR_ALLOCATED;
    }

    /* Make the device name into a unicode string and open it */

    RtlInitUnicodeString(
        core::ptr::addr_of_mut!(beep_device_name),
        RUSTCONV_WSTR_261985D313CC.as_ptr(),
    );

    /* InitializeObjectAttributes(&attribs, &beep_device_name, FACT_0001, NULL, NULL) expanded */
    attribs.Length = core::mem::size_of::<_OBJECT_ATTRIBUTES>() as ULONG;
    attribs.RootDirectory = core::ptr::null_mut::<core::ffi::c_void>() as HANDLE;
    attribs.ObjectName = core::ptr::addr_of_mut!(beep_device_name) as PUNICODE_STRING;
    attribs.Attributes = 0 as ULONG;
    attribs.SecurityDescriptor = core::ptr::null_mut::<core::ffi::c_void>() as PVOID;
    attribs.SecurityQualityOfService = core::ptr::null_mut::<core::ffi::c_void>() as PVOID;

    status = NtCreateFile(
        core::ptr::addr_of_mut!(kernel_device),
        FILE_READ_DATA | FILE_WRITE_DATA,
        core::ptr::addr_of_mut!(attribs),
        core::ptr::addr_of_mut!(status_block),
        core::ptr::null_mut(),
        0,
        FILE_SHARE_READ | FILE_SHARE_WRITE,
        FILE_OPEN_IF,
        0,
        core::ptr::null_mut(),
        0,
    );

    if !(status >= 0) {
        DPRINT!(
            b"Could not connect to BEEP device - %d\n\0" as *const u8
                as *const core::ffi::c_char as *mut core::ffi::c_char,
            GetLastError() as core::ffi::c_int,
        );
        return MMSYSERR_ERROR;
    }

    DPRINT!(
        b"Opened!\n\0" as *const u8 as *const core::ffi::c_char as *mut core::ffi::c_char,
    );

    /* Allocate and initialize the device info */

    heap = GetProcessHeap();

    the_device = HeapAlloc(
        heap,
        HEAP_ZERO_MEMORY,
        core::mem::size_of::<DeviceInfo>() as SIZE_T,
    ) as *mut DeviceInfo;

    if the_device.is_null() {
        DPRINT!(
            b"Out of memory\n\0" as *const u8 as *const core::ffi::c_char
                as *mut core::ffi::c_char,
        );
        return MMSYSERR_NOMEM;
    }

    /* Initialize */
    (*the_device).kernel_device = kernel_device;
    (*the_device).playing_notes_count = 0;
    (*the_device).note_list = core::ptr::null_mut::<NoteNode>();
    (*the_device).thread_handle = core::ptr::null_mut();
    (*the_device).terminate_thread = (FALSE as i32);
    (*the_device).running_status = 0;

    // TODO
    (*the_device).mme_handle = (*open_desc).hMidi as HDRVR;
    (*the_device).callback = (*open_desc).dwCallback;
    (*the_device).instance = (*open_desc).dwInstance as DWORD;
    (*the_device).flags = flags;

    /* Store the pointer in the user data */
    *private_data = the_device;

    /* This is threading-related code */
    (*the_device).work_available = CreateEvent(
        core::ptr::null_mut(),
        (TRUE as i32),
        (FALSE as i32),
        core::ptr::null_mut(),
    );

    if (*the_device).work_available.is_null() {
        DPRINT!(
            b"CreateEvent failed\n\0" as *const u8 as *const core::ffi::c_char
                as *mut core::ffi::c_char,
        );
        HeapFree(heap, 0, the_device as PVOID);
        return MMSYSERR_NOMEM;
    }

    (*the_device).thread_handle = CreateThread(
        core::ptr::null_mut(),
        0,
        Some(ProcessPlayingNotes),
        the_device as PVOID,
        0,
        core::ptr::null_mut(),
    );

    if (*the_device).thread_handle.is_null() {
        DPRINT!(
            b"CreateThread failed\n\0" as *const u8 as *const core::ffi::c_char
                as *mut core::ffi::c_char,
        );
        CloseHandle((*the_device).work_available);
        HeapFree(heap, 0, the_device as PVOID);
        return MMSYSERR_NOMEM;
    }

    /* Now we call the client application to say the device is open */
    DPRINT!(
        b"Sending MOM_OPEN\n\0" as *const u8 as *const core::ffi::c_char
            as *mut core::ffi::c_char,
    );
    DPRINT!(
        b"Success? %d\n\0" as *const u8 as *const core::ffi::c_char as *mut core::ffi::c_char,
        CallClient(
            the_device,
            MOM_OPEN,
            0,
            0,
        ) as core::ffi::c_int,
    );

    return MMSYSERR_NOERROR;
}
#[no_mangle]
pub unsafe extern "C" fn CloseDevice(device_info: *mut DeviceInfo) -> MMRESULT {
    let heap: HANDLE = GetProcessHeap();

    /* If we're working in threaded mode we need to wait for thread to die */
    (*the_device).terminate_thread = TRUE as BOOL;
    SetEvent((*device_info).work_available);

    WaitForSingleObject((*the_device).thread_handle, INFINITE as DWORD);
    CloseHandle((*the_device).thread_handle);
    CloseHandle((*the_device).work_available);

    /* Let the client application know the device is closing */
    DPRINT!(b"Sending MOM_CLOSE\n\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char);
    CallClient(
        device_info,
        MOM_CLOSE as DWORD_PTR,
        0 as DWORD_PTR,
        0 as DWORD_PTR,
    );

    NtClose((*device_info).kernel_device);

    /* Free resources */
    HeapFree(heap, 0 as DWORD, device_info as PVOID);

    the_device = ::core::ptr::null_mut::<DeviceInfo>();

    return MMSYSERR_NOERROR as MMRESULT;
}
#[no_mangle]
pub unsafe extern "C" fn StopNote(device_info: *mut DeviceInfo, note: UCHAR) -> MMRESULT {
    let heap: HANDLE = GetProcessHeap();
    let mut node: *mut NoteNode;
    let mut prev_node: *mut NoteNode = core::ptr::null_mut();

    DPRINT!(b"StopNote\n\0".as_ptr() as *const i8 as *mut i8);

    EnterCriticalSection(core::ptr::addr_of_mut!(device_lock));

    node = (*device_info).note_list;

    while !node.is_null() {
        if (*node).note == note {
            /* Found the note - just remove the node from the list */

            DPRINT!(
                b"Stopping note %d\n\0".as_ptr() as *const i8 as *mut i8,
                (*node).note as i32,
            );

            if !prev_node.is_null() {
                (*prev_node).next = (*node).next;
            } else {
                (*device_info).note_list = (*node).next;
            }

            HeapFree(heap, 0, node as LPVOID);

            (*device_info).playing_notes_count = (*device_info).playing_notes_count.wrapping_sub(1);

            DPRINT!(
                b"Note stopped - now playing %d notes\n\0".as_ptr() as *const i8 as *mut i8,
                (*device_info).playing_notes_count as i32,
            );

            if (*device_info).playing_notes_count == 0 {
                ResetEvent((*device_info).work_available);
            }

            LeaveCriticalSection(core::ptr::addr_of_mut!(device_lock));
            (*device_info).refresh_notes = (TRUE as i32);

            return MMSYSERR_NOERROR;
        }

        prev_node = node;
        node = (*node).next;
    }

    LeaveCriticalSection(core::ptr::addr_of_mut!(device_lock));

    /* Hmm, a good idea? */

    return MMSYSERR_NOERROR;
}
#[no_mangle]
pub unsafe extern "C" fn PlayNote(
    device_info: *mut DeviceInfo,
    note: UCHAR,
    velocity: UCHAR,
) -> MMRESULT {
    let heap: HANDLE = GetProcessHeap();

    let mut node: *mut NoteNode;

    DPRINT!(b"PlayNote\n\0".as_ptr() as *const i8);

    if velocity == 0 {
        DPRINT!(b"Zero velocity\n\0".as_ptr() as *const i8);

        /* Velocity zero is effectively a "note off" */
        StopNote(device_info, note);
    } else {
        /* Start playing the note */
        let mut new_node: *mut NoteNode;

        EnterCriticalSection(core::ptr::addr_of_mut!(device_lock));

        node = (*device_info).note_list;

        while !node.is_null() {
            node = (*node).next;
        }

        new_node = HeapAlloc(
            heap,
            HEAP_ZERO_MEMORY,
            (core::mem::size_of::<NoteNode>()) as SIZE_T,
        ) as *mut NoteNode;

        if new_node.is_null() {
            LeaveCriticalSection(core::ptr::addr_of_mut!(device_lock));
            return MMSYSERR_NOMEM;
        }

        (*new_node).note = note;
        (*new_node).velocity = velocity;

        /*
            Prepend to the playing notes list. If exceeding polyphony,
            remove the oldest note (which will be at the tail.)
        */

        if !(*device_info).note_list.is_null() {
            (*(*device_info).note_list).previous = new_node;
        }

        (*new_node).next = (*device_info).note_list;
        (*new_node).previous = core::ptr::null_mut();

        (*device_info).note_list = new_node;
        (*device_info).playing_notes_count = (*device_info).playing_notes_count.wrapping_add(1);

        SetEvent((*device_info).work_available);

        LeaveCriticalSection(core::ptr::addr_of_mut!(device_lock));

        DPRINT!(
            b"Note started - now playing %d notes\n\0".as_ptr() as *const i8,
            (*device_info).playing_notes_count as i32,
        );
        (*device_info).refresh_notes = (TRUE as i32);
    }

    return MMSYSERR_NOERROR;
}
#[no_mangle]
pub unsafe extern "C" fn ProcessShortMidiMessage(
    device_info: *mut DeviceInfo,
    message: DWORD,
) -> MMRESULT {
    let mut status: DWORD;

    let mut category: DWORD;
    let mut channel: DWORD;
    let mut data1: DWORD;
    let mut data2: DWORD;

    status = message & 0x000000FF;

    /* Deal with running status */

    if status < (MIDI_NOTE_OFF as u32) {
        status = (*device_info).running_status as DWORD;
    }

    /* Ensure the status is sane! */

    if status < (MIDI_NOTE_OFF as u32) {
        /* It's garbage, ignore it */
        return MMSYSERR_NOERROR;
    }

    /* Figure out the message category and channel */

    category = status & 0xF0;
    channel = status & 0x0F; /* we don't use this */

    data1 = (message & 0x0000FF00) >> 8;
    data2 = (message & 0x00FF0000) >> 16;

    DPRINT!(
        b"0x%x, %d, %d\n\0".as_ptr() as *const i8,
        status as i32,
        data1 as i32,
        data2 as i32,
    );

    /* Filter drums (which are *usually* on channel 10) */
    if channel == 10 {
        return MMSYSERR_NOERROR;
    }

    /* Pass to the appropriate message handler */

    match category {
        __c if __c == MIDI_NOTE_ON as _ => {
            PlayNote(device_info, data1 as UCHAR, data2 as UCHAR);
        }

        __c if __c == MIDI_NOTE_OFF as _ => {
            StopNote(device_info, data1 as UCHAR);
        }

        _ => {}
    }

    return MMSYSERR_NOERROR;
}
#[no_mangle]
pub unsafe extern "C" fn ProcessLongMidiMessage(
    mut device_info: *mut DeviceInfo,
    mut header: *mut MIDIHDR,
) -> MMRESULT {
    let mut index: ::core::ffi::c_uint = 0 as ::core::ffi::c_uint;
    let mut midi_bytes: *mut UCHAR =
        (*header).lpData as *mut UCHAR;
    let mut msg_index: ::core::ffi::c_uint = 0 as ::core::ffi::c_uint;
    let mut msg: [UCHAR; 3] = [0; 3];

    /* Initialize the buffer */
    msg[2 as usize] = 0 as UCHAR;
    msg[1 as usize] = msg[2 as usize];
    msg[0 as usize] = msg[1 as usize];

    if (*header).dwFlags & MHDR_PREPARED as DWORD == 0 {
        DPRINT!(
            b"Not prepared!\n\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
        );
        return MIDIERR_UNPREPARED as MMRESULT;
    }

    DPRINT!(
        b"Processing %d bytes of MIDI\n\0".as_ptr() as *const ::core::ffi::c_char
            as *mut ::core::ffi::c_char,
        (*header).dwBufferLength as ::core::ffi::c_int,
    );

    while (index as DWORD) < (*header).dwBufferLength {
        /* New status byte? ( = new event) */
        if *midi_bytes.offset(index as isize) as ::core::ffi::c_int
            & 0x80 as ::core::ffi::c_int
            != 0
        {
            let mut short_msg: DWORD = 0;

            /* Deal with the existing event */

            if msg[0 as usize] as ::core::ffi::c_int
                & 0x80 as ::core::ffi::c_int
                != 0
            {
                short_msg = PACK_MIDI!(
                    msg[0 as usize] as DWORD,
                    msg[1 as usize] as DWORD,
                    msg[2 as usize] as DWORD,
                );

                DPRINT!(
                    b"Complete msg is 0x%x %d %d\n\0".as_ptr() as *const ::core::ffi::c_char
                        as *mut ::core::ffi::c_char,
                    msg[0 as usize] as ::core::ffi::c_int,
                    msg[1 as usize] as ::core::ffi::c_int,
                    msg[2 as usize] as ::core::ffi::c_int,
                );
                ProcessShortMidiMessage(device_info, short_msg);
            }

            /* Set new running status and start recording the event */
            DPRINT!(
                b"Set new running status\n\0".as_ptr() as *const ::core::ffi::c_char
                    as *mut ::core::ffi::c_char,
            );
            (*device_info).running_status = *midi_bytes.offset(index as isize);
            msg[0 as usize] = *midi_bytes.offset(index as isize);
            msg_index = 1 as ::core::ffi::c_uint;
        }
        /* Unexpected data byte? ( = re-use previous status) */
        else if msg_index == 0 as ::core::ffi::c_uint {
            if (*device_info).running_status as ::core::ffi::c_int
                & 0x80 as ::core::ffi::c_int
                != 0
            {
                DPRINT!(
                    b"Retrieving running status\n\0".as_ptr() as *const ::core::ffi::c_char
                        as *mut ::core::ffi::c_char,
                );
                msg[0 as usize] = (*device_info).running_status;
                msg[1 as usize] = *midi_bytes.offset(index as isize);
                msg_index = 2 as ::core::ffi::c_uint;
            } else {
                DPRINT!(
                    b"garbage\n\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
                );
            }
        }
        /* Expected data ( = append to message until buffer full) */
        else {
            DPRINT!(
                b"Next byte...\n\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
            );
            msg[msg_index as usize] = *midi_bytes.offset(index as isize);
            msg_index = msg_index.wrapping_add(1);

            if msg_index > 2 as ::core::ffi::c_uint {
                let mut short_msg: DWORD = 0;

                short_msg = PACK_MIDI!(
                    msg[0 as usize] as DWORD,
                    msg[1 as usize] as DWORD,
                    msg[2 as usize] as DWORD,
                );

                DPRINT!(
                    b"Complete msg is 0x%x %d %d\n\0".as_ptr() as *const ::core::ffi::c_char
                        as *mut ::core::ffi::c_char,
                    msg[0 as usize] as ::core::ffi::c_int,
                    msg[1 as usize] as ::core::ffi::c_int,
                    msg[2 as usize] as ::core::ffi::c_int,
                );
                ProcessShortMidiMessage(device_info, short_msg);

                /* Reinit */
                msg_index = 0 as ::core::ffi::c_uint;
                msg[2 as usize] = 0 as UCHAR;
                msg[1 as usize] = msg[2 as usize];
                msg[0 as usize] = msg[1 as usize];
            }
        }

        index = index.wrapping_add(1);
    }

    /*
        We're meant to clear MHDR_DONE and set MHDR_INQUEUE but since we
        deal with everything here and now we might as well just say so.
    */
    (*header).dwFlags |= MHDR_DONE as DWORD;
    (*header).dwFlags &= !(MHDR_INQUEUE as DWORD);

    DPRINT!(
        b"Success? %d\n\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
        CallClient(
            the_device,
            MOM_DONE as DWORD_PTR,
            header as DWORD_PTR,
            0 as DWORD_PTR,
        ),
    );

    return MMSYSERR_NOERROR as MMRESULT;
}

#[no_mangle]
pub unsafe extern "system" fn modMessage(
    device_id: UINT,
    message: UINT,
    private_data: DWORD_PTR,
    parameter1: DWORD_PTR,
    parameter2: DWORD_PTR,
) -> MMRESULT {
    match message {
        MODM_GETNUMDEVS => {
            /* Only one internal PC speaker device (and even that's too much) */
            DPRINT!(b"MODM_GETNUMDEVS\n\0".as_ptr() as *const i8);
            return 1 as MMRESULT;
        }

        MODM_GETDEVCAPS => {
            DPRINT!(b"MODM_GETDEVCAPS\n\0".as_ptr() as *const i8);
            return GetDeviceCapabilities(parameter1 as *mut MIDIOUTCAPS);
        }

        MODM_OPEN => {
            DPRINT!(b"MODM_OPEN\n\0".as_ptr() as *const i8);

            return OpenDevice(
                private_data as *mut *mut DeviceInfo,
                parameter1 as *mut MIDIOPENDESC,
                parameter2 as DWORD,
            );
        }

        MODM_CLOSE => {
            DPRINT!(b"MODM_CLOSE\n\0".as_ptr() as *const i8);
            return CloseDevice(private_data as *mut DeviceInfo);
        }

        MODM_DATA => {
            return ProcessShortMidiMessage(private_data as *mut DeviceInfo, parameter1 as DWORD);
        }

        MODM_PREPARE => {
            /* We don't bother with this */
            let hdr: *mut MIDIHDR = parameter1 as *mut MIDIHDR;
            (*hdr).dwFlags |= MHDR_PREPARED as DWORD;
            return MMSYSERR_NOERROR as MMRESULT;
        }

        MODM_UNPREPARE => {
            let hdr: *mut MIDIHDR = parameter1 as *mut MIDIHDR;
            (*hdr).dwFlags &= !(MHDR_PREPARED as DWORD);
            return MMSYSERR_NOERROR as MMRESULT;
        }

        MODM_LONGDATA => {
            DPRINT!(b"LONGDATA\n\0".as_ptr() as *const i8);
            return ProcessLongMidiMessage(
                private_data as *mut DeviceInfo,
                parameter1 as *mut MIDIHDR,
            );
        }

        MODM_RESET => {
            /* TODO */
        }

        _ => {}
    }

    DPRINT!(
        b"Not supported %d\n\0".as_ptr() as *const i8,
        message,
    );

    return MMSYSERR_NOTSUPPORTED as MMRESULT;
}

#[no_mangle]
pub unsafe extern "system" fn DriverProc(
    driver_id: DWORD,
    driver_handle: HDRVR,
    message: UINT,
    parameter1: LONG,
    parameter2: LONG,
) -> LONG {
    if message == DRV_LOAD {
        DPRINT!(b"DRV_LOAD\n\0".as_ptr() as *mut i8);
        the_device = core::ptr::null_mut();
        return 1;
    }
    if message == DRV_FREE {
        DPRINT!(b"DRV_FREE\n\0".as_ptr() as *mut i8);
        return 1;
    }
    if message == DRV_OPEN {
        DPRINT!(b"DRV_OPEN\n\0".as_ptr() as *mut i8);
        InitializeCriticalSection(core::ptr::addr_of_mut!(device_lock));
        return 1;
    }
    if message == DRV_CLOSE {
        DPRINT!(b"DRV_CLOSE\n\0".as_ptr() as *mut i8);
        return 1;
    }
    if message == DRV_ENABLE {
        DPRINT!(b"DRV_ENABLE\n\0".as_ptr() as *mut i8);
        return 1;
    }
    if message == DRV_DISABLE {
        DPRINT!(b"DRV_DISABLE\n\0".as_ptr() as *mut i8);
        return 1;
    }
    /*
        We don't provide configuration capabilities. This used to be
        for things like I/O port, IRQ, DMA settings, etc.
    */
    if message == DRV_QUERYCONFIGURE {
        DPRINT!(b"DRV_QUERYCONFIGURE\n\0".as_ptr() as *mut i8);
        return 0;
    }
    if message == DRV_CONFIGURE {
        DPRINT!(b"DRV_CONFIGURE\n\0".as_ptr() as *mut i8);
        return 0;
    }
    if message == DRV_INSTALL {
        DPRINT!(b"DRV_INSTALL\n\0".as_ptr() as *mut i8);
        return (DRVCNF_RESTART) as LONG;
    }

    DPRINT!(b"???\n\0".as_ptr() as *mut i8);

    return DefDriverProc(
        driver_id as DWORD_PTR,
        driver_handle,
        message,
        parameter1 as LPARAM,
        parameter2 as LPARAM,
    ) as LONG;
}
