
add_subdirectory(cdfs)
add_subdirectory(fastfat)
add_subdirectory(fs_rec)
add_subdirectory(msfs)
add_subdirectory(mup)
add_subdirectory(npfs)
# nfs / udfs / vfatfs deleted (2026-06-20, filesystems lane): untranslated imported/niche
# non-target FS drivers — nfs = imported UMich ms-nfs41-client (→ userland later); udfs =
# UDF C++ (DVD/Blu-ray, niche); vfatfs = FATX/Xbox-only here (normal FAT → fastfat). Same
# precedent as the btrfs/ext2/ntfs deletion (14a0563b291). Recoverable from ReactOS + rustconv.
