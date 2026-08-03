/* msvcrt_time_types.h — the Win32 time vocabulary time.c reaches for.
 *
 * Separate from msvcrt_file_types.h for the reason that header states about itself:
 * each of these is a struct a caller READS AND WRITES FIELD BY FIELD, so its layout is
 * part of the contract, and it is checked on the Rust side by msvcrt-sys's bindgen of
 * the SAME headers plus ref_integrity's offset comparison. `SYSTEMTIME` is the worked
 * example — eight `WORD`s whose ORDER is the whole meaning, and a wDay/wDayOfWeek swap
 * would link, run, and produce a calendar wrong by a variable number of days.
 *
 * FILETIME and LARGE_INTEGER already live in msvcrt_file_types.h; this header assumes
 * it is included after that one.
 */
#ifndef MSVCRT_DIFFTEST_TIME_TYPES_H
#define MSVCRT_DIFFTEST_TIME_TYPES_H

typedef struct _SYSTEMTIME {
  WORD wYear;
  WORD wMonth;
  WORD wDayOfWeek;
  WORD wDay;
  WORD wHour;
  WORD wMinute;
  WORD wSecond;
  WORD wMilliseconds;
} SYSTEMTIME, *PSYSTEMTIME, *LPSYSTEMTIME;

typedef struct _TIME_ZONE_INFORMATION {
  LONG       Bias;
  WCHAR      StandardName[32];
  SYSTEMTIME StandardDate;
  LONG       StandardBias;
  WCHAR      DaylightName[32];
  SYSTEMTIME DaylightDate;
  LONG       DaylightBias;
} TIME_ZONE_INFORMATION, *PTIME_ZONE_INFORMATION, *LPTIME_ZONE_INFORMATION;

/* winbase.h:307-310. TIME_ZONE_ID_INVALID is 0xFFFFFFFF and NOT -1: `_ftime64`
 * compares GetTimeZoneInformation's DWORD return against it, and a signed -1 written
 * from host habit compares false against every possible return. */
#define TIME_ZONE_ID_UNKNOWN  0
#define TIME_ZONE_ID_STANDARD 1
#define TIME_ZONE_ID_DAYLIGHT 2
#define TIME_ZONE_ID_INVALID  ((DWORD)0xFFFFFFFF)

#ifndef LOCALE_NEUTRAL
#define LOCALE_NEUTRAL 0
#endif

/* <sys/timeb.h>: what _ftime32 / _ftime64 fill in. Two DISTINCT structs — the 32-bit
 * one is not a narrowing view of the 64-bit one, and `_ftime32` copies field by field
 * for exactly that reason. */
struct __timeb32 {
  __time32_t time;
  unsigned short millitm;
  short timezone;
  short dstflag;
};

struct __timeb64 {
  __time64_t time;
  unsigned short millitm;
  short timezone;
  short dstflag;
};

/* ── the harness knobs timeimpl.c exports ──────────────────────────────────
 *
 * Declared here, in the header both the reference TU and the tests include, for the
 * reason msvcrt_shim_decls.h gives: a knob whose definition drifts from what a caller
 * spells becomes an ABI mismatch surfacing as a wrong answer in an unrelated row,
 * rather than a compile error. */
void      ref_time_set_now(ULONGLONG ticks);
ULONGLONG ref_time_ticks_from_unix(long long unix_secs, unsigned int millis);
void      ref_time_set_zone(int preset);
int       ref_time_zone_count(void);
BOOL      ref_time_in_dst(const TIME_ZONE_INFORMATION *tzi, const SYSTEMTIME *utc);

#endif /* MSVCRT_DIFFTEST_TIME_TYPES_H */
