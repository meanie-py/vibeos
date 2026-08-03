/*
 * timeimpl.c — the Win32 clock and time-zone layer underneath time.c.
 *
 * time.c implements the whole CRT calendar itself (mktime/gmtime/localtime/strftime,
 * the DST rules, the `_tzname` strings).  What it does NOT implement is the layer
 * below: it asks Win32 what time it is, what the zone is, and how to convert a
 * SYSTEMTIME to a FILETIME.  This file is that layer.
 *
 * Compiled WITHOUT the ref_* rename prelude, like fileimpl.c and refimpl.c, so these
 * land under the plain Win32 names time.c calls.  Safe for the same reason: msvcrt
 * exports none of them, so there is no code-under-test for a reference to reach.
 *
 * ── why the clock is FROZEN and not the host's ──────────────────────────────
 *
 * The subject and the reference are two bodies of code in ONE process, called one
 * after the other.  If both asked the host what time it is they would get two
 * different answers, and every row over `_time64`, `_ftime64`, `clock`, `_strdate`
 * or `asctime` would be a coin flip that usually agrees.  "Usually agrees" is the
 * worst possible oracle: it goes green in CI and red at midnight.
 *
 * So the instant is a KNOB (`ref_time_set_now`), set by the test before each row and
 * read by both sides.  That is the same rule as fileimpl.c's `ref_set_console_fd` —
 * where the honest answer depends on the environment rather than on POSIX, the
 * environment gets an explicit setting instead of a hardcoded guess — and it buys
 * something a real clock cannot: a row can sit on 1970-01-01T00:00:00Z, on a leap
 * day, on the second a DST transition fires, or on `_MAX__TIME64_T`.
 *
 * ── and why the ZONE is a knob too ─────────────────────────────────────────
 *
 * `_tzset`, `is_dst`, `mktime_helper` and `_ftime64` all branch on the zone, and the
 * interesting branches are the ones a UTC-only model never reaches: a southern
 * hemisphere zone whose DST window WRAPS the year end, a zone with a half-hour bias,
 * and a `GetTimeZoneInformation` that FAILS.  Each is a preset below.  A shim that
 * always succeeded with Bias 0 would make `is_dst` unreachable and the harness would
 * report a confident green over the half of time.c that has the arithmetic in it.
 *
 * Nothing here is transcribed from Windows.  The calendar is computed (the standard
 * days-from-civil algorithm), and the zone presets are declared data whose only job
 * is to drive real branches — both sides call through here, so what is graded is
 * time.c's LOGIC around the conversion, never the conversion itself.
 */
#include <msvcrt.h>

#include <string.h>

#include <msvcrt_file_types.h>
#include <msvcrt_crt_consts.h>
#include <msvcrt_time_types.h>

#include <time.h>

/* refimpl.c owns the single last-error slot both sides read. */
void msvcrt_ref_set_last_error(DWORD err);
#define SET_ERR(e) msvcrt_ref_set_last_error(e)

/* ── the frozen instant ─────────────────────────────────────────────────── */

/* 100ns ticks since 1601-01-01, the FILETIME epoch. 0 means "ask the host", which is
 * the state a suite that never calls the knob is in; every graded row sets it. */
static ULONGLONG ref_now_ticks = 0;

#define TICKSPERSEC        10000000ULL
#define SECS_1601_TO_1970  11644473600ULL

void ref_time_set_now(ULONGLONG ticks) { ref_now_ticks = ticks; }

/* Convenience for the tests: an instant expressed as a Unix time. */
ULONGLONG ref_time_ticks_from_unix(long long unix_secs, unsigned int millis)
{
    return ((ULONGLONG)(unix_secs + (long long)SECS_1601_TO_1970)) * TICKSPERSEC
         + (ULONGLONG)millis * 10000ULL;
}

static ULONGLONG now_ticks(void)
{
    if (ref_now_ticks) return ref_now_ticks;
    /* Unfrozen: the host clock, whole seconds. Only reachable from a suite that never
     * set the knob — deliberately coarse, so a row that forgot to freeze is more
     * likely to be caught by a disagreement than to pass by luck. */
    return ((ULONGLONG)time(NULL) + SECS_1601_TO_1970) * TICKSPERSEC;
}

/* ── the calendar ───────────────────────────────────────────────────────── */

/* Days from 1970-01-01 to y-m-d (proleptic Gregorian), for y >= 1601. The shift-by-
 * March trick makes the leap day the LAST day of the internal year, which is what
 * removes every special case from the month-length arithmetic. */
