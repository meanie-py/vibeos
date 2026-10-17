# mi.dll: Windows exports exactly these two (measured on Windows 11). Everything else is
# reached through the function tables mi_clientFT_V1 points at.
@ cdecl MI_Application_InitializeV1(long ptr ptr ptr)
@ extern mi_clientFT_V1
