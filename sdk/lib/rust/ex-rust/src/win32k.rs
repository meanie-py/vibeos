use ntkrnl_sys::base::*;
use ntkrnl_sys::constants::*;
use ntkrnl_sys::mm::*;
use ntkrnl_sys::ps::PsGetCurrentProcess;
use ntkrnl_sys::rtl::*;
use ntkrnl_sys::{opaque_ptr, opaque_ptr_const};

use ntkrnl_sys::ddk::{


    _EPROCESS, _UNICODE_STRING,
    ACCESS_MASK,
    _GENERIC_MAPPING,
    _KAPC_STATE,
    KPROCESSOR_MODE,
    MmAttachSession,
    MmDetachSession,
    MmGetSessionById,
    MmQuitNextSession,
    OBJECT_TYPE_INITIALIZER,
    ObCreateObjectType,
    OB_OPEN_REASON,
    PACCESS_STATE,
    PEPROCESS,
    PKWIN32_SESSION_CALLOUT,
    POBJECT_TYPE,
    PSECURITY_QUALITY_OF_SERVICE,
    PUNICODE_STRING,
    PsGetCurrentProcessSessionId,
    RtlInitUnicodeString,
    WIN32_CLOSEMETHOD_PARAMETERS,
    WIN32_DELETEMETHOD_PARAMETERS,
    WIN32_OKAYTOCLOSEMETHOD_PARAMETERS,
    WIN32_OPENMETHOD_PARAMETERS,
    WIN32_PARSEMETHOD_PARAMETERS,
};

// w! — NUL-terminated UTF-16 array from an ASCII literal (2-aligned 'static;
// a b"...\0" byte string cast to *const u16 is both ANSI-not-wide and
// potentially odd-aligned — see field manual / basesrv init.rs CSDVersion).
macro_rules! w {
    ($s:literal) => {{
        const __W: &[u16] = &{
            const BYTES: &[u8] = $s.as_bytes();
            let mut out = [0u16; { BYTES.len() + 1 }];
            let mut i = 0;
            while i < BYTES.len() {
                out[i] = BYTES[i] as u16;
                i += 1;
            }
            out
        };
        __W
    }};
}


#[repr(C)]
struct WIN32_KERNEL_OBJECT_HEADER {
    SessionId: ULONG,
}

type PWIN32_KERNEL_OBJECT_HEADER = *mut WIN32_KERNEL_OBJECT_HEADER;

#[no_mangle]
pub static mut ExWindowStationObjectType: POBJECT_TYPE = core::ptr::null_mut();

#[no_mangle]
pub static mut ExDesktopObjectType: POBJECT_TYPE = core::ptr::null_mut();

static mut ExpWindowStationMapping: _GENERIC_MAPPING = _GENERIC_MAPPING {
    GenericRead: STANDARD_RIGHTS_READ,
    GenericWrite: STANDARD_RIGHTS_WRITE,
    GenericExecute: STANDARD_RIGHTS_EXECUTE,
    GenericAll: STANDARD_RIGHTS_REQUIRED,
};

static mut ExpDesktopMapping: _GENERIC_MAPPING = _GENERIC_MAPPING {
    GenericRead: STANDARD_RIGHTS_READ,
    GenericWrite: STANDARD_RIGHTS_WRITE,
    GenericExecute: STANDARD_RIGHTS_EXECUTE,
    GenericAll: STANDARD_RIGHTS_REQUIRED,
};

#[no_mangle]
static mut ExpWindowStationObjectParse: PKWIN32_SESSION_CALLOUT = None;
#[no_mangle]
static mut ExpWindowStationObjectDelete: PKWIN32_SESSION_CALLOUT = None;
#[no_mangle]
static mut ExpWindowStationObjectOkToClose: PKWIN32_SESSION_CALLOUT = None;
#[no_mangle]
static mut ExpDesktopObjectOkToClose: PKWIN32_SESSION_CALLOUT = None;
#[no_mangle]
static mut ExpDesktopObjectDelete: PKWIN32_SESSION_CALLOUT = None;
#[no_mangle]
static mut ExpDesktopObjectOpen: PKWIN32_SESSION_CALLOUT = None;
#[no_mangle]
static mut ExpDesktopObjectClose: PKWIN32_SESSION_CALLOUT = None;

