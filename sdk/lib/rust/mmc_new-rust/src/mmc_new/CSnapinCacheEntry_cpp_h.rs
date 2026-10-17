use crate::prelude::*;
use core::ffi::c_void;

impl CSnapinCacheEntry {
    pub unsafe fn __ctor_0(&mut self, guidString: &CStringW, aboutGuidString: &CStringW, name: &CStringW, hImageList: HIMAGELIST) {
        let this_: *mut Self = self;
        (*this_).m_GuidString = CStringW::from_ptr(guidString.GetString());
        (*this_).m_AboutGuidString = CStringW::from_ptr(aboutGuidString.GetString());
        (*this_).m_Name = CStringW::from_ptr(name.GetString());
        (*this_).m_Loaded = (FALSE as i32);
        {
            CLSIDFromString((*this_).m_GuidString.GetString(), core::ptr::addr_of_mut!((*this_).m_Guid));
            CLSIDFromString((*this_).m_AboutGuidString.GetString(), core::ptr::addr_of_mut!((*this_).m_AboutGuid));

            let mut pSnapinAbout: CComPtr<ISnapinAbout> = CComPtr::new();
            let mut hr: HRESULT = CoCreateInstance(
                core::ptr::addr_of!((*this_).m_AboutGuid),
                core::ptr::null_mut(),
                CLSCTX_INPROC,
                &IID_ISnapinAbout,
                pSnapinAbout.GetAddressOf() as *mut *mut core::ffi::c_void,
            );
            DPRINT!("CoCreateInstance %lx\n", hr);
            if (hr) >= 0 {
                let mut Str: LPOLESTR = core::ptr::null_mut();

                hr = (*pSnapinAbout.Get()).GetProvider(core::ptr::addr_of_mut!(Str));
                DPRINT!("GetProvider %lx\n", hr);
                if (hr) >= 0 {
                    (*this_).m_Provider = CStringW::from_ptr(Str);
                    CoTaskMemFree(Str as *mut core::ffi::c_void);
                }

                hr = (*pSnapinAbout.Get()).GetSnapinDescription(core::ptr::addr_of_mut!(Str));
                DPRINT!("GetDescription %lx\n", hr);
                if (hr) >= 0 {
                    (*this_).m_Description = CStringW::from_ptr(Str);
                    CoTaskMemFree(Str as *mut core::ffi::c_void);
                }

                hr = (*pSnapinAbout.Get()).GetSnapinVersion(core::ptr::addr_of_mut!(Str));
                DPRINT!("GetVersion %lx\n", hr);
                if (hr) >= 0 {
                    (*this_).m_Version = CStringW::from_ptr(Str);
                    CoTaskMemFree(Str as *mut core::ffi::c_void);
                }

                let mut hSmallImage: HBITMAP = core::ptr::null_mut();
                let mut hSmallImageOpen: HBITMAP = core::ptr::null_mut();
                let mut hLargeImage: HBITMAP = core::ptr::null_mut();
                let mut colorMask: COLORREF = 0;

                hr = (*pSnapinAbout.Get()).GetStaticFolderImage(
                    core::ptr::addr_of_mut!(hSmallImage),
                    core::ptr::addr_of_mut!(hSmallImageOpen),
                    core::ptr::addr_of_mut!(hLargeImage),
                    core::ptr::addr_of_mut!(colorMask),
                );
                if (hr) >= 0 {
                    (*this_).m_ImageNormal = ImageList_AddMasked(
                        hImageList,
                        hSmallImage,
                        colorMask,
                    );

                    (*this_).m_ImageOpen = ImageList_AddMasked(
                        hImageList,
                        hSmallImageOpen,
                        colorMask,
                    );

                    DeleteObject(hSmallImage as _);
                    DeleteObject(hSmallImageOpen as _);
                    DeleteObject(hLargeImage as _);
                }

                hr = (*pSnapinAbout.Get()).GetSnapinImage(core::ptr::addr_of_mut!((*this_).m_hIcon));
            }
        }
    }
    pub unsafe fn __dtor__body(&mut self) {
        let this_: *mut Self = self;
        DeleteObject(((*this_).m_hIcon).cast());
    }
}
