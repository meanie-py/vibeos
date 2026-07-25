/*
 * <poppack.h> — pop whatever packing was pushed.  See refstub/pshpack8.h for
 * why the PSDK's copy is unreachable from a reference TU.
 */
#if !defined(RC_INVOKED)
#pragma pack(pop)
#endif
