/*
 * PROJECT:         ReactOS HAL
 * LICENSE:         GPL-2.0-or-later
 * FILE:            hal/halarm64/generic/hal.h
 * PURPOSE:         Hardware Abstraction Layer Header for ARM64
 */

#pragma once

/* C Headers */
#include <stdio.h>

/* WDK HAL Compilation hack */
#include <excpt.h>
#include <ntdef.h>
#undef NTSYSAPI
#define NTSYSAPI __declspec(dllimport)

/* IFS/DDK/NDK Headers */
#include <ntifs.h>
#include <bugcodes.h>
#include <ntdddisk.h>
#include <arc/arc.h>
#include <iotypes.h>
#include <kefuncs.h>
#include <halfuncs.h>
#include <inbvfuncs.h>
#include <iofuncs.h>
#include <ldrtypes.h>
#include <obfuncs.h>

/* EOF */
