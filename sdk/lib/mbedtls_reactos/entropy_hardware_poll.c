/*
 * mbedTLS entropy source for ReactOS.
 *
 * Copyright 2026 The vibeos contributors
 *
 * This library is free software; you can redistribute it and/or
 * modify it under the terms of the GNU Lesser General Public
 * License as published by the Free Software Foundation; either
 * version 2.1 of the License, or (at your option) any later version.
 *
 * This library is distributed in the hope that it will be useful,
 * but WITHOUT ANY WARRANTY; without even the implied warranty of
 * MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the GNU
 * Lesser General Public License for more details.
 *
 * --
 *
 * WHY THIS EXISTS. mbedTLS 3.x's built-in Windows entropy source calls
 * BCryptGenRandom, and on this OS that is a DEPENDENCY CYCLE: bcrypt.dll already
 * imports mbedtls.dll for its hashing, so mbedtls.dll importing bcrypt.dll back
 * would close the loop. 2.16 had no such problem because its Windows path used
 * CryptGenRandom out of advapi32.
 *
 * So MBEDTLS_NO_PLATFORM_ENTROPY turns the built-in source off and
 * MBEDTLS_ENTROPY_HARDWARE_ALT hands the job here, to RtlGenRandom -- exported by
 * advapi32 under its historical ordinal name SystemFunction036, which mbedtls.dll
 * already links. That is the same generator Windows' own CryptGenRandom is built
 * over, so this is a shorter path to the same bytes rather than a weaker one.
 */

#include <windef.h>
#include <winbase.h>
#include <stddef.h>

/* advapi32's RtlGenRandom. Declared here rather than via <ntsecapi.h> to keep this
 * adapter's include surface to what it actually uses. */
BOOLEAN WINAPI SystemFunction036(PVOID RandomBuffer, ULONG RandomBufferLength);

int mbedtls_hardware_poll(void *data, unsigned char *output, size_t len, size_t *olen)
{
    (void)data;

    *olen = 0;

    /* RtlGenRandom takes a ULONG length. Fill in chunks rather than truncating, so a
     * caller asking for more than ULONG_MAX still gets what it asked for. */
    while (len > 0)
    {
        ULONG chunk = (len > 0x10000000u) ? 0x10000000u : (ULONG)len;

        if (!SystemFunction036(output, chunk))
            return -1; /* MBEDTLS_ERR_ENTROPY_SOURCE_FAILED, without the header */

        output += chunk;
        *olen  += chunk;
        len    -= chunk;
    }

    return 0;
}
