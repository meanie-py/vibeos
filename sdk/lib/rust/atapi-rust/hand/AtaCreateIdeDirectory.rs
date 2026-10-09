// HAND-PORTED (atapi.c AtaCreateIdeDirectory): the banked body called the declaring macro
// as an expression and `port fix` could not repair it in two rounds. Re-applied by
// `rustconv port` after every reassembly (port.apply_hand_bodies).
unsafe extern "C" fn AtaCreateIdeDirectory() {
    let mut Handle: HANDLE = core::ptr::null_mut();
    let mut ObjectAttributes: OBJECT_ATTRIBUTES = core::mem::zeroed();
    DECLARE_PAGED_UNICODE_STRING!(DirectoryName, RUSTCONV_WSTR_604F52A85CB9);

    PAGED_CODE!();

    InitializeObjectAttributes!(
        &mut ObjectAttributes,
        &DirectoryName as *const UNICODE_STRING as PUNICODE_STRING,
        OBJ_CASE_INSENSITIVE | OBJ_PERMANENT | OBJ_KERNEL_HANDLE,
        core::ptr::null_mut(),
        core::ptr::null_mut()
    );

    let Status = ZwCreateDirectoryObject(&mut Handle, DIRECTORY_ALL_ACCESS, &mut ObjectAttributes);
    if NT_SUCCESS!(Status) {
        /* We don't need a handle for a permanent object */
        ZwClose(Handle);
    }
    // Directory creation failures are ignored: another driver may already have created it,
    // and a fatal error surfaces later from IoCreateDevice().
}
