use crate::sc::usage::ContinueUsage;
use crate::sc::control::Control;
use crate::sc::usage::ControlUsage;
use crate::sc::create::Create;
use crate::sc::delete::Delete;
use crate::sc::usage::DeleteUsage;
use crate::sc::depend::EnumDepend;
use crate::sc::usage::EnumDependUsage;
use crate::sc::name::GetDisplayName;
use crate::sc::usage::GetDisplayNameUsage;
use crate::sc::name::GetKeyName;
use crate::sc::usage::GetKeyNameUsage;
use crate::sc::usage::InterrogateUsage;
use crate::sc::usage::MainUsage;
use crate::sc::usage::PauseUsage;
use crate::sc::query::Query;
use crate::sc::config::QueryConfig;
use crate::sc::usage::QueryConfigUsage;
use crate::sc::description::QueryDescription;
use crate::sc::usage::QueryDescriptionUsage;
use crate::sc::failure::QueryFailure;
use crate::sc::usage::QueryFailureUsage;
use crate::sc::sdset::SdSet;
use crate::sc::usage::SdSetUsage;
use crate::sc::sdshow::SdShow;
use crate::sc::usage::SdShowUsage;
use crate::sc::config::SetConfig;
use crate::sc::description::SetDescription;
use crate::sc::usage::SetDescriptionUsage;
use crate::sc::failure::SetFailure;
use crate::sc::start::Start;
use crate::sc::usage::StartUsage;
use crate::sc::usage::StopUsage;
use crate::prelude::*;
use core::ffi::c_void;

// ---- file-scope preamble (translated) ----
// @c2rust file-scope data (compiler-exact tables/statics)
#[no_mangle]
pub static mut hSCManager: SC_HANDLE = ::core::ptr::null::<SC_HANDLE__>() as *mut SC_HANDLE__;
#[no_mangle]
pub unsafe extern "C" fn ReportLastError() {
    let mut lpMsgBuf: LPVOID = core::ptr::null_mut();
    let mut RetVal: DWORD;

    let ErrorCode: DWORD = GetLastError();
    if ErrorCode != ERROR_SUCCESS {
        RetVal = FormatMessage(
            FORMAT_MESSAGE_ALLOCATE_BUFFER |
            FORMAT_MESSAGE_FROM_SYSTEM |
            FORMAT_MESSAGE_IGNORE_INSERTS,
            core::ptr::null_mut(),
            ErrorCode,
            (MAKELANGID!(LANG_NEUTRAL, SUBLANG_DEFAULT)) as _, // Default language
            core::ptr::addr_of_mut!(lpMsgBuf) as LPTSTR,
            0,
            core::ptr::null_mut(),
        );

        if RetVal != 0 {
            static FMT: [u16; 3] = ['%' as u16, 's' as u16, 0];
            _tprintf((FMT.as_ptr()), lpMsgBuf as LPTSTR);
            LocalFree(lpMsgBuf);
        }
    }
}

