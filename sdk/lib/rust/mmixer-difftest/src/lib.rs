//! mmixer, shipping Rust against the genuine C: the volume conversion (sup.c) here, the
//! topology walkers (topology.c) in `topology`.
//!
//! `MMixerSetGetVolumeControlDetails` maps the mixer API's 0..=0xFFFF volume (controls.c sets
//! the control's `Bounds.dwMaximum = 0xFFFF`) onto the driver's dB range and back. The set
//! path and the C-defined part of the get path are graded against the C; where the C's get
//! leaves the control's own bounds -- 0x10000 at full volume, a wrapped ULONG for a driver
//! value outside [Min, Max] -- the Rust clamps, and the test pins that the C still does not.
#![allow(non_snake_case, non_camel_case_types, non_upper_case_globals, unused_mut)]

#[cfg(test)]
mod topology;

#[cfg(test)]
mod tests {
    use std::{cell::RefCell, ffi::c_void};

    type LONG = i32;
    type PLONG = *mut i32;
    type ULONG = u32;
    type DWORD = u32;
    type INT64 = i64;
    type HANDLE = *mut c_void;
    type MIXER_STATUS = u32;
    const MM_STATUS_SUCCESS: u32 = 0;
    const MM_STATUS_INVALID_PARAMETER: u32 = 5;
    const MM_STATUS_UNSUCCESSFUL: u32 = 6;
    const KSPROPERTY_AUDIO_VOLUMELEVEL: u32 = 4;
    const MM_MIXM_CONTROL_CHANGE: u32 = 0x3D1;

    #[repr(C)]
    pub struct MIXER_CONTEXT {
        unused: i32,
    }
    type PMIXER_CONTEXT = *mut MIXER_CONTEXT;
    #[repr(C)]
    pub struct MIXER_INFO {
        unused: i32,
    }
    type LPMIXER_INFO = *mut MIXER_INFO;
    #[repr(C)]
    pub struct MIXERLINE_EXT {
        unused: i32,
    }
    type LPMIXERLINE_EXT = *mut MIXERLINE_EXT;
    #[repr(C)]
    pub struct MIXERCONTROLW {
        dwControlID: DWORD,
    }
    #[repr(C)]
    pub struct MIXERCONTROL_EXT {
        Control: MIXERCONTROLW,
        hDevice: HANDLE,
        ExtraData: *mut c_void,
    }
    type LPMIXERCONTROL_EXT = *mut MIXERCONTROL_EXT;
    #[repr(C)]
    pub struct MIXERCONTROLDETAILS {
        cChannels: DWORD,
        cbDetails: DWORD,
        paDetails: *mut c_void,
    }
    type LPMIXERCONTROLDETAILS = *mut MIXERCONTROLDETAILS;
    #[repr(C)]
    #[derive(Clone, Copy, Debug, PartialEq)]
    pub struct MIXERCONTROLDETAILS_UNSIGNED {
        dwValue: DWORD,
    }
    type LPMIXERCONTROLDETAILS_UNSIGNED = *mut MIXERCONTROLDETAILS_UNSIGNED;
    #[repr(C)]
    pub struct MIXERVOLUME_DATA {
        SignedMinimum: LONG,
        SignedMaximum: LONG,
    }
    type LPMIXERVOLUME_DATA = *mut MIXERVOLUME_DATA;

    /// What the driver reports on a get, per channel; what each set wrote, and the notifies.
    #[derive(Default)]
    struct Fixture {
        hardware: Vec<LONG>,
        sets: Vec<(ULONG, ULONG, LONG)>,
        notifies: Vec<(ULONG, ULONG)>,
        status: MIXER_STATUS,
    }
    thread_local! { static FIXTURE: RefCell<Fixture> = RefCell::default(); }

    #[no_mangle]
    unsafe extern "C" fn fixture_set_get(
        _ctx: PMIXER_CONTEXT,
        _device: HANDLE,
        node: ULONG,
        set: ULONG,
        property: ULONG,
        channel: ULONG,
        value: PLONG,
    ) -> MIXER_STATUS {
        assert_eq!(property, KSPROPERTY_AUDIO_VOLUMELEVEL);
        FIXTURE.with(|f| {
            let mut f = f.borrow_mut();
            if set != 0 {
                f.sets.push((node, channel, *value));
            } else {
                *value = f.hardware[channel as usize];
            }
            f.status
        })
    }
    #[no_mangle]
    unsafe extern "C" fn fixture_notify(_c: PMIXER_CONTEXT, _i: LPMIXER_INFO, kind: ULONG, id: ULONG) {
        FIXTURE.with(|f| f.borrow_mut().notifies.push((kind, id)));
    }
    unsafe fn MMixerSetGetControlDetails(
        c: PMIXER_CONTEXT,
        d: HANDLE,
        n: ULONG,
        s: ULONG,
        p: ULONG,
        ch: ULONG,
        v: PLONG,
    ) -> MIXER_STATUS {
        fixture_set_get(c, d, n, s, p, ch, v)
    }
    unsafe fn MMixerNotifyControlChange(c: PMIXER_CONTEXT, i: LPMIXER_INFO, k: ULONG, id: ULONG) {
        fixture_notify(c, i, k, id)
    }

