/* Deterministic NtQuerySystemTime for grading sdk/lib/rtl/encode.c on the host.
   Mirrored exactly by encode_host_time.rs. A call-counted script: a time whose byte 1
   is nonzero, one where the hash search must reach byte 2, one with bytes 1..7 zero
   (so the hash falls back to 1), and a failing NTSTATUS. */
#include <rtl.h>
static const LONGLONG encode_host_times[4] = {0x0000000000001200LL, 0x0000000000AB0000LL, 0x00000000000000FFLL, 0x7FFFFFFFFFFFFFFFLL};
static const NTSTATUS encode_host_status[4] = {0, 0, 0, (NTSTATUS)0xC0000001};
static unsigned encode_host_calls;
static NTSTATUS NTAPI NtQuerySystemTime(PLARGE_INTEGER time)
{
    unsigned i = encode_host_calls++ & 3;
    time->QuadPart = encode_host_times[i];
    return encode_host_status[i];
}