unsafe fn ExpWin32SessionCallout(
    Object: PVOID,
    CalloutProcedure: PKWIN32_SESSION_CALLOUT,
    Parameter: PVOID,
) -> NTSTATUS {
    let Win32ObjectHeader: PWIN32_KERNEL_OBJECT_HEADER = Object as PWIN32_KERNEL_OBJECT_HEADER;
    let mut SessionEntry: PVOID = core::ptr::null_mut();
    let mut ApcState: _KAPC_STATE = core::mem::zeroed();
    let mut Status: NTSTATUS;

    if (*(PsGetCurrentProcess() as *mut _EPROCESS)).__bindgen_anon_2.__bindgen_anon_1.ProcessInSession() == 0 ||
        (PsGetCurrentProcessSessionId() != (*Win32ObjectHeader).SessionId)
    {
        SessionEntry = MmGetSessionById((*Win32ObjectHeader).SessionId);
        if SessionEntry.is_null() {
            return STATUS_NOT_FOUND;
        }

        Status = MmAttachSession(SessionEntry, &mut ApcState as *mut _KAPC_STATE);
        if Status < 0 {
            MmQuitNextSession(SessionEntry);
            return Status;
        }
    }

    Status = (CalloutProcedure.unwrap())(Parameter);

    if !SessionEntry.is_null() {
        MmDetachSession(SessionEntry, &mut ApcState as *mut _KAPC_STATE);
        MmQuitNextSession(SessionEntry);
    }

    Status
}

#[no_mangle]
pub unsafe extern "system" fn ExpDesktopOkToClose(
    Process: PEPROCESS,
    Object: PVOID,
    Handle: HANDLE,
    AccessMode: KPROCESSOR_MODE,
) -> BOOLEAN {
    let Process = opaque_ptr(Process);
    let Object = opaque_ptr(Object);
    let Handle = opaque_ptr(Handle);
    let mut Parameters: WIN32_OKAYTOCLOSEMETHOD_PARAMETERS = core::mem::zeroed();

    Parameters.Process = Process;
    Parameters.Object = Object;
    Parameters.Handle = Handle;
    Parameters.PreviousMode = AccessMode;

    let Status = ExpWin32SessionCallout(
        Object,
        ExpDesktopObjectOkToClose,
        &mut Parameters as *mut WIN32_OKAYTOCLOSEMETHOD_PARAMETERS as PVOID,
    );

    if Status >= 0 { 1u8 } else { 0u8 }
}

#[no_mangle]
pub unsafe extern "system" fn ExpWindowStationOkToClose(
    Process: PEPROCESS,
    Object: PVOID,
    Handle: HANDLE,
    AccessMode: KPROCESSOR_MODE,
) -> BOOLEAN {
    let Process = opaque_ptr(Process);
    let Object = opaque_ptr(Object);
    let Handle = opaque_ptr(Handle);
    let mut Parameters: WIN32_OKAYTOCLOSEMETHOD_PARAMETERS = core::mem::zeroed();

    Parameters.Process = Process;
    Parameters.Object = Object;
    Parameters.Handle = Handle;
    Parameters.PreviousMode = AccessMode;

    let Status = ExpWin32SessionCallout(
        Object,
        ExpWindowStationObjectOkToClose,
        &mut Parameters as *mut WIN32_OKAYTOCLOSEMETHOD_PARAMETERS as PVOID,
    );

    if Status >= 0 { 1u8 } else { 0u8 }
}

#[no_mangle]
pub unsafe extern "system" fn ExpWinStaObjectDelete(DeletedObject: PVOID) {
    let DeletedObject = opaque_ptr(DeletedObject);
    let mut Parameters: WIN32_DELETEMETHOD_PARAMETERS = core::mem::zeroed();

    Parameters.Object = DeletedObject;

    ExpWin32SessionCallout(
        DeletedObject,
        ExpWindowStationObjectDelete,
        &mut Parameters as *mut WIN32_DELETEMETHOD_PARAMETERS as PVOID,
    );
}

