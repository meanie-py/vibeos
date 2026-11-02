use crate::prelude::*;
use core::ffi::c_void;

#[no_mangle]
pub unsafe extern "system" fn PsaEnumerateProcessModules(
    ProcessHandle: HANDLE,
    Callback: PPROCMOD_ENUM_ROUTINE,
    CallbackContext: PVOID,
) -> NTSTATUS {
    let mut Status: NTSTATUS = 0;
    'failure: {
        /* current process - use direct memory copy */
        /* FIXME - compare process id instead of a handle */
        if ProcessHandle == NtCurrentProcess() {
            let ListHead: PLIST_ENTRY;
            let mut Current: PLIST_ENTRY;

            ListHead = core::ptr::addr_of_mut!((*(*NtCurrentPeb()).Ldr).InLoadOrderModuleList);
            Current = (*ListHead).Flink;

            while Current != ListHead {
                let LoaderModule: PLDR_DATA_TABLE_ENTRY =
                    CONTAINING_RECORD!(Current, LDR_DATA_TABLE_ENTRY, InLoadOrderLinks);

                /* return the current module to the callback */
                Status = (Callback.unwrap_unchecked())(ProcessHandle, LoaderModule, CallbackContext);

                if !NT_SUCCESS!(Status) {
                    break 'failure;
                }

                Current = (*LoaderModule).InLoadOrderLinks.Flink;
            }
        } else {
            let mut BasicInformation: PROCESS_BASIC_INFORMATION =
                core::mem::zeroed::<PROCESS_BASIC_INFORMATION>();
            let mut LoaderData: PPEB_LDR_DATA = core::ptr::null_mut();
            let mut LoaderModule: LDR_DATA_TABLE_ENTRY =
                core::mem::zeroed::<LDR_DATA_TABLE_ENTRY>();
            let ListHead: PLIST_ENTRY;
            let mut Current: PLIST_ENTRY = core::ptr::null_mut();

            /* query the process basic information (includes the PEB address) */
            Status = NtQueryInformationProcess(
                ProcessHandle,
                ProcessBasicInformation,
                core::ptr::addr_of_mut!(BasicInformation) as PVOID,
                core::mem::size_of::<PROCESS_BASIC_INFORMATION>() as ULONG,
                core::ptr::null_mut(),
            );

            if !NT_SUCCESS!(Status) {
                DPRINT!(FAILED_WITH_STATUS, "NtQueryInformationProcess", Status);
                break 'failure;
            }

            /* get the address of the PE Loader data */
            Status = NtReadVirtualMemory(
                ProcessHandle,
                core::ptr::addr_of_mut!((*BasicInformation.PebBaseAddress).Ldr) as PVOID,
                core::ptr::addr_of_mut!(LoaderData) as PVOID,
                core::mem::size_of::<_PEB_LDR_DATA>() as SIZE_T,
                core::ptr::null_mut(),
            );

            if !NT_SUCCESS!(Status) {
                DPRINT!(FAILED_WITH_STATUS, "NtReadVirtualMemory", Status);
                break 'failure;
            }

            /* head of the module list: the last element in the list will point to this */
            ListHead = core::ptr::addr_of_mut!((*LoaderData).InLoadOrderModuleList);

            /* get the address of the first element in the list */
            Status = NtReadVirtualMemory(
                ProcessHandle,
                core::ptr::addr_of_mut!((*LoaderData).InLoadOrderModuleList.Flink) as PVOID,
                core::ptr::addr_of_mut!(Current) as PVOID,
                core::mem::size_of::<_LIST_ENTRY>() as SIZE_T,
                core::ptr::null_mut(),
            );

            while Current != ListHead {
                /* read the current module */
                Status = NtReadVirtualMemory(
                    ProcessHandle,
                    CONTAINING_RECORD!(Current, LDR_DATA_TABLE_ENTRY, InLoadOrderLinks) as PVOID,
                    core::ptr::addr_of_mut!(LoaderModule) as PVOID,
                    core::mem::size_of::<LDR_DATA_TABLE_ENTRY>() as SIZE_T,
                    core::ptr::null_mut(),
                );

                if !NT_SUCCESS!(Status) {
                    DPRINT!(FAILED_WITH_STATUS, "NtReadVirtualMemory", Status);
                    break 'failure;
                }

                /* return the current module to the callback */
                Status = (Callback.unwrap_unchecked())(
                    ProcessHandle,
                    core::ptr::addr_of_mut!(LoaderModule),
                    CallbackContext,
                );

                if !NT_SUCCESS!(Status) {
                    break 'failure;
                }

                /* address of the next module in the list */
                Current = LoaderModule.InLoadOrderLinks.Flink;
            }
        }

        return STATUS_SUCCESS;
    }

    /* Failure: */
    return Status;
}
