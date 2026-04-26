# EX Subsystem SEH Pattern Analysis

Reference for post-translation review of ntoskrnl/ex/ C-to-Rust translation.
All 80 `_SEH2_TRY` blocks classified by pattern and Rust replacement strategy.

## Pattern Summary

| Pattern | Count | Rust Replacement | Confidence |
|---------|-------|------------------|------------|
| A: Probe-then-bail | 27 | `probe_write_user::<T>(ptr, mode)?` | Mechanical |
| B: Write-back scalar | 22 | `write_user::<T>(ptr, value, mode)?` | Mechanical |
| C: Read user input | 5 | `probe_read_user::<T>(ptr, mode)?` | Mechanical |
| D: Complex body | 22 | Decompose into A+B+C calls | Review needed |
| E: Other | 4 | Case-by-case | Manual review |

**Key insight**: With DUMMY_PSEH, all 80 SEH handlers are dead code on all 3 architectures.
The Rust translation must pre-validate user pointers rather than catching faults.

## API Surface

```rust
// Already exists in ntkrnl-sys/src/probe.rs:
probe_read_user::<T>(ptr: *const T, mode: KPROCESSOR_MODE) -> Result<T, NTSTATUS>
probe_write_user::<T>(ptr: *mut T, mode: KPROCESSOR_MODE) -> Result<(), NTSTATUS>
probe_user_buffer(buf: PVOID, len: ULONG, align: ULONG, mode: KPROCESSOR_MODE) -> Result<(), NTSTATUS>

// NEW additions:
write_user::<T>(ptr: *mut T, value: T, mode: KPROCESSOR_MODE) -> Result<(), NTSTATUS>
write_user_nofail::<T>(ptr: *mut T, value: T, mode: KPROCESSOR_MODE)  // ignore failure
```

## Per-File Block Inventory

### event.c (11 blocks)

| # | Function | Pattern | SEH Body | Rust Replacement |
|---|----------|---------|----------|------------------|
| 1 | NtCreateEvent | A | `ProbeForWriteHandle(EventHandle)` | `probe_write_user::<HANDLE>(EventHandle, mode)?` |
| 2 | NtCreateEvent | B | `*EventHandle = hEvent` | `write_user(EventHandle, hEvent, mode)?` |
| 3 | NtOpenEvent | A | `ProbeForWriteHandle(EventHandle)` | `probe_write_user::<HANDLE>(EventHandle, mode)?` |
| 4 | NtOpenEvent | B | `*EventHandle = hEvent` | `write_user(EventHandle, hEvent, mode)?` |
| 5 | NtPulseEvent | A | `ProbeForWriteLong(PreviousState)` | `probe_write_user::<LONG>(PreviousState, mode)?` |
| 6 | NtPulseEvent | B | `*PreviousState = Prev` | `write_user(PreviousState, Prev, mode)?` |
| 7 | NtQueryEvent | D1 | Fill BasicInfo (EventType, EventState, ReturnLength) | 3x `write_user` calls |
| 8 | NtResetEvent | A | `ProbeForWriteLong(PreviousState)` | `probe_write_user::<LONG>(PreviousState, mode)?` |
| 9 | NtResetEvent | B | `*PreviousState = Prev` | `write_user(PreviousState, Prev, mode)?` |
| 10 | NtSetEvent | A | `ProbeForWriteLong(PreviousState)` | `probe_write_user::<LONG>(PreviousState, mode)?` |
| 11 | NtSetEvent | B | `*PreviousState = Prev` | `write_user(PreviousState, Prev, mode)?` |

### sem.c (7 blocks)

| # | Function | Pattern | SEH Body | Rust Replacement |
|---|----------|---------|----------|------------------|
| 1 | NtCreateSemaphore | A | `ProbeForWriteHandle(SemaphoreHandle)` | `probe_write_user` |
| 2 | NtCreateSemaphore | B | `*SemaphoreHandle = hSemaphore` | `write_user` |
| 3 | NtOpenSemaphore | A | `ProbeForWriteHandle(SemaphoreHandle)` | `probe_write_user` |
| 4 | NtOpenSemaphore | B | `*SemaphoreHandle = hSemaphore` | `write_user` |
| 5 | NtQuerySemaphore | D1 | Fill BasicInfo (CurrentCount, MaximumCount, ReturnLength) | 3x `write_user` |
| 6 | NtReleaseSemaphore | A | `ProbeForWriteLong(PreviousCount)` | `probe_write_user` |
| 7 | NtReleaseSemaphore | **D2** | `KeReleaseSemaphore()` + `*PreviousCount = PrevCount` | **Review**: KeReleaseSemaphore can raise. With DUMMY_PSEH already unprotected. Just call + write_user. |