#[no_mangle]
pub unsafe extern "system" fn ExpWinStaObjectParse(
    ParseObject: PVOID,
    ObjectType: PVOID,
    AccessState: PACCESS_STATE,
    AccessMode: KPROCESSOR_MODE,
    Attributes: ULONG,
    CompleteName: PUNICODE_STRING,
    RemainingName: PUNICODE_STRING,
    Context: PVOID,
    SecurityQos: PSECURITY_QUALITY_OF_SERVICE,
    Object: *mut PVOID,
) -> NTSTATUS {
    let ParseObject = opaque_ptr(ParseObject);
    let ObjectType = opaque_ptr(ObjectType);
    let AccessState = opaque_ptr(AccessState);
    let CompleteName = opaque_ptr(CompleteName);
    let RemainingName = opaque_ptr(RemainingName);
    let Context = opaque_ptr(Context);
    let Object = opaque_ptr(Object);
    let mut Parameters: WIN32_PARSEMETHOD_PARAMETERS = core::mem::zeroed();

    Parameters.ParseObject = ParseObject;
    Parameters.ObjectType = ObjectType;
    Parameters.AccessState = AccessState;
    Parameters.AccessMode = AccessMode;
    Parameters.Attributes = Attributes;
    Parameters.CompleteName = CompleteName;
    Parameters.RemainingName = RemainingName;
    Parameters.Context = Context;
    Parameters.SecurityQos = SecurityQos;
    Parameters.Object = Object;

    ExpWin32SessionCallout(
        ParseObject,
        ExpWindowStationObjectParse,
        &mut Parameters as *mut WIN32_PARSEMETHOD_PARAMETERS as PVOID,
    )
}

#[no_mangle]
pub unsafe extern "system" fn ExpDesktopDelete(DeletedObject: PVOID) {
    let DeletedObject = opaque_ptr(DeletedObject);
    let mut Parameters: WIN32_DELETEMETHOD_PARAMETERS = core::mem::zeroed();

    Parameters.Object = DeletedObject;

    ExpWin32SessionCallout(
        DeletedObject,
        ExpDesktopObjectDelete,
        &mut Parameters as *mut WIN32_DELETEMETHOD_PARAMETERS as PVOID,
    );
}

#[no_mangle]
// upstream-sync 2026-09-20: OB_OPEN_METHOD went NT6 (sdk/include/ndk/obtypes.h) — a new
// second parameter `KPROCESSOR_MODE AccessMode`, and GrantedAccess by POINTER instead of
// by value. ntoskrnl/ex/win32k.c's ExpDesktopOpen has both. Keeping the old five-parameter
// by-value shape here is not a cosmetic lag: on i386 `extern "system"` is stdcall and the
// callee pops, so a six-argument call into a five-parameter callee corrupts the stack, and
// the callee reads a pointer as a value.
pub unsafe extern "system" fn ExpDesktopOpen(
    Reason: OB_OPEN_REASON,
    _AccessMode: KPROCESSOR_MODE,
    Process: PEPROCESS,
    ObjectBody: PVOID,
    GrantedAccess: *mut ACCESS_MASK,
    HandleCount: ULONG,
) -> NTSTATUS {
    let Process = opaque_ptr(Process);
    let ObjectBody = opaque_ptr(ObjectBody);
    let mut Parameters: WIN32_OPENMETHOD_PARAMETERS = core::mem::zeroed();

    Parameters.OpenReason = Reason;
    Parameters.Process = Process;
    Parameters.Object = ObjectBody;
    Parameters.GrantedAccess = if GrantedAccess.is_null() { 0 } else { *GrantedAccess };
    Parameters.HandleCount = HandleCount;

    ExpWin32SessionCallout(
        ObjectBody,
        ExpDesktopObjectOpen,
        &mut Parameters as *mut WIN32_OPENMETHOD_PARAMETERS as PVOID,
    )
}

