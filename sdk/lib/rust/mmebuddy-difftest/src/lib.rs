//! mmebuddy's wave streaming engine -- DoWaveStreaming commits each queued header in
//! FrameSize chunks, CompleteIO counts the completions back and returns finished headers
//! -- the shipping Rust against the genuine wave/streaming.c.
//!
//! One scripted world drives either side: a committer that queues every chunk FIFO, as
//! WriteFileEx completes them, and a CompleteWaveHeader that unlinks and records. Plain
//! headers grade the Rust against the C and an independent model of what a waveOut
//! client hears. Looped headers (WHDR_BEGINLOOP|WHDR_ENDLOOP) grade the Rust against the
//! model only, and pin the C defects the Rust's divergence comments describe -- a finite
//! loop that never runs down, a repeat played after the NEXT header -- so if the C is
//! ever fixed, the test says so.
#![allow(
    non_snake_case,
    non_camel_case_types,
    non_upper_case_globals,
    unused_mut,
    unused_assignments,
    clippy::missing_safety_doc
)]

#[cfg(test)]
mod tests {
    use std::{cell::RefCell, collections::VecDeque, ffi::c_void};

    type DWORD = u32;
    type BOOL = i32;
    type MMRESULT = u32;
    type MMDEVICE_TYPE = u8;
    type PVOID = *mut c_void;
    type LPVOID = *mut c_void;
    type DWORD_PTR = usize;
    const FALSE: BOOL = 0;
    const INFINITE: DWORD = 0xFFFF_FFFF;
    const MMSYSERR_NOERROR: MMRESULT = 0;
    const WHDR_DONE: DWORD = 0x01;
    const WHDR_PREPARED: DWORD = 0x02;
    const WHDR_BEGINLOOP: DWORD = 0x04;
    const WHDR_ENDLOOP: DWORD = 0x08;
    const WHDR_INQUEUE: DWORD = 0x10;
    const WAVE_IN_DEVICE_TYPE: u32 = 1;
    const WAVE_OUT_DEVICE_TYPE: u32 = 2;

    #[repr(C)]
    struct WAVEHDR {
        lpData: *mut i8,
        dwBufferLength: DWORD,
        dwBytesRecorded: DWORD,
        dwUser: DWORD_PTR,
        dwFlags: DWORD,
        dwLoops: DWORD,
        lpNext: *mut WAVEHDR,
        reserved: DWORD_PTR,
    }
    type PWAVEHDR = *mut WAVEHDR;

    #[repr(C)]
    #[derive(Default)]
    struct WAVEHDR_EXTENSION {
        BytesCommitted: DWORD,
        BytesCompleted: DWORD,
        LoopsToCommit: DWORD,
        LoopsToComplete: DWORD,
    }
    type PWAVEHDR_EXTENSION = *mut WAVEHDR_EXTENSION;

    #[repr(C)]
    struct OVERLAPPED {
        Internal: DWORD_PTR,
        InternalHigh: DWORD_PTR,
        Offset: DWORD,
        OffsetHigh: DWORD,
        hEvent: PVOID,
    }
    type LPOVERLAPPED = *mut OVERLAPPED;
    type LPOVERLAPPED_COMPLETION_ROUTINE =
        Option<unsafe extern "system" fn(DWORD, DWORD, LPOVERLAPPED)>;

    // The bindgen island spells the C's anonymous one-member unions like this.
    #[repr(C)]
    union Head {
        HeadWaveHeader: PWAVEHDR,
    }
    #[repr(C)]
    union Tail {
        TailWaveHeader: PWAVEHDR,
    }
    #[repr(C)]
    struct SOUND_DEVICE_INSTANCE {
        __bindgen_anon_1: Head,
        __bindgen_anon_2: Tail,
        OutstandingBuffers: DWORD,
        LoopsRemaining: DWORD,
        FrameSize: DWORD,
        BufferCount: DWORD,
        ResetInProgress: BOOL,
        bPaused: BOOL,
    }
    type PSOUND_DEVICE_INSTANCE = *mut SOUND_DEVICE_INSTANCE;

