use crate::prelude::*;

// Local extern decls removed (CC_MISMATCH sweep): memset/memcpy/CoTaskMemAlloc
// come from the prelude with the real prototypes; SysStringByteLen /
// SysAllocStringByteLen / SysFreeString / VariantCopy are defined in-crate
// (extern "system") and visible via the prelude glob re-exports; the
// IUnknown_* / IRecordInfo_* COBJMACROS forms are commacros_gen.rs wrappers.

// ---- file-scope preamble (translated) ----
pub const SAFEARRAY_HIDDEN_SIZE: usize = core::mem::size_of::<GUID>();

pub const ignored_copy_features: u16 = 
    0x0001 | // FADF_AUTO
    0x0002 | // FADF_STATIC
    0x0004 | // FADF_EMBEDDED
    0x0010 | // FADF_FIXEDSIZE
    0x0020;  // FADF_CREATEVECTOR

unsafe fn SAFEARRAY_Malloc(size: ULONG) -> *mut core::ffi::c_void {
    
    let ret = CoTaskMemAlloc(size as SIZE_T);
    if !ret.is_null() {
        memset(ret, 0, size as usize);
    }
    ret
}

unsafe fn SAFEARRAY_Free(ptr: *mut core::ffi::c_void) {
    CoTaskMemFree(ptr);
}

unsafe fn SAFEARRAY_GetVTSize(vt: VARTYPE) -> DWORD {
    match vt {
        VT_I1 | VT_UI1 => core::mem::size_of::<BYTE>() as DWORD,
        VT_BOOL | VT_I2 | VT_UI2 => core::mem::size_of::<SHORT>() as DWORD,
        VT_I4 | VT_UI4 | VT_R4 | VT_ERROR => core::mem::size_of::<LONG>() as DWORD,
        VT_R8 | VT_I8 | VT_UI8 => core::mem::size_of::<LONG64>() as DWORD,
        VT_INT | VT_UINT => core::mem::size_of::<INT>() as DWORD,
        VT_INT_PTR | VT_UINT_PTR => core::mem::size_of::<UINT_PTR>() as DWORD,
        VT_CY => core::mem::size_of::<CY>() as DWORD,
        VT_DATE => core::mem::size_of::<DATE>() as DWORD,
        VT_BSTR => core::mem::size_of::<BSTR>() as DWORD,
        VT_DISPATCH => core::mem::size_of::<LPDISPATCH>() as DWORD,
        VT_VARIANT => core::mem::size_of::<VARIANT>() as DWORD,
        VT_UNKNOWN => core::mem::size_of::<LPUNKNOWN>() as DWORD,
        VT_DECIMAL => core::mem::size_of::<DECIMAL>() as DWORD,
        VT_RECORD => 32,
        _ => 0,
    }
}

unsafe fn SAFEARRAY_SetHiddenDWORD(psa: *mut SAFEARRAY, dw: DWORD) {
    let lpDw = psa as LPDWORD;
    *lpDw.offset(-1) = dw;
}

unsafe fn SAFEARRAY_GetHiddenDWORD(psa: *mut SAFEARRAY) -> DWORD {
    let lpDw = psa as LPDWORD;
    *lpDw.offset(-1)
}

unsafe fn SAFEARRAY_GetCellCount(psa: *const SAFEARRAY) -> ULONG {
    let mut psab: *const SAFEARRAYBOUND = (*psa).rgsabound.as_ptr();
    let mut cCount: USHORT = (*psa).cDims;
    let mut ulNumCells: ULONG = 1;

    while cCount > 0 {
        cCount -= 1;
        if (*psab).cElements == 0 {
            return 0;
        }
        ulNumCells *= (*psab).cElements;
        psab = psab.offset(1);
    }
    ulNumCells
}

unsafe fn SAFEARRAY_AllocDescriptor(ulSize: ULONG, ppsaOut: *mut *mut SAFEARRAY) -> HRESULT {
    let ptr = SAFEARRAY_Malloc(ulSize + SAFEARRAY_HIDDEN_SIZE as ULONG) as *mut u8;

    if ptr.is_null() {
        *ppsaOut = core::ptr::null_mut();
        return E_OUTOFMEMORY;
    }

    *ppsaOut = ptr.add(SAFEARRAY_HIDDEN_SIZE) as *mut SAFEARRAY;
    S_OK
}

unsafe fn SAFEARRAY_SetFeatures(vt: VARTYPE, psa: *mut SAFEARRAY) {
    if vt == VT_DISPATCH as u16 {
        (*psa).fFeatures = FADF_HAVEIID as u16;
        SafeArraySetIID(psa, &IID_IDispatch);
    } else if vt == VT_UNKNOWN as u16 {
        (*psa).fFeatures = FADF_HAVEIID as u16;
        SafeArraySetIID(psa, &IID_IUnknown);
    } else if vt == VT_RECORD as u16 {
        (*psa).fFeatures = FADF_RECORD as u16;
    } else {
        (*psa).fFeatures = FADF_HAVEVARTYPE as u16;
        SAFEARRAY_SetHiddenDWORD(psa, vt as DWORD);
    }
}

unsafe fn SAFEARRAY_Create(vt: VARTYPE, cDims: UINT, rgsabound: *const SAFEARRAYBOUND, ulSize: ULONG) -> *mut SAFEARRAY {
    let mut psa: *mut SAFEARRAY = core::ptr::null_mut();
    let mut i: u32;

    if rgsabound.is_null() {
        return core::ptr::null_mut();
    }

    if SUCCEEDED(SafeArrayAllocDescriptorEx(vt, cDims, &mut psa)) {
        match vt {
            VT_BSTR => (*psa).fFeatures |= FADF_BSTR,
            VT_UNKNOWN => (*psa).fFeatures |= FADF_UNKNOWN,
            VT_DISPATCH => (*psa).fFeatures |= FADF_DISPATCH,
            VT_VARIANT => (*psa).fFeatures |= FADF_VARIANT,
            _ => {}
        }

        i = 0;
        while i < cDims {
            memcpy(
                (*psa).rgsabound.as_mut_ptr().add(i as usize) as LPVOID,
                rgsabound.add((cDims - 1 - i) as usize) as *const core::ffi::c_void,
                core::mem::size_of::<SAFEARRAYBOUND>(),
            );
            i += 1;
        }

        if ulSize != 0 {
            (*psa).cbElements = ulSize;
        }

        if (*psa).cbElements == 0 || FAILED(SafeArrayAllocData(psa)) {
            SafeArrayDestroyDescriptor(psa);
            psa = core::ptr::null_mut();
        }
    }
    psa
}

