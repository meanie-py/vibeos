use crate::*;
use core::ptr::{null, null_mut};

fn w(s: &str) -> Vec<u16> {
    s.encode_utf16().chain(Some(0)).collect()
}

fn create(ev: &str, ty: i32) -> (HRESULT, *mut c_void) {
    let p = w(ev);
    let mut h = 0xdead_beef_usize as *mut c_void;
    let r = unsafe { WerReportCreate(p.as_ptr(), ty, null(), &mut h) };
    (r, h)
}

#[test]
fn create_and_close() {
    let (hr, h) = create("APPCRASH", 1);
    assert_eq!(hr, S_OK);
    assert!(!h.is_null());
    unsafe {
        assert_eq!(WerReportCloseHandle(h), S_OK);
        assert_eq!(WerReportCloseHandle(h), E_INVALIDARG, "double close");
        assert_eq!(WerReportCloseHandle(null_mut()), E_INVALIDARG);
        assert_eq!(WerReportCloseHandle(0x1000 as *mut c_void), E_INVALIDARG);
    }
}

#[test]
fn create_validation() {
    unsafe {
        let ev = w("APPCRASH");
        // phandle NULL
        assert_eq!(WerReportCreate(ev.as_ptr(), 1, null(), null_mut()), E_INVALIDARG);
        // NULL / empty event type: handle nulled
        let mut h = 0xdead_beef_usize as *mut c_void;
        assert_eq!(WerReportCreate(null(), 1, null(), &mut h), E_INVALIDARG);
        assert!(h.is_null());
        h = 0xdead_beef_usize as *mut c_void;
        let e = w("");
        assert_eq!(WerReportCreate(e.as_ptr(), 1, null(), &mut h), E_INVALIDARG);
        assert!(h.is_null());
    }
    // every valid type (0..=4) works, 5 / 42 / negative do not, and the handle is nulled
    for t in 0..WER_REPORT_INVALID {
        let (hr, h) = create("APPCRASH", t);
        assert_eq!(hr, S_OK, "type {t}");
        unsafe { assert_eq!(WerReportCloseHandle(h), S_OK) };
    }
    for t in [WER_REPORT_INVALID, 42, -1] {
        let (hr, h) = create("APPCRASH", t);
        assert_eq!(hr, E_INVALIDARG, "type {t}");
        assert!(h.is_null());
    }
}

#[test]
fn many_reports_and_out_of_order_close() {
    let hs: Vec<_> = (0..8).map(|i| create(&format!("Evt{i}"), 1)).collect();
    assert!(hs.iter().all(|(hr, h)| *hr == S_OK && !h.is_null()));
    // close middle, then tail, then head, then the rest: exercises list unlinking
    for i in [3usize, 0, 7, 1, 2, 4, 5, 6] {
        unsafe { assert_eq!(WerReportCloseHandle(hs[i].1), S_OK, "close {i}") };
    }
    for (_, h) in &hs {
        unsafe { assert_eq!(WerReportCloseHandle(*h), E_INVALIDARG) };
    }
}

#[test]
fn report_copies_event_type_and_info() {
    unsafe {
        let mut info: WerReportInformation = core::mem::zeroed();
        info.dw_size = size_of::<WerReportInformation>() as u32;
        info.application_name[..3].copy_from_slice(&[b'A' as u16, b'p' as u16, b'p' as u16]);
        let ev = w("MyLongEventName");
        let mut h = null_mut();
        assert_eq!(WerReportCreate(ev.as_ptr(), 2, &info, &mut h), S_OK);
        let r = h as *mut Report;
        assert_eq!((*r).report_type, 2);
        assert_eq!((*r).info.application_name[1], b'p' as u16);
        let got = std::slice::from_raw_parts((*r).event_type.as_ptr(), ev.len());
        assert_eq!(got, &ev[..]);
        assert_eq!(WerReportCloseHandle(h), S_OK);
    }
}

#[test]
fn submit_reports_wer_disabled() {
    unsafe {
        let (_, h) = create("APPCRASH", 1);
        let mut res = 0;
        assert_eq!(WerReportSubmit(h, 2, 0, &mut res), S_OK);
        assert_eq!(res, 5, "WerDisabled");
        // NULL result pointer
        assert_eq!(WerReportSubmit(h, 2, 0, null_mut()), E_INVALIDARG);
        // as in Wine the handle is not validated
        res = 0;
        assert_eq!(WerReportSubmit(null_mut(), 1, 0, &mut res), S_OK);
        assert_eq!(res, 5);
        assert_eq!(WerReportCloseHandle(h), S_OK);
    }
}

#[test]
fn stubs_like_wine() {
    unsafe {
        let p = w("x");
        assert_eq!(WerReportAddFile(null_mut(), p.as_ptr(), 5, 0), S_OK);
        assert_eq!(WerReportSetParameter(null_mut(), 0, p.as_ptr(), p.as_ptr()), S_OK);
        assert_eq!(WerReportSetUIOption(null_mut(), 1, p.as_ptr()), E_NOTIMPL);
        assert_eq!(WerReportAddDump(null_mut(), null_mut(), null_mut(), 2, null_mut(), null_mut(), 0), E_NOTIMPL);
    }
}

#[test]
fn excluded_applications() {
    unsafe {
        let (add, rem) = (WerAddExcludedApplication, WerRemoveExcludedApplication);
        let n = w("winetest_wer.exe");
        // invalid names
        assert_eq!(add(null(), 0), E_INVALIDARG);
        assert_eq!(add(w("").as_ptr(), 0), E_INVALIDARG);
        assert_eq!(rem(null(), 0), E_INVALIDARG);
        assert_eq!(add(w("C:\\Windows\\").as_ptr(), 0), E_INVALIDARG);
        assert_eq!(rem(w("C:\\Windows\\").as_ptr(), 0), E_INVALIDARG);
        // add (twice: idempotent), remove, remove again
        assert_eq!(add(n.as_ptr(), 0), S_OK);
        assert_eq!(add(n.as_ptr(), 0), S_OK);
        assert_eq!(rem(n.as_ptr(), 0), S_OK);
        assert_eq!(rem(n.as_ptr(), 0), HR_ENVVAR_NOT_FOUND);
        // path form stores only the last component; hives are independent
        assert_eq!(add(w("C:\\Windows\\winetest_wer.exe").as_ptr(), 1), S_OK);
        assert_eq!(rem(n.as_ptr(), 0), HR_ENVVAR_NOT_FOUND);
        assert_eq!(rem(n.as_ptr(), 1), S_OK);
        // bare directory path: adds "Windows"
        assert_eq!(add(w("C:\\Windows").as_ptr(), 0), S_OK);
        assert_eq!(rem(w("D:\\x\\Windows").as_ptr(), 0), S_OK);
        // unwritable key
        *plat::DENY.lock().unwrap() = true;
        assert_eq!(add(n.as_ptr(), 0), E_ACCESSDENIED);
        assert_eq!(rem(n.as_ptr(), 0), E_ACCESSDENIED);
        *plat::DENY.lock().unwrap() = false;
    }
}

#[test]
fn exclude_key_string() {
    let s: String = String::from_utf16(&EXCLUDE_KEY[..EXCLUDE_KEY.len() - 1]).unwrap();
    assert_eq!(s, "Software\\Microsoft\\Windows Error Reporting\\ExcludedApplications");
    assert_eq!(*EXCLUDE_KEY.last().unwrap(), 0);
}