    include!(concat!(env!("OUT_DIR"), "/volume_shipping.rs"));

    type Convert = unsafe extern "C" fn(
        PMIXER_CONTEXT,
        LPMIXER_INFO,
        ULONG,
        ULONG,
        LPMIXERCONTROL_EXT,
        LPMIXERCONTROLDETAILS,
        LPMIXERLINE_EXT,
    ) -> MIXER_STATUS;
    extern "C" {
        fn ref_MMixerSetGetVolumeControlDetails(
            c: PMIXER_CONTEXT,
            i: LPMIXER_INFO,
            n: ULONG,
            s: ULONG,
            x: LPMIXERCONTROL_EXT,
            d: LPMIXERCONTROLDETAILS,
            l: LPMIXERLINE_EXT,
        ) -> MIXER_STATUS;
    }

    #[derive(Debug, PartialEq)]
    struct Run {
        status: MIXER_STATUS,
        values: Vec<DWORD>,
        sets: Vec<(ULONG, ULONG, LONG)>,
        notifies: Vec<(ULONG, ULONG)>,
    }

    struct Case<'a> {
        range: Option<(LONG, LONG)>,
        set: bool,
        input: &'a [DWORD],
        hardware: &'a [LONG],
        cb_details: DWORD,
        null_details: bool,
    }

    unsafe fn run(f: Convert, case: &Case) -> Run {
        FIXTURE.with(|fx| {
            *fx.borrow_mut() = Fixture { hardware: case.hardware.to_vec(), ..Default::default() };
        });
        let mut volume = case.range.map(|(lo, hi)| MIXERVOLUME_DATA { SignedMinimum: lo, SignedMaximum: hi });
        let mut control = MIXERCONTROL_EXT {
            Control: MIXERCONTROLW { dwControlID: 77 },
            hDevice: std::ptr::null_mut(),
            ExtraData: volume.as_mut().map_or(std::ptr::null_mut(), |v| v as *mut _ as *mut c_void),
        };
        let mut values: Vec<MIXERCONTROLDETAILS_UNSIGNED> =
            case.input.iter().map(|&dwValue| MIXERCONTROLDETAILS_UNSIGNED { dwValue }).collect();
        let mut details = MIXERCONTROLDETAILS {
            cChannels: values.len() as DWORD,
            cbDetails: case.cb_details,
            paDetails: if case.null_details { std::ptr::null_mut() } else { values.as_mut_ptr().cast() },
        };
        let (mut ctx, mut info, mut line) = (MIXER_CONTEXT { unused: 0 }, MIXER_INFO { unused: 0 }, MIXERLINE_EXT { unused: 0 });
        let status = f(&mut ctx, &mut info, 9, case.set as ULONG, &mut control, &mut details, &mut line);
        let (sets, notifies) = FIXTURE.with(|fx| {
            let fx = fx.borrow();
            (fx.sets.clone(), fx.notifies.clone())
        });
        Run { status, values: values.iter().map(|v| v.dwValue).collect(), sets, notifies }
    }

    fn shipping() -> Convert {
        MMixerSetGetVolumeControlDetails
    }

    const RANGES: [(LONG, LONG); 6] = [
        (-96 * 0x10000, 0), // the C's own default (controls.c:202)
        (-6291456, 786432), // -96 dB .. +12 dB
        (-0x10000, 0x10000),
        (0, 100),
        (-100, -1),
        (1, 2),
    ];
    const LOGICAL: [DWORD; 9] = [0, 1, 2, 0x7FFF, 0x8000, 0x8001, 0xFFFD, 0xFFFE, 0xFFFF];

    /// The get the C means, kept inside the control's bounds (0..=0xFFFF).
    fn model_get(value: LONG, lo: LONG, hi: LONG) -> DWORD {
        let wide = ((value as i64 - lo as i64 + 1) * 0x10000) / (hi as i64 - lo as i64);
        wide.clamp(0, 0xFFFF) as DWORD
    }

    #[test]
    fn set_matches_c_for_every_range_and_channel_count() {
        unsafe {
            for (lo, hi) in RANGES {
                for channels in 1..=3 {
                    for &v in &LOGICAL {
                        let input: Vec<DWORD> = (0..channels).map(|c| v.wrapping_sub(c as u32 * 3) & 0xFFFF).collect();
                        let case = Case { range: Some((lo, hi)), set: true, input: &input, hardware: &[], cb_details: 4, null_details: false };
                        let c = run(ref_MMixerSetGetVolumeControlDetails, &case);
                        let rust = run(shipping(), &case);
                        assert_eq!(rust, c, "range {lo}..{hi} input {input:?}");
                        assert_eq!(rust.notifies, vec![(MM_MIXM_CONTROL_CHANGE, 77)]);
                    }
                }
            }
        }
    }

    #[test]
    fn get_matches_c_inside_the_bounds_and_clamps_where_c_leaves_them() {
        unsafe {
            for (lo, hi) in RANGES {
                let span = hi as i64 - lo as i64;
                let mut probes: Vec<LONG> = vec![lo, lo + 1, hi - 1, hi, lo - 1, hi + 1, lo - 0x10000, hi + 0x10000];
                probes.extend((1..16).map(|k| (lo as i64 + span * k / 16) as LONG));
                for value in probes {
                    let case = Case { range: Some((lo, hi)), set: false, input: &[0xDEAD], hardware: &[value], cb_details: 4, null_details: false };
                    let c = run(ref_MMixerSetGetVolumeControlDetails, &case);
                    let rust = run(shipping(), &case);
                    let ctx = format!("range {lo}..{hi} value {value}");
                    let want = model_get(value, lo, hi);
                    assert_eq!(rust.values, vec![want], "{ctx}");
                    assert_eq!((rust.status, &rust.sets, &rust.notifies), (c.status, &c.sets, &c.notifies), "{ctx}");
                    let wide = ((value as i64 - lo as i64 + 1) * 0x10000) / span;
                    if (0..=0xFFFF).contains(&wide) {
                        assert_eq!(rust.values, c.values, "the C is defined here: {ctx}");
                    } else {
                        assert_ne!(c.values, rust.values, "the C left the bounds here: {ctx}");
                    }
                }
            }
        }
    }

    #[test]
    fn full_volume_reads_back_one_past_the_maximum_in_c() {
        unsafe {
            let case = Case { range: Some((-96 * 0x10000, 0)), set: false, input: &[0], hardware: &[0], cb_details: 4, null_details: false };
            assert_eq!(run(ref_MMixerSetGetVolumeControlDetails, &case).values, [0x10000]);
            assert_eq!(run(shipping(), &case).values, [0xFFFF]);
        }
    }

    #[test]
    fn a_set_then_get_round_trips_like_the_c_wherever_the_c_stays_in_bounds() {
        unsafe {
            for (lo, hi) in RANGES {
                for &v in &LOGICAL {
                    let set = Case { range: Some((lo, hi)), set: true, input: &[v], hardware: &[], cb_details: 4, null_details: false };
                    let hardware = run(shipping(), &set).sets[0].2;
                    let get = Case { range: Some((lo, hi)), set: false, input: &[0], hardware: &[hardware], cb_details: 4, null_details: false };
                    let c = run(ref_MMixerSetGetVolumeControlDetails, &get).values[0];
                    let rust = run(shipping(), &get).values[0];
                    assert_eq!(rust, c.min(0xFFFF), "range {lo}..{hi} logical {v:#x}");
                }
            }
        }
    }

    #[test]
    fn refusals_match_c() {
        unsafe {
            let base = Case { range: Some((-96 * 0x10000, 0)), set: true, input: &[5], hardware: &[0], cb_details: 4, null_details: false };
            for (case, status) in [
                (Case { cb_details: 8, ..base }, MM_STATUS_INVALID_PARAMETER),
                (Case { range: None, input: &[5], hardware: &[0], cb_details: 4, null_details: false, set: true }, MM_STATUS_UNSUCCESSFUL),
                (Case { range: Some((-96 * 0x10000, 0)), set: true, input: &[5], hardware: &[0], cb_details: 4, null_details: true }, MM_STATUS_UNSUCCESSFUL),
            ] {
                let c = run(ref_MMixerSetGetVolumeControlDetails, &case);
                let rust = run(shipping(), &case);
                assert_eq!(rust, c);
                assert_eq!(rust.status, status);
                assert!(rust.sets.is_empty() && rust.notifies.is_empty());
            }
        }
    }
}