static long long days_from_civil(int y, unsigned m, unsigned d)
{
    y -= m <= 2;
    const long long era = (y >= 0 ? y : y - 399) / 400;
    const unsigned yoe = (unsigned)(y - era * 400);            /* [0, 399] */
    const unsigned doy = (153u * (m + (m > 2 ? -3 : 9)) + 2u) / 5u + d - 1u;
    const unsigned doe = yoe * 365u + yoe / 4u - yoe / 100u + doy;
    return era * 146097LL + (long long)doe - 719468LL;
}

static void civil_from_days(long long z, int *y, unsigned *m, unsigned *d)
{
    z += 719468;
    const long long era = (z >= 0 ? z : z - 146096) / 146097;
    const unsigned doe = (unsigned)(z - era * 146097);
    const unsigned yoe = (doe - doe / 1460u + doe / 36524u - doe / 146096u) / 365u;
    const long long yy = (long long)yoe + era * 400;
    const unsigned doy = doe - (365u * yoe + yoe / 4u - yoe / 100u);
    const unsigned mp = (5u * doy + 2u) / 153u;
    *d = doy - (153u * mp + 2u) / 5u + 1u;
    *m = mp + (mp < 10 ? 3 : -9);
    *y = (int)(yy + (*m <= 2));
}

/* ── the zone presets ───────────────────────────────────────────────────── */

/* A recurring rule is `wYear == 0`, `wDay` = which occurrence of `wDayOfWeek` in
 * `wMonth` (1..4, or 5 meaning LAST). time.c's `is_dst` builds exactly this shape by
 * hand, so a shim that only understood absolute dates would make it always answer
 * "not DST" and the whole DST half of localtime would go ungraded. */
static const TIME_ZONE_INFORMATION ZONES[] = {
    /* 0: UTC. No DST — `_tzset` takes the `!DaylightDate.wMonth` arm. */
    { 0, L"UTC", { 0 }, 0, L"UTC", { 0 }, 0 },
    /* 1: US Pacific, post-2007 rules. Bias 480 = UTC-8. DST starts the 2nd Sunday in
     *    March and ends the 1st Sunday in November, both at 02:00 local. */
    { 480, L"Pacific Standard Time",
      { 0, 11, 0, 1, 2, 0, 0, 0 }, 0,
      L"Pacific Daylight Time",
      { 0, 3, 0, 2, 2, 0, 0, 0 }, -60 },
    /* 2: India. Bias -330 = UTC+5:30, no DST — the half-hour offset is the case that
     *    catches an implementation doing whole-hour arithmetic. */
    { -330, L"India Standard Time", { 0 }, 0, L"India Standard Time", { 0 }, 0 },
    /* 3: Sydney. Southern hemisphere, so the DST window WRAPS the year end: it starts
     *    the 1st Sunday in October and ends the 1st Sunday in April. A rule engine
     *    that assumes start < end gets this exactly backwards, all year. */
    { -600, L"AUS Eastern Standard Time",
      { 0, 4, 0, 1, 3, 0, 0, 0 }, 0,
      L"AUS Eastern Daylight Time",
      { 0, 10, 0, 1, 2, 0, 0, 0 }, -60 },
};
#define ZONE_COUNT ((int)(sizeof(ZONES) / sizeof(ZONES[0])))

/* -1 selects "GetTimeZoneInformation FAILS", which is a real Windows outcome and the
 * only way `_tzset`'s early-out arm is reachable. */
static int ref_zone = 0;

void ref_time_set_zone(int preset) { ref_zone = preset; }
int  ref_time_zone_count(void)     { return ZONE_COUNT; }

/* ── SYSTEMTIME <-> FILETIME ────────────────────────────────────────────── */

static void ticks_to_systemtime(ULONGLONG ticks, SYSTEMTIME *st)
{
    ULONGLONG secs = ticks / TICKSPERSEC;
    unsigned  frac = (unsigned)(ticks % TICKSPERSEC);
    long long days = (long long)(secs / 86400ULL);
    unsigned  rem  = (unsigned)(secs % 86400ULL);
    int y; unsigned m, d;

    civil_from_days(days - 134774LL, &y, &m, &d);   /* 1601-01-01 is day -134774 */
    st->wYear         = (WORD)y;
    st->wMonth        = (WORD)m;
    st->wDay          = (WORD)d;
    /* 1601-01-01 was a Monday, so day 0 of this count is wDayOfWeek 1. */
    st->wDayOfWeek    = (WORD)(((days % 7) + 1) % 7);
    st->wHour         = (WORD)(rem / 3600);
    st->wMinute       = (WORD)((rem / 60) % 60);
    st->wSecond       = (WORD)(rem % 60);
    st->wMilliseconds = (WORD)(frac / 10000);
}

