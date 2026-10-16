@ stdcall -private DllCanUnloadNow()
@ stdcall -private DllGetClassObject(ptr ptr ptr)
@ stdcall -private DllRegisterServer()
@ stdcall -private DllUnregisterServer()
@ stub MFConvertColorInfoFromDXVA
@ stdcall MFConvertColorInfoToDXVA(ptr ptr) mfplat.MFConvertColorInfoToDXVA
@ stub MFConvertFromFP16Array
@ stub MFConvertToFP16Array
@ stdcall MFCopyImage(ptr long ptr long long long) mfplat.MFCopyImage
@ stdcall MFCreateDXSurfaceBuffer(ptr ptr long ptr) mfplat.MFCreateDXSurfaceBuffer
@ stdcall MFCreateVideoMediaType(ptr ptr) mfplat.MFCreateVideoMediaType
@ stub MFCreateVideoMediaTypeFromBitMapInfoHeader
@ stdcall MFCreateVideoMediaTypeFromSubtype(ptr ptr) mfplat.MFCreateVideoMediaTypeFromSubtype
@ stub MFCreateVideoMediaTypeFromVideoInfoHeader2
@ stdcall MFCreateVideoMediaTypeFromVideoInfoHeader(ptr long long long long int64 ptr ptr) mfplat.MFCreateVideoMediaTypeFromVideoInfoHeader
@ stdcall MFCreateVideoMixer(ptr ptr ptr ptr)
@ stdcall MFCreateVideoMixerAndPresenter(ptr ptr ptr ptr ptr ptr)
@ stub MFCreateVideoOTA
@ stub MFCreateVideoPresenter2
@ stdcall MFCreateVideoPresenter(ptr ptr ptr ptr)
@ stdcall MFCreateVideoSampleAllocator(ptr ptr)
@ stdcall MFCreateVideoSampleFromSurface(ptr ptr)
@ stdcall MFGetPlaneSize(long long long ptr) mfplat.MFGetPlaneSize
@ stdcall MFGetStrideForBitmapInfoHeader(long long ptr) mfplat.MFGetStrideForBitmapInfoHeader
@ stub MFGetUncompressedVideoFormat
@ stub MFInitVideoFormat
@ stdcall MFInitVideoFormat_RGB(ptr long long long) mfplat.MFInitVideoFormat_RGB
@ stdcall MFIsFormatYUV(long)
