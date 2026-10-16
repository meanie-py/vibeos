/*
 * selfmod32.exe — the i386 runtime-code-generation guest for WoW64 E17
 * (docs/engineering/active-lanes/wow64-plan.md). Built with the llvm-mingw i686
 * target, no CRT (plain _start entry, same as Hello32.c/File32.c), importing
 * only kernel32, loaded through the genuine 32-bit loader under FEX.
 *
 * What it proves: the CPU layer's MEMORY-NOTIFICATION contract. Up to E17
 * wow64.dll wired exactly one of it — BTCpuNotifyMapViewOfSection — which
 * covers images and nothing else. On a JIT host that is not a missing
 * optimisation, it is a missing capability: FEX's decoder refuses to translate
 * any guest address outside the executable set it has been told about, and it
 * caches translated blocks keyed on guest address. So a 32-bit guest that
 * builds code at runtime — ATL/MFC thunks, VB6/Delphi trampolines, packers and
 * self-extractors, any JIT — either could not run its code at all, or ran the
 * version it generated three rewrites ago.
 *
 * Each rung isolates ONE notification, and most of them fail SILENTLY WRONG
 * rather than loudly: the stale-translation rungs return the previous answer,
 * which is exactly the failure mode a crash-only test would miss.
 *
 * On a native host (amd64's wow64cpu) none of these notifications exist and the
 * whole file is a no-op contract — every rung must still pass, which makes
 * amd64 the control for "the rungs measure the emulator, not the guest".
 *
 * Verdict contract (same as hello32/file32/thread32): exit code 0x600D on PASS;
 * distinct 0xBADx codes name the failing rung so serial alone triages.
 */

#include <windows.h>

#define SM_FAIL_ALLOC        0xBAD1 /* VirtualAlloc RWX itself failed          */
#define SM_FAIL_EXEC         0xBAD2 /* generated code did not run / wrong value */
#define SM_FAIL_FLUSH        0xBAD3 /* rewrite+FlushInstructionCache ran STALE  */
#define SM_FAIL_PROTECT      0xBAD4 /* RW -> RX transition never became runnable */
#define SM_FAIL_REALLOC      0xBAD5 /* free+realloc at the same base ran STALE  */
#define SM_FAIL_READFILE     0xBAD6 /* ReadFile into code memory ran STALE      */
#define SM_FAIL_WPM          0xBAD7 /* WriteProcessMemory into code ran STALE   */
#define SM_FAIL_COUNT        0xBAD8 /* *ReturnSize cell clobbered past 4 bytes  */
#define SM_FAIL_FILE         0xBAD9 /* scratch file setup failed (not the SUT)  */
#define SM_PASS              0x600D

typedef DWORD (*CODEFN)(void);

static void Report(const char *msg)
{
    /* OutputDebugStringA reaches serial via the guest DbgPrint forwarding
     * (int 2d DebugService -> [GUEST32] in the wow64 VEH). */
    OutputDebugStringA(msg);
}

static void ReportHex(const char *label, DWORD v)
{
    static const char Digits[] = "0123456789abcdef";
    char buf[96];
    int i = 0, n;
    while (label[i] && i < 72) { buf[i] = label[i]; i++; }
    buf[i++] = ' '; buf[i++] = '0'; buf[i++] = 'x';
    for (n = 28; n >= 0; n -= 4)
        buf[i++] = Digits[(v >> n) & 0xf];
    buf[i++] = '\n';
    buf[i] = 0;
    Report(buf);
}

/* The whole payload: `mov eax, imm32` / `ret`. Six bytes, no relocations, no
 * stack use — if it runs at all it can only return the constant, so a wrong
 * answer means a STALE TRANSLATION and nothing else. */
#define CODE_LEN 6

static void EmitConst(BYTE *p, DWORD value)
{
    p[0] = 0xB8;                        /* mov eax, imm32 */
    p[1] = (BYTE)(value      );
    p[2] = (BYTE)(value >>  8);
    p[3] = (BYTE)(value >> 16);
    p[4] = (BYTE)(value >> 24);
    p[5] = 0xC3;                        /* ret            */
}

