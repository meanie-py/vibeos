# virtdisk.dll for an OS with no virtual-disk (VHD/VHDX/ISO) provider. The three entry points
# Raspberry Pi Imager imports are real (sdk/lib/rust/virtdisk-rust); the rest are stubs as in
# Wine's virtdisk.spec.
@ stub AddVirtualDiskParent
@ stub AttachVirtualDisk
@ stub BreakMirrorVirtualDisk
@ stub CompactVirtualDisk
@ stub CreateVirtualDisk
@ stub DeleteVirtualDiskMetadata
@ stdcall DetachVirtualDisk(ptr long long)
@ stub EnumerateVirtualDiskMetadata
@ stub ExpandVirtualDisk
@ stub GetAllAttachedVirtualDiskPhysicalPaths
@ stdcall GetStorageDependencyInformation(ptr long long ptr ptr)
@ stub GetVirtualDiskInformation
@ stub GetVirtualDiskMetadata
@ stub GetVirtualDiskOperationProgress
@ stub GetVirtualDiskPhysicalPath
@ stub MergeVirtualDisk
@ stub MirrorVirtualDisk
@ stdcall OpenVirtualDisk(ptr wstr long long ptr ptr)
@ stub ResizeVirtualDisk
@ stub SetVirtualDiskInformation
@ stub SetVirtualDiskMetadata
