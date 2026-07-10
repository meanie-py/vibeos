/*
 * Kernel-mode SEH boot self-test (born as the DUMMY_PSEH-verdict PoC —
 * docs/engineering/dummy-pseh-verdict.md, field manual §9.27).
 *
 * Compiled with -fms-extensions (see the c_shims wiring in
 * ntoskrnl-cargo/build.rs + ntkrnlmp-cargo/build.rs) so __try/__except are
 * the REAL clang SEH keywords, not the pseh2.h DUMMY macros — this file
 * deliberately includes no headers so no macro layer can interfere. Codegen
 * emits a .pdata/.xdata scope table + __filt_SehPocRun and references
 * __C_specific_handler (linked via sdk/lib/crt/except/amd64/ehandler.c).
 *
 * Called once from Phase1InitializationDiscard (ex/init.rs, amd64).
 * Exercises the full software-exception chain: ExRaiseStatus ->
 * KiDispatchException -> RtlDispatchException -> RtlVirtualUnwind/.pdata
 * walk -> __C_specific_handler -> filter -> RtlUnwindEx -> handler.
 * Prints `[SEH-POC] ... (PASS)` on serial; a FAIL or a bugcheck here means
 * the exception machinery regressed (historically: requires-uwtable
 * missing from the *-kernel.json rustc targets -> no .pdata for Rust
 * frames -> recursive-dispatch stack overflow, BUGCHECK 0x7F).
 */

void ExRaiseStatus(long Status);
unsigned long DbgPrint(const char *Format, ...);

#define SEH_POC_STATUS 0xE0DEAD01L

void SehPocRun(void)
{
    volatile int caught = 0;
    volatile long code = 0;

    __try {
        ExRaiseStatus(SEH_POC_STATUS);
        caught = -1; /* unreachable if the raise dispatches */
    } __except (_exception_code() == SEH_POC_STATUS ? 1 : 0) {
        caught = 1;
        code = _exception_code();
    }

    DbgPrint("[SEH-POC] native __try/__except: caught=%d code=0x%lx (%s)\n",
             caught, code,
             (caught == 1 && code == SEH_POC_STATUS) ? "PASS" : "FAIL");
}