static void PathJoinA(char *dst, const char *dir, const char *rest)
{
    int i = 0, j = 0;
    while (dir[j]) dst[i++] = dir[j++];
    dst[i++] = '\\';
    j = 0;
    while (rest[j]) dst[i++] = rest[j++];
    dst[i] = 0;
}

int start(void) /* i386 symbol _start (-e _start); x64 symbol start (-e start) */
{
    BYTE *code, *base2;
    CODEFN fn;
    DWORD got, old;
    char scratch[MAX_PATH], win[MAX_PATH];
    HANDLE file;
    DWORD io;
    BYTE filebuf[CODE_LEN];

    Report("[selfmod32] start: runtime code generation under WoW64\n");

    /* -- Rung 1: generate code into a fresh RWX allocation and RUN it -------
     * The allocation notification is what puts the range in the CPU layer's
     * executable set. Without it the emulator declines to decode the address
     * at all — this rung does not return a wrong value, it dies. */
    code = (BYTE *)VirtualAlloc(NULL, 0x1000, MEM_RESERVE | MEM_COMMIT,
                                PAGE_EXECUTE_READWRITE);
    if (!code)
    {
        Report("[selfmod32] FAIL: VirtualAlloc(RWX) returned NULL\n");
        ExitProcess(SM_FAIL_ALLOC);
    }
    ReportHex("[selfmod32] RWX page at", (DWORD)(ULONG_PTR)code);
    EmitConst(code, 0x11111111);
    fn = (CODEFN)code;
    got = fn();
    ReportHex("[selfmod32] rung1 generated code returned", got);
    if (got != 0x11111111)
    {
        Report("[selfmod32] FAIL: rung1 — generated code did not run correctly\n");
        ExitProcess(SM_FAIL_EXEC);
    }
    Report("[selfmod32] rung1 PASS: guest can execute VirtualAlloc'd code\n");

    /* -- Rung 2: rewrite it and flush ---------------------------------------
     * The code at this address is now translated and cached. Rewriting it and
     * calling FlushInstructionCache is what a well-behaved code generator
     * does, and it is the ONLY signal the architecture gives us. A CPU layer
     * that ignores it returns the previous constant — silently. */
    EmitConst(code, 0x22222222);
    FlushInstructionCache(GetCurrentProcess(), code, CODE_LEN);
    got = fn();
    ReportHex("[selfmod32] rung2 rewritten code returned", got);
    if (got != 0x22222222)
    {
        Report("[selfmod32] FAIL: rung2 — STALE translation survived "
               "FlushInstructionCache\n");
        ExitProcess(SM_FAIL_FLUSH);
    }
    Report("[selfmod32] rung2 PASS: FlushInstructionCache invalidates\n");

    /* -- Rung 3: write as data, THEN make it executable ---------------------
     * The commonest real pattern, and the one the alloc notification cannot
     * cover: this region was allocated non-executable, so only the PROTECT
     * notification can ever add it to the executable set. */
    base2 = (BYTE *)VirtualAlloc(NULL, 0x1000, MEM_RESERVE | MEM_COMMIT,
                                 PAGE_READWRITE);
    if (!base2)
    {
        Report("[selfmod32] FAIL: VirtualAlloc(RW) returned NULL\n");
        ExitProcess(SM_FAIL_ALLOC);
    }
    EmitConst(base2, 0x33333333);
    old = 0;
    if (!VirtualProtect(base2, 0x1000, PAGE_EXECUTE_READ, &old))
    {
        Report("[selfmod32] FAIL: VirtualProtect(RW -> RX) failed\n");
        ExitProcess(SM_FAIL_PROTECT);
    }
    FlushInstructionCache(GetCurrentProcess(), base2, CODE_LEN);
    got = ((CODEFN)base2)();
    ReportHex("[selfmod32] rung3 RW->RX code returned", got);
    if (got != 0x33333333)
    {
        Report("[selfmod32] FAIL: rung3 — RW->RX region never became runnable\n");
        ExitProcess(SM_FAIL_PROTECT);
    }
    Report("[selfmod32] rung3 PASS: NtProtectVirtualMemory publishes new code\n");

    /* -- Rung 4: free the page and take the SAME address back ---------------
     * Translations are cached by GUEST address. If freeing does not drop them,
     * the next tenant of that address inherits the previous tenant's code —
     * the exact failure a long-running host process with a recycling code heap
     * hits, and it produces a plausible wrong answer rather than a crash. */
    if (!VirtualFree(base2, 0, MEM_RELEASE))
    {
        Report("[selfmod32] FAIL: VirtualFree(MEM_RELEASE) failed\n");
        ExitProcess(SM_FAIL_REALLOC);
    }
    if (VirtualAlloc(base2, 0x1000, MEM_RESERVE | MEM_COMMIT,
                     PAGE_EXECUTE_READWRITE) != base2)
    {
        /* Not the system under test — if the address did not come back we
         * cannot ask the question. Say so instead of scoring a pass. */
        Report("[selfmod32] FAIL: could not re-reserve the same base; rung4 "
               "cannot be evaluated\n");
        ExitProcess(SM_FAIL_REALLOC);
    }
    EmitConst(base2, 0x44444444);
    got = ((CODEFN)base2)();
    ReportHex("[selfmod32] rung4 recycled address returned", got);
    if (got != 0x44444444)
    {
        Report("[selfmod32] FAIL: rung4 — STALE translation survived free+realloc\n");
        ExitProcess(SM_FAIL_REALLOC);
    }
    Report("[selfmod32] rung4 PASS: NtFreeVirtualMemory drops cached code\n");

    /* -- Rung 5: let the KERNEL write the code, via ReadFile ----------------
     * A loader reading a section straight into executable memory. The bytes
     * appear behind the CPU layer's back, and if it write-traps the page for
     * self-modifying-code detection the kernel's write has to be let through
     * first. code[] is already translated from rungs 1-2, so this is a genuine
     * overwrite of live cached code. */
    win[0] = 0;
    GetWindowsDirectoryA(win, MAX_PATH - 32);
    PathJoinA(scratch, win, "syswow64\\selfmod32.tmp");
    file = CreateFileA(scratch, GENERIC_READ | GENERIC_WRITE, 0, NULL,
                       CREATE_ALWAYS, FILE_ATTRIBUTE_NORMAL, NULL);
    if (file == INVALID_HANDLE_VALUE)
    {
        DWORD err = GetLastError();
        if (err == 19 /* ERROR_WRITE_PROTECT */ || err == 5 /* ERROR_ACCESS_DENIED */)
        {
            /* %WINDIR% is a writable ramdisk on arm64 but a read-only CD on the
             * amd64 livecd. The scratch file is this rung's FIXTURE, not its
             * subject — what is under test is whether a KERNEL write into live
             * code invalidates the CPU layer's translation. So skip loudly on a
             * read-only volume and let rungs 6-7 (which need no file) still run,
             * exactly as file32 does for its own write rungs.
             *
             * This is why the rung was green when E17 landed and red here: the
             * ladder only runs past the desktop marker under `smoke --until`,
             * so an amd64-only failure in a late rung stays invisible to the
             * ordinary gate. */
            ReportHex("[selfmod32] rung5 SKIPPED (read-only volume), err =", err);
            goto rung6;
        }
        ReportHex("[selfmod32] FAIL: could not create the scratch file, err =", err);
        ExitProcess(SM_FAIL_FILE);
    }
    EmitConst(filebuf, 0x55555555);
    io = 0;
    if (!WriteFile(file, filebuf, CODE_LEN, &io, NULL) || io != CODE_LEN ||
        SetFilePointer(file, 0, NULL, FILE_BEGIN) != 0)
    {
        CloseHandle(file);
        Report("[selfmod32] FAIL: scratch file write/seek failed\n");
        ExitProcess(SM_FAIL_FILE);
    }
    io = 0;
    if (!ReadFile(file, code, CODE_LEN, &io, NULL) || io != CODE_LEN)
    {
        CloseHandle(file);
        Report("[selfmod32] FAIL: ReadFile into code memory failed\n");
        ExitProcess(SM_FAIL_FILE);
    }
    CloseHandle(file);
    DeleteFileA(scratch);
    FlushInstructionCache(GetCurrentProcess(), code, CODE_LEN);
    got = fn();
    ReportHex("[selfmod32] rung5 kernel-written code returned", got);
    if (got != 0x55555555)
    {
        Report("[selfmod32] FAIL: rung5 — STALE translation survived a ReadFile "
               "into code memory\n");
        ExitProcess(SM_FAIL_READFILE);
    }
    Report("[selfmod32] rung5 PASS: NtReadFile into code memory publishes it\n");

rung6:
    /* -- Rung 6: WriteProcessMemory onto our own code ----------------------
     * The debugger/hot-patch path, and the one that exercises three
     * notifications at once: kernel32 wraps the write in
     * NtProtectVirtualMemory(PAGE_EXECUTE_READWRITE) either side and finishes
     * with NtFlushInstructionCache. */
    {
        BYTE patch[CODE_LEN];
        SIZE_T wrote = 0;

        EmitConst(patch, 0x66666666);
        if (!WriteProcessMemory(GetCurrentProcess(), code, patch, CODE_LEN, &wrote))
        {
            Report("[selfmod32] FAIL: WriteProcessMemory onto own code failed\n");
            ExitProcess(SM_FAIL_WPM);
        }
        got = fn();
        ReportHex("[selfmod32] rung6 patched code returned", got);
        if (got != 0x66666666)
        {
            Report("[selfmod32] FAIL: rung6 — STALE translation survived "
                   "WriteProcessMemory\n");
            ExitProcess(SM_FAIL_WPM);
        }
        Report("[selfmod32] rung6 PASS: WriteProcessMemory hot-patch publishes\n");
    }

    /* -- Rung 7: the syscall's own count cell, called DIRECTLY --------------
     * *NumberOfBytesWritten is a ULONG_PTR: 4 bytes to the guest, 8 to the
     * host, so a flat thunk hands the host a 4-byte cell and it writes 8.
     * This has to bypass kernel32 to mean anything — WriteProcessMemory passes
     * the syscall its OWN local and copies the result out, so the clobber
     * lands in kernel32's stack frame where no caller-side canary can see it.
     * cells[1] is the canary; it sits immediately after the count cell. */
    {
        typedef LONG (WINAPI *RWVM_FN)(HANDLE, PVOID, PVOID, SIZE_T, SIZE_T *);
        HMODULE nt = GetModuleHandleA("ntdll.dll");
        RWVM_FN NtWVM = (RWVM_FN)GetProcAddress(nt, "NtWriteVirtualMemory");
        RWVM_FN NtRVM = (RWVM_FN)GetProcAddress(nt, "NtReadVirtualMemory");
        /* [0] the count cell, at the caller's SIZE_T width; [1] the canary
         * right behind it. */
        volatile SIZE_T cells[2];
        BYTE patch[CODE_LEN];
        BYTE readback[CODE_LEN];   /* never the same buffer as `patch` */
        LONG st;
        LONG write_st;   /* the direct write's own status; `st` is reused below */

        if (!NtWVM || !NtRVM)
        {
            Report("[selfmod32] FAIL: ntdll32 has no Nt{Read,Write}VirtualMemory\n");
            ExitProcess(SM_FAIL_COUNT);
        }

        EmitConst(patch, 0x77777777);
        cells[0] = 0;
        cells[1] = 0xC0DEFACE;
        st = NtWVM(GetCurrentProcess(), code, patch, CODE_LEN,
                   (SIZE_T *)&cells[0]);
        write_st = st;
        ReportHex("[selfmod32] rung7 NtWriteVirtualMemory =", (DWORD)st);
        ReportHex("[selfmod32] rung7 bytes written =", (DWORD)cells[0]);
        ReportHex("[selfmod32] rung7 canary past the count cell =", (DWORD)cells[1]);

        /* A REFUSED write is a correct answer here, and on a JIT host it is the
         * EXPECTED one. FEX write-protects an RWX page to detect self-modifying
         * code, so the page really is read-only at the moment of the syscall and
         * a kernel that refuses is honouring a real protection -- which is
         * exactly why kernel32's WriteProcessMemory (rung 6, which passes)
         * wraps its write in NtProtectVirtualMemory. This rung calls the syscall
         * DIRECTLY and so gets no such wrapper.
         *
         * This used to read `st < 0` as failure, and that was right when the
         * kernel was WRONG: task #76 recorded MiDoPoolCopy returning 0x00000000
         * with 6 bytes written and the bytes unchanged -- success reported for a
         * write that never happened, because the translated Rust had dropped the
         * _SEH2_EXCEPT and could not observe its own copy faulting. With the
         * probe/copy faults now observable it returns STATUS_PARTIAL_COPY with 0
         * written, which is the truth. The rung was asserting the bug.
         *
         * Since ProbeForWrite touches each target page again, as NT's does
         * (ac29134e356), the refusal comes from the PROBE rather than the copy, and
         * MmCopyVirtualMemory returns the probe's own exception code with 0
         * written: STATUS_ACCESS_VIOLATION, which is what Windows answers too
         * (WriteProcessMemory into a read-only page fails with ERROR_NOACCESS).
         * Both are honest refusals.
         *
         * Accepting either BLINDLY would be the other mistake, so the refusal is
         * only accepted when the page is genuinely not writable. */
        if ((st == (LONG)0x8000000D            /* STATUS_PARTIAL_COPY */
             || st == (LONG)0xC0000005)        /* STATUS_ACCESS_VIOLATION */
            && cells[0] == 0)
        {
            MEMORY_BASIC_INFORMATION mbi;
            SIZE_T got = VirtualQuery((LPCVOID)code, &mbi, sizeof(mbi));
            ReportHex("[selfmod32] rung7 page protect =", got ? mbi.Protect : 0);
            if (!got || (mbi.Protect & (PAGE_READWRITE | PAGE_WRITECOPY |
                                        PAGE_EXECUTE_READWRITE |
                                        PAGE_EXECUTE_WRITECOPY)))
            {
                Report("[selfmod32] FAIL: rung7 — refused a write to a page that "
                       "IS writable\n");
                ExitProcess(SM_FAIL_COUNT);
            }
            Report("[selfmod32] rung7 OK: kernel refused a write to an "
                   "SMC-trapped page, which is correct\n");
        }
        else if (st < 0 || cells[0] != CODE_LEN)
        {
            Report("[selfmod32] FAIL: rung7 — direct NtWriteVirtualMemory failed\n");
            ExitProcess(SM_FAIL_COUNT);
        }

        /* The canary is the POINT of this rung and is checked either way: a
         * refused write must not scribble 8 bytes into the guest's 4-byte count
         * cell any more than a successful one may. */
        if (cells[1] != 0xC0DEFACE)
        {
            Report("[selfmod32] FAIL: rung7 — *NumberOfBytesWritten wrote past "
                   "the guest's 4-byte cell\n");
            ExitProcess(SM_FAIL_COUNT);
        }

        cells[0] = 0;
        cells[1] = 0xC0DEFACE;
        st = NtRVM(GetCurrentProcess(), code, readback, CODE_LEN,
                   (SIZE_T *)&cells[0]);
        ReportHex("[selfmod32] rung7 bytes read =", (DWORD)cells[0]);
        ReportHex("[selfmod32] rung7 read canary =", (DWORD)cells[1]);
        if (st < 0 || cells[0] != CODE_LEN || cells[1] != 0xC0DEFACE)
        {
            Report("[selfmod32] FAIL: rung7 — *NumberOfBytesRead wrote past the "
                   "guest's 4-byte cell\n");
            ExitProcess(SM_FAIL_COUNT);
        }
        Report("[selfmod32] rung7 PASS: the count cells stay 4 bytes wide\n");

        /* Did the bytes actually LAND? "the write failed" and "the write
         * landed but the translation is stale" produce the identical wrong
         * answer from fn(), so read the page back through a plain guest load
         * before blaming the emulator.
         *
         * This is expected to be NO on a JIT host, and the reason is worth
         * knowing: the emulator write-protects RWX pages to detect
         * self-modifying code, and that protection is real — it is in the
         * page tables, so the KERNEL sees it too. A direct
         * NtWriteVirtualMemory onto a page the emulator has trapped is a
         * write to a read-only page and must fail, exactly as it would on
         * Windows. That is precisely why WriteProcessMemory (rung 6) wraps
         * the write in NtProtectVirtualMemory instead of writing straight
         * through. What must NOT happen is the kernel reporting success
         * and a full byte count while writing nothing. */
        ReportHex("[selfmod32] rung7 code[1..4] after the direct write =",
                  (DWORD)code[1] | ((DWORD)code[2] << 8) |
                  ((DWORD)code[3] << 16) | ((DWORD)code[4] << 24));
        if (code[1] != 0x77 || code[2] != 0x77 || code[3] != 0x77 || code[4] != 0x77)
        {
            MEMORY_BASIC_INFORMATION mbi;
            mbi.Protect = 0;
            VirtualQuery(code, &mbi, sizeof(mbi));
            Report("[selfmod32] rung7 NOTE: the direct write did not land — the "
                   "emulator's SMC trap is a real page protection and the "
                   "kernel honours it\n");
            ReportHex("[selfmod32] rung7 page protection seen by the kernel =",
                      mbi.Protect);
            /* Only a FAIL since it reads the write's own status: until then it
             * read NtReadVirtualMemory's, printed on every FEX boot, and was
             * ignored as noise. */
            if (write_st >= 0)
            {
                Report("[selfmod32] FAIL: rung7 — NtWriteVirtualMemory "
                       "returned SUCCESS but the write did not land. A refused "
                       "write must say so.\n");
                ExitProcess(SM_FAIL_COUNT);
            }

            /* Undo the trap the way WriteProcessMemory does, then require the
             * write to land AND the new code to run — so the rung still gates
             * the dirty/flush notifications rather than quietly passing. */
            old = 0;
            if (!VirtualProtect(code, 0x1000, PAGE_EXECUTE_READWRITE, &old))
            {
                Report("[selfmod32] FAIL: rung7 — VirtualProtect to un-trap failed\n");
                ExitProcess(SM_FAIL_WPM);
            }
            st = NtWVM(GetCurrentProcess(), code, patch, CODE_LEN,
                       (SIZE_T *)&cells[0]);
            if (st < 0 || code[1] != 0x77)
            {
                Report("[selfmod32] FAIL: rung7 — the write still did not land "
                       "on an explicitly writable page\n");
                ExitProcess(SM_FAIL_WPM);
            }
        }

        /* The direct write bypassed kernel32's flush, so this also asks
         * whether NtWriteVirtualMemory ALONE marks the code dirty. */
        FlushInstructionCache(GetCurrentProcess(), code, CODE_LEN);
        got = fn();
        ReportHex("[selfmod32] rung7 directly-written code returned", got);
        if (got != 0x77777777)
        {
            Report("[selfmod32] FAIL: rung7 — the bytes are in memory but the "
                   "STALE translation survived\n");
            ExitProcess(SM_FAIL_WPM);
        }
    }

    Report("[selfmod32] PASS: the guest can generate, rewrite, recycle and "
           "hot-patch its own code\n");
    ExitProcess(SM_PASS);
    return 0;
}