#[no_mangle]
pub unsafe extern "system" fn ExpDesktopClose(
    Process: PEPROCESS,
    Object: PVOID,
    GrantedAccess: ACCESS_MASK,
    ProcessHandleCount: ULONG,
    SystemHandleCount: ULONG,
) {
    let Process = opaque_ptr(Process);
    let Object = opaque_ptr(Object);
    let mut Parameters: WIN32_CLOSEMETHOD_PARAMETERS = core::mem::zeroed();

    Parameters.Process = Process;
    Parameters.Object = Object;
    Parameters.AccessMask = GrantedAccess;
    Parameters.ProcessHandleCount = ProcessHandleCount;
    Parameters.SystemHandleCount = SystemHandleCount;

    ExpWin32SessionCallout(
        Object,
        ExpDesktopObjectClose,
        &mut Parameters as *mut WIN32_CLOSEMETHOD_PARAMETERS as PVOID,
    );
}

#[no_mangle]
pub unsafe extern "system" fn ExpWin32kInit() -> BOOLEAN {
    let mut ObjectTypeInitializer: OBJECT_TYPE_INITIALIZER = core::mem::zeroed();
    let mut Name: _UNICODE_STRING = core::mem::zeroed();
    let Status: NTSTATUS;

    RtlZeroMemory(
        &mut ObjectTypeInitializer as *mut OBJECT_TYPE_INITIALIZER as PVOID,
        core::mem::size_of::<OBJECT_TYPE_INITIALIZER>(),
    );
    RtlInitUnicodeString(&mut Name as *mut _UNICODE_STRING, w!("WindowStation").as_ptr() as *const u16);
    ObjectTypeInitializer.Length = core::mem::size_of::<OBJECT_TYPE_INITIALIZER>() as u16;
    ObjectTypeInitializer.GenericMapping = ExpWindowStationMapping;
    ObjectTypeInitializer.PoolType = NonPagedPool as u32;
    ObjectTypeInitializer.DeleteProcedure = core::mem::transmute(ExpWinStaObjectDelete as usize);
    ObjectTypeInitializer.ParseProcedure = core::mem::transmute(ExpWinStaObjectParse as usize);
    ObjectTypeInitializer.OkayToCloseProcedure = core::mem::transmute(ExpWindowStationOkToClose as usize);
    ObjectTypeInitializer.SecurityRequired = 1u8;
    ObjectTypeInitializer.InvalidAttributes = OBJ_OPENLINK | OBJ_PERMANENT | OBJ_EXCLUSIVE;
    ObjectTypeInitializer.ValidAccessMask = STANDARD_RIGHTS_REQUIRED;
    Status = ObCreateObjectType(
        &mut Name as *mut _UNICODE_STRING,
        &mut ObjectTypeInitializer as *mut OBJECT_TYPE_INITIALIZER,
        core::ptr::null_mut(),
        &raw mut ExWindowStationObjectType as *mut POBJECT_TYPE,
    );
    if Status < 0 {
        return 0u8;
    }

    RtlInitUnicodeString(&mut Name as *mut _UNICODE_STRING, w!("Desktop").as_ptr() as *const u16);
    ObjectTypeInitializer.GenericMapping = ExpDesktopMapping;
    ObjectTypeInitializer.DeleteProcedure = core::mem::transmute(ExpDesktopDelete as usize);
    ObjectTypeInitializer.ParseProcedure = None;
    ObjectTypeInitializer.OkayToCloseProcedure = core::mem::transmute(ExpDesktopOkToClose as usize);
    ObjectTypeInitializer.OpenProcedure = core::mem::transmute(ExpDesktopOpen as usize);
    ObjectTypeInitializer.CloseProcedure = core::mem::transmute(ExpDesktopClose as usize);
    let Status = ObCreateObjectType(
        &mut Name as *mut _UNICODE_STRING,
        &mut ObjectTypeInitializer as *mut OBJECT_TYPE_INITIALIZER,
        core::ptr::null_mut(),
        &raw mut ExDesktopObjectType as *mut POBJECT_TYPE,
    );
    if Status < 0 {
        return 0u8;
    }

    1u8
}

