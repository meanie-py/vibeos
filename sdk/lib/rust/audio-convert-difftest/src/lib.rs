//! The PCM converters kmixer.sys and wdmaud.drv each carry a copy of: the shipping Rust
//! against the genuine pin.c and mixer.c.
//!
//! The C is defined only for 8-bit channel up-mixing and 16 -> 32 quality conversion;
//! everywhere else it overruns its buffer or computes the wrong sample (the Rust's doc
//! comments say how). So those rows grade the Rust against the C, and the rest grade it
//! against an independent full-scale PCM model while pinning the C defect that the
//! divergence answers -- if the C is ever fixed, the test says so. Every allocation is
//! padded and canaried so the C's overruns are measured, not suffered.
#![allow(
    non_snake_case,
    non_camel_case_types,
    non_upper_case_globals,
    unused_mut,
    clippy::missing_safety_doc
)]

#[cfg(test)]
mod tests {
    use std::{
        alloc::{alloc, dealloc, Layout},
        cell::RefCell,
        ffi::c_void,
    };

    type ULONG = u32;
    type PULONG = *mut u32;
    type USHORT = u16;
    type UCHAR = u8;
    type PUCHAR = *mut u8;
    type SIZE_T = usize;
    type PVOID = *mut c_void;
    macro_rules! ASSERT {
        ($e:expr) => {
            assert!($e)
        };
    }
    macro_rules! DPRINT1 {
        ($($t:tt)*) => {};
    }

    const CANARY: u8 = 0xa5;

    struct Allocation {
        ptr: *mut u8,
        layout: Layout,
        requested: usize,
    }
    thread_local! { static ALLOCATIONS: RefCell<Vec<Allocation>> = RefCell::default(); }

    /// Allocation with 4x + 64 bytes of canaried slack: the widest C overrun (a 32-bit
    /// up-mix into a byte-sized buffer) writes 4x plus one sample.
    #[no_mangle]
    unsafe extern "C" fn fixture_allocate(pool: u32, size: usize) -> PVOID {
        assert_eq!(pool, 0);
        let layout = Layout::from_size_align(size * 4 + 64, 16).unwrap();
        let ptr = alloc(layout);
        assert!(!ptr.is_null());
        ptr.write_bytes(CANARY, layout.size());
        ALLOCATIONS.with(|s| s.borrow_mut().push(Allocation { ptr, layout, requested: size }));
        ptr.cast()
    }

    mod kmixer {
        use super::*;
        type NTSTATUS = i32;
        const NonPagedPool: u32 = 0;
        pub const STATUS_SUCCESS: i32 = 0;
        pub const STATUS_NOT_IMPLEMENTED: i32 = 0xc0000002u32 as i32;
        pub const STATUS_INVALID_PARAMETER: i32 = 0xc000000du32 as i32;
        pub const STATUS_INSUFFICIENT_RESOURCES: i32 = 0xc000009au32 as i32;
        unsafe fn ExAllocatePool(pool: u32, size: SIZE_T) -> PVOID {
            fixture_allocate(pool, size)
        }
        include!(concat!(env!("OUT_DIR"), "/kmixer_shipping.rs"));
    }

    mod wdmaud {
        use super::*;
        type DWORD = u32;
        pub const ERROR_SUCCESS: u32 = 0;
        pub const ERROR_NOT_ENOUGH_MEMORY: u32 = 8;
        pub const ERROR_NOT_SUPPORTED: u32 = 50;
        pub const ERROR_INVALID_PARAMETER: u32 = 87;
        unsafe fn GetProcessHeap() -> PVOID {
            std::ptr::null_mut()
        }
        unsafe fn HeapAlloc(heap: PVOID, flags: u32, size: SIZE_T) -> PVOID {
            assert!(heap.is_null() && flags == 0);
            fixture_allocate(0, size)
        }
        include!(concat!(env!("OUT_DIR"), "/wdmaud_shipping.rs"));
    }

