/* Host fixture for wave/streaming.c's DoWaveStreaming and CompleteIO: the types and
 * macros the two bodies use, laid out exactly as the Rust test's #[repr(C)] mirrors so
 * one scripted world drives either side. Device lookups, CompleteWaveHeader and the
 * allocator are the Rust test's (fx_*). */
#include <stdint.h>
#include <stddef.h>
#include <string.h>
typedef void VOID;
typedef uint32_t DWORD;
typedef int32_t BOOL;
typedef uint32_t MMRESULT;
typedef uint8_t MMDEVICE_TYPE;
typedef void *PVOID, *LPVOID;
typedef char *LPSTR;
typedef uintptr_t DWORD_PTR;
#define IN
#define CALLBACK
#define TRUE 1
#define FALSE 0
#define INFINITE 0xFFFFFFFFu
#define MMSYSERR_NOERROR 0
#define MMSUCCESS(r) ((r) == MMSYSERR_NOERROR)
#define WHDR_DONE 0x01
#define WHDR_PREPARED 0x02
#define WHDR_BEGINLOOP 0x04
#define WHDR_ENDLOOP 0x08
#define WHDR_INQUEUE 0x10
#define WAVE_IN_DEVICE_TYPE 1
#define WAVE_OUT_DEVICE_TYPE 2

typedef struct wavehdr_tag {
    LPSTR lpData;
    DWORD dwBufferLength;
    DWORD dwBytesRecorded;
    DWORD_PTR dwUser;
    DWORD dwFlags;
    DWORD dwLoops;
    struct wavehdr_tag *lpNext;
    DWORD_PTR reserved;
} WAVEHDR, *PWAVEHDR;

typedef struct {
    DWORD BytesCommitted;
    DWORD BytesCompleted;
    DWORD LoopsToCommit;
    DWORD LoopsToComplete;
} WAVEHDR_EXTENSION, *PWAVEHDR_EXTENSION;

typedef struct { DWORD_PTR Internal, InternalHigh; DWORD Offset, OffsetHigh; PVOID hEvent; } OVERLAPPED, *LPOVERLAPPED;
typedef VOID (*LPOVERLAPPED_COMPLETION_ROUTINE)(DWORD, DWORD, LPOVERLAPPED);

typedef struct _SOUND_DEVICE_INSTANCE {
    PWAVEHDR HeadWaveHeader;
    PWAVEHDR TailWaveHeader;
    DWORD OutstandingBuffers;
    DWORD LoopsRemaining;
    DWORD FrameSize;
    DWORD BufferCount;
    BOOL ResetInProgress;
    BOOL bPaused;
} SOUND_DEVICE_INSTANCE, *PSOUND_DEVICE_INSTANCE;

typedef struct _SOUND_OVERLAPPED {
    OVERLAPPED Standard;
    PSOUND_DEVICE_INSTANCE SoundDeviceInstance;
    PWAVEHDR Header;
    LPOVERLAPPED_COMPLETION_ROUTINE OriginalCompletionRoutine;
    PVOID CompletionContext;
} SOUND_OVERLAPPED, *PSOUND_OVERLAPPED;

typedef struct _SOUND_DEVICE SOUND_DEVICE, *PSOUND_DEVICE;
typedef MMRESULT (*WAVE_COMMIT_FUNC)(PSOUND_DEVICE_INSTANCE, PVOID, DWORD, PSOUND_OVERLAPPED,
                                     LPOVERLAPPED_COMPLETION_ROUTINE);
typedef struct { WAVE_COMMIT_FUNC CommitWaveBuffer; } MMFUNCTION_TABLE, *PMMFUNCTION_TABLE;

MMRESULT fx_GetSoundDeviceFromInstance(PSOUND_DEVICE_INSTANCE, PSOUND_DEVICE *);
MMRESULT fx_GetSoundDeviceType(PSOUND_DEVICE, MMDEVICE_TYPE *);
MMRESULT fx_GetSoundDeviceFunctionTable(PSOUND_DEVICE, PMMFUNCTION_TABLE *);
VOID fx_CompleteWaveHeader(PSOUND_DEVICE_INSTANCE, PWAVEHDR);
PVOID fx_AllocateMemory(size_t);
VOID fx_FreeMemory(PVOID);
#define GetSoundDeviceFromInstance fx_GetSoundDeviceFromInstance
#define GetSoundDeviceType fx_GetSoundDeviceType
#define GetSoundDeviceFunctionTable fx_GetSoundDeviceFunctionTable
#define CompleteWaveHeader fx_CompleteWaveHeader
#define FreeMemory fx_FreeMemory
#define AllocateStruct(thing) ((thing *)fx_AllocateMemory(sizeof(thing)))
#define ZeroMemory(p, n) memset((p), 0, (n))
#define SND_ASSERT(x) ((void)0)
#define SND_TRACE(...) ((void)0)
#define SND_WARN(...) ((void)0)
#define DoWaveStreaming ref_DoWaveStreaming
#define CompleteIO ref_CompleteIO
VOID ref_DoWaveStreaming(PSOUND_DEVICE_INSTANCE);
VOID ref_CompleteIO(DWORD, DWORD, LPOVERLAPPED);