unsafe fn SAFEARRAY_CreateVector(vt: VARTYPE, lLbound: LONG, cElements: ULONG, ulSize: ULONG) -> *mut SAFEARRAY {
    let mut psa: *mut SAFEARRAY = core::ptr::null_mut();

    if ulSize != 0 || vt == VT_RECORD {
        if SUCCEEDED(SAFEARRAY_AllocDescriptor(
            (core::mem::size_of::<SAFEARRAY>() as ULONG)
                .wrapping_add(ulSize.wrapping_mul(cElements)),
            &mut psa,
        ))
        {
            SAFEARRAY_SetFeatures(vt, psa);

            (*psa).cDims = 1;
            (*psa).fFeatures |= FADF_CREATEVECTOR;
            (*psa).pvData = psa.add(1) as LPVOID;
            (*psa).cbElements = ulSize;
            (*psa).rgsabound[0].cElements = cElements;
            (*psa).rgsabound[0].lLbound = lLbound;

            match vt {
                VT_BSTR => (*psa).fFeatures |= FADF_BSTR,
                VT_UNKNOWN => (*psa).fFeatures |= FADF_UNKNOWN,
                VT_DISPATCH => (*psa).fFeatures |= FADF_DISPATCH,
                VT_VARIANT => (*psa).fFeatures |= FADF_VARIANT,
                _ => {}
            }
        }
    }
    psa
}

unsafe fn SAFEARRAY_DestroyData(psa: *mut SAFEARRAY, ulStartCell: ULONG) -> HRESULT {
    if !(*psa).pvData.is_null() && ((*psa).fFeatures & FADF_DATADELETED) == 0 {
        let mut ulCellCount = SAFEARRAY_GetCellCount(psa);

        if ulStartCell > ulCellCount {
            FIXME!(b"unexpected ulCellCount %d, start %d\n\0".as_ptr() as *const i8, ulCellCount, ulStartCell);
            return E_UNEXPECTED;
        }

        ulCellCount -= ulStartCell;

        if ((*psa).fFeatures & (FADF_UNKNOWN | FADF_DISPATCH)) != 0 {
            let mut lpUnknown = ((*psa).pvData as *mut LPUNKNOWN).add(ulStartCell as usize);

            let mut count = ulCellCount;
            while count > 0 {
                count -= 1;
                if !(*lpUnknown).is_null() {
                    IUnknown_Release(*lpUnknown);
                }
                lpUnknown = lpUnknown.add(1);
            }
        } else if ((*psa).fFeatures & FADF_RECORD) != 0 {
            let mut lpRecInfo: *mut IRecordInfo = core::ptr::null_mut();

            if SUCCEEDED(SafeArrayGetRecordInfo(psa, &mut lpRecInfo)) {
                let mut pRecordData = (*psa).pvData as PBYTE;
                let mut count = ulCellCount;
                while count > 0 {
                    count -= 1;
                    IRecordInfo_RecordClear(lpRecInfo, pRecordData as LPVOID);
                    pRecordData = pRecordData.add((*psa).cbElements as usize);
                }
                IRecordInfo_Release(lpRecInfo);
            }
        } else if ((*psa).fFeatures & FADF_BSTR) != 0 {
            let mut lpBstr = ((*psa).pvData as *mut BSTR).add(ulStartCell as usize);

            let mut count = ulCellCount;
            while count > 0 {
                count -= 1;
                SysFreeString(*lpBstr);
                lpBstr = lpBstr.add(1);
            }
        } else if ((*psa).fFeatures & FADF_VARIANT) != 0 {
            let mut lpVariant = ((*psa).pvData as *mut VARIANT).add(ulStartCell as usize);

            let mut count = ulCellCount;
            while count > 0 {
                count -= 1;
                let hRet = VariantClear(lpVariant);

                if FAILED(hRet) {
                    FIXME!(b"VariantClear of element failed!\n\0".as_ptr() as *const i8);
                }
                lpVariant = lpVariant.add(1);
            }
        }
    }
    S_OK
}

