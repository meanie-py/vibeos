#pragma once
#include "portcls.h"
#include <cstdlib>
#include <new>
#include <cstring>
#include "source_tag.hpp"
#define ASSERT(x) do { if (!(x)) RtlAssert(nullptr,nullptr,__LINE__,nullptr); } while(0)
#define PC_ASSERT_IRQL_EQUAL(x) ASSERT(KeGetCurrentIrql()==(x))
static inline LONG InterlockedIncrement(volatile LONG *x) { return __atomic_add_fetch(x, 1, __ATOMIC_SEQ_CST); }
static inline LONG InterlockedDecrement(volatile LONG *x) { return __atomic_sub_fetch(x, 1, __ATOMIC_SEQ_CST); }
static inline bool IsEqualGUIDAligned(REFIID a, REFIID b) { return std::memcmp(&a,&b,sizeof(GUID)) == 0; }
static inline NTSTATUS RtlStringFromGUID(REFIID id, UNICODE_STRING *s) { return RtlStringFromGUID(&id,s); }
static inline PVOID AllocateItem(POOL_TYPE p, SIZE_T n, ULONG t) { return fixture_allocate(p,n,t); }
static inline void FreeItem(PVOID p, ULONG t) { fixture_free(p,t,1); }
inline void *operator new(size_t n, POOL_TYPE p, ULONG t) { return fixture_allocate(p,n,t); }
#include "unknown_base.hpp"