    #[repr(C)]
    struct SOUND_OVERLAPPED {
        Standard: OVERLAPPED,
        SoundDeviceInstance: PSOUND_DEVICE_INSTANCE,
        Header: PWAVEHDR,
        OriginalCompletionRoutine: LPOVERLAPPED_COMPLETION_ROUTINE,
        CompletionContext: PVOID,
    }
    type PSOUND_OVERLAPPED = *mut SOUND_OVERLAPPED;

    type PSOUND_DEVICE = *mut c_void;
    type WAVE_COMMIT_FUNC = Option<
        unsafe extern "C" fn(
            PSOUND_DEVICE_INSTANCE,
            PVOID,
            DWORD,
            PSOUND_OVERLAPPED,
            LPOVERLAPPED_COMPLETION_ROUTINE,
        ) -> MMRESULT,
    >;
    #[repr(C)]
    struct MMFUNCTION_TABLE {
        CommitWaveBuffer: WAVE_COMMIT_FUNC,
    }
    type PMMFUNCTION_TABLE = *mut MMFUNCTION_TABLE;

    macro_rules! SND_ASSERT {
        ($e:expr) => {
            let _ = $e;
        };
    }
    macro_rules! SND_TRACE {
        ($($a:expr),* $(,)?) => {{ $(let _ = $a;)* }};
    }
    macro_rules! SND_WARN {
        ($($a:expr),* $(,)?) => {{ $(let _ = $a;)* }};
    }
    macro_rules! MMSUCCESS {
        ($p0:expr $(,)?) => {
            (($p0 == 0) as i32)
        };
    }
    macro_rules! AllocateStruct {
        ($size:expr) => {
            fx_AllocateMemory($size)
        };
    }
    macro_rules! ZeroMemory {
        ($p:expr, $n:expr) => {
            core::ptr::write_bytes($p as *mut u8, 0, $n)
        };
    }

    /// The shipping mmebuddy-rust streaming functions, verbatim.
    #[allow(unused_variables, dead_code)]
    mod shipping {
        use super::*;
        include!(concat!(env!("OUT_DIR"), "/streaming_shipping.rs"));
    }

    extern "C" {
        fn ref_DoWaveStreaming(instance: PSOUND_DEVICE_INSTANCE);
        fn ref_CompleteIO(error: DWORD, bytes: DWORD, overlapped: LPOVERLAPPED);
    }

    // ---- the scripted world ------------------------------------------------------------

    struct Pending {
        overlap: LPOVERLAPPED,
        header: PWAVEHDR,
        length: DWORD,
        routine: LPOVERLAPPED_COMPLETION_ROUTINE,
    }
    #[derive(Default)]
    struct World {
        device_type: u8,
        table: Option<Box<MMFUNCTION_TABLE>>,
        pending: VecDeque<Pending>,
        played: Vec<(usize, u32, u32)>,
        done: Vec<usize>,
        errors: Vec<String>,
    }
    thread_local! { static WORLD: RefCell<World> = RefCell::default(); }

    unsafe fn index(header: PWAVEHDR) -> usize {
        (*header).dwUser
    }