    /// A status, as the outcome both copies' codes name.
    #[derive(Clone, Copy, Debug, PartialEq)]
    enum Status {
        Success,
        NotImplemented,
        Invalid,
        NoMemory,
        Other(u32),
    }

    type Channel = unsafe extern "C" fn(PUCHAR, ULONG, ULONG, ULONG, ULONG, *mut PVOID, PULONG) -> u32;
    type Quality = unsafe extern "C" fn(PUCHAR, ULONG, ULONG, ULONG, *mut PVOID, PULONG) -> u32;

    /// One copy of the converters: shipping Rust, its genuine C as amd64/arm64 and as
    /// i386 compile it, and what its status codes mean.
    struct Copy {
        name: &'static str,
        channel: Channel,
        quality: Quality,
        c_channel: Channel,
        c_quality: Quality,
        c_x86_quality: Quality,
        status: fn(u32) -> Status,
    }

    macro_rules! c_refs {
        ($($f:ident),*) => { extern "C" { $(fn $f();)* } };
    }
    c_refs!(
        ref_kmixer_PerformChannelConversion,
        ref_kmixer_PerformQualityConversion,
        ref_kmixer_x86_PerformQualityConversion,
        ref_wdmaud_PerformChannelConversion,
        ref_wdmaud_PerformQualityConversion,
        ref_wdmaud_x86_PerformQualityConversion
    );

    /// NTSTATUS and DWORD are both 32 bits returned in the same register; the C side is
    /// declared above only by name, so its real prototype is the one pinned here.
    fn copies() -> [Copy; 2] {
        unsafe {
            use std::mem::transmute as t;
            [
                Copy {
                    name: "kmixer",
                    channel: t(kmixer::PerformChannelConversion as unsafe extern "C" fn(_, _, _, _, _, _, _) -> _),
                    quality: t(kmixer::PerformQualityConversion as unsafe extern "C" fn(_, _, _, _, _, _) -> _),
                    c_channel: t(ref_kmixer_PerformChannelConversion as unsafe extern "C" fn()),
                    c_quality: t(ref_kmixer_PerformQualityConversion as unsafe extern "C" fn()),
                    c_x86_quality: t(ref_kmixer_x86_PerformQualityConversion as unsafe extern "C" fn()),
                    status: |s| match s as i32 {
                        kmixer::STATUS_SUCCESS => Status::Success,
                        kmixer::STATUS_NOT_IMPLEMENTED => Status::NotImplemented,
                        kmixer::STATUS_INVALID_PARAMETER => Status::Invalid,
                        kmixer::STATUS_INSUFFICIENT_RESOURCES => Status::NoMemory,
                        _ => Status::Other(s),
                    },
                },
                Copy {
                    name: "wdmaud",
                    channel: t(wdmaud::PerformChannelConversion as unsafe extern "C" fn(_, _, _, _, _, _, _) -> _),
                    quality: t(wdmaud::PerformQualityConversion as unsafe extern "C" fn(_, _, _, _, _, _) -> _),
                    c_channel: t(ref_wdmaud_PerformChannelConversion as unsafe extern "C" fn()),
                    c_quality: t(ref_wdmaud_PerformQualityConversion as unsafe extern "C" fn()),
                    c_x86_quality: t(ref_wdmaud_x86_PerformQualityConversion as unsafe extern "C" fn()),
                    status: |s| match s {
                        wdmaud::ERROR_SUCCESS => Status::Success,
                        wdmaud::ERROR_NOT_SUPPORTED => Status::NotImplemented,
                        wdmaud::ERROR_INVALID_PARAMETER => Status::Invalid,
                        wdmaud::ERROR_NOT_ENOUGH_MEMORY => Status::NoMemory,
                        _ => Status::Other(s),
                    },
                },
            ]
        }
    }

    #[derive(Debug, PartialEq)]
    struct Run {
        status: Status,
        /// None: `*Result` was not written.
        result_length: Option<u32>,
        /// Requested size of each allocation, in order.
        requested: Vec<usize>,
        /// The returned buffer, its requested size only.
        bytes: Vec<u8>,
        /// Bytes past the requested size that the call wrote.
        overrun: usize,
    }