unsafe fn SAFEARRAY_CopyData(psa: *mut SAFEARRAY, dest: *mut SAFEARRAY) -> HRESULT {

    let mut hr: HRESULT = 0;

    if (*psa).pvData.is_null() {
        return 0;
    }

    if (*dest).pvData.is_null() || (*psa).fFeatures & 0x0001 != 0 {
        return 0x80070057u32 as HRESULT;
    } else {
        let mut ulCellCount = SAFEARRAY_GetCellCount(psa);

        (*dest).fFeatures = ((*dest).fFeatures & 0x0800) | ((*psa).fFeatures & !ignored_copy_features);

        if (*psa).fFeatures & 0x0080 != 0 {
            let mut src_var = (*psa).pvData as *mut VARIANT;
            let mut dest_var = (*dest).pvData as *mut VARIANT;

            while ulCellCount != 0 {
                ulCellCount -= 1;
                let hRet = VariantCopy(dest_var, src_var);
                if FAILED(hRet) {
                    FIXME!(b"VariantCopy failed with 0x%08x, element %u\0".as_ptr() as *const i8, hRet, ulCellCount);
                }
                src_var = src_var.offset(1);
                dest_var = dest_var.offset(1);
            }
        } else if (*psa).fFeatures & 0x0100 != 0 {
            let mut src_bstr = (*psa).pvData as *mut BSTR;
            let mut dest_bstr = (*dest).pvData as *mut BSTR;

            while ulCellCount != 0 {
                ulCellCount -= 1;
                SysFreeString(*dest_bstr);
                if !(*src_bstr).is_null() {
                    *dest_bstr = SysAllocStringByteLen(*src_bstr as *const i8, SysStringByteLen(*src_bstr));
                    if (*dest_bstr).is_null() {
                        return 0x8007000Eu32 as HRESULT;
                    }
                } else {
                    *dest_bstr = core::ptr::null_mut();
                }
                src_bstr = src_bstr.offset(1);
                dest_bstr = dest_bstr.offset(1);
            }
        } else if (*psa).fFeatures & 0x0020 != 0 {
            let mut dest_data = (*dest).pvData as *mut PBYTE;
            let mut src_data = (*psa).pvData as *mut PBYTE;
            let mut record: *mut IRecordInfo = core::ptr::null_mut();

            SafeArrayGetRecordInfo(psa, &mut record);
            while ulCellCount != 0 {
                ulCellCount -= 1;
                hr = IRecordInfo_RecordCopy(record, src_data as *mut core::ffi::c_void, dest_data as *mut core::ffi::c_void);
                if FAILED(hr) {
                    break;
                }
                src_data = src_data.offset((*psa).cbElements as isize);
                dest_data = dest_data.offset((*psa).cbElements as isize);
            }

            SafeArraySetRecordInfo(dest, record);
            (*dest).cbElements = (*psa).cbElements;
            IRecordInfo_Release(record);
        } else if (*psa).fFeatures & (0x0200 | 0x0400) != 0 {
            let mut dest_unk = (*dest).pvData as *mut *mut IUnknown;
            let mut src_unk = (*psa).pvData as *mut *mut IUnknown;

            while ulCellCount != 0 {
                ulCellCount -= 1;
                if !(*dest_unk).is_null() {
                    IUnknown_Release(*dest_unk);
                }
                *dest_unk = *src_unk;
                if !(*dest_unk).is_null() {
                    IUnknown_AddRef(*dest_unk);
                }
                src_unk = src_unk.offset(1);
                dest_unk = dest_unk.offset(1);
            }
        } else {
            memcpy((*dest).pvData as *mut core::ffi::c_void, (*psa).pvData as *const core::ffi::c_void, (ulCellCount * (*psa).cbElements) as usize);
        }

        if (*psa).fFeatures & 0x0040 != 0 {
            let mut guid: GUID = core::mem::zeroed();
            SafeArrayGetIID(psa, &mut guid);
            SafeArraySetIID(dest, &guid);
        } else if (*psa).fFeatures & 0x0001 != 0 {
            SAFEARRAY_SetHiddenDWORD(dest, SAFEARRAY_GetHiddenDWORD(psa));
        }
    }

    hr
}

#[no_mangle]
pub unsafe extern "system" fn SafeArrayAllocDescriptor(cDims: UINT, ppsaOut: *mut *mut SAFEARRAY) -> HRESULT {
    let allocSize: LONG;
    let hr: HRESULT;

    TRACE!(b"(%d,%p)\n\0".as_ptr() as PCHAR, cDims, ppsaOut);

    if cDims == 0 || cDims >= 0x10000 {
        return E_INVALIDARG;
    }

    if ppsaOut.is_null() {
        return E_POINTER;
    }

    allocSize = (core::mem::size_of::<SAFEARRAY>() + core::mem::size_of::<SAFEARRAYBOUND>() * (cDims as usize - 1)) as LONG;

    hr = SAFEARRAY_AllocDescriptor(allocSize as ULONG, ppsaOut);
    if FAILED(hr) {
        return hr;
    }

    (**ppsaOut).cDims = cDims as USHORT;

    TRACE!(b"(%d): %u bytes allocated for descriptor.\n\0".as_ptr() as PCHAR, cDims, allocSize);
    S_OK
}

#[no_mangle]
pub unsafe extern "system" fn SafeArrayAllocDescriptorEx(vt: VARTYPE, cDims: UINT, ppsaOut: *mut *mut SAFEARRAY) -> HRESULT {
    let cbElements: ULONG;
    let hRet: HRESULT;

    TRACE!(b"(%s,%u,%p)\n\0".as_ptr() as *const i8, debugstr_vt(vt), cDims, ppsaOut);

    cbElements = SAFEARRAY_GetVTSize(vt);
    if cbElements == 0 {
        WARN!(b"Creating a descriptor with an invalid VARTYPE!\n\0".as_ptr() as *const i8);
    }

    hRet = SafeArrayAllocDescriptor(cDims, ppsaOut);

    if SUCCEEDED(hRet) {
        SAFEARRAY_SetFeatures(vt, *ppsaOut);
        (**ppsaOut).cbElements = cbElements;
    }
    hRet
}

#[no_mangle]
pub unsafe extern "system" fn SafeArrayAllocData(psa: *mut SAFEARRAY) -> HRESULT {
    let mut hRet: HRESULT = E_INVALIDARG;
    
    if !psa.is_null() {
        let ulSize: ULONG = SAFEARRAY_GetCellCount(psa);
        
        (*psa).pvData = SAFEARRAY_Malloc(ulSize.wrapping_mul((*psa).cbElements));
        
        if !(*psa).pvData.is_null() {
            hRet = S_OK;
        } else {
            hRet = E_OUTOFMEMORY;
        }
    }
    return hRet;
}

