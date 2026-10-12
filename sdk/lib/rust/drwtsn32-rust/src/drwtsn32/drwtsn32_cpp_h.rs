use crate::prelude::*;
// Deterministic UTF-16 literals from the selected C++ helper closure.
pub static CPP_COM_WSTR_9A6EFE4D86A5: [WCHAR; 3] = [0x005C, 0x002F, 0x0000];
pub static CPP_COM_WSTR_D440467AE734: [WCHAR; 3] = [0x003F, 0x003F, 0x0000];

use core::ffi::c_void;





#[no_mangle]
pub unsafe extern "C" fn UpdateFromEvent(evt: &mut DEBUG_EVENT, data: &mut DumpData) -> bool {
    match evt.dwDebugEventCode {
        CREATE_PROCESS_DEBUG_EVENT => {
            data.ProcessPath.resize((MAX_PATH * 2) as _);
            let len: DWORD = GetModuleFileNameExW(
                evt.u.CreateProcessInfo.hProcess,
                core::ptr::null_mut(),
                data.ProcessPath.at(0),
                data.ProcessPath.size() as DWORD,
            );
            if len != 0 {
                data.ProcessPath.resize(len as usize);
                let pos: usize = data.ProcessPath.find_last_of(CPP_COM_WSTR_9A6EFE4D86A5.as_ptr());
                if pos != npos {
                    data.ProcessName = data.ProcessPath.substr(pos + 1, npos);
                }
            } else {
                data.ProcessPath.assign(CPP_COM_WSTR_D440467AE734.as_ptr());
            }
            if data.ProcessName.empty() {
                data.ProcessName.assign(data.ProcessPath.c_str());
            }

            CloseHandle(evt.u.CreateProcessInfo.hFile);
            data.ProcessID = evt.dwProcessId;
            data.ProcessHandle = evt.u.CreateProcessInfo.hProcess;
            data.Threads.insert_or_assign(
                evt.dwThreadId,
                ThreadData__make_0(evt.u.CreateProcessInfo.hThread),
            );
        }
        CREATE_THREAD_DEBUG_EVENT => {
            data.Threads.insert_or_assign(
                evt.dwThreadId,
                ThreadData__make_0(evt.u.CreateThread.hThread),
            );
        }
        EXIT_THREAD_DEBUG_EVENT => {
            let it = data.Threads.find(&evt.dwThreadId);
            if !it.is_null() {
                data.Threads.erase(&evt.dwThreadId);
            }
        }
        LOAD_DLL_DEBUG_EVENT => {
            CloseHandle(evt.u.LoadDll.hFile);
            let mut n: usize = 0;
            while n < data.Modules.size() {
                if (*data.Modules.at(n)).BaseAddress == evt.u.LoadDll.lpBaseOfDll {
                    (*data.Modules.at(n)).Unloaded = false;
                    return true;
                }
                n += 1;
            }
            data.Modules.push_back(ModuleData__make_0(evt.u.LoadDll.lpBaseOfDll));
        }
        UNLOAD_DLL_DEBUG_EVENT => {
            let mut n: usize = 0;
            while n < data.Modules.size() {
                if (*data.Modules.at(n)).BaseAddress == evt.u.UnloadDll.lpBaseOfDll {
                    (*data.Modules.at(n)).Unloaded = true;
                }
                n += 1;
            }
        }
        OUTPUT_DEBUG_STRING_EVENT => {
            // ignore
        }
        EXCEPTION_DEBUG_EVENT => {
            if evt.u.Exception.dwFirstChance != 0 {
                match evt.u.Exception.ExceptionRecord.ExceptionCode as NTSTATUS {
                    EXCEPTION_BREAKPOINT => {
                        if !data.FirstBPHit {
                            data.FirstBPHit = true;

                            if !data.Event.is_null() {
                                SetEvent(data.Event);
                                CloseHandle(data.Event);
                                data.Event = core::ptr::null_mut();
                            }
                            return true;
                        }
                    }
                    MS_VC_EXCEPTION_THREAD_NAME => {
                        /* Thread name */
                        return true;
                    }
                    DBG_CONTROL_C | DBG_CONTROL_BREAK => {
                        return true;
                    }
                    _ => {}
                }
            }
            data.ExceptionInfo = evt.u.Exception;
            data.ThreadID = evt.dwThreadId;
            return false;
        }
        EXIT_PROCESS_DEBUG_EVENT => {
            //assert(FALSE);
            return false;
        }
        RIP_EVENT => {
            //assert(FALSE);
            return false;
        }
        _ => {
            assert!(false);
        }
    }
    true
}

impl ModuleData {
    pub unsafe fn __ctor_0(&mut self, addr: *mut core::ffi::c_void) {
        let this_: *mut Self = self;
        {
            (*this_).BaseAddress = addr;
            (*this_).Size = 0;
            (*this_).Unloaded = false;
        }
    }
    pub unsafe fn Update(&mut self, hProcess: HANDLE) {
        let this_: *mut Self = self;
        let mut mi: MODULEINFO = core::mem::zeroed();
        GetModuleInformation(hProcess, (*this_).BaseAddress as HMODULE, core::ptr::addr_of_mut!(mi), core::mem::size_of::<MODULEINFO>() as DWORD);
        assert!((*this_).BaseAddress == mi.lpBaseOfDll);
        (*this_).Size = mi.SizeOfImage;
    
        (*this_).ModuleName.resize(MAX_PATH as usize);
        let dwLen: DWORD = GetModuleFileNameExA(hProcess, (*this_).BaseAddress as HMODULE, (*this_).ModuleName.at(0) as LPSTR, (*this_).ModuleName.size() as DWORD);
        (*this_).ModuleName.resize(dwLen as usize);
    }
}

impl ThreadData {
    pub unsafe fn __ctor_0(&mut self, handle: HANDLE) {
        let this_: *mut Self = self;
        (*this_).Handle = handle;
        {
            memset(core::ptr::addr_of_mut!((*this_).Context) as *mut core::ffi::c_void, 0, core::mem::size_of::<CONTEXT>());
        }
    }
    pub unsafe fn Update(&mut self) {
        let this_: *mut Self = self;
        (*this_).Context.ContextFlags = CONTEXT_INTEGER | CONTEXT_CONTROL | CONTEXT_DEBUG_REGISTERS;
        GetThreadContext((*this_).Handle, core::ptr::addr_of_mut!((*this_).Context));
    }
}

impl DumpData {
    pub unsafe fn __ctor_0(&mut self) {
        let this_: *mut Self = self;
        (*this_).ProcessID = 0;
        (*this_).ThreadID = 0;
        (*this_).ProcessHandle = core::ptr::null_mut();
        (*this_).Event = core::ptr::null_mut();
        (*this_).FirstBPHit = false;
        {
            memset(core::ptr::addr_of_mut!((*this_).ExceptionInfo) as *mut core::ffi::c_void, 0, core::mem::size_of_val(&(*this_).ExceptionInfo));
        }
    }
}