static BOOL systemtime_to_ticks(const SYSTEMTIME *st, ULONGLONG *out)
{
    static const unsigned char mlen[12] = {31,28,31,30,31,30,31,31,30,31,30,31};
    unsigned maxday;

    /* Windows VALIDATES, and it matters: `mktime_helper` calls this on a struct tm the
     * caller supplied, and relies on FALSE to reject an impossible date. A shim that
     * accepted anything would make `_mktime64`'s EINVAL arm unreachable. */
    if (st->wYear < 1601 || st->wYear > 30827) return FALSE;
    if (st->wMonth < 1 || st->wMonth > 12)     return FALSE;
    if (st->wHour > 23 || st->wMinute > 59 || st->wSecond > 59
            || st->wMilliseconds > 999)        return FALSE;
    maxday = mlen[st->wMonth - 1];
    if (st->wMonth == 2 && (st->wYear % 4 == 0
            && (st->wYear % 100 != 0 || st->wYear % 400 == 0)))
        maxday = 29;
    if (st->wDay < 1 || st->wDay > maxday)     return FALSE;

    *out = ((ULONGLONG)(days_from_civil(st->wYear, st->wMonth, st->wDay) + 134774LL)
                * 86400ULL
            + (ULONGLONG)st->wHour * 3600ULL
            + (ULONGLONG)st->wMinute * 60ULL
            + st->wSecond) * TICKSPERSEC
           + (ULONGLONG)st->wMilliseconds * 10000ULL;
    return TRUE;
}

static ULONGLONG ft_to_ticks(const FILETIME *ft)
{
    return ((ULONGLONG)ft->dwHighDateTime << 32) | ft->dwLowDateTime;
}

static void ticks_to_ft(ULONGLONG t, FILETIME *ft)
{
    ft->dwLowDateTime  = (DWORD)(t & 0xffffffffULL);
    ft->dwHighDateTime = (DWORD)(t >> 32);
}

/* ── Win32 ──────────────────────────────────────────────────────────────── */

void WINAPI GetSystemTimeAsFileTime(FILETIME *ft) { ticks_to_ft(now_ticks(), ft); }

/* The "precise" variant differs from the ordinary one only in resolution on real
 * Windows; against a frozen instant they are the same answer, which is the point. */
void WINAPI GetSystemTimePreciseAsFileTime(FILETIME *ft) { ticks_to_ft(now_ticks(), ft); }

NTSTATUS WINAPI NtQuerySystemTime(LARGE_INTEGER *t)
{
    t->QuadPart = (LONGLONG)now_ticks();
    return 0;   /* STATUS_SUCCESS */
}

BOOL WINAPI SystemTimeToFileTime(const SYSTEMTIME *st, FILETIME *ft)
{
    ULONGLONG t;
    if (!systemtime_to_ticks(st, &t)) { SET_ERR(ERROR_INVALID_PARAMETER); return FALSE; }
    ticks_to_ft(t, ft);
    return TRUE;
}

BOOL WINAPI FileTimeToSystemTime(const FILETIME *ft, SYSTEMTIME *st)
{
    ULONGLONG t = ft_to_ticks(ft);
    /* Windows rejects a FILETIME with the sign bit set rather than wrapping, and
     * time.c leans on that: `_gmtime64_s` range-checks BEFORE converting, so a shim
     * that silently wrapped would hide a missing check rather than expose it. */
    if (t & 0x8000000000000000ULL) { SET_ERR(ERROR_INVALID_PARAMETER); return FALSE; }
    ticks_to_systemtime(t, st);
    return TRUE;
}

DWORD WINAPI GetTimeZoneInformation(TIME_ZONE_INFORMATION *tzi)
{
    SYSTEMTIME utc;

    if (ref_zone < 0 || ref_zone >= ZONE_COUNT) {
        SET_ERR(ERROR_INVALID_PARAMETER);
        return TIME_ZONE_ID_INVALID;
    }
    *tzi = ZONES[ref_zone];
    if (!tzi->DaylightDate.wMonth) return TIME_ZONE_ID_UNKNOWN;

    ticks_to_systemtime(now_ticks(), &utc);
    return ref_time_in_dst(tzi, &utc) ? TIME_ZONE_ID_DAYLIGHT : TIME_ZONE_ID_STANDARD;
}