#[no_mangle]
pub unsafe extern "system" fn SafeArrayDestroyDescriptor(psa: *mut SAFEARRAY) -> HRESULT {
    if !psa.is_null() {
        let lpv = (psa as *mut u8).offset(-(SAFEARRAY_HIDDEN_SIZE as isize)) as LPVOID;

        if (*psa).cLocks != 0 {
            return DISP_E_ARRAYISLOCKED;
        }

        if (*psa).fFeatures & FADF_RECORD != 0 {
            SafeArraySetRecordInfo(psa, core::ptr::null_mut());
        }

        if (*psa).fFeatures & FADF_CREATEVECTOR != 0 &&
           (*psa).fFeatures & FADF_DATADELETED == 0 {
            SAFEARRAY_DestroyData(psa, 0);
        }

        SAFEARRAY_Free(lpv);
    }
    S_OK
}

#[no_mangle]
pub unsafe extern "system" fn SafeArrayLock(psa: *mut SAFEARRAY) -> HRESULT {

    TRACE!(b"(%p)\n\0".as_ptr(), psa);

    if psa.is_null() {
        return E_INVALIDARG;
    }

    let ulLocks = InterlockedIncrement(core::ptr::addr_of_mut!((*psa).cLocks) as *mut i32);

    if ulLocks > 0xffff {
        WARN!(b"Out of locks!\n\0".as_ptr());
        InterlockedDecrement(core::ptr::addr_of_mut!((*psa).cLocks) as *mut i32);
        return E_UNEXPECTED;
    }
    S_OK
}

#[no_mangle]
pub unsafe extern "system" fn SafeArrayUnlock(psa: *mut SAFEARRAY) -> HRESULT {
    TRACE!(b"(%p)\n\0".as_ptr() as *const i8, psa);

    if psa.is_null() {
        return E_INVALIDARG;
    }

    if InterlockedDecrement(core::ptr::addr_of_mut!((*psa).cLocks) as *mut i32) < 0 {
        WARN!(b"Unlocked but no lock held!\n\0".as_ptr() as *const i8);
        InterlockedIncrement(core::ptr::addr_of_mut!((*psa).cLocks) as *mut i32);
        return E_UNEXPECTED;
    }
    S_OK
}

#[no_mangle]
pub unsafe extern "system" fn SafeArrayPutElement(
    psa: *mut SAFEARRAY,
    rgIndices: *mut LONG,
    pvData: *mut core::ffi::c_void,
) -> HRESULT {

    let mut hRet: HRESULT;

    TRACE!(b"(%p,%p,%p)\n\0".as_ptr(), psa, rgIndices, pvData);

    if psa.is_null() || rgIndices.is_null() {
        return E_INVALIDARG;
    }

    hRet = SafeArrayLock(psa);

    if SUCCEEDED(hRet) {
        let mut lpvDest: *mut core::ffi::c_void = core::ptr::null_mut();

        hRet = SafeArrayPtrOfIndex(psa, rgIndices, &mut lpvDest);

        if SUCCEEDED(hRet) {
            if ((*psa).fFeatures & FADF_VARIANT) != 0 {
                let lpVariant = pvData as *mut VARIANT;
                let lpDest = lpvDest as *mut VARIANT;

                hRet = VariantCopy(lpDest, lpVariant);
                if FAILED(hRet) {
                    FIXME!(b"VariantCopy failed with 0x%x\n\0".as_ptr(), hRet);
                }
            } else if ((*psa).fFeatures & FADF_BSTR) != 0 {
                let lpBstr = pvData as BSTR;
                let lpDest = lpvDest as *mut BSTR;

                SysFreeString(*lpDest);

                *lpDest = SysAllocStringByteLen(lpBstr as *const i8, SysStringByteLen(lpBstr));
                if (*lpDest).is_null() {
                    hRet = E_OUTOFMEMORY;
                }
            } else if ((*psa).fFeatures & (FADF_UNKNOWN | FADF_DISPATCH)) != 0 {
                let lpUnknown = pvData as *mut IUnknown;
                let lpDest = lpvDest as *mut *mut IUnknown;

                if !lpUnknown.is_null() {
                    IUnknown_AddRef(lpUnknown);
                }
                if !(*lpDest).is_null() {
                    IUnknown_Release(*lpDest);
                }
                *lpDest = lpUnknown;
            } else if ((*psa).fFeatures & FADF_RECORD) != 0 {
                let mut record: *mut IRecordInfo = core::ptr::null_mut();

                SafeArrayGetRecordInfo(psa, &mut record);
                hRet = IRecordInfo_RecordCopy(record, pvData, lpvDest);
                IRecordInfo_Release(record);
            } else {
                memcpy(lpvDest, pvData, (*psa).cbElements as usize);
            }
        }
        SafeArrayUnlock(psa);
    }
    hRet
}