    fn sentinel() -> PVOID {
        std::ptr::NonNull::<c_void>::dangling().as_ptr()
    }

    unsafe fn finish(status: Status, result: PVOID, result_length: u32) -> Run {
        ALLOCATIONS.with(|s| {
            let mut run = Run {
                status,
                result_length: (result != sentinel()).then_some(result_length),
                requested: Vec::new(),
                bytes: Vec::new(),
                overrun: 0,
            };
            for a in s.borrow_mut().drain(..) {
                let all = std::slice::from_raw_parts(a.ptr, a.layout.size());
                run.requested.push(a.requested);
                run.overrun += all[a.requested..].iter().rposition(|&b| b != CANARY).map_or(0, |i| i + 1);
                if a.ptr.cast() == result {
                    run.bytes = all[..a.requested].to_vec();
                }
                dealloc(a.ptr, a.layout);
            }
            run
        })
    }

    impl Copy {
        /// The input sits in a padded buffer too: the C reads at unscaled offsets.
        unsafe fn channel(&self, f: Channel, input: &[u8], old: u32, new: u32, bits: u32) -> Run {
            let mut buffer = input.to_vec();
            buffer.resize(input.len() * 4 + 64, 0);
            let (mut result, mut length) = (sentinel(), 0xdead_beef);
            let status = f(buffer.as_mut_ptr(), input.len() as u32, old, new, bits, &mut result, &mut length);
            finish((self.status)(status), result, length)
        }

        unsafe fn quality(&self, f: Quality, input: &[u8], old: u32, new: u32) -> Run {
            let mut buffer = input.to_vec();
            buffer.resize(input.len() + 64, 0);
            let (mut result, mut length) = (sentinel(), 0xdead_beef);
            let status = f(buffer.as_mut_ptr(), input.len() as u32, old, new, &mut result, &mut length);
            finish((self.status)(status), result, length)
        }
    }

    /// Deterministic bytes; xorshift so every lane differs.
    fn data(len: usize, seed: u32) -> Vec<u8> {
        let mut x = seed.wrapping_mul(2654435761) | 1;
        (0..len)
            .map(|_| {
                x ^= x << 13;
                x ^= x >> 17;
                x ^= x << 5;
                x as u8
            })
            .collect()
    }

    /// Output channel c of every frame is input channel c % old.
    fn channel_model(input: &[u8], old: usize, new: usize, width: usize) -> Vec<u8> {
        let frames = input.len() / width / old;
        let mut out = Vec::new();
        for f in 0..frames {
            for c in 0..new {
                let at = (f * old + c % old) * width;
                out.extend_from_slice(&input[at..at + width]);
            }
        }
        out
    }

    #[test]
    fn eight_bit_upmix_matches_c_and_drops_its_trailing_overrun() {
        for copy in copies() {
            unsafe {
                for old in 1..=3u32 {
                    for new in old + 1..=6 {
                        for len in 0..=40usize {
                            let input = data(len, len as u32 * 31 + old * 7 + new);
                            let c = copy.channel(copy.c_channel, &input, old, new, 8);
                            let rust = copy.channel(copy.channel, &input, old, new, 8);
                            let ctx = format!("{} old={old} new={new} len={len}", copy.name);
                            assert_eq!(rust.status, Status::Success, "{ctx}");
                            assert_eq!(
                                (rust.status, rust.result_length, &rust.requested, &rust.bytes),
                                (c.status, c.result_length, &c.requested, &c.bytes),
                                "{ctx}"
                            );
                            assert_eq!(rust.overrun, 0, "{ctx}");
                            // The C's do/while fills one channel too many: on the last frame
                            // that is one byte past its allocation.
                            assert_eq!(c.overrun, usize::from(len / old as usize > 0), "{ctx}");
                        }
                    }
                }
            }
        }
    }