### mutant.c (7 blocks)

| # | Function | Pattern | SEH Body | Rust Replacement |
|---|----------|---------|----------|------------------|
| 1 | NtCreateMutant | A | `ProbeForWriteHandle(MutantHandle)` | `probe_write_user` |
| 2 | NtCreateMutant | B | `*MutantHandle = hMutant` | `write_user` |
| 3 | NtOpenMutant | A | `ProbeForWriteHandle(MutantHandle)` | `probe_write_user` |
| 4 | NtOpenMutant | B | `*MutantHandle = hMutant` | `write_user` |
| 5 | NtQueryMutant | D1 | Fill BasicInfo (CurrentCount, OwnedByCaller, AbandonedState, ResultLength) | 4x `write_user` |
| 6 | NtReleaseMutant | A | `ProbeForWriteLong(PreviousCount)` | `probe_write_user` |
| 7 | NtReleaseMutant | **D2** | `KeReleaseMutant()` + `*PreviousCount = Prev` | **Review**: KeReleaseMutant can raise STATUS_MUTANT_NOT_OWNED. Already unprotected with DUMMY_PSEH. |

### timer.c (9 blocks)

| # | Function | Pattern | SEH Body | Rust Replacement |
|---|----------|---------|----------|------------------|
| 1 | NtCancelTimer | A | `ProbeForWriteBoolean(CurrentState)` | `probe_write_user` |
| 2 | NtCancelTimer | B | `*CurrentState = State` | `write_user_nofail` (EXCEPT is `(void)0`) |
| 3 | NtCreateTimer | A | `ProbeForWriteHandle(TimerHandle)` | `probe_write_user` |
| 4 | NtCreateTimer | B | `*TimerHandle = hTimer` | `write_user_nofail` (EXCEPT is `(void)0`) |
| 5 | NtOpenTimer | A | `ProbeForWriteHandle(TimerHandle)` | `probe_write_user` |
| 6 | NtOpenTimer | B | `*TimerHandle = hTimer` | `write_user_nofail` (EXCEPT is `(void)0`) |
| 7 | NtQueryTimer | D1 | Fill BasicInfo (TimeRemaining computed, SignalState, ReturnLength) | 3x `write_user` |
| 8 | NtSetTimer | C | `ProbeForReadLargeInteger(DueTime)` + `ProbeForWriteBoolean(PreviousState)` | `probe_read_user` + `probe_write_user` |
| 9 | NtSetTimer | B | `*PreviousState = State` | `write_user_nofail` (EXCEPT is `(void)0`) |

### profile.c (7 blocks)

| # | Function | Pattern | SEH Body | Rust Replacement |
|---|----------|---------|----------|------------------|
| 1 | NtCreateProfile | A | `ProbeForWriteHandle` + `ProbeForWrite(Buffer)` | 2x probe calls |
| 2 | NtCreateProfile | B | `*ProfileHandle = hProfile` | `write_user` |
| 3 | NtQueryPerformanceCounter | A | `ProbeForWriteLargeInteger` x2 | 2x `probe_write_user` |
| 4 | NtQueryPerformanceCounter | D1 | `KeQueryPerformanceCounter` + write 2 outputs | `write_user` x2 |
| 5 | NtStartProfile | **E** | `MmProbeAndLockPages(Mdl)` — kernel MDL lock | **Manual**: not a user-ptr pattern. Leave as raw call. |
| 6 | NtQueryIntervalProfile | A | `ProbeForWriteUlong(Interval)` | `probe_write_user` |
| 7 | NtQueryIntervalProfile | B | `*Interval = ReturnInterval` | `write_user` |

### evtpair.c (4 blocks)

| # | Function | Pattern | SEH Body | Rust Replacement |
|---|----------|---------|----------|------------------|
| 1 | NtCreateEventPair | A | `ProbeForWriteHandle(EventPairHandle)` | `probe_write_user` |
| 2 | NtCreateEventPair | B | `*EventPairHandle = hEventPair` | `write_user` |
| 3 | NtOpenEventPair | A | `ProbeForWriteHandle(EventPairHandle)` | `probe_write_user` |
| 4 | NtOpenEventPair | B | `*EventPairHandle = hEventPair` | `write_user` |

### keyedevt.c (4 blocks)