#[no_mangle]
pub unsafe extern "system" fn SafeArrayGetElement(
    psa: *mut SAFEARRAY,
    rgIndices: *mut LONG,
    pvData: *mut core::ffi::c_void,
) -> HRESULT {

    TRACE!(b"(%p,%p,%p)\n\0".as_ptr(), psa, rgIndices, pvData);

    if psa.is_null() || rgIndices.is_null() || pvData.is_null() {
        return E_INVALIDARG;
    }

    let mut hRet = SafeArrayLock(psa);

    if SUCCEEDED(hRet) {
        let mut lpvSrc: *mut core::ffi::c_void = core::ptr::null_mut();

        hRet = SafeArrayPtrOfIndex(psa, rgIndices, &mut lpvSrc);

        if SUCCEEDED(hRet) {
            if (*psa).fFeatures & FADF_VARIANT != 0 {
                let lpVariant = lpvSrc as *mut VARIANT;
                let lpDest = pvData as *mut VARIANT;

                (*V_VT_ref(lpDest)) = VT_EMPTY;
                hRet = VariantCopy(lpDest, lpVariant);
                if FAILED(hRet) {
                    FIXME!(b"VariantCopy failed with 0x%x\n\0".as_ptr(), hRet);
                }
            } else if (*psa).fFeatures & FADF_BSTR != 0 {
                let lpBstr = lpvSrc as *mut BSTR;
                let lpDest = pvData as *mut BSTR;

                if !(*lpBstr).is_null() {
                    *lpDest = SysAllocStringByteLen(*lpBstr as *const i8, SysStringByteLen(*lpBstr));
                    if (*lpDest).is_null() {
                        hRet = E_OUTOFMEMORY;
                    }
                } else {
                    *lpDest = core::ptr::null_mut();
                }
            } else if (*psa).fFeatures & (FADF_UNKNOWN | FADF_DISPATCH) != 0 {
                let src_unk = lpvSrc as *mut *mut IUnknown;
                let dest_unk = pvData as *mut *mut IUnknown;

                if !(*src_unk).is_null() {
                    IUnknown_AddRef(*src_unk);
                }
                *dest_unk = *src_unk;
            } else if (*psa).fFeatures & FADF_RECORD != 0 {
                let mut record: *mut IRecordInfo = core::ptr::null_mut();

                SafeArrayGetRecordInfo(psa, &mut record);
                hRet = IRecordInfo_RecordCopy(record, lpvSrc, pvData);
                IRecordInfo_Release(record);
            } else {
                memcpy(pvData, lpvSrc, (*psa).cbElements as usize);
            }
        }
        SafeArrayUnlock(psa);
    }
    hRet
}

#[no_mangle]
pub unsafe extern "system" fn SafeArrayGetUBound(
    psa: *mut SAFEARRAY,
    nDim: UINT,
    plUbound: *mut LONG,
) -> HRESULT {
    TRACE!(b"(%p,%d,%p)\n\0".as_ptr(), psa, nDim, plUbound);

    if psa.is_null() || plUbound.is_null() {
        return E_INVALIDARG;
    }

    if nDim == 0 || nDim > (*psa).cDims as UINT {
        return DISP_E_BADINDEX;
    }

    let idx = ((*psa).cDims as UINT - nDim) as usize;
    // oaidl.idl: `[size_is(cDims)] SAFEARRAYBOUND rgsabound[];` -- a true flexible
    // array. bindgen's `[T; 1]` panicked for every dimension past the first, so any
    // multi-dimensional SAFEARRAY faulted here.
    let bound = (&raw const (*psa).rgsabound[0]).add(idx);
    *plUbound = (*bound).lLbound + (*bound).cElements as LONG - 1;

    S_OK
}

#[no_mangle]
pub unsafe extern "system" fn SafeArrayGetLBound(
    psa: *mut SAFEARRAY,
    nDim: UINT,
    plLbound: *mut LONG,
) -> HRESULT {
    TRACE!(b"(%p,%d,%p)\n\0".as_ptr() as PCHAR, psa, nDim, plLbound);

    if psa.is_null() || plLbound.is_null() {
        return E_INVALIDARG;
    }

    if nDim == 0 || nDim > (*psa).cDims as UINT {
        return DISP_E_BADINDEX;
    }

    *plLbound = (*(&raw const (*psa).rgsabound[0]).add(((*psa).cDims as UINT - nDim) as usize)).lLbound;
    S_OK
}

#[no_mangle]
pub unsafe extern "system" fn SafeArrayGetDim(psa: *mut SAFEARRAY) -> UINT {
    TRACE!(b"(%p) returning %d\n\0".as_ptr() as PCHAR, psa, if !psa.is_null() { (*psa).cDims } else { 0u32 });
    if !psa.is_null() { (*psa).cDims as UINT } else { 0 }
}

#[no_mangle]
pub unsafe extern "system" fn SafeArrayGetElemsize(psa: *mut SAFEARRAY) -> UINT {
    TRACE!(
        b"(%p) returning %d\n\0".as_ptr() as PCHAR,
        psa,
        if !psa.is_null() { (*psa).cbElements } else { 0u32 }
    );
    if !psa.is_null() {
        (*psa).cbElements
    } else {
        0
    }
}

#[no_mangle]
pub unsafe extern "system" fn SafeArrayAccessData(psa: *mut SAFEARRAY, ppvData: *mut *mut core::ffi::c_void) -> HRESULT {
    let hr: HRESULT;

    TRACE!(b"(%p,%p)\n\0".as_ptr(), psa, ppvData);

    if psa.is_null() || ppvData.is_null() {
        return E_INVALIDARG;
    }

    hr = SafeArrayLock(psa);
    *ppvData = if SUCCEEDED(hr) { (*psa).pvData } else { core::ptr::null_mut() };

    return hr;
}

#[no_mangle]
pub unsafe extern "system" fn SafeArrayUnaccessData(psa: *mut SAFEARRAY) -> HRESULT {
    SafeArrayUnlock(psa)
}

#[no_mangle]
pub unsafe extern "system" fn SafeArrayPtrOfIndex(
    psa: *mut SAFEARRAY,
    rgIndices: *mut LONG,
    ppvData: *mut *mut core::ffi::c_void,
) -> HRESULT {
    let mut dim: USHORT;
    let mut cell: ULONG = 0;
    let mut dimensionSize: ULONG = 1;
    let mut psab: *mut SAFEARRAYBOUND;
    let c1: LONG;

    TRACE!(
        b"(%p,%p,%p)\n\0".as_ptr() as *const i8,
        psa,
        rgIndices,
        ppvData,
    );

    if psa.is_null() || rgIndices.is_null() || ppvData.is_null() {
        return E_INVALIDARG;
    }

    psab = (*psa).rgsabound.as_mut_ptr().offset(((*psa).cDims - 1) as isize);
    c1 = *rgIndices;
    let mut rgIndices = rgIndices.offset(1);

    if c1 < (*psab).lLbound || c1 >= (*psab).lLbound + (*psab).cElements as LONG {
        return DISP_E_BADINDEX;
    }

    dim = 1;
    while dim < (*psa).cDims {
        dimensionSize *= (*psab).cElements;

        psab = psab.offset(-1);

        if (*psab).cElements == 0
            || *rgIndices < (*psab).lLbound
            || *rgIndices >= (*psab).lLbound + (*psab).cElements as LONG
        {
            return DISP_E_BADINDEX;
        }

        cell += ((*rgIndices - (*psab).lLbound) as ULONG) * dimensionSize;
        rgIndices = rgIndices.offset(1);
        dim += 1;
    }

    cell += (c1 - (*psa).rgsabound.as_ptr().offset(((*psa).cDims - 1) as isize).read().lLbound) as ULONG;

    *ppvData = ((*psa).pvData as *mut u8).offset((cell * (*psa).cbElements) as isize) as *mut core::ffi::c_void;
    S_OK
}

