/*
 * PROJECT:     ReactOS Kernel Security Support Provider Interface Driver
 * LICENSE:     MIT (https://spdx.org/licenses/MIT)
 * PURPOSE:     Driver entry point for ksecdd
 * COPYRIGHT:   Copyright 2014 Timo Kreuzer <timo.kreuzer@reactos.org>
 */

/* INCLUDES *******************************************************************/

#include "ksecdd.h"

#define NDEBUG
#include <debug.h>

/* GLOBALS ********************************************************************/

PDEVICE_OBJECT KsecDeviceObject;


/* FUNCTIONS ******************************************************************/

NTSTATUS
NTAPI
DriverEntry(
    _In_ PDRIVER_OBJECT DriverObject,
    _In_ PUNICODE_STRING RegistryPath)
{
    UNICODE_STRING DeviceName = RTL_CONSTANT_STRING(L"\\Device\\KsecDD");
    NTSTATUS Status;

    /* Debugcon traces for i386 boot debugging */
#ifdef _M_IX86
    __asm__ __volatile__("outb %b0, %w1" : : "a"((unsigned char)'K'), "Nd"((unsigned short)0xE9));
    __asm__ __volatile__("outb %b0, %w1" : : "a"((unsigned char)'0'), "Nd"((unsigned short)0xE9));
#endif

    /* Create the KsecDD device */
    Status = IoCreateDevice(DriverObject,
                            0,
                            &DeviceName,
                            FILE_DEVICE_KSEC,
                            0x100u,
                            FALSE,
                            &KsecDeviceObject);
#ifdef _M_IX86
    __asm__ __volatile__("outb %b0, %w1" : : "a"((unsigned char)'K'), "Nd"((unsigned short)0xE9));
    __asm__ __volatile__("outb %b0, %w1" : : "a"((unsigned char)'1'), "Nd"((unsigned short)0xE9));
#endif
    if (!NT_SUCCESS(Status))
    {
        DPRINT1("Failed to create KsecDD device: 0x%lx\n", Status);
        return Status;
    }

    /* Set up dispatch table */
    DriverObject->MajorFunction[IRP_MJ_CREATE] = KsecDdDispatch;
    DriverObject->MajorFunction[IRP_MJ_CLOSE] = KsecDdDispatch;
    DriverObject->MajorFunction[IRP_MJ_READ] = KsecDdDispatch;
    DriverObject->MajorFunction[IRP_MJ_WRITE] = KsecDdDispatch;
    DriverObject->MajorFunction[IRP_MJ_QUERY_INFORMATION] = KsecDdDispatch;
    DriverObject->MajorFunction[IRP_MJ_QUERY_VOLUME_INFORMATION] = KsecDdDispatch;
    DriverObject->MajorFunction[IRP_MJ_DEVICE_CONTROL] = KsecDdDispatch;

#ifdef _M_IX86
    __asm__ __volatile__("outb %b0, %w1" : : "a"((unsigned char)'K'), "Nd"((unsigned short)0xE9));
    __asm__ __volatile__("outb %b0, %w1" : : "a"((unsigned char)'2'), "Nd"((unsigned short)0xE9));
#endif

    /* Initialize */
    KsecInitializeEncryptionSupport();

#ifdef _M_IX86
    __asm__ __volatile__("outb %b0, %w1" : : "a"((unsigned char)'K'), "Nd"((unsigned short)0xE9));
    __asm__ __volatile__("outb %b0, %w1" : : "a"((unsigned char)'3'), "Nd"((unsigned short)0xE9));
#endif

    return STATUS_SUCCESS;
}