| # | Function | Pattern | SEH Body | Rust Replacement |
|---|----------|---------|----------|------------------|
| 1 | NtCreateKeyedEvent | **D3** | `ProbeForWrite(OutHandle)` + `*OutHandle = Handle` in same TRY. EXCEPT calls `ObCloseHandle`. | `probe_write_user` + `write_user`. EXCEPT cleanup: add explicit `ObCloseHandle` on error path. |
| 2 | NtOpenKeyedEvent | **D3** | Same as above | Same |
| 3 | NtWaitForKeyedEvent | C | `ProbeForRead(Timeout)` + `TimeoutCopy = *Timeout` | `probe_read_user::<LARGE_INTEGER>` |
| 4 | NtReleaseKeyedEvent | C | Same timeout capture | `probe_read_user::<LARGE_INTEGER>` |

### atom.c (5 blocks)

| # | Function | Pattern | SEH Body | Rust Replacement |
|---|----------|---------|----------|------------------|
| 1 | NtAddAtom | **D5** | `ProbeForRead(AtomName)` + pool alloc + `RtlCopyMemory` + null-term + `ProbeForWriteUshort(Atom)` | **Review**: Decompose into probe_user_buffer + manual copy + probe_write_user. Pool alloc inside SEH is already broken with DUMMY_PSEH (ExAllocatePoolWithQuotaTag can raise). |
| 2 | NtAddAtom | B | `*Atom = SafeAtom` | `write_user` |
| 3 | NtFindAtom | **D5** | Same as NtAddAtom #1 | Same |
| 4 | NtFindAtom | B | `*Atom = SafeAtom` | `write_user` |
| 5 | NtQueryInformationAtom | **D6** | Entire function body: probe + switch + query + fill + ReturnLength | **Review**: Break into probe phase + kernel work phase + write-back phase. |

### time.c (6 blocks)

| # | Function | Pattern | SEH Body | Rust Replacement |
|---|----------|---------|----------|------------------|
| 1 | NtSetSystemTime | C | `ProbeForReadLargeInteger(SystemTime)` + optional `ProbeForWriteLargeInteger(PreviousTime)` | `probe_read_user` + `probe_write_user` |
| 2 | NtSetSystemTime | B | `*PreviousTime = OldSystemTime` | `write_user` |
| 3 | NtQuerySystemTime | **D3** | `ProbeForWriteLargeInteger(SystemTime)` + `KeQuerySystemTime(SystemTime)` writes directly to user ptr | **Review**: Probe first, then KeQuerySystemTime to local, then write_user. |
| 4 | NtQueryTimerResolution | **D3** | Probe 3 ptrs + write 3 values | 3x `probe_write_user` + 3x `write_user` |
| 5 | NtSetTimerResolution | A | `ProbeForWriteUlong(CurrentResolution)` | `probe_write_user` |
| 6 | NtSetTimerResolution | B | `*CurrentResolution = NewResolution` | `write_user` |

### locale.c (3 blocks)

| # | Function | Pattern | SEH Body | Rust Replacement |
|---|----------|---------|----------|------------------|
| 1 | NtQueryDefaultLocale | **D4** | Probe inside TRY + write `*DefaultLocaleId` | `probe_write_user` + `write_user` |
| 2 | NtQueryInstallUILanguage | **D4** | Probe inside TRY + write `*LanguageId` | `probe_write_user` + `write_user` |
| 3 | NtQueryDefaultUILanguage | **D4** | Probe inside TRY + conditional write `*LanguageId` | `probe_write_user` + `write_user` |

### uuid.c (4 blocks)

| # | Function | Pattern | SEH Body | Rust Replacement |
|---|----------|---------|----------|------------------|
| 1 | NtAllocateLocallyUniqueId | A | `ProbeForWrite(LocallyUniqueId, sizeof(LUID), sizeof(ULONG))` | `probe_write_user` |
| 2 | NtAllocateUuids | A | Probe 4 output ptrs | 4x `probe_write_user` |
| 3 | NtAllocateUuids | D1 | Write 4 computed values: Time, Range, Sequence, Seed | 4x `write_user` + `RtlCopyMemory` for Seed |
| 4 | NtSetUuidSeed | **D7** | Security context + privilege check + `RtlRaiseStatus` + probe + copy | **Review**: Complex. Security check raises on failure (already broken with DUMMY_PSEH). Decompose into security check phase + probe_read phase. |

### sysinfo.c (7 blocks)