#[no_mangle]
pub unsafe extern "system" fn SafeArrayDestroyData(psa: *mut SAFEARRAY) -> HRESULT {
    TRACE!(b"(%p)\n\0".as_ptr(), psa);

    if psa.is_null() {
        return E_INVALIDARG;
    }

    if (*psa).cLocks != 0 {
        return DISP_E_ARRAYISLOCKED;
    }

    let hr = SAFEARRAY_DestroyData(psa, 0);
    if FAILED(hr) {
        return hr;
    }

    if !(*psa).pvData.is_null() {
        if ((*psa).fFeatures & FADF_STATIC) != 0 {
            ZeroMemory(
                (*psa).pvData,
                (SAFEARRAY_GetCellCount(psa) * (*psa).cbElements) as usize,
            );
            return S_OK;
        }

        if ((*psa).fFeatures & FADF_CREATEVECTOR) == 0 {
            SAFEARRAY_Free((*psa).pvData);
            (*psa).pvData = core::ptr::null_mut();
        } else {
            (*psa).fFeatures |= FADF_DATADELETED;
        }
    }

    S_OK
}

#[no_mangle]
pub unsafe extern "system" fn SafeArrayCopyData(
    psaSource: *mut SAFEARRAY,
    psaTarget: *mut SAFEARRAY,
) -> HRESULT {
    let mut dim: i32;

    TRACE!(
        b"(%p,%p)\n\0".as_ptr() as *const i8,
        psaSource,
        psaTarget,
    );

    if psaSource.is_null()
        || psaTarget.is_null()
        || (*psaSource).cDims != (*psaTarget).cDims
        || (*psaSource).cbElements != (*psaTarget).cbElements
    {
        return E_INVALIDARG;
    }

    dim = (*psaSource).cDims as i32 - 1;
    while dim >= 0 {
        if (*(&raw const (*psaSource).rgsabound[0]).add(dim as usize)).cElements
            != (*(&raw const (*psaTarget).rgsabound[0]).add(dim as usize)).cElements
        {
            return E_INVALIDARG;
        }
        dim -= 1;
    }

    SAFEARRAY_CopyData(psaSource, psaTarget)
}

#[no_mangle]
pub unsafe extern "system" fn SafeArrayDestroy(psa: *mut SAFEARRAY) -> HRESULT {
    if psa.is_null() {
        return S_OK;
    }

    if (*psa).cLocks > 0 {
        return DISP_E_ARRAYISLOCKED;
    }

    SafeArrayDestroyData(psa);
    SafeArrayDestroyDescriptor(psa);
    S_OK
}

#[no_mangle]
pub unsafe extern "system" fn SafeArrayCopy(psa: *mut SAFEARRAY, ppsaOut: *mut *mut SAFEARRAY) -> HRESULT {

    const E_INVALIDARG: HRESULT = 0x80070057u32 as i32;
    const S_OK: HRESULT = 0;
    const E_OUTOFMEMORY: HRESULT = 0x8007000Eu32 as i32;
    const FADF_RECORD: u16 = 0x0020;
    const FADF_HAVEIID: u16 = 0x0040;
    const FADF_HAVEVARTYPE: u16 = 0x0080;

    TRACE!(b"(%p,%p)\n\0".as_ptr(), psa, ppsaOut);

    if ppsaOut.is_null() {
        return E_INVALIDARG;
    }

    *ppsaOut = core::ptr::null_mut();

    if psa.is_null() {
        return S_OK;
    }

    if (*psa).cbElements == 0 {
        return E_INVALIDARG;
    }

    let mut hRet: HRESULT;

    if ((*psa).fFeatures & (FADF_RECORD | FADF_HAVEIID | FADF_HAVEVARTYPE)) != 0 {
        let mut vt: VARTYPE = 0;

        hRet = SafeArrayGetVartype(psa, &mut vt);
        if SUCCEEDED(hRet) {
            hRet = SafeArrayAllocDescriptorEx(vt, (*psa).cDims as UINT, ppsaOut);
        }
    } else {
        hRet = SafeArrayAllocDescriptor((*psa).cDims as UINT, ppsaOut);
        if SUCCEEDED(hRet) {
            (**ppsaOut).fFeatures = (*psa).fFeatures & !ignored_copy_features;
            (**ppsaOut).cbElements = (*psa).cbElements;
        }
    }

    if SUCCEEDED(hRet) {
        memcpy(
            (**ppsaOut).rgsabound.as_mut_ptr() as *mut core::ffi::c_void,
            (*psa).rgsabound.as_ptr() as *const core::ffi::c_void,
            ((*psa).cDims as usize) * core::mem::size_of::<SAFEARRAYBOUND>(),
        );

        (**ppsaOut).pvData = SAFEARRAY_Malloc(SAFEARRAY_GetCellCount(psa) * (*psa).cbElements);
        if (**ppsaOut).pvData.is_null() {
            SafeArrayDestroyDescriptor(*ppsaOut);
            *ppsaOut = core::ptr::null_mut();
            return E_OUTOFMEMORY;
        }

        let hRet = SAFEARRAY_CopyData(psa, *ppsaOut);
        if FAILED(hRet) {
            SAFEARRAY_Free((**ppsaOut).pvData);
            SafeArrayDestroyDescriptor(*ppsaOut);
            *ppsaOut = core::ptr::null_mut();
            return hRet;
        }
    }

    hRet
}

