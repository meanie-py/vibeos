// rustconv-assembly: whole-tu
use crate::prelude::*;
use core::ffi::c_void;
#[no_mangle]
pub unsafe extern "C" fn InsSpacePos(szInsStr: PCWSTR, nPos: i32) -> PWSTR {
    let mut pszDestStr: PWSTR;
    let mut nDestStrCnt: i32 = 0;
    let mut nStrCnt: i32;
    let nStrSize: i32;

    pszDestStr = HeapAlloc(GetProcessHeap(), 0, (MAX_SAMPLES_STR_SIZE * (core::mem::size_of::<u16>()) as u32) as SIZE_T) as PWSTR;
    if pszDestStr == core::ptr::null_mut() {
        return core::ptr::null_mut();
    }

    wcscpy(pszDestStr, szInsStr);

    nStrSize = wcslen(szInsStr) as i32;

    nStrCnt = 0;
    while nStrCnt < nStrSize {
        if nStrCnt == nStrSize - nPos {
            *pszDestStr.offset(nDestStrCnt as isize) = ' ' as u16;
            nDestStrCnt += 1;
        }

        *pszDestStr.offset(nDestStrCnt as isize) = *szInsStr.offset(nStrCnt as isize);
        nDestStrCnt += 1;
        nStrCnt += 1;
    }

    *pszDestStr.offset(nDestStrCnt as isize) = 0;

    pszDestStr
}
#[no_mangle]
pub unsafe extern "C" fn InsSpacesFmt(szSourceStr: PCWSTR, szFmtStr: PCWSTR) -> PWSTR {
    let mut pszDestStr: PWSTR;
    let mut pszTempStr: PWSTR;
    let mut szFmtVal: [u16; 255] = [0; 255];
    let mut nFmtCount: u32 = 0;
    let mut nValCount: i32 = 0;
    let mut nLastVal: i32 = 0;
    let mut nSpaceOffset: i32 = 0;
    let mut wasNul: i32 = (FALSE) as i32;

    pszDestStr = HeapAlloc(GetProcessHeap(), 0, (255 * core::mem::size_of::<u16>()) as SIZE_T) as PWSTR;
    if pszDestStr == core::ptr::null_mut() {
        return core::ptr::null_mut();
    }

    wcscpy(pszDestStr, szSourceStr);

    if *szFmtStr == 0 {
        return pszDestStr;
    }

    nFmtCount = 0;
    while nFmtCount <= wcslen(szFmtStr) as u32 {
        if *szFmtStr.offset(nFmtCount as isize) == 0x003B || *szFmtStr.offset(nFmtCount as isize) == 0x0000 {
            if _wtoi(szFmtVal.as_ptr()) == 0 && wasNul == 0 {
                wasNul = (TRUE) as i32;
                break;
            }

            if wasNul != 0 {
                nSpaceOffset += nLastVal;
            } else {
                nSpaceOffset += _wtoi(szFmtVal.as_ptr());
            }

            szFmtVal[nValCount as usize] = 0x0000;
            nValCount = 0;

            pszTempStr = InsSpacePos(pszDestStr, nSpaceOffset);
            wcscpy(pszDestStr, pszTempStr);
            HeapFree(GetProcessHeap(), 0, (pszTempStr).cast());

            if wasNul == 0 {
                nSpaceOffset += 1;
                nLastVal = _wtoi(szFmtVal.as_ptr());
            }
        } else {
            szFmtVal[nValCount as usize] = *szFmtStr.offset(nFmtCount as isize);
            nValCount += 1;
        }
        nFmtCount += 1;
    }

    if wasNul != 0 && nLastVal != 0 {
        nFmtCount = (nSpaceOffset + nLastVal) as u32;
        while (nFmtCount as usize) < wcslen(pszDestStr) as usize {
            pszTempStr = InsSpacePos(pszDestStr, nFmtCount as i32);
            wcscpy(pszDestStr, pszTempStr);
            HeapFree(GetProcessHeap(), 0, (pszTempStr).cast());
            nFmtCount = (nFmtCount as i32 + nLastVal + 1) as u32;
        }
    }

    pszDestStr
}
#[no_mangle]
pub unsafe extern "C" fn ReplaceSubStr(
    szSourceStr: PCWSTR,
    szStrToReplace: PCWSTR,
    szTempl: PCWSTR,
) -> PWSTR {
    let mut szDestStr: PWSTR;
    let mut nCharCnt: u32;
    let mut nSubStrCnt: u32;
    let mut nDestStrCnt: u32;
    let mut nFirstCharCnt: u32;

    szDestStr = HeapAlloc(GetProcessHeap(), 0, (MAX_SAMPLES_STR_SIZE * (core::mem::size_of::<u16>()) as u32) as SIZE_T) as PWSTR;
    if szDestStr == core::ptr::null_mut() {
        return core::ptr::null_mut();
    }

    nDestStrCnt = 0;
    nFirstCharCnt = 0;

    wcscpy(szDestStr, w!(""));

    while nFirstCharCnt < wcslen(szSourceStr) as u32 {
        if *szSourceStr.add(nFirstCharCnt as usize) == *szTempl.add(0 as usize) {
            nSubStrCnt = 0;
            nCharCnt = nFirstCharCnt;
            while nCharCnt < nFirstCharCnt + wcslen(szTempl) as u32 {
                if *szSourceStr.add(nCharCnt as usize) == *szTempl.add(nSubStrCnt as usize) {
                    nSubStrCnt += 1;
                } else {
                    break;
                }

                if wcslen(szTempl) as u32 == nSubStrCnt {
                    wcscat(szDestStr, szStrToReplace);
                    nDestStrCnt = wcslen(szDestStr) as u32;
                    nFirstCharCnt += wcslen(szTempl) as u32 - 1;
                    break;
                }
                nCharCnt += 1;
            }
        } else {
            *szDestStr.add(nDestStrCnt as usize) = *szSourceStr.add(nFirstCharCnt as usize);
            nDestStrCnt += 1;
            *szDestStr.add(nDestStrCnt as usize) = 0u16;
        }

        nFirstCharCnt += 1;
    }

    szDestStr
}
#[no_mangle]
pub unsafe extern "C" fn GetSelectedComboBoxText(
    hwndDlg: HWND,
    nIdDlgItem: INT,
    Buffer: PWSTR,
    uSize: UINT,
) {
    let hChildWnd: HWND;
    let tmp: PWSTR;
    let nIndex: INT;
    let uReqSize: UINT;

    hChildWnd = GetDlgItem(hwndDlg, nIdDlgItem);
    if hChildWnd == core::ptr::null_mut() {
        return;
    }

    nIndex = (SendMessageW(hChildWnd, CB_GETCURSEL, 0, 0)) as i32;
    if nIndex == CB_ERR {
        SendMessageW(hChildWnd, WM_GETTEXT, (uSize) as WPARAM, Buffer as LPARAM);
    } else {
        uReqSize = SendMessageW(hChildWnd, CB_GETLBTEXTLEN, nIndex as WPARAM, 0) as UINT + 1;

        tmp = HeapAlloc(GetProcessHeap(), HEAP_ZERO_MEMORY, (uReqSize * (core::mem::size_of::<u16>()) as u32) as SIZE_T) as PWSTR;
        if tmp != core::ptr::null_mut() {
            SendMessageW(hChildWnd, CB_GETLBTEXT, nIndex as WPARAM, tmp as LPARAM);

            wcsncpy(Buffer, tmp, (uSize) as usize);

            HeapFree(GetProcessHeap(), 0, tmp as *mut _);
        }
    }
}
#[no_mangle]
pub unsafe extern "C" fn GetSelectedComboBoxIndex(
    hwndDlg: HWND,
    nIdDlgItem: INT,
    pValue: PINT,
) {
    *pValue = (SendDlgItemMessageW(hwndDlg, nIdDlgItem, CB_GETCURSEL, 0, 0)) as i32;
}