    unsafe fn GetSoundDeviceFromInstance(_: PSOUND_DEVICE_INSTANCE, device: *mut PSOUND_DEVICE) -> MMRESULT {
        *device = 1 as PSOUND_DEVICE;
        MMSYSERR_NOERROR
    }
    unsafe fn GetSoundDeviceType(_: PSOUND_DEVICE, kind: *mut MMDEVICE_TYPE) -> MMRESULT {
        *kind = WORLD.with(|w| w.borrow().device_type);
        MMSYSERR_NOERROR
    }
    unsafe fn GetSoundDeviceFunctionTable(_: PSOUND_DEVICE, table: *mut PMMFUNCTION_TABLE) -> MMRESULT {
        *table = WORLD.with(|w| {
            let mut w = w.borrow_mut();
            let t = w.table.get_or_insert_with(|| {
                Box::new(MMFUNCTION_TABLE { CommitWaveBuffer: Some(fx_commit) })
            });
            &mut **t as PMMFUNCTION_TABLE
        });
        MMSYSERR_NOERROR
    }
    /// header.c's CompleteWaveHeader: unlink (head or relink), mark done, record.
    unsafe fn CompleteWaveHeader(instance: PSOUND_DEVICE_INSTANCE, header: PWAVEHDR) {
        let inst = &mut *instance;
        let (mut prev, mut cur) = (core::ptr::null_mut::<WAVEHDR>(), inst.__bindgen_anon_1.HeadWaveHeader);
        while !cur.is_null() && cur != header {
            prev = cur;
            cur = (*cur).lpNext;
        }
        if cur.is_null() {
            let msg = format!("CompleteWaveHeader: header {} is not queued", index(header));
            WORLD.with(|w| w.borrow_mut().errors.push(msg));
            return;
        }
        if prev.is_null() {
            inst.__bindgen_anon_1.HeadWaveHeader = (*header).lpNext;
        } else {
            (*prev).lpNext = (*header).lpNext;
        }
        if (*header).lpNext.is_null() {
            inst.__bindgen_anon_2.TailWaveHeader = prev;
        }
        (*header).dwFlags &= !WHDR_INQUEUE;
        (*header).dwFlags |= WHDR_DONE;
        WORLD.with(|w| w.borrow_mut().done.push(index(header)));
    }
    unsafe fn FreeMemory(p: PVOID) {
        fx_FreeMemory(p)
    }

    #[no_mangle]
    unsafe extern "C" fn fx_GetSoundDeviceFromInstance(i: PSOUND_DEVICE_INSTANCE, d: *mut PSOUND_DEVICE) -> MMRESULT {
        GetSoundDeviceFromInstance(i, d)
    }
    #[no_mangle]
    unsafe extern "C" fn fx_GetSoundDeviceType(d: PSOUND_DEVICE, k: *mut MMDEVICE_TYPE) -> MMRESULT {
        GetSoundDeviceType(d, k)
    }
    #[no_mangle]
    unsafe extern "C" fn fx_GetSoundDeviceFunctionTable(d: PSOUND_DEVICE, t: *mut PMMFUNCTION_TABLE) -> MMRESULT {
        GetSoundDeviceFunctionTable(d, t)
    }
    #[no_mangle]
    unsafe extern "C" fn fx_CompleteWaveHeader(i: PSOUND_DEVICE_INSTANCE, h: PWAVEHDR) {
        CompleteWaveHeader(i, h)
    }
    #[no_mangle]
    unsafe extern "C" fn fx_AllocateMemory(size: usize) -> PVOID {
        libc_calloc(size)
    }
    #[no_mangle]
    unsafe extern "C" fn fx_FreeMemory(p: PVOID) {
        drop(Box::from_raw(p as *mut [u64; 16]));
    }
    unsafe fn libc_calloc(size: usize) -> PVOID {
        assert!(size <= core::mem::size_of::<[u64; 16]>());
        Box::into_raw(Box::new([0u64; 16])) as PVOID
    }

    /// The committer: queue the chunk for FIFO completion, as WriteFileEx does.
    unsafe extern "C" fn fx_commit(
        _: PSOUND_DEVICE_INSTANCE,
        offset: PVOID,
        length: DWORD,
        overlap: PSOUND_OVERLAPPED,
        routine: LPOVERLAPPED_COMPLETION_ROUTINE,
    ) -> MMRESULT {
        let header = (*overlap).Header;
        let at = (offset as usize - (*header).lpData as usize) as u32;
        WORLD.with(|w| {
            let mut w = w.borrow_mut();
            w.played.push((index(header), at, length));
            w.pending.push_back(Pending { overlap: overlap as LPOVERLAPPED, header, length, routine });
        });
        MMSYSERR_NOERROR
    }

    // ---- driving a side ------------------------------------------------------------------

    #[derive(Clone, Copy, PartialEq)]
    enum Side {
        C,
        Rust,
    }

