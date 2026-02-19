/*
 * debug.h — Host shim replacing ReactOS debug.h
 */
#ifndef _DEBUG_HOST_H_
#define _DEBUG_HOST_H_

#include <stdio.h>

#ifdef NDEBUG
#define DPRINT(...)   ((void)0)
#define DPRINT1(...)  ((void)0)
#else
#define DPRINT(...)   ((void)0)
#define DPRINT1(fmt, ...) fprintf(stderr, fmt, ##__VA_ARGS__)
#endif

#ifndef ASSERT
#ifdef NDEBUG
#define ASSERT(x) ((void)0)
#else
#define ASSERT(x) assert(x)
#endif
#endif

#endif
