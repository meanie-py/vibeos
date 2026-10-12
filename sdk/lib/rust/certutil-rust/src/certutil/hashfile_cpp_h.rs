use crate::prelude::*;
// Deterministic UTF-16 literals from the selected C++ helper closure.
pub static CPP_COM_WSTR_01CE4B899B50: [WCHAR; 46] = [0x0043, 0x0065, 0x0072, 0x0074, 0x0055, 0x0074, 0x0069, 0x006C, 0x003A, 0x0020, 0x002D, 0x0068, 0x0061, 0x0073, 0x0068, 0x0066, 0x0069, 0x006C, 0x0065, 0x0020, 0x0063, 0x006F, 0x006D, 0x006D, 0x0061, 0x006E, 0x0064, 0x0020, 0x006E, 0x006F, 0x0020, 0x0061, 0x006C, 0x0067, 0x006F, 0x0072, 0x0069, 0x0074, 0x0068, 0x006D, 0x003A, 0x0020, 0x0025, 0x0064, 0x000A, 0x0000];
pub static CPP_COM_WSTR_075DE2B906DB: [WCHAR; 2] = [0x000A, 0x0000];
pub static CPP_COM_WSTR_85FAA0DB7345: [WCHAR; 18] = [0x0053, 0x0048, 0x0041, 0x0031, 0x0020, 0x0068, 0x0061, 0x0073, 0x0068, 0x0020, 0x006F, 0x0066, 0x0020, 0x0025, 0x0073, 0x003A, 0x000A, 0x0000];
pub static CPP_COM_WSTR_87F7957388DB: [WCHAR; 48] = [0x0043, 0x0065, 0x0072, 0x0074, 0x0055, 0x0074, 0x0069, 0x006C, 0x003A, 0x0020, 0x002D, 0x0068, 0x0061, 0x0073, 0x0068, 0x0066, 0x0069, 0x006C, 0x0065, 0x0020, 0x0063, 0x006F, 0x006D, 0x006D, 0x0061, 0x006E, 0x0064, 0x0020, 0x0066, 0x0061, 0x0069, 0x006C, 0x0065, 0x0064, 0x0020, 0x0074, 0x006F, 0x0020, 0x0068, 0x0061, 0x0073, 0x0068, 0x003A, 0x0020, 0x0025, 0x0064, 0x000A, 0x0000];
pub static CPP_COM_WSTR_AA2A982F0618: [WCHAR; 5] = [0x0025, 0x0030, 0x0032, 0x0078, 0x0000];
pub static CPP_COM_WSTR_AA41D3896C89: [WCHAR; 44] = [0x0043, 0x0065, 0x0072, 0x0074, 0x0055, 0x0074, 0x0069, 0x006C, 0x003A, 0x0020, 0x002D, 0x0068, 0x0061, 0x0073, 0x0068, 0x0066, 0x0069, 0x006C, 0x0065, 0x0020, 0x0063, 0x006F, 0x006D, 0x006D, 0x0061, 0x006E, 0x0064, 0x0020, 0x006E, 0x006F, 0x0020, 0x0063, 0x006F, 0x006E, 0x0074, 0x0065, 0x0078, 0x0074, 0x003A, 0x0020, 0x0025, 0x0064, 0x000A, 0x0000];
pub static CPP_COM_WSTR_AEB8A888AB9E: [WCHAR; 56] = [0x0043, 0x0065, 0x0072, 0x0074, 0x0055, 0x0074, 0x0069, 0x006C, 0x003A, 0x0020, 0x002D, 0x0068, 0x0061, 0x0073, 0x0068, 0x0066, 0x0069, 0x006C, 0x0065, 0x0020, 0x0063, 0x006F, 0x006D, 0x006D, 0x0061, 0x006E, 0x0064, 0x0020, 0x0066, 0x0061, 0x0069, 0x006C, 0x0065, 0x0064, 0x0020, 0x0074, 0x006F, 0x0020, 0x0065, 0x0078, 0x0074, 0x0072, 0x0061, 0x0063, 0x0074, 0x0020, 0x0068, 0x0061, 0x0073, 0x0068, 0x003A, 0x0020, 0x0025, 0x0064, 0x000A, 0x0000];
pub static CPP_COM_WSTR_B3C3F647EE07: [WCHAR; 40] = [0x0043, 0x0065, 0x0072, 0x0074, 0x0055, 0x0074, 0x0069, 0x006C, 0x003A, 0x0020, 0x002D, 0x0068, 0x0061, 0x0073, 0x0068, 0x0066, 0x0069, 0x006C, 0x0065, 0x0020, 0x0063, 0x006F, 0x006D, 0x006D, 0x0061, 0x006E, 0x0064, 0x0020, 0x0066, 0x0061, 0x0069, 0x006C, 0x0065, 0x0064, 0x003A, 0x0020, 0x0025, 0x0064, 0x000A, 0x0000];

