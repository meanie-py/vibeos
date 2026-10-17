/* Host fixture for mmixer's volume conversion: just the fields the function reads, laid out
 * exactly as the Rust test's #[repr(C)] mirrors so one object serves both sides. The two
 * calls out (MMixerSetGetControlDetails, MMixerNotifyControlChange) go to the Rust test's
 * fixture, which both sides share. */
#include <stdint.h>
#include <stddef.h>
#define IN
#define OUT
typedef int32_t LONG, *PLONG;
typedef uint32_t ULONG, DWORD;
typedef int64_t INT64;
typedef void *PVOID, *HANDLE;
typedef enum {
    MM_STATUS_SUCCESS = 0, MM_STATUS_NOTINITIALIZED, MM_STATUS_NOT_IMPLEMENTED,
    MM_STATUS_NO_MORE_DEVICES, MM_STATUS_MORE_ENTRIES, MM_STATUS_INVALID_PARAMETER,
    MM_STATUS_UNSUCCESSFUL, MM_STATUS_NO_MEMORY
} MIXER_STATUS;
typedef struct { int unused; } MIXER_CONTEXT, *PMIXER_CONTEXT;
typedef struct { int unused; } MIXER_INFO, *LPMIXER_INFO;
typedef struct { int unused; } MIXERLINE_EXT, *LPMIXERLINE_EXT;
typedef struct { DWORD dwControlID; } MIXERCONTROLW;
typedef struct { MIXERCONTROLW Control; HANDLE hDevice; PVOID ExtraData; } MIXERCONTROL_EXT, *LPMIXERCONTROL_EXT;
typedef struct { DWORD cChannels; DWORD cbDetails; PVOID paDetails; } MIXERCONTROLDETAILS, *LPMIXERCONTROLDETAILS;
typedef struct { DWORD dwValue; } MIXERCONTROLDETAILS_UNSIGNED, *LPMIXERCONTROLDETAILS_UNSIGNED;
typedef struct { LONG SignedMinimum; LONG SignedMaximum; } MIXERVOLUME_DATA, *LPMIXERVOLUME_DATA;
#define KSPROPERTY_AUDIO_VOLUMELEVEL 4
#define MM_MIXM_CONTROL_CHANGE 0x3D1
MIXER_STATUS fixture_set_get(PMIXER_CONTEXT, HANDLE, ULONG, ULONG, ULONG, ULONG, PLONG);
void fixture_notify(PMIXER_CONTEXT, LPMIXER_INFO, ULONG, ULONG);
#define MMixerSetGetControlDetails fixture_set_get
#define MMixerNotifyControlChange fixture_notify