// ============================================================================
// ORIGINAL C SOURCE
// ============================================================================
//
// #include "intl.h"
//
// /* Insert the space  */
// PWSTR
// InsSpacePos(PCWSTR szInsStr, const int nPos)
// {
//     PWSTR pszDestStr;
//     INT nDestStrCnt = 0;
//     INT nStrCnt;
//     INT nStrSize;
//
//     pszDestStr = (PWSTR)HeapAlloc(GetProcessHeap(), 0, MAX_SAMPLES_STR_SIZE * sizeof(WCHAR));
//     if (pszDestStr == NULL)
//         return NULL;
//
//     wcscpy(pszDestStr, szInsStr);
//
//     nStrSize = wcslen(szInsStr);
//
//     for (nStrCnt = 0; nStrCnt < nStrSize; nStrCnt++)
//     {
//         if (nStrCnt == nStrSize - nPos)
//         {
//             pszDestStr[nDestStrCnt] = L' ';
//             nDestStrCnt++;
//         }
//
//         pszDestStr[nDestStrCnt] = szInsStr[nStrCnt];
//         nDestStrCnt++;
//     }
//
//     pszDestStr[nDestStrCnt] = L'\0';
//
//     return pszDestStr;
// }
//
// /* Insert the spaces by format string separated by ';' */
// PWSTR
// InsSpacesFmt(PCWSTR szSourceStr, PCWSTR szFmtStr)
// {
//     PWSTR pszDestStr;
//     PWSTR pszTempStr;
//     WCHAR szFmtVal[255];
//     UINT nFmtCount = 0;
//     INT nValCount = 0;
//     INT nLastVal = 0;
//     INT nSpaceOffset = 0;
//     BOOL wasNul=FALSE;
//
//     pszDestStr = (PWSTR)HeapAlloc(GetProcessHeap(), 0, 255 * sizeof(WCHAR));
//     if (pszDestStr == NULL)
//         return NULL;
//
//     wcscpy(pszDestStr, szSourceStr);
//
//     /* If format is clean return source string */
//     if (!*szFmtStr)
//         return pszDestStr;
//
//     /* Search for all format values */
//     for (nFmtCount = 0; nFmtCount <= wcslen(szFmtStr); nFmtCount++)
//     {
//         if (szFmtStr[nFmtCount] == L';' || szFmtStr[nFmtCount] == L'\0')
//         {
//             if (_wtoi(szFmtVal) == 0 && !wasNul)
//             {
//                 wasNul=TRUE;
//                 break;
//             }
//
//             /* If was 0, repeat spaces */
//             if (wasNul)
//             {
//                 nSpaceOffset += nLastVal;
//             }
//             else
//             {
//                 nSpaceOffset += _wtoi(szFmtVal);
//             }
//
//             szFmtVal[nValCount] = L'\0';
//             nValCount=0;
//
//             /* Insert space to finded position plus all pos before */
//             pszTempStr = InsSpacePos(pszDestStr, nSpaceOffset);
//             wcscpy(pszDestStr,pszTempStr);
//             HeapFree(GetProcessHeap(), 0, pszTempStr);
//
//             /* Num of spaces total increment */
//             if (!wasNul)
//             {
//                 nSpaceOffset++;
//                 nLastVal = _wtoi(szFmtVal);
//             }
//         }
//         else
//         {
//             szFmtVal[nValCount++] = szFmtStr[nFmtCount];
//         }
//     }
//
//     /* Create spaces for rest part of string */
//     if (wasNul && nLastVal != 0)
//     {
//         for (nFmtCount = nSpaceOffset + nLastVal; nFmtCount < wcslen(pszDestStr); nFmtCount += nLastVal + 1)
//         {
//             pszTempStr = InsSpacePos(pszDestStr, nFmtCount);
//             wcscpy(pszDestStr, pszTempStr);
//             HeapFree(GetProcessHeap(), 0, pszTempStr);
//         }
//     }
//
//     return pszDestStr;
// }
//
// /* Replace given template in source string with string to replace and return received string */
// PWSTR
// ReplaceSubStr(PCWSTR szSourceStr,
//               PCWSTR szStrToReplace,
//               PCWSTR szTempl)
// {
//     PWSTR szDestStr;
//     UINT nCharCnt;
//     UINT nSubStrCnt;
//     UINT nDestStrCnt;
//     UINT nFirstCharCnt;
//
//     szDestStr = (PWSTR)HeapAlloc(GetProcessHeap(), 0, MAX_SAMPLES_STR_SIZE * sizeof(WCHAR));
//     if (szDestStr == NULL)
//         return NULL;
//
//     nDestStrCnt = 0;
//     nFirstCharCnt = 0;
//
//     wcscpy(szDestStr, L"");
//
//     while (nFirstCharCnt < wcslen(szSourceStr))
//     {
//         if (szSourceStr[nFirstCharCnt] == szTempl[0])
//         {
//             nSubStrCnt = 0;
//             for (nCharCnt = nFirstCharCnt; nCharCnt < nFirstCharCnt + wcslen(szTempl); nCharCnt++)
//             {
//                 if (szSourceStr[nCharCnt] == szTempl[nSubStrCnt])
//                 {
//                     nSubStrCnt++;
//                 }
//                 else
//                 {
//                     break;
//                 }
//
//                 if (wcslen(szTempl) == nSubStrCnt)
//                 {
//                     wcscat(szDestStr, szStrToReplace);
//                     nDestStrCnt = wcslen(szDestStr);
//                     nFirstCharCnt += wcslen(szTempl) - 1;
//                     break;
//                 }
//             }
//         }
//         else
//         {
//             szDestStr[nDestStrCnt++] = szSourceStr[nFirstCharCnt];
//             szDestStr[nDestStrCnt] = L'\0';
//         }
//
//         nFirstCharCnt++;
//     }
//
//     return szDestStr;
// }
//
//
// VOID
// GetSelectedComboBoxText(
//     HWND hwndDlg,
//     INT nIdDlgItem,
//     PWSTR Buffer,
//     UINT uSize)
// {
//     HWND hChildWnd;
//     PWSTR tmp;
//     INT nIndex;
//     UINT uReqSize;
//
//     /* Get handle to time format control */
//     hChildWnd = GetDlgItem(hwndDlg, nIdDlgItem);
//     if (hChildWnd == NULL)
//         return;
//
//     /* Get index to selected time format */
//     nIndex = SendMessageW(hChildWnd, CB_GETCURSEL, 0, 0);
//     if (nIndex == CB_ERR)
//     {
//         /* No selection? Get content of the edit control */
//         SendMessageW(hChildWnd, WM_GETTEXT, uSize, (LPARAM)Buffer);
//     }
//     else
//     {
//         /* Get requested size, including the null terminator;
//          * it shouldn't be required because the previous CB_LIMITTEXT,
//          * but it would be better to check it anyways */
//         uReqSize = SendMessageW(hChildWnd, CB_GETLBTEXTLEN, (WPARAM)nIndex, 0) + 1;
//
//         /* Allocate enough space to be more safe */
//         tmp = (PWSTR)HeapAlloc(GetProcessHeap(), HEAP_ZERO_MEMORY, uReqSize * sizeof(WCHAR));
//         if (tmp != NULL)
//         {
//             /* Get selected time format text */
//             SendMessageW(hChildWnd, CB_GETLBTEXT, (WPARAM)nIndex, (LPARAM)tmp);
//
//             /* Finally, copy the result into the output */
//             wcsncpy(Buffer, tmp, uSize);
//
//             HeapFree(GetProcessHeap(), 0, tmp);
//         }
//     }
// }
//
//
// VOID
// GetSelectedComboBoxIndex(
//     HWND hwndDlg,
//     INT nIdDlgItem,
//     PINT pValue)
// {
//     *pValue = SendDlgItemMessageW(hwndDlg, nIdDlgItem, CB_GETCURSEL, 0, 0);
// }
//
// /* EOF */
//
// ============================================================================
// END ORIGINAL C SOURCE
// ============================================================================