unsafe extern "C" fn ScControl(
    mut Server: LPCTSTR,
    mut Command: LPCTSTR,
    mut ServiceArgs: *mut LPCTSTR,
    mut ArgCount: DWORD,
) -> INT {
    let mut ServiceName: LPCTSTR = ::core::ptr::null();

    if !Server.is_null() {
        _tprintf((['R' as u16, 'e' as u16, 'm' as u16, 'o' as u16, 't' as u16, 'e' as u16, ' ' as u16, 's' as u16, 'e' as u16, 'r' as u16, 'v' as u16, 'i' as u16, 'c' as u16, 'e' as u16, ' ' as u16, 'c' as u16, 'o' as u16, 'n' as u16, 't' as u16, 'r' as u16, 'o' as u16, 'l' as u16, ' ' as u16, 'i' as u16, 's' as u16, ' ' as u16, 'n' as u16, 'o' as u16, 't' as u16, ' ' as u16, 'y' as u16, 'e' as u16, 't' as u16, ' ' as u16, 'i' as u16, 'm' as u16, 'p' as u16, 'l' as u16, 'e' as u16, 'm' as u16, 'e' as u16, 'n' as u16, 't' as u16, 'e' as u16, 'd' as u16, '\n' as u16, 0].as_ptr()));
        return 2;
    }

    if lstrcmpi(Command, (['q' as u16, 'u' as u16, 'e' as u16, 'r' as u16, 'y' as u16, 0].as_ptr())) == 0 {
        Query(ServiceArgs, ArgCount, (FALSE) as BOOL);
    } else if lstrcmpi(Command, (['q' as u16, 'u' as u16, 'e' as u16, 'r' as u16, 'y' as u16, 'e' as u16, 'x' as u16, 0].as_ptr())) == 0 {
        Query(ServiceArgs, ArgCount, (TRUE) as BOOL);
    } else if lstrcmpi(Command, (['s' as u16, 't' as u16, 'a' as u16, 'r' as u16, 't' as u16, 0].as_ptr())) == 0 {
        if ArgCount > 0 {
            ServiceName = *ServiceArgs;
            ServiceArgs = ServiceArgs.add(1);
            ArgCount = ArgCount.wrapping_sub(1);

            Start(ServiceName, ServiceArgs, ArgCount as INT);
        } else {
            StartUsage();
        }
    } else if lstrcmpi(Command, (['p' as u16, 'a' as u16, 'u' as u16, 's' as u16, 'e' as u16, 0].as_ptr())) == 0 {
        if ArgCount > 0 {
            ServiceName = *ServiceArgs;
            ServiceArgs = ServiceArgs.add(1);
            ArgCount = ArgCount.wrapping_sub(1);

            Control(SERVICE_CONTROL_PAUSE, ServiceName, ServiceArgs, ArgCount as INT);
        } else {
            PauseUsage();
        }
    } else if lstrcmpi(Command, (['i' as u16, 'n' as u16, 't' as u16, 'e' as u16, 'r' as u16, 'r' as u16, 'o' as u16, 'g' as u16, 'a' as u16, 't' as u16, 'e' as u16, 0].as_ptr())) == 0 {
        if ArgCount > 0 {
            ServiceName = *ServiceArgs;
            ServiceArgs = ServiceArgs.add(1);
            ArgCount = ArgCount.wrapping_sub(1);

            Control(SERVICE_CONTROL_INTERROGATE, ServiceName, ServiceArgs, ArgCount as INT);
        } else {
            InterrogateUsage();
        }
    } else if lstrcmpi(Command, (['s' as u16, 't' as u16, 'o' as u16, 'p' as u16, 0].as_ptr())) == 0 {
        if ArgCount > 0 {
            ServiceName = *ServiceArgs;
            ServiceArgs = ServiceArgs.add(1);
            ArgCount = ArgCount.wrapping_sub(1);

            Control(SERVICE_CONTROL_STOP, ServiceName, ServiceArgs, ArgCount as INT);
        } else {
            StopUsage();
        }
    } else if lstrcmpi(Command, (['c' as u16, 'o' as u16, 'n' as u16, 't' as u16, 'i' as u16, 'n' as u16, 'u' as u16, 'e' as u16, 0].as_ptr())) == 0 {
        if ArgCount > 0 {
            ServiceName = *ServiceArgs;
            ServiceArgs = ServiceArgs.add(1);
            ArgCount = ArgCount.wrapping_sub(1);

            Control(SERVICE_CONTROL_CONTINUE, ServiceName, ServiceArgs, ArgCount as INT);
        } else {
            ContinueUsage();
        }
    } else if lstrcmpi(Command, (['d' as u16, 'e' as u16, 'l' as u16, 'e' as u16, 't' as u16, 'e' as u16, 0].as_ptr())) == 0 {
        if ArgCount > 0 {
            ServiceName = *ServiceArgs;
            ServiceArgs = ServiceArgs.add(1);
            ArgCount = ArgCount.wrapping_sub(1);

            Delete(ServiceName);
        } else {
            DeleteUsage();
        }
    } else if lstrcmpi(Command, (['c' as u16, 'r' as u16, 'e' as u16, 'a' as u16, 't' as u16, 'e' as u16, 0].as_ptr())) == 0 {
        Create(ServiceArgs, ArgCount as INT);
    } else if lstrcmpi(Command, (['c' as u16, 'o' as u16, 'n' as u16, 't' as u16, 'r' as u16, 'o' as u16, 'l' as u16, 0].as_ptr())) == 0 {
        let mut ControlCode: INT = 0;

        if ArgCount > 1 {
            ServiceName = *ServiceArgs;
            ServiceArgs = ServiceArgs.add(1);
            ArgCount = ArgCount.wrapping_sub(1);

            if lstrcmpi(*ServiceArgs.offset(0 as isize), (['p' as u16, 'a' as u16, 'r' as u16, 'a' as u16, 'm' as u16, 'c' as u16, 'h' as u16, 'a' as u16, 'n' as u16, 'g' as u16, 'e' as u16, 0].as_ptr())) == 0 {
                ControlCode = (SERVICE_CONTROL_PARAMCHANGE as i32);
            } else if lstrcmpi(*ServiceArgs.offset(0 as isize), (['n' as u16, 'e' as u16, 't' as u16, 'b' as u16, 'i' as u16, 'n' as u16, 'd' as u16, 'a' as u16, 'd' as u16, 'd' as u16, 0].as_ptr())) == 0 {
                ControlCode = (SERVICE_CONTROL_NETBINDADD as i32);
            } else if lstrcmpi(*ServiceArgs.offset(0 as isize), (['n' as u16, 'e' as u16, 't' as u16, 'b' as u16, 'i' as u16, 'n' as u16, 'd' as u16, 'r' as u16, 'e' as u16, 'm' as u16, 'o' as u16, 'v' as u16, 'e' as u16, 0].as_ptr())) == 0 {
                ControlCode = (SERVICE_CONTROL_NETBINDREMOVE as i32);
            } else if lstrcmpi(*ServiceArgs.offset(0 as isize), (['n' as u16, 'e' as u16, 't' as u16, 'b' as u16, 'i' as u16, 'n' as u16, 'd' as u16, 'e' as u16, 'n' as u16, 'a' as u16, 'b' as u16, 'l' as u16, 'e' as u16, 0].as_ptr())) == 0 {
                ControlCode = (SERVICE_CONTROL_NETBINDENABLE as i32);
            } else if lstrcmpi(*ServiceArgs.offset(0 as isize), (['n' as u16, 'e' as u16, 't' as u16, 'b' as u16, 'i' as u16, 'n' as u16, 'd' as u16, 'd' as u16, 'i' as u16, 's' as u16, 'a' as u16, 'b' as u16, 'l' as u16, 'e' as u16, 0].as_ptr())) == 0 {
                ControlCode = (SERVICE_CONTROL_NETBINDDISABLE as i32);
            } else {
                ControlCode = _ttoi(*ServiceArgs.offset(0 as isize)) as INT;
                if (ControlCode < 128) || (ControlCode > 255) {
                    ControlCode = 0;
                }
            }

            ServiceArgs = ServiceArgs.add(1);
            ArgCount = ArgCount.wrapping_sub(1);

            if ControlCode != 0 {
                Control((ControlCode) as DWORD, ServiceName, ServiceArgs, ArgCount as INT);
            } else {
                ControlUsage();
            }
        } else {
            ControlUsage();
        }
    } else if lstrcmpi(Command, (['s' as u16, 'd' as u16, 's' as u16, 'h' as u16, 'o' as u16, 'w' as u16, 0].as_ptr())) == 0 {
        if ArgCount > 0 {
            ServiceName = *ServiceArgs;
            ServiceArgs = ServiceArgs.add(1);
            ArgCount = ArgCount.wrapping_sub(1);

            SdShow(ServiceName);
        } else {
            SdShowUsage();
        }
    } else if lstrcmpi(Command, (['s' as u16, 'd' as u16, 's' as u16, 'e' as u16, 't' as u16, 0].as_ptr())) == 0 {
        let mut SecurityDescriptor: LPCTSTR = ::core::ptr::null();

        if ArgCount > 1 {
            ServiceName = *ServiceArgs;
            ServiceArgs = ServiceArgs.add(1);
            ArgCount = ArgCount.wrapping_sub(1);

            SecurityDescriptor = *ServiceArgs;
            ServiceArgs = ServiceArgs.add(1);
            ArgCount = ArgCount.wrapping_sub(1);

            SdSet(ServiceName, SecurityDescriptor);
        } else {
            SdSetUsage();
        }
    } else if lstrcmpi(Command, (['q' as u16, 'c' as u16, 0].as_ptr())) == 0 {
        if ArgCount > 0 {
            ServiceName = *ServiceArgs;
            ServiceArgs = ServiceArgs.add(1);
            ArgCount = ArgCount.wrapping_sub(1);

            QueryConfig(ServiceName);
        } else {
            QueryConfigUsage();
        }
    } else if lstrcmpi(Command, (['q' as u16, 'd' as u16, 'e' as u16, 's' as u16, 'c' as u16, 'r' as u16, 'i' as u16, 'p' as u16, 't' as u16, 'i' as u16, 'o' as u16, 'n' as u16, 0].as_ptr())) == 0 {
        if ArgCount > 0 {
            ServiceName = *ServiceArgs;
            ServiceArgs = ServiceArgs.add(1);
            ArgCount = ArgCount.wrapping_sub(1);

            QueryDescription(ServiceName);
        } else {
            QueryDescriptionUsage();
        }
    } else if lstrcmpi(Command, (['q' as u16, 'f' as u16, 'a' as u16, 'i' as u16, 'l' as u16, 'u' as u16, 'r' as u16, 'e' as u16, 0].as_ptr())) == 0 {
        if ArgCount > 0 {
            ServiceName = *ServiceArgs;
            ServiceArgs = ServiceArgs.add(1);
            ArgCount = ArgCount.wrapping_sub(1);

            QueryFailure(ServiceName);
        } else {
            QueryFailureUsage();
        }
    } else if lstrcmpi(Command, (['d' as u16, 'e' as u16, 's' as u16, 'c' as u16, 'r' as u16, 'i' as u16, 'p' as u16, 't' as u16, 'i' as u16, 'o' as u16, 'n' as u16, 0].as_ptr())) == 0 {
        let mut Description: LPCTSTR = ::core::ptr::null();

        if ArgCount > 0 {
            ServiceName = *ServiceArgs;
            ServiceArgs = ServiceArgs.add(1);
            ArgCount = ArgCount.wrapping_sub(1);

            if ArgCount > 0 {
                Description = *ServiceArgs;
                ServiceArgs = ServiceArgs.add(1);
                ArgCount = ArgCount.wrapping_sub(1);
            }

            SetDescription(ServiceName, Description);
        } else {
            SetDescriptionUsage();
        }
    } else if lstrcmpi(Command, (['c' as u16, 'o' as u16, 'n' as u16, 'f' as u16, 'i' as u16, 'g' as u16, 0].as_ptr())) == 0 {
        SetConfig(ServiceArgs, ArgCount as INT);
    } else if lstrcmpi(Command, (['f' as u16, 'a' as u16, 'i' as u16, 'l' as u16, 'u' as u16, 'r' as u16, 'e' as u16, 0].as_ptr())) == 0 {
        SetFailure(ServiceArgs, ArgCount as INT);
    } else if lstrcmpi(Command, (['G' as u16, 'e' as u16, 't' as u16, 'D' as u16, 'i' as u16, 's' as u16, 'p' as u16, 'l' as u16, 'a' as u16, 'y' as u16, 'N' as u16, 'a' as u16, 'm' as u16, 'e' as u16, 0].as_ptr())) == 0 {
        if ArgCount > 0 {
            ServiceName = *ServiceArgs;
            ServiceArgs = ServiceArgs.add(1);
            ArgCount = ArgCount.wrapping_sub(1);

            GetDisplayName(ServiceName);
        } else {
            GetDisplayNameUsage();
        }
    } else if lstrcmpi(Command, (['G' as u16, 'e' as u16, 't' as u16, 'K' as u16, 'e' as u16, 'y' as u16, 'N' as u16, 'a' as u16, 'm' as u16, 'e' as u16, 0].as_ptr())) == 0 {
        if ArgCount > 0 {
            ServiceName = *ServiceArgs;
            ServiceArgs = ServiceArgs.add(1);
            ArgCount = ArgCount.wrapping_sub(1);

            GetKeyName(ServiceName);
        } else {
            GetKeyNameUsage();
        }
    } else if lstrcmpi(Command, (['E' as u16, 'n' as u16, 'u' as u16, 'm' as u16, 'D' as u16, 'e' as u16, 'p' as u16, 'e' as u16, 'n' as u16, 'd' as u16, 0].as_ptr())) == 0 {
        if ArgCount > 0 {
            ServiceName = *ServiceArgs;
            ServiceArgs = ServiceArgs.add(1);
            ArgCount = ArgCount.wrapping_sub(1);

            EnumDepend(ServiceName);
        } else {
            EnumDependUsage();
        }
    } else {
        MainUsage();
    }

    return 0;
}

#[export_name = "wmain"]
pub unsafe extern "C" fn _tmain(argc: ::core::ffi::c_int, argv: *mut LPCTSTR) -> ::core::ffi::c_int {
    let mut Server: LPCTSTR = core::ptr::null();
    let mut Command: LPCTSTR = core::ptr::null();
    let mut Args: *mut LPCTSTR = core::ptr::null_mut();

    if argc < 2 {
        MainUsage();
        return -1;
    }

    /* get server name */
    if (*(*argv.offset(1 as isize)).offset(0 as isize) == '\\' as u16)
        && (*(*argv.offset(1 as isize)).offset(1 as isize) == '\\' as u16)
    {
        if argc < 3 {
            MainUsage();
            return -1;
        }

        Server = *argv.offset(1 as isize);
        Command = *argv.offset(2 as isize);
        if argc > 3 {
            Args = argv.offset(3 as isize);
        }

        return ScControl(
            Server,
            Command,
            Args,
            (argc - 3) as DWORD,
        );
    } else {
        Command = *argv.offset(1 as isize);
        if argc > 2 {
            Args = argv.offset(2 as isize);
        }

        return ScControl(
            Server,
            Command,
            Args,
            (argc - 2) as DWORD,
        );
    }
}