    #[derive(Clone)]
    struct Scenario {
        device: u32,
        frame: u32,
        buffers: u32,
        /// (dwBufferLength, loop flags, dwLoops)
        headers: Vec<(u32, DWORD, DWORD)>,
        steps: usize,
    }

    #[derive(Debug, PartialEq)]
    struct Outcome {
        played: Vec<(usize, u32, u32)>,
        done: Vec<usize>,
        errors: Vec<String>,
        /// Every committed chunk completed within the step budget.
        drained: bool,
        outstanding: u32,
    }

    unsafe fn stream(side: Side, instance: PSOUND_DEVICE_INSTANCE) {
        match side {
            Side::C => ref_DoWaveStreaming(instance),
            Side::Rust => shipping::DoWaveStreaming(instance),
        }
    }

    fn run(side: Side, sc: &Scenario) -> Outcome {
        WORLD.with(|w| {
            *w.borrow_mut() = World { device_type: sc.device as u8, ..World::default() };
        });
        let mut data: Vec<Vec<i8>> = sc.headers.iter().map(|h| vec![0; h.0 as usize]).collect();
        let mut extensions: Vec<Box<WAVEHDR_EXTENSION>> =
            sc.headers.iter().map(|_| Box::default()).collect();
        let mut headers: Vec<Box<WAVEHDR>> = sc
            .headers
            .iter()
            .enumerate()
            .map(|(i, &(length, flags, loops))| {
                Box::new(WAVEHDR {
                    lpData: data[i].as_mut_ptr(),
                    dwBufferLength: length,
                    dwBytesRecorded: 0,
                    dwUser: i,
                    dwFlags: flags | WHDR_PREPARED,
                    dwLoops: loops,
                    lpNext: core::ptr::null_mut(),
                    reserved: &mut *extensions[i] as *mut _ as DWORD_PTR,
                })
            })
            .collect();
        let mut instance = Box::new(SOUND_DEVICE_INSTANCE {
            __bindgen_anon_1: Head { HeadWaveHeader: core::ptr::null_mut() },
            __bindgen_anon_2: Tail { TailWaveHeader: core::ptr::null_mut() },
            OutstandingBuffers: 0,
            LoopsRemaining: 0,
            FrameSize: sc.frame,
            BufferCount: sc.buffers,
            ResetInProgress: FALSE,
            bPaused: FALSE,
        });
        let inst: PSOUND_DEVICE_INSTANCE = &mut *instance;
        unsafe {
            for header in headers.iter_mut() {
                let h: PWAVEHDR = &mut **header;
                // WriteWaveHeader: SanitizeWaveHeader, then (Rust only) the loop count,
                // then EnqueueWaveHeader.
                *((*h).reserved as PWAVEHDR_EXTENSION) = WAVEHDR_EXTENSION::default();
                if side == Side::Rust {
                    shipping::InitializeWaveHeaderLoops(sc.device as u8, h);
                }
                (*h).dwFlags &= !WHDR_DONE;
                (*h).lpNext = core::ptr::null_mut();
                (*h).dwFlags |= WHDR_INQUEUE;
                let tail = (*inst).__bindgen_anon_2.TailWaveHeader;
                if (*inst).__bindgen_anon_1.HeadWaveHeader.is_null() {
                    (*inst).__bindgen_anon_1.HeadWaveHeader = h;
                } else {
                    (*tail).lpNext = h;
                }
                (*inst).__bindgen_anon_2.TailWaveHeader = h;
                stream(side, inst);
            }
            let mut steps = 0;
            while steps < sc.steps {
                let Some(p) = WORLD.with(|w| w.borrow_mut().pending.pop_front()) else { break };
                if (*p.header).dwFlags & WHDR_INQUEUE == 0 {
                    let msg = format!("completion for header {} after it was returned", index(p.header));
                    WORLD.with(|w| w.borrow_mut().errors.push(msg));
                }
                match side {
                    Side::C => ref_CompleteIO(0, p.length, p.overlap),
                    Side::Rust => p.routine.expect("completion routine")(0, p.length, p.overlap),
                }
                steps += 1;
            }
        }
        let drained = WORLD.with(|w| w.borrow().pending.is_empty());
        let outstanding = instance.OutstandingBuffers;
        let world = WORLD.with(|w| std::mem::take(&mut *w.borrow_mut()));
        drop((headers, extensions, data));
        Outcome { played: world.played, done: world.done, errors: world.errors, drained, outstanding }
    }