    #[test]
    fn wider_and_downmixed_frames_follow_the_frame_model_where_c_overruns() {
        for copy in copies() {
            unsafe {
                for width in [1usize, 2, 3, 4] {
                    for old in 1..=4u32 {
                        for new in (1..=6u32).filter(|&n| n != old) {
                            for frames in [0usize, 1, 2, 7] {
                                for extra in [0usize, 1] {
                                    let len = frames * old as usize * width + extra * usize::from(width > 1);
                                    let input = data(len, (width * 1000 + frames) as u32 + old * 10 + new);
                                    let bits = width as u32 * 8;
                                    let ctx = format!("{} bits={bits} old={old} new={new} len={len}", copy.name);
                                    let rust = copy.channel(copy.channel, &input, old, new, bits);
                                    let want = channel_model(&input, old as usize, new as usize, width);
                                    assert_eq!(rust.status, Status::Success, "{ctx}");
                                    assert_eq!(rust.requested, vec![want.len()], "{ctx}");
                                    assert_eq!(rust.result_length, Some(want.len() as u32), "{ctx}");
                                    assert_eq!(rust.bytes, want, "{ctx}");
                                    assert_eq!(rust.overrun, 0, "{ctx}");
                                    if width == 1 && new > old {
                                        continue; // graded against the C above
                                    }
                                    let c = copy.channel(copy.c_channel, &input, old, new, bits);
                                    // Every other C path sizes the buffer in samples, not bytes.
                                    assert_eq!(c.requested, vec![frames * new as usize], "{ctx}");
                                    assert_eq!(c.result_length, Some((frames * new as usize) as u32), "{ctx}");
                                    // One 8-bit frame lands where it should; the stride is what is wrong.
                                    if frames > 1 || (frames == 1 && width > 1) {
                                        assert!(c.overrun > 0 || c.bytes != want, "the C is right here now: {ctx}");
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }
    }

    #[test]
    fn channel_conversion_refuses_what_it_cannot_do() {
        for copy in copies() {
            unsafe {
                let input = data(48, 5);
                for bits in [0u32, 4, 12, 20, 64] {
                    let rust = copy.channel(copy.channel, &input, 1, 2, bits);
                    assert_eq!(
                        (rust.status, rust.result_length, rust.requested.len()),
                        (Status::NotImplemented, None, 0),
                        "{} bits={bits}",
                        copy.name
                    );
                }
                // The C answers an up-mix of an unhandled width with success and no result.
                let c = copy.channel(copy.c_channel, &input, 1, 2, 12);
                assert_eq!((c.status, c.result_length), (Status::Success, None), "{}", copy.name);
                for (old, new) in [(0u32, 2u32), (2, 0)] {
                    let rust = copy.channel(copy.channel, &input, old, new, 16);
                    assert_eq!((rust.status, rust.result_length), (Status::Invalid, None), "{}", copy.name);
                }
            }
        }
    }

    /// A sample as a full-scale signed 32-bit value: 8-bit PCM is unsigned around 0x80.
    fn widen(input: &[u8], width: usize, i: usize) -> i32 {
        let s = &input[i * width..(i + 1) * width];
        match width {
            1 => (s[0] as i32 - 128) << 24,
            2 => (i16::from_le_bytes([s[0], s[1]]) as i32) << 16,
            _ => i32::from_le_bytes([s[0], s[1], s[2], s[3]]),
        }
    }
    fn narrow(v: i32, width: usize) -> Vec<u8> {
        match width {
            1 => vec![((v >> 24) + 128) as u8],
            2 => ((v >> 16) as i16).to_le_bytes().to_vec(),
            _ => v.to_le_bytes().to_vec(),
        }
    }
    fn quality_model(input: &[u8], old: usize, new: usize) -> Vec<u8> {
        (0..input.len() / old).flat_map(|i| narrow(widen(input, old, i), new)).collect()
    }

    const PAIRS: [(u32, u32); 6] = [(8, 16), (8, 32), (16, 8), (16, 32), (32, 8), (32, 16)];

    #[test]
    fn sixteen_to_thirty_two_matches_c() {
        for copy in copies() {
            unsafe {
                for len in 0..=67usize {
                    let input = data(len, len as u32 + 1);
                    let c = copy.quality(copy.c_quality, &input, 16, 32);
                    let rust = copy.quality(copy.quality, &input, 16, 32);
                    assert_eq!(rust, c, "{} len={len}", copy.name);
                    assert_eq!(rust.overrun, 0);
                }
            }
        }
    }

    #[test]
    fn every_pair_follows_the_full_scale_model() {
        for copy in copies() {
            unsafe {
                for (old, new) in PAIRS {
                    for len in (0..=33usize).chain([255, 256, 1027]) {
                        let input = data(len, old * 100 + new + len as u32);
                        let ctx = format!("{} {old}->{new} len={len}", copy.name);
                        let rust = copy.quality(copy.quality, &input, old, new);
                        let want = quality_model(&input, old as usize / 8, new as usize / 8);
                        assert_eq!(rust.status, Status::Success, "{ctx}");
                        assert_eq!(rust.bytes, want, "{ctx}");
                        assert_eq!(rust.result_length, Some(want.len() as u32), "{ctx}");
                        assert_eq!(rust.overrun, 0, "{ctx}");
                        // Same buffer size, length and status as the C on every pair; only
                        // the samples differ.
                        let c = copy.quality(copy.c_quality, &input, old, new);
                        assert_eq!(
                            (c.status, &c.requested, c.result_length),
                            (rust.status, &rust.requested, rust.result_length),
                            "{ctx}"
                        );
                    }
                }
            }
        }
    }

    #[test]
    fn the_c_defects_the_divergence_answers_are_still_there() {
        for copy in copies() {
            unsafe {
                // Silence in, silence out. The C keeps it only for 16 -> 32 and 32 -> 16
                // (where it outputs nothing but silence).
                let silence = |w: u32| -> Vec<u8> {
                    match w {
                        8 => vec![0x80; 8],
                        16 => vec![0; 16],
                        _ => vec![0; 32],
                    }
                };
                for (old, new) in PAIRS {
                    let rust = copy.quality(copy.quality, &silence(old), old, new);
                    let c = copy.quality(copy.c_quality, &silence(old), old, new);
                    let ctx = format!("{} {old}->{new}", copy.name);
                    assert_eq!(rust.bytes, silence(new)[..rust.bytes.len()], "{ctx}");
                    assert_eq!(c.bytes == rust.bytes, matches!((old, new), (16, 32) | (32, 16)), "{ctx}");
                }
                // 32 -> 16 keeps the high half; the C drops it into a USHORT first.
                let loud = 0x7fff_0000u32.to_le_bytes();
                assert_eq!(copy.quality(copy.c_quality, &loud, 32, 16).bytes, [0, 0]);
                assert_eq!(copy.quality(copy.quality, &loud, 32, 16).bytes, [0xff, 0x7f]);
                // i386's _X86_ byte swap makes the same stream come out differently there.
                let input = data(8, 3);
                let amd64 = copy.quality(copy.c_quality, &input, 16, 32);
                let i386 = copy.quality(copy.c_x86_quality, &input, 16, 32);
                assert_ne!(amd64.bytes, i386.bytes, "{}", copy.name);
            }
        }
    }

    #[test]
    fn unsupported_pairs_refuse_like_the_c() {
        for copy in copies() {
            unsafe {
                let input = data(24, 9);
                for (old, new) in [(8u32, 24u32), (24, 16), (16, 24), (32, 64), (64, 32)] {
                    let c = copy.quality(copy.c_quality, &input, old, new);
                    let rust = copy.quality(copy.quality, &input, old, new);
                    assert_eq!(rust, c, "{} {old}->{new}", copy.name);
                    assert_eq!((rust.status, rust.result_length), (Status::NotImplemented, None));
                }
                // The C divides by zero below 8 bits; the Rust refuses first.
                for (old, new) in [(0u32, 8u32), (4, 8)] {
                    let rust = copy.quality(copy.quality, &input, old, new);
                    assert_eq!(rust.status, Status::NotImplemented, "{} {old}->{new}", copy.name);
                }
            }
        }
    }
}