/* Resolve a recurring rule to the actual SYSTEMTIME it fires at in `year`. */
static void resolve_rule(const SYSTEMTIME *rule, int year, SYSTEMTIME *out)
{
    *out = *rule;
    out->wYear = (WORD)year;
    if (rule->wYear) return;             /* already an absolute date */

    {
        /* wDay is the Nth occurrence of wDayOfWeek in the month, 5 = last. */
        long long first = days_from_civil(year, rule->wMonth, 1);
        int first_dow = (int)(((first % 7) + 11) % 7);   /* 1970-01-01 was a Thursday */
        int delta = ((int)rule->wDayOfWeek - first_dow + 7) % 7;
        int day = 1 + delta + 7 * ((int)rule->wDay - 1);
        int y; unsigned m, d;
        /* An Nth-occurrence that runs past the month end means "the last one". */
        while (1) {
            civil_from_days(days_from_civil(year, rule->wMonth, 1) + day - 1, &y, &m, &d);
            if (m == rule->wMonth) break;
            day -= 7;
        }
        out->wDay = (WORD)day;
        out->wDayOfWeek = rule->wDayOfWeek;
    }
}

/* Is `utc` inside the zone's daylight window? Exposed because
 * GetTimeZoneInformation needs it too. */
BOOL ref_time_in_dst(const TIME_ZONE_INFORMATION *tzi, const SYSTEMTIME *utc)
{
    SYSTEMTIME dst_st, std_st;
    ULONGLONG utc_t, dst_t, std_t;

    if (!tzi->DaylightDate.wMonth || !tzi->StandardDate.wMonth) return FALSE;
    if (!systemtime_to_ticks(utc, &utc_t)) return FALSE;

    resolve_rule(&tzi->DaylightDate, utc->wYear, &dst_st);
    resolve_rule(&tzi->StandardDate, utc->wYear, &std_st);
    if (!systemtime_to_ticks(&dst_st, &dst_t)) return FALSE;
    if (!systemtime_to_ticks(&std_st, &std_t)) return FALSE;

    /* The rules are stated in LOCAL time; shift them to UTC before comparing. The
     * daylight rule fires while standard time is in effect and vice versa, which is
     * why the two get different biases. */
    dst_t += (ULONGLONG)((tzi->Bias + tzi->StandardBias) * 60) * TICKSPERSEC;
    std_t += (ULONGLONG)((tzi->Bias + tzi->DaylightBias) * 60) * TICKSPERSEC;

    if (dst_t <= std_t)                       /* northern hemisphere */
        return utc_t >= dst_t && utc_t < std_t;
    return utc_t >= dst_t || utc_t < std_t;   /* southern: the window wraps the year */
}

BOOL WINAPI SystemTimeToTzSpecificLocalTime(const TIME_ZONE_INFORMATION *tzi,
                                            const SYSTEMTIME *utc, SYSTEMTIME *local)
{
    TIME_ZONE_INFORMATION cur;
    ULONGLONG t;
    LONG bias;

    if (!tzi) {
        if (GetTimeZoneInformation(&cur) == TIME_ZONE_ID_INVALID) return FALSE;
        tzi = &cur;
    }
    if (!systemtime_to_ticks(utc, &t)) { SET_ERR(ERROR_INVALID_PARAMETER); return FALSE; }

    bias = tzi->Bias + (ref_time_in_dst(tzi, utc) ? tzi->DaylightBias : tzi->StandardBias);
    /* Bias is UTC = local + bias, in MINUTES, so local time runs the other way. */
    t -= (ULONGLONG)((long long)bias * 60) * TICKSPERSEC;
    ticks_to_systemtime(t, local);
    return TRUE;
}

void WINAPI GetLocalTime(SYSTEMTIME *st)
{
    SYSTEMTIME utc;
    TIME_ZONE_INFORMATION tzi;

    ticks_to_systemtime(now_ticks(), &utc);
    if (GetTimeZoneInformation(&tzi) == TIME_ZONE_ID_INVALID
            || !SystemTimeToTzSpecificLocalTime(&tzi, &utc, st))
        *st = utc;
}

/* ── GetDateFormatW / GetTimeFormatW ────────────────────────────────────── */