// ============================================================================
// ORIGINAL C SOURCE
// ============================================================================
//
// /*
//  * COPYRIGHT:       See COPYING in the top level directory
//  * PROJECT:         ReactOS Kernel
//  * FILE:            ntoskrnl/ex/win32k.c
//  * PURPOSE:         Executive Win32 Object Support (Desktop/WinStation)
//  * PROGRAMMERS:     Alex Ionescu (alex@relsoft.net)
//  */
//
// #include <ntoskrnl.h>
// #define NDEBUG
// #include <debug.h>
//
// typedef struct _WIN32_KERNEL_OBJECT_HEADER
// {
//     ULONG SessionId;
// } WIN32_KERNEL_OBJECT_HEADER, *PWIN32_KERNEL_OBJECT_HEADER;
//
//
// /* DATA **********************************************************************/
//
// POBJECT_TYPE ExWindowStationObjectType = NULL;
// POBJECT_TYPE ExDesktopObjectType = NULL;
//
// _GENERIC_MAPPING ExpWindowStationMapping =
// {
//     STANDARD_RIGHTS_READ,
//     STANDARD_RIGHTS_WRITE,
//     STANDARD_RIGHTS_EXECUTE,
//     STANDARD_RIGHTS_REQUIRED
// };
//
// _GENERIC_MAPPING ExpDesktopMapping =
// {
//     STANDARD_RIGHTS_READ,
//     STANDARD_RIGHTS_WRITE,
//     STANDARD_RIGHTS_EXECUTE,
//     STANDARD_RIGHTS_REQUIRED
// };
//
// PKWIN32_SESSION_CALLOUT ExpWindowStationObjectParse = NULL;
// PKWIN32_SESSION_CALLOUT ExpWindowStationObjectDelete = NULL;
// PKWIN32_SESSION_CALLOUT ExpWindowStationObjectOkToClose = NULL;
// PKWIN32_SESSION_CALLOUT ExpDesktopObjectOkToClose = NULL;
// PKWIN32_SESSION_CALLOUT ExpDesktopObjectDelete = NULL;
// PKWIN32_SESSION_CALLOUT ExpDesktopObjectOpen = NULL;
// PKWIN32_SESSION_CALLOUT ExpDesktopObjectClose = NULL;
//
// /* FUNCTIONS ****************************************************************/
//
// NTSTATUS
// NTAPI
// ExpWin32SessionCallout(
//     _In_ PVOID Object,
//     _In_ PKWIN32_SESSION_CALLOUT CalloutProcedure,
//     _Inout_opt_ PVOID Parameter)
// {
//     PWIN32_KERNEL_OBJECT_HEADER Win32ObjectHeader;
//     PVOID SessionEntry = NULL;
//     _KAPC_STATE ApcState;
//     NTSTATUS Status;
//
//     /* The objects have a common header. And the kernel accesses it!
//        Thanks MS for this kind of retarded "design"! */
//     Win32ObjectHeader = Object;
//
//     /* Check if we are not already in the correct session */
//     if (!PsGetCurrentProcess()->ProcessInSession ||
//         (PsGetCurrentProcessSessionId() != Win32ObjectHeader->SessionId))
//     {
//         /* Get the session from the objects session Id */
//         DPRINT("SessionId == %d\n", Win32ObjectHeader->SessionId);
//         SessionEntry = MmGetSessionById(Win32ObjectHeader->SessionId);
//         if (SessionEntry == NULL)
//         {
//             /* The requested session does not even exist! */
//             ASSERT(FALSE);
//             return STATUS_NOT_FOUND;
//         }
//
//         /* Attach to the session */
//         Status = MmAttachSession(SessionEntry, &ApcState);
//         if (!NT_SUCCESS(Status))
//         {
//             DPRINT1("Could not attach to 0x%p, object %p, callout 0x%p\n",
//                     SessionEntry,
//                     Win32ObjectHeader,
//                     CalloutProcedure);
//
//             /* Cleanup and return */
//             MmQuitNextSession(SessionEntry);
//             ASSERT(FALSE);
//             return Status;
//         }
//     }
//
//     /* Call the callout routine */
//     Status = CalloutProcedure(Parameter);
//
//     /* Check if we have a session */
//     if (SessionEntry != NULL)
//     {
//         /* Detach from the session and quit using it */
//         MmDetachSession(SessionEntry, &ApcState);
//         MmQuitNextSession(SessionEntry);
//     }
//
//     /* Return the callback status */
//     return Status;
// }
//
// BOOLEAN
// NTAPI
// ExpDesktopOkToClose( IN PEPROCESS Process OPTIONAL,
//                      IN PVOID Object,
//                      IN HANDLE Handle,
//                      IN KPROCESSOR_MODE AccessMode)
// {
//     WIN32_OKAYTOCLOSEMETHOD_PARAMETERS Parameters;
//     NTSTATUS Status;
//
//     Parameters.Process = Process;
//     Parameters.Object = Object;
//     Parameters.Handle = Handle;
//     Parameters.PreviousMode = AccessMode;
//
//     Status = ExpWin32SessionCallout(Object,
//                                     ExpDesktopObjectOkToClose,
//                                     &Parameters);
//
//     return NT_SUCCESS(Status);
// }
//
// BOOLEAN
// NTAPI
// ExpWindowStationOkToClose( IN PEPROCESS Process OPTIONAL,
//                      IN PVOID Object,
//                      IN HANDLE Handle,
//                      IN KPROCESSOR_MODE AccessMode)
// {
//     WIN32_OKAYTOCLOSEMETHOD_PARAMETERS Parameters;
//     NTSTATUS Status;
//
//     Parameters.Process = Process;
//     Parameters.Object = Object;
//     Parameters.Handle = Handle;
//     Parameters.PreviousMode = AccessMode;
//
//     Status = ExpWin32SessionCallout(Object,
//                                     ExpWindowStationObjectOkToClose,
//                                     &Parameters);
//
//     return NT_SUCCESS(Status);
// }
//
// VOID
// NTAPI
// ExpWinStaObjectDelete(PVOID DeletedObject)
// {
//     WIN32_DELETEMETHOD_PARAMETERS Parameters;
//
//     /* Fill out the callback structure */
//     Parameters.Object = DeletedObject;
//
//     ExpWin32SessionCallout(DeletedObject,
//                            ExpWindowStationObjectDelete,
//                            &Parameters);
// }
//
// NTSTATUS
// NTAPI
// ExpWinStaObjectParse(IN PVOID ParseObject,
//                      IN PVOID ObjectType,
//                      IN OUT PACCESS_STATE AccessState,
//                      IN KPROCESSOR_MODE AccessMode,
//                      IN ULONG Attributes,
//                      IN OUT PUNICODE_STRING CompleteName,
//                      IN OUT PUNICODE_STRING RemainingName,
//                      IN OUT PVOID Context OPTIONAL,
//                      IN PSECURITY_QUALITY_OF_SERVICE SecurityQos OPTIONAL,
//                      OUT PVOID *Object)
// {
//     WIN32_PARSEMETHOD_PARAMETERS Parameters;
//
//     /* Fill out the callback structure */
//     Parameters.ParseObject = ParseObject;
//     Parameters.ObjectType = ObjectType;
//     Parameters.AccessState = AccessState;
//     Parameters.AccessMode = AccessMode;
//     Parameters.Attributes = Attributes;
//     Parameters.CompleteName = CompleteName;
//     Parameters.RemainingName = RemainingName;
//     Parameters.Context = Context;
//     Parameters.SecurityQos = SecurityQos;
//     Parameters.Object = Object;
//
//     return ExpWin32SessionCallout(ParseObject,
//                                   ExpWindowStationObjectParse,
//                                   &Parameters);
// }
// VOID
// NTAPI
// ExpDesktopDelete(PVOID DeletedObject)
// {
//     WIN32_DELETEMETHOD_PARAMETERS Parameters;
//
//     /* Fill out the callback structure */
//     Parameters.Object = DeletedObject;
//
//     ExpWin32SessionCallout(DeletedObject,
//                            ExpDesktopObjectDelete,
//                            &Parameters);
// }
//
// NTSTATUS
// NTAPI
// ExpDesktopOpen(IN OB_OPEN_REASON Reason,
//                IN PEPROCESS Process OPTIONAL,
//                IN PVOID ObjectBody,
//                IN ACCESS_MASK GrantedAccess,
//                IN ULONG HandleCount)
// {
//     WIN32_OPENMETHOD_PARAMETERS Parameters;
//
//     Parameters.OpenReason = Reason;
//     Parameters.Process = Process;
//     Parameters.Object = ObjectBody;
//     Parameters.GrantedAccess = GrantedAccess;
//     Parameters.HandleCount = HandleCount;
//
//     return ExpWin32SessionCallout(ObjectBody,
//                                   ExpDesktopObjectOpen,
//                                   &Parameters);
// }
//
// VOID
// NTAPI
// ExpDesktopClose(IN PEPROCESS Process OPTIONAL,
//                 IN PVOID Object,
//                 IN ACCESS_MASK GrantedAccess,
//                 IN ULONG ProcessHandleCount,
//                 IN ULONG SystemHandleCount)
// {
//     WIN32_CLOSEMETHOD_PARAMETERS Parameters;
//
//     Parameters.Process = Process;
//     Parameters.Object = Object;
//     Parameters.AccessMask = GrantedAccess;
//     Parameters.ProcessHandleCount = ProcessHandleCount;
//     Parameters.SystemHandleCount = SystemHandleCount;
//
//     ExpWin32SessionCallout(Object,
//                            ExpDesktopObjectClose,
//                            &Parameters);
// }
//
// CODE_SEG("INIT")
// BOOLEAN
// NTAPI
// ExpWin32kInit(VOID)
// {
//     OBJECT_TYPE_INITIALIZER ObjectTypeInitializer;
//     _UNICODE_STRING Name;
//     NTSTATUS Status;
//     DPRINT("Creating Win32 Object Types\n");
//
//     /* Create the window station Object Type */
//     RtlZeroMemory(&ObjectTypeInitializer, sizeof(ObjectTypeInitializer));
//     RtlInitUnicodeString(&Name, L"WindowStation");
//     ObjectTypeInitializer.Length = sizeof(ObjectTypeInitializer);
//     ObjectTypeInitializer.GenericMapping = ExpWindowStationMapping;
//     ObjectTypeInitializer.PoolType = NonPagedPool;
//     ObjectTypeInitializer.DeleteProcedure = ExpWinStaObjectDelete;
//     ObjectTypeInitializer.ParseProcedure = ExpWinStaObjectParse;
//     ObjectTypeInitializer.OkayToCloseProcedure = ExpWindowStationOkToClose;
//     ObjectTypeInitializer.SecurityRequired = TRUE;
//     ObjectTypeInitializer.InvalidAttributes = OBJ_OPENLINK |
//                                               OBJ_PERMANENT |
//                                               OBJ_EXCLUSIVE;
//     ObjectTypeInitializer.ValidAccessMask = STANDARD_RIGHTS_REQUIRED;
//     Status = ObCreateObjectType(&Name,
//                                 &ObjectTypeInitializer,
//                                 NULL,
//                                 &ExWindowStationObjectType);
//     if (!NT_SUCCESS(Status)) return FALSE;
//
//     /* Create desktop object type */
//     RtlInitUnicodeString(&Name, L"Desktop");
//     ObjectTypeInitializer.GenericMapping = ExpDesktopMapping;
//     ObjectTypeInitializer.DeleteProcedure = ExpDesktopDelete;
//     ObjectTypeInitializer.ParseProcedure = NULL;
//     ObjectTypeInitializer.OkayToCloseProcedure = ExpDesktopOkToClose;
//     ObjectTypeInitializer.OpenProcedure = ExpDesktopOpen;
//     ObjectTypeInitializer.CloseProcedure = ExpDesktopClose;
//     Status = ObCreateObjectType(&Name,
//                                 &ObjectTypeInitializer,
//                                 NULL,
//                                 &ExDesktopObjectType);
//     if (!NT_SUCCESS(Status)) return FALSE;
//
//     return TRUE;
// }
//
// /* EOF */
//
// ============================================================================
// END ORIGINAL C SOURCE
// ============================================================================