use core::ffi::c_void;
#[no_mangle]
pub unsafe extern "C" fn hash_file(Filename: LPCWSTR) -> BOOL {
    let mut hProv: HCRYPTPROV = core::mem::zeroed();
    let mut bSuccess: BOOL = (FALSE as i32);

    let hFile: HANDLE = CreateFileW(Filename, GENERIC_READ, FILE_SHARE_READ, core::ptr::null_mut(),
                               OPEN_EXISTING, FILE_FLAG_SEQUENTIAL_SCAN, core::ptr::null_mut());

    if hFile == INVALID_HANDLE_VALUE {
        ConPrintf(core::ptr::addr_of_mut!(csStdOut), CPP_COM_WSTR_B3C3F647EE07.as_ptr(), GetLastError());
        return bSuccess;
    }

    if CryptAcquireContextW(core::ptr::addr_of_mut!(hProv), core::ptr::null(), core::ptr::null(), PROV_RSA_FULL, CRYPT_VERIFYCONTEXT) != 0 {
        let mut hHash: HCRYPTHASH = core::mem::zeroed();

        if CryptCreateHash(hProv, CALG_SHA1, 0, 0, core::ptr::addr_of_mut!(hHash)) != 0 {
            let mut Buffer: [BYTE; 2048] = [0; 2048];
            let mut cbRead: DWORD = 0;

            loop {
                bSuccess = ReadFile(hFile, Buffer.as_mut_ptr() as LPVOID, core::mem::size_of::<[BYTE; 2048]>() as DWORD,
                    core::ptr::addr_of_mut!(cbRead), core::ptr::null_mut());
                if bSuccess == 0 {
                    break;
                }

                if cbRead == 0 {
                    break;
                }

                if CryptHashData(hHash, Buffer.as_mut_ptr(), cbRead, 0) == 0 {
                    bSuccess = (FALSE as i32);
                    ConPrintf(core::ptr::addr_of_mut!(csStdOut), CPP_COM_WSTR_87F7957388DB.as_ptr(), GetLastError());
                    break;
                }
            }

            if bSuccess != 0 {
                let mut rgbHash: [BYTE; 20] = [0; 20];
                let mut cbHash: DWORD = 0;
                let mut n: DWORD = 0;

                if CryptGetHashParam(hHash, HP_HASHVAL, rgbHash.as_mut_ptr(), core::ptr::addr_of_mut!(cbHash), 0) != 0 {
                    ConPrintf(core::ptr::addr_of_mut!(csStdOut), CPP_COM_WSTR_85FAA0DB7345.as_ptr(), Filename);
                    n = 0;
                    while n < cbHash {
                        ConPrintf(core::ptr::addr_of_mut!(csStdOut), CPP_COM_WSTR_AA2A982F0618.as_ptr(), rgbHash[n as usize] as core::ffi::c_uint);
                        n += 1;
                    }
                    ConPuts(core::ptr::addr_of_mut!(csStdOut), CPP_COM_WSTR_075DE2B906DB.as_ptr());
                }
                else {
                    ConPrintf(core::ptr::addr_of_mut!(csStdOut), CPP_COM_WSTR_AEB8A888AB9E.as_ptr(), GetLastError());
                    bSuccess = (FALSE as i32);
                }
            }

            CryptDestroyHash(hHash);
        }
        else {
            ConPrintf(core::ptr::addr_of_mut!(csStdOut), CPP_COM_WSTR_01CE4B899B50.as_ptr(), GetLastError());
        }

        CryptReleaseContext(hProv, 0);
    }
    else {
        ConPrintf(core::ptr::addr_of_mut!(csStdOut), CPP_COM_WSTR_AA41D3896C89.as_ptr(), GetLastError());
    }

    CloseHandle(hFile);
    return bSuccess;
}
