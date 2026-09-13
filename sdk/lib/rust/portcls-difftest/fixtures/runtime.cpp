#include "ntddk.h"
#include <cstdlib>
#include <new>
static unsigned long assertions;
static uint8_t irql;
static unsigned calls, frees, live, fail_at, guid_frees;
static uint64_t events;
extern "C" const GUID IID_IResourceList={1,0,0,{0}}, IID_IUnknown={2,0,0,{0}};
extern "C" uint8_t KeGetCurrentIrql() { return irql; }
extern "C" void RtlAssert(PVOID, PVOID, ULONG, char *) { ++assertions; }
extern "C" unsigned long fixture_assertion_count() { return assertions; }
extern "C" void fixture_set_irql(uint8_t level) { irql = level; }
extern "C" ULONG DbgPrint(const char *, ...) { return 0; }
extern "C" PVOID fixture_allocate(POOL_TYPE pool, SIZE_T size, ULONG tag) {
    ++calls; events = events*131 + tag + pool;
    if (calls == fail_at) return nullptr;
    ++live; return std::calloc(1,size);
}
extern "C" void fixture_free(PVOID p, ULONG tag, int tagged) {
    if (p) { ++frees; --live; events = events*131 + (tagged ? tag : 17); std::free(p); }
}
void operator delete(void *p) noexcept { fixture_free(p,0,0); }
void operator delete(void *p, size_t) noexcept { fixture_free(p,0,0); }
extern "C" NTSTATUS RtlStringFromGUID(const GUID *, UNICODE_STRING *s) {
    static uint16_t text[]={ 'G',0 }; s->Buffer=text; s->Length=2; s->MaximumLength=4; return STATUS_SUCCESS;
}
extern "C" void RtlFreeUnicodeString(UNICODE_STRING *) { ++guid_frees; }
extern "C" void fixture_reset(unsigned failure) {
    if (live) std::abort();
    calls=frees=guid_frees=0; assertions=0; events=0; fail_at=failure;
}
extern "C" void fixture_failure(unsigned failure) { fail_at=failure; }
extern "C" void fixture_stats(uint64_t *out) {
    out[0]=calls; out[1]=frees; out[2]=live; out[3]=assertions; out[4]=events; out[5]=guid_frees;
}
