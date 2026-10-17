#define STANDALONE
#include <apitest.h>

extern void func_mi(void);

const struct test winetest_testlist[] =
{
    { "mi", func_mi },
    { 0, 0 }
};
