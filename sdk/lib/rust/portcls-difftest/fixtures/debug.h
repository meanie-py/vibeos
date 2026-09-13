#pragma once
/* Neither fixture consumes log output. Disabled DPRINT matches the TU's NDEBUG. */
#define DPRINT(...) do {} while(0)
#define DPRINT1(...) DbgPrint(__VA_ARGS__)