    /// What a client hears: each header in order, `dwLoops` times when it carries both
    /// loop flags on an output device (0 and 1 mean once), in FrameSize chunks.
    fn model(sc: &Scenario) -> Outcome {
        let mut played = Vec::new();
        for (i, &(length, flags, loops)) in sc.headers.iter().enumerate() {
            let looped = sc.device == WAVE_OUT_DEVICE_TYPE
                && flags & (WHDR_BEGINLOOP | WHDR_ENDLOOP) == WHDR_BEGINLOOP | WHDR_ENDLOOP;
            let passes = if looped { loops.max(1) } else { 1 };
            assert_ne!(loops, INFINITE, "model: an infinite loop has no finite outcome");
            for _ in 0..passes {
                let mut at = 0;
                while at < length {
                    let n = sc.frame.min(length - at);
                    played.push((i, at, n));
                    at += n;
                }
            }
        }
        Outcome {
            played,
            done: (0..sc.headers.len()).collect(),
            errors: vec![],
            drained: true,
            outstanding: 0,
        }
    }

    fn scenario(device: u32, frame: u32, buffers: u32, headers: &[(u32, DWORD, DWORD)]) -> Scenario {
        Scenario { device, frame, buffers, headers: headers.to_vec(), steps: 10_000 }
    }

    const LOOP: DWORD = WHDR_BEGINLOOP | WHDR_ENDLOOP;

    // ---- plain streaming: Rust == C == model -------------------------------------------

    #[test]
    fn plain_headers_match_the_c() {
        let mut graded = 0;
        for device in [WAVE_OUT_DEVICE_TYPE, WAVE_IN_DEVICE_TYPE] {
            for frame in [4, 7, 16] {
                for buffers in [1, 2, 3, 8] {
                    for headers in [
                        vec![(16, 0, 0)],
                        vec![(16, 0, 0), (5, 0, 0), (33, 0, 0)],
                        vec![(1, 0, 0), (1, 0, 0)],
                    ] {
                        let sc = scenario(device, frame, buffers, &headers);
                        let (c, rust) = (run(Side::C, &sc), run(Side::Rust, &sc));
                        assert_eq!(rust, c, "device {device} frame {frame} buffers {buffers} {headers:?}");
                        assert_eq!(rust, model(&sc));
                        graded += 1;
                    }
                }
            }
        }
        assert_eq!(graded, 72);
    }

    /// Loop flags mean nothing on a capture device, or without ENDLOOP (multi-header
    /// loops are unimplemented on both sides: they play once).
    #[test]
    fn loop_flags_that_do_not_loop_match_the_c() {
        for (device, flags) in [
            (WAVE_IN_DEVICE_TYPE, LOOP),
            (WAVE_OUT_DEVICE_TYPE, WHDR_BEGINLOOP),
            (WAVE_OUT_DEVICE_TYPE, WHDR_ENDLOOP),
        ] {
            for buffers in [1, 4] {
                let sc = scenario(device, 4, buffers, &[(10, flags, 3), (6, 0, 0)]);
                let (c, rust) = (run(Side::C, &sc), run(Side::Rust, &sc));
                assert_eq!(rust, c, "device {device} flags {flags:#x} buffers {buffers}");
                assert_eq!(rust, model(&sc));
            }
        }
    }

    /// dwLoops 0 is one pass on both sides.
    #[test]
    fn zero_loops_match_the_c() {
        for buffers in [1, 2, 8] {
            let sc = scenario(WAVE_OUT_DEVICE_TYPE, 4, buffers, &[(12, LOOP, 0), (4, 0, 0)]);
            assert_eq!(run(Side::Rust, &sc), run(Side::C, &sc));
            assert_eq!(run(Side::Rust, &sc), model(&sc));
        }
    }