#[no_mangle]
pub unsafe extern "system" fn SafeArrayRedim(psa: *mut SAFEARRAY, psabound: *mut SAFEARRAYBOUND) -> HRESULT {

    TRACE!(b"(%p,%p)\n\0".as_ptr(), psa, psabound);

    if psa.is_null() || psabound.is_null() {
        return E_INVALIDARG;
    }

    // Wine 10.0, as Windows: a fixed-size array cannot be resized for the same
    // reason a locked one cannot, and says so with the same HRESULT (was
    // E_INVALIDARG).
    if (*psa).fFeatures & FADF_FIXEDSIZE != 0 || (*psa).cLocks > 0 {
        return DISP_E_ARRAYISLOCKED;
    }

    let hr = SafeArrayLock(psa);
    if FAILED(hr) {
        return hr;
    }

    let oldBounds = (*psa).rgsabound.as_mut_ptr();
    (*oldBounds).lLbound = (*psabound).lLbound;

    if (*psabound).cElements != (*oldBounds).cElements {
        if (*psabound).cElements < (*oldBounds).cElements {
            let ulStartCell = (*psabound).cElements *
                (SAFEARRAY_GetCellCount(psa) / (*oldBounds).cElements);
            SAFEARRAY_DestroyData(psa, ulStartCell);
        } else {
            let ulOldSize: ULONG;
            let ulNewSize: ULONG;

            ulOldSize = SAFEARRAY_GetCellCount(psa) * (*psa).cbElements;
            if ulOldSize != 0 {
                ulNewSize = (ulOldSize / (*oldBounds).cElements) * (*psabound).cElements;
            } else {
                let oldelems = (*oldBounds).cElements;
                (*oldBounds).cElements = (*psabound).cElements;
                ulNewSize = SAFEARRAY_GetCellCount(psa) * (*psa).cbElements;
                (*oldBounds).cElements = oldelems;
            }

            let pvNewData = SAFEARRAY_Malloc(ulNewSize);
            if pvNewData.is_null() {
                SafeArrayUnlock(psa);
                return E_OUTOFMEMORY;
            }

            memcpy(pvNewData, (*psa).pvData, ulOldSize as usize);
            SAFEARRAY_Free((*psa).pvData);
            (*psa).pvData = pvNewData;
        }
        (*oldBounds).cElements = (*psabound).cElements;
    }

    SafeArrayUnlock(psa);
    S_OK
}

#[no_mangle]
pub unsafe extern "system" fn SafeArrayGetVartype(psa: *mut SAFEARRAY, pvt: *mut VARTYPE) -> HRESULT {
    if psa.is_null() || pvt.is_null() {
        return E_INVALIDARG;
    }

    if (*psa).fFeatures & FADF_RECORD != 0 {
        *pvt = VT_RECORD;
    } else if ((*psa).fFeatures & (FADF_HAVEIID | FADF_DISPATCH)) == (FADF_HAVEIID | FADF_DISPATCH) {
        *pvt = VT_DISPATCH;
    } else if (*psa).fFeatures & FADF_HAVEIID != 0 {
        *pvt = VT_UNKNOWN;
    } else if (*psa).fFeatures & FADF_HAVEVARTYPE != 0 {
        let vt = SAFEARRAY_GetHiddenDWORD(psa);
        *pvt = vt as VARTYPE;
    } else {
        return E_INVALIDARG;
    }

    S_OK
}

#[no_mangle]
pub unsafe extern "system" fn SafeArraySetRecordInfo(psa: *mut SAFEARRAY, pRinfo: *mut IRecordInfo) -> HRESULT {

    let dest = psa as *mut *mut IRecordInfo;

    TRACE!(b"(%p,%p)\n\0".as_ptr(), psa, pRinfo);

    if psa.is_null() || ((*psa).fFeatures & FADF_RECORD) == 0 {
        return E_INVALIDARG;
    }

    if !pRinfo.is_null() {
        IRecordInfo_AddRef(pRinfo);
    }

    if !(*dest.offset(-1)).is_null() {
        IRecordInfo_Release(*dest.offset(-1));
    }

    *dest.offset(-1) = pRinfo;
    S_OK
}

#[no_mangle]
pub unsafe extern "system" fn SafeArrayGetRecordInfo(
    psa: *mut SAFEARRAY,
    pRinfo: *mut *mut IRecordInfo,
) -> HRESULT {

    let src = psa as *mut *mut IRecordInfo;

    TRACE!(b"(%p,%p)\n\0".as_ptr(), psa, pRinfo);

    if psa.is_null() || pRinfo.is_null() || ((*psa).fFeatures & FADF_RECORD) == 0 {
        return E_INVALIDARG;
    }

    *pRinfo = *src.offset(-1);

    if !(*pRinfo).is_null() {
        IRecordInfo_AddRef(*pRinfo);
    }
    S_OK
}
#[no_mangle]
pub unsafe extern "system" fn SafeArraySetIID(psa: *mut SAFEARRAY, guid: *const GUID) -> HRESULT {

    let dest = psa as *mut GUID;

    TRACE!(b"(%p,%s)\n\0".as_ptr() as *const i8, psa, debugstr_guid(guid));

    if psa.is_null() || guid.is_null() || ((*psa).fFeatures & FADF_HAVEIID) == 0 {
        return E_INVALIDARG;
    }

    *dest.offset(-1) = *guid;
    S_OK
}