/*
 * time.c reaches these from exactly two places, each with a LITERAL picture:
 *
 *     _wstrdate: GetDateFormatW(LOCALE_NEUTRAL, 0, NULL, L"MM'/'dd'/'yy", date, 9)
 *     _wstrtime: GetTimeFormatW(LOCALE_NEUTRAL, 0, NULL, L"HH':'mm':'ss", time, 9)
 *
 * so this models the picture grammar those two need — the repeated-letter fields plus
 * single-quoted literals — and HARD FAILS on anything else rather than silently
 * emitting a best guess. A shim that quietly ignored an unknown field would turn a
 * future caller's wrong output into a passing row.
 */
static int format_picture(const SYSTEMTIME *st, const WCHAR *fmt, WCHAR *out, int cch,
                          BOOL date_fields)
{
    WCHAR buf[128];
    int n = 0;

    while (*fmt) {
        if (*fmt == '\'') {                       /* quoted literal */
            fmt++;
            while (*fmt && *fmt != '\'') {
                if (n >= (int)(sizeof(buf) / sizeof(buf[0])) - 1) return 0;
                buf[n++] = *fmt++;
            }
            if (*fmt == '\'') fmt++;
            continue;
        }
        {
            WCHAR c = *fmt;
            int run = 0, value = -1;
            while (fmt[run] == c) run++;
            if (date_fields) {
                if (c == 'y') value = (run <= 2) ? st->wYear % 100 : st->wYear;
                else if (c == 'M') value = st->wMonth;
                else if (c == 'd') value = st->wDay;
            } else {
                if (c == 'H') value = st->wHour;
                else if (c == 'h') value = st->wHour % 12 ? st->wHour % 12 : 12;
                else if (c == 'm') value = st->wMinute;
                else if (c == 's') value = st->wSecond;
            }
            if (value < 0) {
                /* Not a field this shim models, and not quoted. Fail loud. */
                SET_ERR(ERROR_INVALID_PARAMETER);
                return 0;
            }
            {
                int width = run, digits = 0, i;
                WCHAR tmp[8];
                do { tmp[digits++] = (WCHAR)('0' + value % 10); value /= 10; }
                while (value);
                for (i = digits; i < width; i++) {
                    if (n >= (int)(sizeof(buf) / sizeof(buf[0])) - 1) return 0;
                    buf[n++] = '0';
                }
                for (i = digits - 1; i >= 0; i--) {
                    if (n >= (int)(sizeof(buf) / sizeof(buf[0])) - 1) return 0;
                    buf[n++] = tmp[i];
                }
            }
            fmt += run;
        }
    }
    buf[n] = 0;

    /* cch == 0 is the "how big a buffer do I need" query, INCLUDING the NUL. */
    if (cch == 0) return n + 1;
    if (n + 1 > cch) { SET_ERR(ERROR_INSUFFICIENT_BUFFER); return 0; }
    memcpy(out, buf, (size_t)(n + 1) * sizeof(WCHAR));
    return n + 1;
}

int WINAPI GetDateFormatW(LCID lcid, DWORD flags, const SYSTEMTIME *st,
                          const WCHAR *fmt, WCHAR *out, int cch)
{
    SYSTEMTIME local;
    (void)lcid; (void)flags;
    if (!fmt) { SET_ERR(ERROR_INVALID_PARAMETER); return 0; }
    if (!st) { GetLocalTime(&local); st = &local; }
    return format_picture(st, fmt, out, cch, TRUE);
}

int WINAPI GetTimeFormatW(LCID lcid, DWORD flags, const SYSTEMTIME *st,
                          const WCHAR *fmt, WCHAR *out, int cch)
{
    SYSTEMTIME local;
    (void)lcid; (void)flags;
    if (!fmt) { SET_ERR(ERROR_INVALID_PARAMETER); return 0; }
    if (!st) { GetLocalTime(&local); st = &local; }
    return format_picture(st, fmt, out, cch, FALSE);
}

/* ── the one A-string helper time.c uses ────────────────────────────────── */

/* `lstrcpynA` copies at most n-1 characters and ALWAYS NUL-terminates — it is not
 * `strncpy`, and `_tzset` depends on the difference: it passes 3 to take a
 * two-character zone abbreviation plus the terminator. */
LPSTR WINAPI lstrcpynA(LPSTR dst, LPCSTR src, int n)
{
    int i;
    if (!dst || !src || n <= 0) return NULL;
    for (i = 0; i < n - 1 && src[i]; i++) dst[i] = src[i];
    dst[i] = 0;
    return dst;
}