    // ---- looped streaming: Rust == model, the C defects pinned -------------------------

    #[test]
    fn a_finite_loop_plays_dwloops_times() {
        for length in [4, 12, 10] {
            for buffers in [1, 2, 3, 8] {
                for loops in [1, 2, 3, 5] {
                    let sc = scenario(WAVE_OUT_DEVICE_TYPE, 4, buffers, &[(length, LOOP, loops)]);
                    let rust = run(Side::Rust, &sc);
                    assert_eq!(rust, model(&sc), "length {length} buffers {buffers} loops {loops}");
                }
            }
        }
    }

    /// The C reloads the count from dwLoops on every completion, so a finite loop of a
    /// whole-frame header never ends: 10,000 completions later it is still playing.
    #[test]
    fn the_c_never_ends_a_finite_loop() {
        for loops in [1, 2, 5] {
            let sc = scenario(WAVE_OUT_DEVICE_TYPE, 4, 2, &[(4, LOOP, loops)]);
            let c = run(Side::C, &sc);
            assert!(!c.drained && c.done.is_empty(), "loops {loops}: the C finished -- fixed?");
            assert!(run(Side::Rust, &sc).drained);
        }
    }

    /// A looped header followed by a plain one: every repeat plays before the next header.
    #[test]
    fn repeats_play_before_the_next_header() {
        for buffers in [1, 2, 3, 8] {
            for length in [4, 12, 10] {
                let headers = [(6, 0, 0), (length, LOOP, 3), (5, 0, 0), (length, LOOP, 2)];
                let sc = scenario(WAVE_OUT_DEVICE_TYPE, 4, buffers, &headers);
                assert_eq!(run(Side::Rust, &sc), model(&sc), "buffers {buffers} length {length}");
            }
        }
        // The C commits the next header as soon as the looped one is fully committed,
        // then rewinds the looped one from CompleteIO: the repeat plays after it.
        let sc = scenario(WAVE_OUT_DEVICE_TYPE, 4, 2, &[(4, LOOP, 1), (4, 0, 0)]);
        let c = run(Side::C, &sc);
        assert_eq!(&c.played[..3], &[(0, 0, 4), (1, 0, 4), (0, 0, 4)], "the C's order -- fixed?");
    }

    /// A multi-chunk looped header: the C's loop branch drops the triggering completion
    /// while partial completions keep counting across passes, so (once dwLoops is not
    /// reloaded) a header can be returned with chunks still in flight. The Rust returns
    /// it only after its last pass's last chunk, and never sees a completion afterwards.
    #[test]
    fn a_looped_header_is_returned_after_its_last_chunk() {
        for buffers in [2, 3, 4, 8] {
            for loops in [2, 3] {
                let sc = scenario(WAVE_OUT_DEVICE_TYPE, 4, buffers, &[(12, LOOP, loops), (4, 0, 0)]);
                let rust = run(Side::Rust, &sc);
                assert!(rust.errors.is_empty(), "{:?}", rust.errors);
                assert_eq!(rust.outstanding, 0);
                assert_eq!(rust, model(&sc));
            }
        }
    }

    /// INFINITE repeats the header until a reset; the next header never plays.
    #[test]
    fn an_infinite_loop_repeats_until_reset() {
        for buffers in [1, 3] {
            let mut sc = scenario(WAVE_OUT_DEVICE_TYPE, 4, buffers, &[(10, LOOP, INFINITE), (4, 0, 0)]);
            sc.steps = 300;
            let rust = run(Side::Rust, &sc);
            assert!(!rust.drained && rust.done.is_empty() && rust.errors.is_empty());
            let pass = [(0, 0, 4), (0, 4, 4), (0, 8, 2)];
            for (i, chunk) in rust.played.iter().enumerate() {
                assert_eq!(*chunk, pass[i % 3], "chunk {i}");
            }
            assert!(rust.played.len() >= 300);
        }
    }
}