| # | Function | Pattern | SEH Body | Rust Replacement |
|---|----------|---------|----------|------------------|
| 1 | ExLockUserBuffer | **E** | `MmProbeAndLockPages(Mdl)` | **Manual**: kernel MDL API, not user-ptr. Leave as raw call with error check. |
| 2 | NtQuerySystemEnvironmentValue | A | Probe VariableName + ValueBuffer + ReturnLength | `probe_user_buffer` + `probe_write_user` |
| 3 | NtQuerySystemEnvironmentValue | D1 | String conversion to user buffer + ReturnLength | Write-back phase after kernel work |
| 4 | QSI[SystemProcessInformation] | **D6** | **~200 lines**: Entire process enumeration filling user buffer | **Review**: Largest single block. All user writes already unprotected. LLM should preserve structure, we verify user buffer probed at entry. |
| 5 | SSI[SystemExtendServiceTable] | C | Capture UNICODE_STRING + probe string buffer | `probe_read_user::<UNICODE_STRING>` + `probe_user_buffer` |
| 6 | NtQuerySystemInformation | **D6** | Probe + dispatch to QSI handler + ReturnLength | Decompose: probe phase, dispatch, write_user for ReturnLength |
| 7 | NtSetSystemInformation | **D6** | Probe + dispatch to SSI handler | Decompose: probe phase, dispatch |

### harderr.c (4 blocks)

| # | Function | Pattern | SEH Body | Rust Replacement |
|---|----------|---------|----------|------------------|
| 1 | ExpRaiseHardError | **E** | Read `Teb->HardErrorMode` (user TEB) | **Manual**: raw TEB access with MmIsAddressValid or write_user pattern |
| 2 | ExRaiseHardError | **D8** | Copy string params to user-mode allocated VA | **Review**: Writing to ZwAllocateVirtualMemory region. probe_user_buffer + manual writes. |
| 3 | NtRaiseHardError | C+D | Probe Response + capture Parameters array + loop string probes | Decompose: `probe_write_user` + `probe_read_user` + loop |
| 4 | NtRaiseHardError | B | `*Response = SafeResponse` | `write_user` |

### dbgctrl.c (2 blocks)

| # | Function | Pattern | SEH Body | Rust Replacement |
|---|----------|---------|----------|------------------|
| 1 | ExpDebuggerWorker | **E** | `ProbeForReadUchar(PageInAddress)` — force page-in | **Manual**: Not user validation. Deliberate page-in attempt. |
| 2 | NtSystemDebugControl | **D6** | Probe + switch dispatch + ReturnLength | Decompose: probe phase, switch, write_user |

## Post-Translation Checklist

After LLM translation, verify each file against this inventory:

### Mechanical checks (grep-able)
- [ ] No raw `ProbeForWrite*` / `ProbeForRead*` calls remain — all replaced with probe API
- [ ] No `_SEH2_TRY` / `_SEH2_EXCEPT` / `_SEH2_END` markers remain
- [ ] Every Pattern A site uses `probe_write_user(...)?` or `probe_read_user(...)?`
- [ ] Every Pattern B site uses `write_user(...)?` or `write_user_nofail(...)`
- [ ] PreviousMode guards preserved: probe calls only execute when `mode != KernelMode`

### Manual review points
- [ ] **D2 (2 blocks)**: sem.c `NtReleaseSemaphore`, mutant.c `NtReleaseMutant` — verify KeRelease* call is NOT wrapped in any error handling (matches DUMMY_PSEH behavior)
- [ ] **D3 keyedevt.c (2 blocks)**: NtCreate/OpenKeyedEvent — verify `ObCloseHandle` cleanup on probe failure path
- [ ] **D5 atom.c (2 blocks)**: NtAddAtom/NtFindAtom — verify pool allocation + string capture decomposed correctly
- [ ] **D6 sysinfo.c (4 blocks)**: NtQuerySystemInformation, SystemProcessInformation, NtSetSystemInformation, NtSystemDebugControl — verify user buffer probed at entry, large body preserved structurally
- [ ] **D7 uuid.c (1 block)**: NtSetUuidSeed — security privilege check preserved
- [ ] **D8 harderr.c (1 block)**: ExRaiseHardError — user-mode VA writes handled
- [ ] **E profile.c (1 block)**: NtStartProfile MmProbeAndLockPages — not converted to probe API
- [ ] **E sysinfo.c (1 block)**: ExLockUserBuffer MmProbeAndLockPages — same
- [ ] **E harderr.c (1 block)**: ExpRaiseHardError TEB read — handled appropriately
- [ ] **E dbgctrl.c (1 block)**: ExpDebuggerWorker page-in — not converted to probe API

### timer.c special case
- [ ] 3 write-back blocks use `(void)0` EXCEPT (silently swallow). Verify these use `write_user_nofail` (not `write_user(...)?` which would propagate error).

### Files with ZERO SEH (no special review needed)
- callback.c, efi.c, exintrin.c, fmutex.c, handle.c, hdlsterm.c, init.c,
  interlocked.c, lookas.c, pushlock.c, resource.c, rundown.c, shutdown.c,
  win32k.c, work.c, xipdisp.c, zone.c (17 files)