#[no_mangle]
pub unsafe extern "system" fn SafeArrayGetIID(psa: *mut SAFEARRAY, pGuid: *mut GUID) -> HRESULT {
    let src = psa as *mut GUID;

    TRACE!(b"(%p,%p)\n\0".as_ptr(), psa, pGuid);

    if psa.is_null() || pGuid.is_null() || ((*psa).fFeatures & FADF_HAVEIID) == 0 {
        return E_INVALIDARG;
    }

    *pGuid = *src.offset(-1);
    S_OK
}

#[no_mangle]
pub unsafe extern "system" fn VectorFromBstr(bstr: BSTR, ppsa: *mut *mut SAFEARRAY) -> HRESULT {

    TRACE!(b"(%p,%p)\n\0".as_ptr(), bstr, ppsa);

    if ppsa.is_null() {
        return E_INVALIDARG;
    }

    let mut sab: SAFEARRAYBOUND = core::mem::zeroed();
    sab.lLbound = 0;
    sab.cElements = SysStringByteLen(bstr);

    *ppsa = SAFEARRAY_Create(VT_UI1, 1, &sab, 0);

    if !(*ppsa).is_null() {
        memcpy((**ppsa).pvData, bstr as *const core::ffi::c_void, sab.cElements as usize);
        return S_OK;
    }
    E_OUTOFMEMORY
}

#[no_mangle]
pub unsafe extern "system" fn BstrFromVector(psa: *mut SAFEARRAY, pbstr: *mut BSTR) -> HRESULT {

    TRACE!(b"(%p,%p)\n\0".as_ptr(), psa, pbstr);

    if pbstr.is_null() {
        return E_INVALIDARG;
    }

    *pbstr = core::ptr::null_mut();

    if psa.is_null() || (*psa).cbElements != 1 || (*psa).cDims != 1 {
        return E_INVALIDARG;
    }

    *pbstr = SysAllocStringByteLen((*psa).pvData as LPCSTR, (*psa).rgsabound[0].cElements);
    if (*pbstr).is_null() {
        return E_OUTOFMEMORY;
    }
    S_OK
}

/// SafeArrayCreate (OLEAUT32.15) — create a new SafeArray.
#[no_mangle]
pub unsafe extern "system" fn SafeArrayCreate(
    vt: VARTYPE,
    cDims: UINT,
    rgsabound: *mut SAFEARRAYBOUND,
) -> *mut SAFEARRAY {
    TRACE!(b"(%d,%u,%p)\n\0".as_ptr(), vt as c_int, cDims, rgsabound);

    if vt == VT_RECORD {
        return core::ptr::null_mut();
    }

    SAFEARRAY_Create(vt, cDims, rgsabound, 0)
}

/// SafeArrayCreateEx (OLEAUT32.42) — create a new SafeArray with extra data.
#[no_mangle]
pub unsafe extern "system" fn SafeArrayCreateEx(
    vt: VARTYPE,
    cDims: UINT,
    rgsabound: *mut SAFEARRAYBOUND,
    pvExtra: LPVOID,
) -> *mut SAFEARRAY {
    let mut ulSize: ULONG = 0;
    let iRecInfo: *mut IRecordInfo = pvExtra as *mut IRecordInfo;

    TRACE!(b"(%d,%u,%p,%p)\n\0".as_ptr(), vt as c_int, cDims, rgsabound, pvExtra);

    if vt == VT_RECORD {
        if iRecInfo.is_null() {
            return core::ptr::null_mut();
        }
        IRecordInfo_GetSize(iRecInfo, &mut ulSize);
    }
    let psa = SAFEARRAY_Create(vt, cDims, rgsabound, ulSize);

    if !pvExtra.is_null() {
        match vt {
            VT_RECORD => {
                SafeArraySetRecordInfo(psa, pvExtra as *mut IRecordInfo);
            }
            VT_UNKNOWN | VT_DISPATCH => {
                SafeArraySetIID(psa, pvExtra as *const GUID);
            }
            _ => {}
        }
    }
    psa
}

/// SafeArrayCreateVector (OLEAUT32.411) — one-dimensional contiguous SafeArray.
#[no_mangle]
pub unsafe extern "system" fn SafeArrayCreateVector(
    vt: VARTYPE,
    lLbound: LONG,
    cElements: ULONG,
) -> *mut SAFEARRAY {
    TRACE!(b"(%d,%d,%u)\n\0".as_ptr(), vt as c_int, lLbound, cElements);

    if vt == VT_RECORD {
        return core::ptr::null_mut();
    }

    SAFEARRAY_CreateVector(vt, lLbound, cElements, SAFEARRAY_GetVTSize(vt))
}

/// SafeArrayCreateVectorEx (OLEAUT32.43) — vector SafeArray with extra data.
#[no_mangle]
pub unsafe extern "system" fn SafeArrayCreateVectorEx(
    vt: VARTYPE,
    lLbound: LONG,
    cElements: ULONG,
    pvExtra: LPVOID,
) -> *mut SAFEARRAY {
    let ulSize: ULONG;
    let iRecInfo: *mut IRecordInfo = pvExtra as *mut IRecordInfo;

    TRACE!(b"(%d,%d,%u,%p)\n\0".as_ptr(), vt as c_int, lLbound, cElements, pvExtra);

    if vt == VT_RECORD {
        if iRecInfo.is_null() {
            return core::ptr::null_mut();
        }
        let mut sz: ULONG = 0;
        IRecordInfo_GetSize(iRecInfo, &mut sz);
        ulSize = sz;
    } else {
        ulSize = SAFEARRAY_GetVTSize(vt);
    }

    let psa = SAFEARRAY_CreateVector(vt, lLbound, cElements, ulSize);

    if !pvExtra.is_null() {
        match vt {
            VT_RECORD => {
                SafeArraySetRecordInfo(psa, iRecInfo);
            }
            VT_UNKNOWN | VT_DISPATCH => {
                SafeArraySetIID(psa, pvExtra as *const GUID);
            }
            _ => {}
        }
    }
    psa
}
