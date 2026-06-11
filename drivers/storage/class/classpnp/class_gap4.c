#include "classp.h"
#include "debug.h"

NTSTATUS
NTAPI 
ClassShutdownFlush(
      PDEVICE_OBJECT DeviceObject,
      PIRP Irp
    )
{
    PCOMMON_DEVICE_EXTENSION commonExtension = DeviceObject->DeviceExtension;

    ULONG isRemoved;

    isRemoved = ClassAcquireRemoveLock(DeviceObject, Irp);
    _Analysis_assume_(isRemoved);
    if(isRemoved) {

        ClassReleaseRemoveLock(DeviceObject, Irp);

        Irp->IoStatus.Status = STATUS_DEVICE_DOES_NOT_EXIST;

        ClassCompleteRequest(DeviceObject, Irp, IO_NO_INCREMENT);

        return STATUS_DEVICE_DOES_NOT_EXIST;
    }

    if (commonExtension->DevInfo->ClassShutdownFlush) {

        //
        // Call the device-specific driver's routine.
        //

        return commonExtension->DevInfo->ClassShutdownFlush(DeviceObject, Irp);
    }

    //
    // Device-specific driver doesn't support this.
    //

    Irp->IoStatus.Status = STATUS_INVALID_DEVICE_REQUEST;

    ClassReleaseRemoveLock(DeviceObject, Irp);
    ClassCompleteRequest(DeviceObject, Irp, IO_NO_INCREMENT);

    return STATUS_INVALID_DEVICE_REQUEST;
} // end ClassShutdownFlush()
