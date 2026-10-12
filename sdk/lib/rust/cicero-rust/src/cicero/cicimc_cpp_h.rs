use crate::prelude::*;
use core::ffi::c_void;


impl CicIMCLock {
    pub unsafe fn ClearCand(&mut self) -> BOOL {
        let this_: *mut Self = self;
        let mut hNewCandInfo: HIMCC;
        let mut hCandInfo: HIMCC = (*(*this_).__base0.m_pIC).hCandInfo;
        if !hCandInfo.is_null() {
            hNewCandInfo = ImmReSizeIMCC(hCandInfo, (CUSTOM_CAND_INFO_SIZE) as DWORD);
            if hNewCandInfo.is_null() {
                ImmDestroyIMCC((*(*this_).__base0.m_pIC).hCandInfo);
                (*(*this_).__base0.m_pIC).hCandInfo = ImmCreateIMCC((CUSTOM_CAND_INFO_SIZE) as DWORD);
                return (FALSE) as BOOL;
            }
        } else {
            hNewCandInfo = ImmCreateIMCC((CUSTOM_CAND_INFO_SIZE) as DWORD);
        }
    
        (*(*this_).__base0.m_pIC).hCandInfo = hNewCandInfo;
        if (*(*this_).__base0.m_pIC).hCandInfo.is_null() {
            return (FALSE) as BOOL;
        }
    
        let mut candInfo = CicIMCCLock__make_0::<CANDIDATEINFO>((*(*this_).__base0.m_pIC).hCandInfo);
        if !candInfo.__bool() {
            ImmDestroyIMCC((*(*this_).__base0.m_pIC).hCandInfo);
            (*(*this_).__base0.m_pIC).hCandInfo = ImmCreateIMCC((CUSTOM_CAND_INFO_SIZE) as DWORD);
            return (FALSE) as BOOL;
        }
    
        candInfo.get().dwSize = CUSTOM_CAND_INFO_SIZE as DWORD;
        candInfo.get().dwCount = 0;
        candInfo.get().dwOffset[0 as usize] = core::mem::size_of::<CANDIDATEINFO>() as _;
    
        // FIXME: Something is trailing after CANDIDATEINFO...
    
        return (TRUE) as BOOL;
    }
}
