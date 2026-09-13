#pragma once
#include <stdint.h>
#include <stddef.h>
#include <string.h>
#define IN
#define OUT
#define OPTIONAL
#define NTAPI
#define PORTCLASSAPI extern "C"
#define STDMETHODIMP NTSTATUS
#define STDMETHODIMP_(x) x
#define VOID void
#define TRUE 1
#define FALSE 0
#define PASSIVE_LEVEL 0
#define NonPagedPool 0
#define STATUS_SUCCESS ((NTSTATUS)0)
#define STATUS_UNSUCCESSFUL ((NTSTATUS)0xc0000001)
#define STATUS_INVALID_PARAMETER ((NTSTATUS)0xc000000d)
#define STATUS_INSUFFICIENT_RESOURCES ((NTSTATUS)0xc000009a)
#define NT_SUCCESS(x) ((x)>=0)
#define RtlCopyMemory memcpy
#define FIELD_OFFSET(t, f) offsetof(t,f)
typedef int32_t NTSTATUS, LONG;
typedef uint32_t ULONG;
typedef int32_t POOL_TYPE, CM_RESOURCE_TYPE;
typedef void *PVOID;
typedef size_t SIZE_T;
typedef struct { uint32_t Data1; uint16_t Data2, Data3; uint8_t Data4[8]; } GUID;
typedef GUID IID;
typedef struct { uint16_t Length, MaximumLength; uint16_t *Buffer; } UNICODE_STRING;
#ifdef __cplusplus
extern "C" {
#endif
uint8_t KeGetCurrentIrql(void);
void RtlAssert(PVOID, PVOID, ULONG, char *);
unsigned long fixture_assertion_count(void);
void fixture_set_irql(uint8_t);
ULONG DbgPrint(const char *, ...);
PVOID fixture_allocate(POOL_TYPE, SIZE_T, ULONG);
void fixture_free(PVOID, ULONG, int);
NTSTATUS RtlStringFromGUID(const GUID *, UNICODE_STRING *);
void RtlFreeUnicodeString(UNICODE_STRING *);
extern const GUID IID_IResourceList, IID_IUnknown;
#ifdef __cplusplus
}
#endif
/* Deliberately small matching host fixtures: test buffer semantics, not NT ABI. */
typedef struct { uint8_t Type, ShareDisposition; uint16_t Flags; uint32_t Data[3]; } CM_PARTIAL_RESOURCE_DESCRIPTOR, *PCM_PARTIAL_RESOURCE_DESCRIPTOR;
typedef struct { uint16_t Version, Revision; ULONG Count; CM_PARTIAL_RESOURCE_DESCRIPTOR PartialDescriptors[1]; } CM_PARTIAL_RESOURCE_LIST;
typedef struct { int32_t InterfaceType; ULONG BusNumber; CM_PARTIAL_RESOURCE_LIST PartialResourceList; } CM_FULL_RESOURCE_DESCRIPTOR;
typedef struct { ULONG Count; CM_FULL_RESOURCE_DESCRIPTOR List[1]; } CM_RESOURCE_LIST, *PCM_RESOURCE_LIST;
