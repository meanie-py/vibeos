;
; ReactOS Operating System
;
; This file contains all win32k native api functions from ReactOS
; Unsupported functions are commented out
;

@ stdcall NtGdiAbortDoc(ptr)
@ stdcall NtGdiAbortPath(ptr)
@ stdcall NtGdiAddFontResourceW(wstr long long long long ptr)
@ stdcall NtGdiAddRemoteFontToDC(ptr ptr long ptr)
@ stdcall NtGdiAddFontMemResourceEx(ptr long ptr long ptr)
@ stdcall NtGdiRemoveMergeFont(ptr ptr)
@ stdcall NtGdiAddRemoteMMInstanceToDC(ptr ptr long)
@ stdcall NtGdiAlphaBlend(ptr long long long long ptr long long long long long ptr)
@ stdcall NtGdiAngleArc(ptr long long long long long)
@ stdcall NtGdiAnyLinkedFonts()
@ stdcall NtGdiFontIsLinked(ptr)
@ stdcall NtGdiArcInternal(long ptr long long long long long long long long)
@ stdcall NtGdiBeginPath(ptr)
@ stdcall NtGdiBitBlt(ptr long long long long ptr long long long long long)
@ stdcall NtGdiCancelDC(ptr)
@ stdcall NtGdiCheckBitmapBits(ptr ptr ptr long long long long ptr)
@ stdcall NtGdiCloseFigure(ptr)
@ stdcall NtGdiClearBitmapAttributes(ptr long)
@ stdcall NtGdiClearBrushAttributes(ptr long)
@ stdcall NtGdiColorCorrectPalette(ptr ptr long long ptr long)
@ stdcall NtGdiCombineRgn(ptr ptr ptr long)
@ stdcall NtGdiCombineTransform(ptr ptr ptr)
@ stdcall NtGdiComputeXformCoefficients(ptr)
@ stdcall NtGdiConsoleTextOut(ptr ptr long ptr)
@ stdcall NtGdiConvertMetafileRect(ptr ptr)
@ stdcall NtGdiCreateBitmap(long long long long ptr)
@ stdcall NtGdiCreateClientObj(long)
@ stdcall NtGdiCreateColorSpace(ptr)
@ stdcall NtGdiCreateColorTransform(ptr ptr ptr long ptr long ptr long)
@ stdcall NtGdiCreateCompatibleBitmap(ptr long long)
@ stdcall NtGdiCreateCompatibleDC(ptr)
@ stdcall NtGdiCreateDIBBrush(ptr long long long long ptr)
@ stdcall NtGdiCreateDIBitmapInternal(ptr long long long ptr ptr long long long long ptr)
@ stdcall NtGdiCreateDIBSection(ptr ptr long ptr long long long ptr ptr)
@ stdcall NtGdiCreateEllipticRgn(long long long long)
@ stdcall NtGdiCreateHalftonePalette(ptr)
@ stdcall NtGdiCreateHatchBrushInternal(long long long)
@ stdcall NtGdiCreateMetafileDC(ptr)
@ stdcall NtGdiCreatePaletteInternal(ptr long)
@ stdcall NtGdiCreatePatternBrushInternal(ptr long long)
@ stdcall NtGdiCreatePen(long long long ptr)
@ stdcall NtGdiCreateRectRgn(long long long long)
@ stdcall NtGdiCreateRoundRectRgn(long long long long long long)
@ stdcall NtGdiCreateServerMetaFile(long long ptr long long long)
@ stdcall NtGdiCreateSolidBrush(long ptr)
@ stdcall NtGdiD3dContextCreate(ptr ptr ptr ptr)
@ stdcall NtGdiD3dContextDestroy(ptr)
@ stdcall NtGdiD3dContextDestroyAll(ptr)
@ stdcall NtGdiD3dValidateTextureStageState(ptr)
@ stdcall NtGdiD3dDrawPrimitives2(ptr ptr ptr ptr ptr ptr ptr)
@ stdcall NtGdiDdGetDriverState(ptr)
@ stdcall NtGdiDdAddAttachedSurface(ptr ptr ptr)
@ stdcall NtGdiDdAlphaBlt(ptr ptr ptr)
@ stdcall NtGdiDdAttachSurface(ptr ptr)
@ stdcall NtGdiDdBeginMoCompFrame(ptr ptr)
@ stdcall NtGdiDdBlt(ptr ptr ptr)
@ stdcall NtGdiDdCanCreateSurface(ptr ptr)
@ stdcall NtGdiDdCanCreateD3DBuffer(ptr ptr)
@ stdcall NtGdiDdColorControl(ptr ptr)
@ stdcall NtGdiDdCreateDirectDrawObject(ptr)
@ stdcall NtGdiDdCreateSurface(ptr ptr ptr ptr ptr ptr ptr ptr)
@ stdcall NtGdiDdCreateD3DBuffer(ptr ptr ptr ptr ptr ptr ptr ptr)
@ stdcall NtGdiDdCreateMoComp(ptr ptr)
@ stdcall NtGdiDdCreateSurfaceObject(ptr ptr ptr ptr ptr long)
@ stdcall NtGdiDdDeleteDirectDrawObject(ptr)
@ stdcall NtGdiDdDeleteSurfaceObject(ptr)
@ stdcall NtGdiDdDestroyMoComp(ptr ptr)
@ stdcall NtGdiDdDestroySurface(ptr long)
@ stdcall NtGdiDdDestroyD3DBuffer(ptr)
@ stdcall NtGdiDdEndMoCompFrame(ptr ptr)
@ stdcall NtGdiDdFlip(ptr ptr ptr ptr ptr)
@ stdcall NtGdiDdFlipToGDISurface(ptr ptr)
@ stdcall NtGdiDdGetAvailDriverMemory(ptr ptr)
@ stdcall NtGdiDdGetBltStatus(ptr ptr)
@ stdcall NtGdiDdGetDC(ptr ptr)
@ stdcall NtGdiDdGetDriverInfo(ptr ptr)
@ stdcall NtGdiDdGetDxHandle(ptr ptr long)
@ stdcall NtGdiDdGetFlipStatus(ptr ptr)
@ stdcall NtGdiDdGetInternalMoCompInfo(ptr ptr)
@ stdcall NtGdiDdGetMoCompBuffInfo(ptr ptr)
@ stdcall NtGdiDdGetMoCompGuids(ptr ptr)
@ stdcall NtGdiDdGetMoCompFormats(ptr ptr)
@ stdcall NtGdiDdGetScanLine(ptr ptr)
@ stdcall NtGdiDdLock(ptr ptr ptr)
@ stdcall NtGdiDdLockD3D(ptr ptr)
@ stdcall NtGdiDdQueryDirectDrawObject(ptr ptr ptr ptr ptr ptr ptr ptr ptr ptr ptr)
@ stdcall NtGdiDdQueryMoCompStatus(ptr ptr)
@ stdcall NtGdiDdReenableDirectDrawObject(ptr ptr)
@ stdcall NtGdiDdReleaseDC(ptr)
@ stdcall NtGdiDdRenderMoComp(ptr ptr)
@ stdcall NtGdiDdResetVisrgn(ptr ptr)
@ stdcall NtGdiDdSetColorKey(ptr ptr)
@ stdcall NtGdiDdSetExclusiveMode(ptr ptr)
@ stdcall NtGdiDdSetGammaRamp(ptr ptr ptr)
@ stdcall NtGdiDdCreateSurfaceEx(ptr ptr long)
@ stdcall NtGdiDdSetOverlayPosition(ptr ptr ptr)
@ stdcall NtGdiDdUnattachSurface(ptr ptr)
@ stdcall NtGdiDdUnlock(ptr ptr)
@ stdcall NtGdiDdUnlockD3D(ptr ptr)
@ stdcall NtGdiDdUpdateOverlay(ptr ptr ptr)
@ stdcall NtGdiDdWaitForVerticalBlank(ptr ptr)
@ stdcall NtGdiDvpCanCreateVideoPort(ptr ptr)
@ stdcall NtGdiDvpColorControl(ptr ptr)
@ stdcall NtGdiDvpCreateVideoPort(ptr ptr)
@ stdcall NtGdiDvpDestroyVideoPort(ptr ptr)
@ stdcall NtGdiDvpFlipVideoPort(ptr ptr ptr ptr)
@ stdcall NtGdiDvpGetVideoPortBandwidth(ptr ptr)
@ stdcall NtGdiDvpGetVideoPortField(ptr ptr)
@ stdcall NtGdiDvpGetVideoPortFlipStatus(ptr ptr)
@ stdcall NtGdiDvpGetVideoPortInputFormats(ptr ptr)
@ stdcall NtGdiDvpGetVideoPortLine(ptr ptr)
@ stdcall NtGdiDvpGetVideoPortOutputFormats(ptr ptr)
@ stdcall NtGdiDvpGetVideoPortConnectInfo(ptr ptr)
@ stdcall NtGdiDvpGetVideoSignalStatus(ptr ptr)
@ stdcall NtGdiDvpUpdateVideoPort(ptr ptr ptr ptr)
@ stdcall NtGdiDvpWaitForVideoPortSync(ptr ptr)
@ stdcall NtGdiDvpAcquireNotification(ptr ptr ptr)
@ stdcall NtGdiDvpReleaseNotification(ptr ptr)
@ stdcall NtGdiDxgGenericThunk(ptr ptr ptr ptr ptr ptr)
@ stdcall NtGdiDeleteClientObj(ptr)
@ stdcall NtGdiDeleteColorSpace(ptr)
@ stdcall NtGdiDeleteColorTransform(ptr ptr)
@ stdcall NtGdiDeleteObjectApp(ptr)
@ stdcall NtGdiDescribePixelFormat(ptr long long ptr)
@ stdcall NtGdiGetPerBandInfo(ptr ptr)
@ stdcall NtGdiDoBanding(ptr long ptr ptr)
@ stdcall NtGdiDoPalette(ptr long long ptr long long)
@ stdcall NtGdiDrawEscape(ptr long long ptr)
@ stdcall NtGdiEllipse(ptr long long long long)
@ stdcall NtGdiEnableEudc(long)
@ stdcall NtGdiEndDoc(ptr)
@ stdcall NtGdiEndPage(ptr)
@ stdcall NtGdiEndPath(ptr)
@ stdcall NtGdiEnumFontChunk(ptr ptr long ptr ptr)
@ stdcall NtGdiEnumFontClose(ptr)
@ stdcall NtGdiEnumFontOpen(ptr long long long wstr long ptr)
@ stdcall NtGdiEnumObjects(ptr long long ptr)
@ stdcall NtGdiEqualRgn(ptr ptr)
@ stdcall NtGdiEudcLoadUnloadLink(wstr long wstr long long long long)
@ stdcall NtGdiExcludeClipRect(ptr long long long long)
@ stdcall NtGdiExtCreatePen(long long long long ptr ptr long ptr long long ptr)
@ stdcall NtGdiExtCreateRegion(ptr long ptr)
@ stdcall NtGdiExtEscape(ptr ptr long long long ptr long ptr)
@ stdcall NtGdiExtFloodFill(ptr long long long long)
@ stdcall NtGdiExtGetObjectW(ptr long ptr)
@ stdcall NtGdiExtSelectClipRgn(ptr ptr long)
@ stdcall NtGdiExtTextOutW(ptr long long long ptr wstr long ptr long)
@ stdcall NtGdiFillPath(ptr)
@ stdcall NtGdiFillRgn(ptr ptr ptr)
@ stdcall NtGdiFlattenPath(ptr)
# @ stdcall NtGdiFlushUserBatch()
@ stdcall NtGdiFlush()
@ stdcall NtGdiForceUFIMapping(ptr ptr)
@ stdcall NtGdiFrameRgn(ptr ptr ptr long long)
@ stdcall NtGdiFullscreenControl(long ptr long ptr ptr)
@ stdcall NtGdiGetAndSetDCDword(ptr long long ptr)
@ stdcall NtGdiGetAppClipBox(ptr ptr)
@ stdcall NtGdiGetBitmapBits(ptr long ptr)
@ stdcall NtGdiGetBitmapDimension(ptr ptr)
@ stdcall NtGdiGetBoundsRect(ptr ptr long)
@ stdcall NtGdiGetCharABCWidthsW(ptr long long ptr long ptr)
@ stdcall NtGdiGetCharacterPlacementW(ptr wstr long long ptr long)
@ stdcall NtGdiGetCharSet(ptr)
@ stdcall NtGdiGetCharWidthW(ptr long long ptr long ptr)
@ stdcall NtGdiGetCharWidthInfo(ptr ptr)
@ stdcall NtGdiGetColorAdjustment(ptr ptr)
@ stdcall NtGdiGetColorSpaceforBitmap(ptr)
@ stdcall NtGdiGetDCDword(ptr long ptr)
@ stdcall NtGdiGetDCforBitmap(ptr)
@ stdcall NtGdiGetDCObject(ptr long)
@ stdcall NtGdiGetDCPoint(ptr long ptr)
@ stdcall NtGdiGetDeviceCaps(ptr long)
@ stdcall NtGdiGetDeviceGammaRamp(ptr ptr)
@ stdcall NtGdiGetDeviceCapsAll(ptr ptr)
@ stdcall NtGdiGetDIBitsInternal(ptr ptr long long ptr ptr long long long)
@ stdcall NtGdiGetETM(ptr ptr)
@ stdcall NtGdiGetEudcTimeStampEx(wstr long long)
@ stdcall NtGdiGetFontData(ptr long long ptr long)
@ stdcall NtGdiGetFontFileData(long long ptr ptr long)
@ stdcall NtGdiGetFontFileInfo(long long ptr long ptr)
@ stdcall NtGdiGetFontRealizationInfo(ptr ptr)
@ stdcall NtGdiGetFontResourceInfoInternalW(wstr long long long ptr ptr long)
@ stdcall NtGdiGetGlyphIndicesW(ptr wstr long ptr long)
@ stdcall NtGdiGetGlyphIndicesWInternal(ptr wstr long ptr long long)
@ stdcall NtGdiGetGlyphOutline(ptr long long ptr long ptr ptr long)
@ stdcall NtGdiGetKerningPairs(ptr long ptr)
@ stdcall NtGdiGetLinkedUFIs(ptr ptr long)
@ stdcall NtGdiGetMiterLimit(ptr ptr)
@ stdcall NtGdiGetMonitorID(ptr long ptr)
@ stdcall NtGdiGetNearestColor(ptr ptr)
@ stdcall NtGdiGetNearestPaletteIndex(ptr ptr)
@ stdcall NtGdiGetObjectBitmapHandle(ptr ptr)
@ stdcall NtGdiGetOutlineTextMetricsInternalW(ptr long ptr ptr)
@ stdcall NtGdiGetPath(ptr ptr ptr long)
@ stdcall NtGdiGetPixel(ptr long long)
@ stdcall NtGdiGetRandomRgn(ptr ptr long)
@ stdcall NtGdiGetRasterizerCaps(ptr long)
@ stdcall NtGdiGetRealizationInfo(ptr ptr ptr)
@ stdcall NtGdiGetRegionData(ptr long ptr)
@ stdcall NtGdiGetRgnBox(ptr ptr)
@ stdcall NtGdiGetServerMetaFileBits(ptr long ptr ptr ptr ptr ptr)
@ stdcall NtGdiGetSpoolMessage(long long long long)
@ stdcall NtGdiGetStats(ptr long long ptr long)
@ stdcall NtGdiGetStockObject(ptr)
@ stdcall NtGdiGetStringBitmapW(ptr wstr long ptr long)
@ stdcall NtGdiGetSystemPaletteUse(ptr)
@ stdcall NtGdiGetTextCharsetInfo(ptr ptr long)
@ stdcall NtGdiGetTextExtent(ptr wstr long ptr long)
@ stdcall NtGdiGetTextExtentExW(ptr wstr long long ptr ptr ptr long)
@ stdcall NtGdiGetTextFaceW(ptr long ptr long)
@ stdcall NtGdiGetTextMetricsW(ptr ptr long)
@ stdcall NtGdiGetTransform(ptr long ptr)
@ stdcall NtGdiGetUFI(ptr ptr ptr ptr ptr ptr)
@ stdcall NtGdiGetEmbUFI(ptr ptr ptr ptr ptr ptr ptr)
@ stdcall NtGdiGetUFIPathname(ptr ptr wstr ptr long ptr ptr ptr ptr ptr)
@ stdcall NtGdiGetEmbedFonts()
@ stdcall NtGdiChangeGhostFont(ptr long)
@ stdcall NtGdiAddEmbFontToDC(ptr ptr)
@ stdcall NtGdiGetFontUnicodeRanges(ptr ptr)
@ stdcall NtGdiGetWidthTable(ptr long ptr long ptr ptr ptr)
@ stdcall NtGdiGradientFill(ptr ptr long ptr long long)
@ stdcall NtGdiHfontCreate(ptr long long long ptr)
@ stdcall NtGdiIcmBrushInfo(ptr ptr ptr ptr ptr ptr ptr long)
@ stdcall NtGdiInit()
@ stdcall NtGdiInitSpool()
@ stdcall NtGdiIntersectClipRect(ptr long long long long)
@ stdcall NtGdiInvertRgn(ptr ptr)
@ stdcall NtGdiLineTo(ptr long long)
@ stdcall NtGdiMakeFontDir(long ptr long wstr long)
@ stdcall NtGdiMakeInfoDC(ptr long)
@ stdcall NtGdiMaskBlt(ptr long long long long ptr long long ptr long long long long)
@ stdcall NtGdiModifyWorldTransform(ptr ptr long)
@ stdcall NtGdiMonoBitmap(ptr)
@ stdcall NtGdiMoveTo(ptr long long ptr)
@ stdcall NtGdiOffsetClipRgn(ptr long long)
@ stdcall NtGdiOffsetRgn(ptr long long)
@ stdcall NtGdiOpenDCW(ptr ptr ptr long ptr long ptr)
@ stdcall NtGdiPatBlt(ptr long long long long long)
@ stdcall NtGdiPolyPatBlt(ptr long ptr long long)
@ stdcall NtGdiPathToRegion(ptr)
@ stdcall NtGdiPlgBlt(ptr ptr ptr long long long long ptr long long long)
@ stdcall NtGdiPolyDraw(ptr ptr ptr long)
@ stdcall NtGdiPolyPolyDraw(ptr ptr ptr long long)
@ stdcall NtGdiPolyTextOutW(ptr ptr long long)
@ stdcall NtGdiPtInRegion(ptr long long)
@ stdcall NtGdiPtVisible(ptr long long)
@ stdcall NtGdiQueryFonts(ptr long ptr)
@ stdcall NtGdiQueryFontAssocInfo(ptr)
@ stdcall NtGdiRectangle(ptr long long long long)
@ stdcall NtGdiRectInRegion(ptr ptr)
@ stdcall NtGdiRectVisible(ptr ptr)
@ stdcall NtGdiRemoveFontResourceW(ptr long long long long ptr)
@ stdcall NtGdiRemoveFontMemResourceEx(ptr)
@ stdcall NtGdiResetDC(ptr ptr ptr ptr ptr)
@ stdcall NtGdiResizePalette(ptr long)
@ stdcall NtGdiRestoreDC(ptr long)
@ stdcall NtGdiRoundRect(ptr long long long long long long)
@ stdcall NtGdiSaveDC(ptr)
@ stdcall NtGdiScaleViewportExtEx(ptr long long long long ptr)
@ stdcall NtGdiScaleWindowExtEx(ptr long long long long ptr)
@ stdcall NtGdiSelectBitmap(ptr ptr)
@ stdcall NtGdiSelectBrush(ptr ptr)
@ stdcall NtGdiSelectClipPath(ptr long)
@ stdcall NtGdiSelectFont(ptr ptr)
@ stdcall NtGdiSelectPen(ptr ptr)
@ stdcall NtGdiSetBitmapAttributes(ptr long)
@ stdcall NtGdiSetBitmapBits(ptr long ptr)
@ stdcall NtGdiSetBitmapDimension(ptr long long ptr)
@ stdcall NtGdiSetBoundsRect(ptr ptr long)
@ stdcall NtGdiSetBrushAttributes(ptr long)
@ stdcall NtGdiSetBrushOrg(ptr long long ptr)
@ stdcall NtGdiSetColorAdjustment(ptr ptr)
@ stdcall NtGdiSetColorSpace(ptr ptr)
@ stdcall NtGdiSetDeviceGammaRamp(ptr ptr)
@ stdcall NtGdiSetDIBitsToDeviceInternal(ptr long long long long long long long long ptr ptr long long long long ptr)
@ stdcall NtGdiSetFontEnumeration(long)
@ stdcall NtGdiSetFontXform(ptr long long)
@ stdcall NtGdiSetIcmMode(ptr long long)
@ stdcall NtGdiSetLinkedUFIs(ptr ptr long)
@ stdcall NtGdiSetMagicColors(ptr long long)
@ stdcall NtGdiSetMetaRgn(ptr)
@ stdcall NtGdiSetMiterLimit(ptr long ptr)
@ stdcall NtGdiGetDeviceWidth(ptr)
@ stdcall NtGdiMirrorWindowOrg(ptr)
@ stdcall NtGdiSetLayout(ptr long long)
@ stdcall NtGdiSetPixel(ptr long long long)
@ stdcall NtGdiSetPixelFormat(ptr long)
@ stdcall NtGdiSetRectRgn(ptr long long long long)
@ stdcall NtGdiSetSystemPaletteUse(ptr long)
@ stdcall NtGdiSetTextJustification(ptr long long)
@ stdcall NtGdiSetupPublicCFONT(ptr ptr long)
@ stdcall NtGdiSetVirtualResolution(ptr long long long long)
@ stdcall NtGdiSetSizeDevice(ptr long long)
@ stdcall NtGdiStartDoc(ptr ptr ptr long)
@ stdcall NtGdiStartPage(ptr)
@ stdcall NtGdiStretchBlt(ptr long long long long ptr long long long long long long)
@ stdcall NtGdiStretchDIBitsInternal(ptr long long long long long long long long ptr ptr long long long long ptr)
@ stdcall NtGdiStrokeAndFillPath(ptr)
@ stdcall NtGdiStrokePath(ptr)
@ stdcall NtGdiSwapBuffers(ptr)
@ stdcall NtGdiTransformPoints(ptr ptr ptr long long)
@ stdcall NtGdiTransparentBlt(ptr long long long long  ptr long long long long long)
@ stdcall NtGdiUnloadPrinterDriver(wstr long)
@ stdcall NtGdiUnmapMemFont(ptr)
@ stdcall NtGdiUnrealizeObject(ptr)
@ stdcall NtGdiUpdateColors(ptr)
@ stdcall NtGdiWidenPath(ptr)
@ stdcall NtUserActivateKeyboardLayout(ptr long)
@ stdcall NtUserAlterWindowStyle(long long long)
@ stdcall NtUserAssociateInputContext(long long long)
@ stdcall NtUserAttachThreadInput(long long long)
@ stdcall NtUserBeginPaint(ptr ptr)
@ stdcall NtUserBitBltSysBmp(ptr long long long long long long long)
@ stdcall NtUserBlockInput(long)
@ stdcall NtUserBuildHimcList(long long long long)
@ stdcall NtUserBuildHwndList(ptr ptr long long long ptr ptr)
@ stdcall NtUserBuildNameList(ptr long ptr ptr)
@ stdcall NtUserBuildPropList(ptr ptr long ptr)
@ stdcall NtUserCallHwnd(ptr long)
@ stdcall NtUserCallHwndLock(ptr long)
@ stdcall NtUserCallHwndOpt(ptr long)
@ stdcall NtUserCallHwndParam(ptr long long)
@ stdcall NtUserCallHwndParamLock(ptr long long)
@ stdcall NtUserCallMsgFilter(ptr long)
@ stdcall NtUserCallNextHookEx(long ptr ptr long)
@ stdcall NtUserCallNoParam(long)
@ stdcall NtUserCallOneParam(ptr long)
@ stdcall NtUserCallTwoParam(ptr ptr long)
@ stdcall NtUserChangeClipboardChain(ptr ptr)
# @ stdcall NtUserChangeDisplaySettings(ptr ptr ptr long ptr)
@ stdcall NtUserCheckImeHotKey(long long)
@ stdcall NtUserCheckMenuItem(ptr long long)
@ stdcall NtUserChildWindowFromPointEx(ptr long long long)
@ stdcall NtUserClipCursor(ptr)
@ stdcall NtUserCloseClipboard()
@ stdcall NtUserCloseDesktop(ptr)
@ stdcall NtUserCloseWindowStation(ptr)
@ stdcall NtUserConsoleControl(long long long)
@ stdcall NtUserConvertMemHandle(ptr long)
@ stdcall NtUserCopyAcceleratorTable(ptr ptr long)
@ stdcall NtUserCountClipboardFormats()
@ stdcall NtUserCreateAcceleratorTable(ptr ptr)
@ stdcall NtUserCreateCaret(ptr ptr long long)
@ stdcall NtUserCreateDesktop(ptr ptr ptr long long)
@ stdcall NtUserCreateInputContext(long)
@ stdcall NtUserCreateLocalMemHandle(ptr ptr long ptr)
@ stdcall NtUserCreateWindowEx(long ptr ptr ptr long long long long long ptr ptr ptr ptr long ptr)
@ stdcall NtUserCreateWindowStation(ptr long long long long long long)
@ stdcall NtUserDdeGetQualityOfService(ptr ptr ptr)
@ stdcall NtUserDdeInitialize(long long long long long)
@ stdcall NtUserDdeSetQualityOfService(ptr ptr ptr)
@ stdcall NtUserDeferWindowPos(ptr ptr ptr long long long long long)
@ stdcall NtUserDefSetText(ptr ptr)
@ stdcall NtUserDeleteMenu(ptr long long)
@ stdcall NtUserDestroyAcceleratorTable(ptr)
@ stdcall NtUserDestroyCursor(ptr long)
@ stdcall NtUserDestroyInputContext(long)
@ stdcall NtUserDestroyMenu(ptr)
@ stdcall NtUserDestroyWindow(ptr)
@ stdcall NtUserDisableThreadIme(long)
@ stdcall NtUserDispatchMessage(ptr)
@ stdcall NtUserDragDetect(ptr ptr long)
@ stdcall NtUserDragObject(ptr ptr long long ptr)
@ stdcall NtUserDrawAnimatedRects(ptr long ptr ptr)
@ stdcall NtUserDrawCaption(ptr ptr ptr long)
@ stdcall NtUserDrawCaptionTemp(ptr ptr ptr ptr ptr ptr long)
@ stdcall NtUserDrawIconEx(ptr long long ptr long long long ptr long long ptr)
@ stdcall NtUserDrawMenuBarTemp(ptr ptr ptr ptr ptr)
@ stdcall NtUserEmptyClipboard()
@ stdcall NtUserEnableMenuItem(ptr long long)
@ stdcall NtUserEnableScrollBar(ptr long long)
@ stdcall NtUserEndDeferWindowPosEx(ptr long)
@ stdcall NtUserEndMenu()
@ stdcall NtUserEndPaint(ptr ptr)
@ stdcall NtUserEnumDisplayDevices(ptr long ptr long)
; @ stdcall NtUserEnumDisplayMonitors(ptr ptr ptr ptr) # FIXME: 4 on XP
@ stdcall NtUserEnumDisplaySettings(ptr long ptr long)
@ stdcall NtUserEvent(long)
@ stdcall NtUserExcludeUpdateRgn(ptr ptr)
@ stdcall NtUserFillWindow(ptr ptr ptr ptr)
@ stdcall NtUserFindExistingCursorIcon(ptr ptr ptr)
@ stdcall NtUserFindWindowEx(ptr ptr ptr ptr long)
@ stdcall NtUserFlashWindowEx(ptr)
@ stdcall NtUserGetAltTabInfo(ptr long ptr wstr long long)
@ stdcall NtUserGetAncestor(ptr long)
@ stdcall NtUserGetAppImeLevel(long)
@ stdcall NtUserGetAsyncKeyState(long)
@ stdcall NtUserGetAtomName(long ptr)
@ stdcall NtUserGetCaretBlinkTime()
@ stdcall NtUserGetCaretPos(ptr)
@ stdcall NtUserGetClassInfo(ptr ptr ptr ptr long)
@ stdcall NtUserGetClassName(ptr long ptr)
@ stdcall NtUserGetClipboardData(long ptr)
@ stdcall NtUserGetClipboardFormatName(long wstr ptr)
@ stdcall NtUserGetClipboardOwner()
@ stdcall NtUserGetClipboardSequenceNumber()
@ stdcall NtUserGetClipboardViewer()
@ stdcall NtUserGetClipCursor(ptr)
@ stdcall NtUserGetComboBoxInfo(ptr ptr)
@ stdcall NtUserGetControlBrush(ptr ptr long)
@ stdcall NtUserGetControlColor(ptr ptr ptr long)
@ stdcall NtUserGetCPD(ptr long ptr)
@ stdcall NtUserGetCursorFrameInfo(ptr long ptr ptr)
@ stdcall NtUserGetCursorInfo(ptr)
@ stdcall NtUserGetDC(ptr)
@ stdcall NtUserGetDCEx(ptr ptr long)
@ stdcall NtUserGetDoubleClickTime()
@ stdcall NtUserGetForegroundWindow()
@ stdcall NtUserGetGuiResources(ptr long)
@ stdcall NtUserGetGUIThreadInfo(long ptr)
@ stdcall NtUserGetIconInfo(ptr ptr ptr ptr ptr long)
@ stdcall NtUserGetIconSize(ptr long ptr ptr)
@ stdcall NtUserGetImeHotKey(long long long long)
@ stdcall NtUserGetImeInfoEx(long long)
@ stdcall NtUserGetInternalWindowPos(ptr ptr ptr)
@ stdcall NtUserGetKeyboardLayoutList(long ptr)
@ stdcall NtUserGetKeyboardLayoutName(ptr)
@ stdcall NtUserGetKeyboardState(ptr)
@ stdcall NtUserGetKeyNameText(long wstr long)
@ stdcall NtUserGetKeyState(long)
@ stdcall NtUserGetListBoxInfo(ptr)
@ stdcall NtUserGetMenuBarInfo(ptr long long ptr)
@ stdcall NtUserGetMenuIndex(ptr ptr)
@ stdcall NtUserGetMenuItemRect(ptr ptr long ptr)
@ stdcall NtUserGetMessage(ptr ptr long long)
@ stdcall NtUserGetMouseMovePointsEx(long ptr ptr long long)
@ stdcall NtUserGetObjectInformation(ptr long ptr long ptr)
@ stdcall NtUserGetOpenClipboardWindow()
@ stdcall NtUserGetPriorityClipboardFormat(ptr long)
@ stdcall NtUserGetProcessWindowStation()
@ stdcall NtUserGetRawInputBuffer(ptr ptr long)
@ stdcall NtUserGetRawInputData(ptr long ptr ptr long)
@ stdcall NtUserGetRawInputDeviceInfo(ptr long ptr ptr)
@ stdcall NtUserGetRawInputDeviceList(ptr ptr long)
@ stdcall NtUserGetRegisteredRawInputDevices(ptr ptr long)
@ stdcall NtUserGetScrollBarInfo(ptr long ptr)
@ stdcall NtUserGetSystemMenu(ptr long)
@ stdcall NtUserGetThreadDesktop(long long)
@ stdcall NtUserGetThreadState(ptr)
@ stdcall NtUserGetTitleBarInfo(ptr ptr)
@ stdcall NtUserGetUpdateRect(ptr ptr long)
@ stdcall NtUserGetUpdateRgn(ptr ptr long)
@ stdcall NtUserGetWindowDC(ptr)
@ stdcall NtUserGetWindowPlacement(ptr ptr)
@ stdcall NtUserGetWOWClass(ptr ptr)
@ stdcall NtUserHardErrorControl(long long long)
@ stdcall NtUserHideCaret(ptr)
@ stdcall NtUserHiliteMenuItem(ptr ptr long long)
@ stdcall NtUserImpersonateDdeClientWindow(ptr ptr)
@ stdcall NtUserInitialize(long ptr ptr)
@ stdcall NtUserInitializeClientPfnArrays(ptr ptr ptr ptr)
; @ stdcall NtUserInitTask(long long long long long long long long long long long long)
@ stdcall NtUserInternalGetWindowText(ptr wstr ptr)
@ stdcall NtUserInvalidateRect(ptr ptr long)
@ stdcall NtUserInvalidateRgn(ptr ptr long)
@ stdcall NtUserIsClipboardFormatAvailable(long)
@ stdcall NtUserKillTimer(ptr ptr)
@ stdcall NtUserLoadKeyboardLayoutEx(ptr long ptr ptr ptr long long)
@ stdcall NtUserLockWindowStation(ptr)
@ stdcall NtUserLockWindowUpdate(ptr)
@ stdcall NtUserLockWorkStation()
@ stdcall NtUserMapVirtualKeyEx(long long long ptr)
@ stdcall NtUserMenuItemFromPoint(ptr ptr long long)
@ stdcall NtUserMessageCall(ptr long ptr ptr ptr long long)
@ stdcall NtUserMinMaximize(ptr long long)
@ stdcall NtUserMNDragLeave()
@ stdcall NtUserMNDragOver(long long)
@ stdcall NtUserModifyUserStartupInfoFlags(long long)
@ stdcall NtUserMoveWindow(ptr long long long long long)
@ stdcall NtUserNotifyIMEStatus(long long long)
@ stdcall NtUserNotifyProcessCreate(long long long long)
@ stdcall NtUserNotifyWinEvent(long long long long)
@ stdcall NtUserOpenClipboard(ptr long)
@ stdcall NtUserOpenDesktop(ptr long long)
@ stdcall NtUserOpenInputDesktop(long long long)
@ stdcall NtUserOpenWindowStation(ptr long)
@ stdcall NtUserPaintDesktop(ptr)
@ stdcall NtUserPeekMessage(ptr ptr long long long)
@ stdcall NtUserPostMessage(ptr long long long)
@ stdcall NtUserPostThreadMessage(long long long long)
@ stdcall NtUserPrintWindow(ptr ptr long)
@ stdcall NtUserProcessConnect(ptr ptr long)
@ stdcall NtUserQueryInformationThread(ptr long ptr long)
@ stdcall NtUserQueryInputContext(long long)
@ stdcall NtUserQuerySendMessage(long)
#@ stdcall NtUserQueryUserCounters(long long long long long)
@ stdcall NtUserQueryWindow(ptr long)
@ stdcall NtUserRealChildWindowFromPoint(ptr long long)
@ stdcall NtUserRealInternalGetMessage(ptr ptr long long long long)
@ stdcall NtUserRealWaitMessageEx(long long)
@ stdcall NtUserRedrawWindow(ptr ptr ptr long)
@ stdcall NtUserRegisterClassExWOW(ptr ptr ptr ptr long long ptr)
@ stdcall NtUserRegisterUserApiHook(ptr ptr long long) # FIXME: 2 params on XP
@ stdcall NtUserRegisterHotKey(ptr long long long)
@ stdcall NtUserRegisterRawInputDevices(ptr long long)
@ stdcall NtUserRegisterTasklist(long)
@ stdcall NtUserRegisterWindowMessage(ptr)
@ stdcall NtUserRemoveMenu(ptr long long)
@ stdcall NtUserRemoveProp(ptr long)
@ stdcall NtUserResolveDesktop(long long long long)
@ stdcall NtUserResolveDesktopForWOW(long)
@ stdcall NtUserSBGetParms(ptr long ptr ptr)
@ stdcall NtUserScrollDC(ptr long long ptr ptr ptr ptr)
@ stdcall NtUserScrollWindowEx(ptr long long ptr ptr ptr ptr long)
@ stdcall NtUserSelectPalette(ptr ptr long)
@ stdcall NtUserSendInput(long ptr long)
@ stdcall NtUserSetActiveWindow(ptr)
@ stdcall NtUserSetAppImeLevel(long long)
@ stdcall NtUserSetCapture(ptr)
@ stdcall NtUserSetClassLong(ptr long ptr long)
@ stdcall NtUserSetClassWord(ptr long long)
@ stdcall NtUserSetClipboardData(long ptr ptr)
@ stdcall NtUserSetClipboardViewer(ptr)
@ stdcall NtUserSetConsoleReserveKeys(long long)
@ stdcall NtUserSetCursor(ptr)
@ stdcall NtUserSetCursorContents(ptr ptr)
@ stdcall NtUserSetCursorIconData(ptr ptr ptr ptr)
@ stdcall -dbg NtUserSetDbgTag(long long)
@ stdcall NtUserSetFocus(ptr)
@ stdcall NtUserSetImeHotKey(long long long long long)
@ stdcall NtUserSetImeInfoEx(long)
@ stdcall NtUserSetImeOwnerWindow(long long)
@ stdcall NtUserSetInformationProcess(long long long long)
@ stdcall NtUserSetInformationThread(ptr long ptr long)
@ stdcall NtUserSetInternalWindowPos(ptr long ptr ptr)
@ stdcall NtUserSetKeyboardState(ptr)
@ stdcall NtUserSetLogonNotifyWindow(ptr)
@ stdcall NtUserSetMenu(ptr ptr long)
@ stdcall NtUserSetMenuContextHelpId(ptr long)
@ stdcall NtUserSetMenuDefaultItem(ptr long long)
@ stdcall NtUserSetMenuFlagRtoL(ptr)
@ stdcall NtUserSetObjectInformation(ptr long ptr long)
@ stdcall NtUserSetParent(ptr ptr)
@ stdcall NtUserSetProcessWindowStation(ptr)
@ stdcall NtUserSetProp(ptr long ptr)
#@ stdcall -dbg NtUserSetRipFlags(long long)
@ stdcall NtUserSetScrollInfo(ptr long ptr long)
@ stdcall NtUserSetShellWindowEx(ptr ptr)
@ stdcall NtUserSetSysColors(long ptr ptr long)
@ stdcall NtUserSetSystemCursor(ptr long)
@ stdcall NtUserSetSystemMenu(ptr ptr)
@ stdcall NtUserSetSystemTimer(ptr ptr long ptr)
@ stdcall NtUserSetThreadDesktop(ptr)
@ stdcall NtUserSetThreadLayoutHandles(long long)
@ stdcall NtUserSetThreadState(long long)
@ stdcall NtUserSetTimer(ptr ptr long ptr)
@ stdcall NtUserSetWindowFNID(ptr long)
@ stdcall NtUserSetWindowLong(ptr long long long)
@ stdcall NtUserSetWindowPlacement(ptr ptr)
@ stdcall NtUserSetWindowPos(ptr ptr long long long long long)
@ stdcall NtUserSetWindowRgn(ptr ptr long)
@ stdcall NtUserSetWindowsHookAW(long ptr long)
@ stdcall NtUserSetWindowsHookEx(ptr ptr long long ptr long)
@ stdcall NtUserSetWindowStationUser(long long long long)
@ stdcall NtUserSetWindowWord(ptr long long)
@ stdcall NtUserSetWinEventHook(long long ptr ptr ptr long long long)
@ stdcall NtUserShowCaret(ptr)
@ stdcall NtUserShowScrollBar(ptr long long)
@ stdcall NtUserShowWindow(ptr long)
@ stdcall NtUserShowWindowAsync(ptr long)
@ stdcall NtUserSoundSentry()
@ stdcall NtUserSwitchDesktop(ptr)
@ stdcall NtUserSystemParametersInfo(long long ptr long)
@ stdcall NtUserTestForInteractiveUser(long)
@ stdcall NtUserThunkedMenuInfo(ptr ptr)
@ stdcall NtUserThunkedMenuItemInfo(ptr long long long ptr ptr)
@ stdcall NtUserToUnicodeEx(long long ptr wstr long long ptr)
@ stdcall NtUserTrackMouseEvent(ptr)
@ stdcall NtUserTrackPopupMenuEx(ptr long long long ptr ptr)
@ stdcall NtUserCalcMenuBar(long long long long long)
@ stdcall NtUserPaintMenuBar(long long long long long long)
@ stdcall NtUserTranslateAccelerator(ptr ptr ptr)
@ stdcall NtUserTranslateMessage(ptr long)
@ stdcall NtUserUnhookWindowsHookEx(ptr)
@ stdcall NtUserUnhookWinEvent(ptr)
@ stdcall NtUserUnloadKeyboardLayout(ptr)
@ stdcall NtUserUnlockWindowStation(ptr)
@ stdcall NtUserUnregisterClass(ptr ptr ptr)
@ stdcall NtUserUnregisterUserApiHook()
@ stdcall NtUserUnregisterHotKey(ptr long)
@ stdcall NtUserUpdateInputContext(long long long)
@ stdcall NtUserUpdateInstance(long long long)
@ stdcall NtUserUpdateLayeredWindow(ptr ptr ptr ptr ptr ptr long ptr long ptr) # FIXME: 9 params on XP
@ stdcall NtUserGetLayeredWindowAttributes(ptr ptr ptr ptr)
@ stdcall NtUserSetLayeredWindowAttributes(ptr long long long)
@ stdcall NtUserUpdatePerUserSystemParameters(long long)
@ stdcall NtUserUserHandleGrantAccess(ptr ptr long)
#@ stdcall NtUserValidateHandleSecure(ptr long)
@ stdcall NtUserValidateRect(ptr ptr)
#@ stdcall NtUserValidateTimerCallback(ptr ptr ptr)
@ stdcall NtUserVkKeyScanEx(long ptr long)
@ stdcall NtUserWaitForInputIdle(ptr long long)
@ stdcall NtUserWaitForMsgAndEvent(long)
@ stdcall NtUserWaitMessage()
@ stdcall NtUserWin32PoolAllocationStats(long long long long long long)
@ stdcall NtUserWindowFromPoint(long long)
@ stdcall NtUserYieldTask()
@ stdcall NtUserRemoteConnect(long long long)
@ stdcall NtUserRemoteRedrawRectangle(long long long long)
@ stdcall NtUserRemoteRedrawScreen()
@ stdcall NtUserRemoteStopScreenUpdates()
@ stdcall NtUserCtxDisplayIOCtl(long long long)
@ stdcall NtGdiEngAssociateSurface(ptr ptr long)
@ stdcall NtGdiEngCreateBitmap(long long long long long ptr)
@ stdcall NtGdiEngCreateDeviceSurface(ptr long long long)
@ stdcall NtGdiEngCreateDeviceBitmap(ptr long long long)
@ stdcall NtGdiEngCreatePalette(long long ptr long long long)
@ stdcall NtGdiEngComputeGlyphSet(ptr ptr ptr)
@ stdcall NtGdiEngCopyBits(ptr ptr ptr ptr ptr ptr)
@ stdcall NtGdiEngDeletePalette(ptr)
@ stdcall NtGdiEngDeleteSurface(ptr)
@ stdcall NtGdiEngEraseSurface(ptr ptr long)
@ stdcall NtGdiEngUnlockSurface(ptr)
@ stdcall NtGdiEngLockSurface(ptr)
@ stdcall NtGdiEngBitBlt(ptr ptr ptr ptr ptr ptr ptr ptr ptr ptr ptr)
@ stdcall NtGdiEngStretchBlt(ptr ptr ptr ptr ptr ptr ptr ptr ptr ptr long)
@ stdcall NtGdiEngPlgBlt(ptr ptr ptr ptr ptr ptr ptr ptr ptr ptr long)
@ stdcall NtGdiEngMarkBandingSurface(ptr)
@ stdcall NtGdiEngStrokePath(ptr ptr ptr ptr ptr ptr ptr ptr)
@ stdcall NtGdiEngFillPath(ptr ptr ptr ptr ptr ptr ptr)
@ stdcall NtGdiEngStrokeAndFillPath(ptr ptr ptr ptr ptr ptr ptr ptr ptr ptr)
@ stdcall NtGdiEngPaint(ptr ptr ptr ptr ptr)
@ stdcall NtGdiEngLineTo(ptr ptr ptr long long long long ptr ptr)
@ stdcall NtGdiEngAlphaBlend(ptr ptr ptr ptr ptr ptr ptr)
@ stdcall NtGdiEngGradientFill(ptr ptr ptr ptr long ptr long ptr ptr long)
@ stdcall NtGdiEngTransparentBlt(ptr ptr ptr ptr ptr ptr long long)
@ stdcall NtGdiEngTextOut(ptr ptr ptr ptr ptr ptr ptr ptr ptr ptr)
@ stdcall NtGdiEngStretchBltROP(ptr ptr ptr ptr ptr ptr ptr ptr ptr ptr long ptr long)
@ stdcall NtGdiXLATEOBJ_cGetPalette(ptr long long ptr)
@ stdcall NtGdiXLATEOBJ_iXlate(ptr long)
@ stdcall NtGdiXLATEOBJ_hGetColorTransform(ptr)
@ stdcall NtGdiCLIPOBJ_bEnum(ptr long long)
@ stdcall NtGdiCLIPOBJ_cEnumStart(ptr long long long long)
@ stdcall NtGdiCLIPOBJ_ppoGetPath(ptr)
@ stdcall NtGdiEngDeletePath(ptr)
@ stdcall NtGdiEngCreateClip()
@ stdcall NtGdiEngDeleteClip(ptr)
@ stdcall NtGdiBRUSHOBJ_ulGetBrushColor(ptr)
@ stdcall NtGdiBRUSHOBJ_pvAllocRbrush(ptr long)
@ stdcall NtGdiBRUSHOBJ_pvGetRbrush(ptr)
@ stdcall NtGdiBRUSHOBJ_hGetColorTransform(ptr)
@ stdcall NtGdiXFORMOBJ_bApplyXform(ptr ptr long long ptr)
@ stdcall NtGdiXFORMOBJ_iGetXform(ptr ptr)
@ stdcall NtGdiFONTOBJ_vGetInfo(ptr long ptr)
@ stdcall NtGdiFONTOBJ_pxoGetXform(ptr)
@ stdcall NtGdiFONTOBJ_cGetGlyphs(ptr long long ptr ptr)
@ stdcall NtGdiFONTOBJ_pifi(ptr)
@ stdcall NtGdiFONTOBJ_pfdg(ptr)
@ stdcall NtGdiFONTOBJ_pQueryGlyphAttrs(ptr long)
@ stdcall NtGdiFONTOBJ_pvTrueTypeFontFile(ptr ptr)
@ stdcall NtGdiFONTOBJ_cGetAllGlyphHandles(ptr ptr)
@ stdcall NtGdiSTROBJ_bEnum(ptr ptr ptr)
@ stdcall NtGdiSTROBJ_bEnumPositionsOnly(ptr ptr ptr)
@ stdcall NtGdiSTROBJ_bGetAdvanceWidths(ptr long long ptr)
@ stdcall NtGdiSTROBJ_vEnumStart(ptr)
@ stdcall NtGdiSTROBJ_dwGetCodePage(ptr)
@ stdcall NtGdiPATHOBJ_vGetBounds(ptr ptr)
@ stdcall NtGdiPATHOBJ_bEnum(ptr ptr)
@ stdcall NtGdiPATHOBJ_vEnumStart(ptr)
@ stdcall NtGdiPATHOBJ_vEnumStartClipLines(ptr ptr ptr ptr)
@ stdcall NtGdiPATHOBJ_bEnumClipLines(ptr long ptr)
@ stdcall NtGdiGetDhpdev(ptr)
@ stdcall NtGdiEngCheckAbort(ptr)
@ stdcall NtGdiHT_Get8BPPFormatPalette(ptr long long long)
@ stdcall NtGdiHT_Get8BPPMaskPalette(ptr long long long long long)
@ stdcall NtGdiUpdateTransform(ptr)
@ stdcall NtGdiSetPUMPDOBJ(ptr long ptr ptr)
@ stdcall NtGdiBRUSHOBJ_DeleteRbrush(ptr ptr)
@ stdcall NtGdiUMPDEngFreeUserMem(ptr)
@ stdcall NtGdiDrawStream(ptr long ptr)


# WoW64 automarshal targets (M17): stubs exist for every kernel-table
# service (sys-stubs.S includes the full w32ksvc list); these exports were
# simply never listed. Arg counts from w32ksvc32.h (i386 @N decoration).
@ stdcall NtGdiGetFontFamilyInfo(long long long long)
@ stdcall NtGdiMakeObjectXferable(long long)
@ stdcall NtGdiOffsetViewportOrgEx(long long long long)
@ stdcall NtGdiOffsetWindowOrgEx(long long long long)
@ stdcall NtGdiSetViewportOrgEx(long long long long)
@ stdcall NtGdiSetWindowOrgEx(long long long long)
@ stdcall NtUserChangeDisplaySettings(long long long long)
@ stdcall NtUserDbgWin32HeapFail(long long)
@ stdcall NtUserDbgWin32HeapStat(long long)
@ stdcall NtUserEnumDisplayMonitors(long long long long long)
@ stdcall NtUserGetMonitorInfo(long long)
@ stdcall NtUserMonitorFromPoint(long long long)
@ stdcall NtUserMonitorFromRect(long long)
@ stdcall NtUserMonitorFromWindow(long long)
@ stdcall NtUserSetDbgTag(long long)
@ stdcall NtUserSetDbgTagCount(long)
@ stdcall NtUserSetRipFlags(long)
@ stdcall NtUserSetScrollBarInfo(long long long)
# Appended, not placed alphabetically: spec2def numbers exports sequentially,
# so an insertion mid-file would renumber every later ordinal. wow64.dll needs
# the *Ptr setters to carry a full 64-bit window/class proc (wow64_pfnmap.c).
@ stdcall -arch=win64 NtUserSetClassLongPtr(ptr long ptr long)
@ stdcall -arch=win64 NtUserSetWindowLongPtr(ptr long ptr long)

# BEGIN win10-surface stubs
# Generated by tools/win10_surface.py from Windows 10.0.26100.9457; do not edit by hand.
# Names that build exports and this spec does not declare. Dormant below
# DLL_EXPORT_VERSION 0xA00; give a name a real line above and re-run to drop it.
@ stub -version=0xA00+ ABI_Get_DpiMetricsArray
@ stub -arch=win64 -version=0xA00+ ABI_Get_ForegroundWindow
@ stub -version=0xA00+ ABI_Get_OemBitmapSetArray
@ stub -arch=win64 -version=0xA00+ ABI_Get_apfnClientWorkerNtDll
@ stub -version=0xA00+ ABI_Get_cbwndExtraInternal
@ stub -arch=win64 -version=0xA00+ ABI_Get_currentMonitorTopologyId
@ stub -arch=win64 -version=0xA00+ ABI_Get_tooltipStrings
@ stub -version=0xA00+ NtBindCompositionSurface
@ stub -version=0xA00+ NtCloseCompositionInputSink
@ stub -version=0xA00+ NtCompositionInputThread
@ stub -version=0xA00+ NtCompositionSetDropTarget
@ stub -version=0xA00+ NtCompositorNotifyExitWindows
@ stub -version=0xA00+ NtConfigureInputSpace
@ stub -version=0xA00+ NtConfirmCompositionSurfaceIndependentFlipEntry
@ stub -version=0xA00+ NtCreateCompositionInputSink
@ stub -version=0xA00+ NtCreateCompositionSurfaceHandle
@ stub -version=0xA00+ NtCreateImplicitCompositionInputSink
@ stub -version=0xA00+ NtDCompositionAddCrossDeviceVisualChild
@ stub -version=0xA00+ NtDCompositionBeginFrame
@ stub -version=0xA00+ NtDCompositionBoostCompositorClock
@ stub -version=0xA00+ NtDCompositionCommitChannel
@ stub -version=0xA00+ NtDCompositionCommitSynchronizationObject
@ stub -version=0xA00+ NtDCompositionConfirmFrame
@ stub -version=0xA00+ NtDCompositionConnectPipe
@ stub -version=0xA00+ NtDCompositionCreateAndBindSharedSection
@ stub -version=0xA00+ NtDCompositionCreateBufferCollection
@ stub -version=0xA00+ NtDCompositionCreateChannel
@ stub -version=0xA00+ NtDCompositionCreateConnection
@ stub -version=0xA00+ NtDCompositionCreateSharedResourceHandle
@ stub -version=0xA00+ NtDCompositionCreateSynchronizationObject
@ stub -version=0xA00+ NtDCompositionDestroyChannel
@ stub -version=0xA00+ NtDCompositionDestroyConnection
@ stub -version=0xA00+ NtDCompositionDuplicateHandleToProcess
@ stub -version=0xA00+ NtDCompositionDuplicateSwapchainHandleToDwm
@ stub -version=0xA00+ NtDCompositionEnableMMCSS
@ stub -version=0xA00+ NtDCompositionGetBatchId
@ stub -version=0xA00+ NtDCompositionGetChannels
@ stub -version=0xA00+ NtDCompositionGetConnectionBatch
@ stub -version=0xA00+ NtDCompositionGetDeletedResources
@ stub -version=0xA00+ NtDCompositionGetFrameId
@ stub -version=0xA00+ NtDCompositionGetFrameIdFromBatchId
@ stub -version=0xA00+ NtDCompositionGetFrameLegacyTokens
@ stub -version=0xA00+ NtDCompositionGetFrameStatistics
@ stub -version=0xA00+ NtDCompositionGetFrameSurfaceUpdates
@ stub -version=0xA00+ NtDCompositionGetMaterialProperty
@ stub -version=0xA00+ NtDCompositionGetStatistics
@ stub -version=0xA00+ NtDCompositionGetTargetStatistics
@ stub -version=0xA00+ NtDCompositionNotifySuperWetInkWork
@ stub -version=0xA00+ NtDCompositionProcessChannelBatchBuffer
@ stub -version=0xA00+ NtDCompositionRegisterThumbnailVisual
@ stub -version=0xA00+ NtDCompositionRegisterVirtualDesktopVisual
@ stub -version=0xA00+ NtDCompositionReleaseAllResources
@ stub -version=0xA00+ NtDCompositionRemoveCrossDeviceVisualChild
@ stub -version=0xA00+ NtDCompositionSendDwmLpcMessage
@ stub -version=0xA00+ NtDCompositionSetBlurredWallpaperSurface
@ stub -version=0xA00+ NtDCompositionSetChannelCommitCompletionEvent
@ stub -version=0xA00+ NtDCompositionSetChannelConnectionId
@ stub -version=0xA00+ NtDCompositionSetChildRootVisual
@ stub -version=0xA00+ NtDCompositionSetDebugCounter
@ stub -version=0xA00+ NtDCompositionSetMaterialProperty
@ stub -version=0xA00+ NtDCompositionSuspendAnimations
@ stub -version=0xA00+ NtDCompositionSyncWait
@ stub -version=0xA00+ NtDCompositionSynchronize
@ stub -version=0xA00+ NtDCompositionTelemetrySetApplicationId
@ stub -version=0xA00+ NtDCompositionUpdatePointerCapture
@ stub -version=0xA00+ NtDCompositionWaitForChannel
@ stub -version=0xA00+ NtDCompositionWaitForCompositorClock
@ stub -version=0xA00+ NtDesktopCaptureBits
@ stub -version=0xA00+ NtDuplicateCompositionInputSink
@ stub -version=0xA00+ NtDxgkAcquireExclusiveEngineAccess
@ stub -version=0xA00+ NtDxgkCancelPresents
@ stub -version=0xA00+ NtDxgkCheckEngineAccess
@ stub -version=0xA00+ NtDxgkCheckSinglePlaneForMultiPlaneOverlaySupport
@ stub -version=0xA00+ NtDxgkConnectDoorbell
@ stub -version=0xA00+ NtDxgkCreateDoorbell
@ stub -version=0xA00+ NtDxgkCreateNativeFence
@ stub -version=0xA00+ NtDxgkCreateTrackedWorkload
@ stub -version=0xA00+ NtDxgkDDisplayEnum2
@ stub -version=0xA00+ NtDxgkDestroyDoorbell
@ stub -version=0xA00+ NtDxgkDestroyTrackedWorkload
@ stub -version=0xA00+ NtDxgkDisableProcessDebugBlobCollection
@ stub -version=0xA00+ NtDxgkDispMgrOperation
@ stub -version=0xA00+ NtDxgkDisplayMuxSwitchExecute
@ stub -version=0xA00+ NtDxgkDisplayMuxSwitchFinish
@ stub -version=0xA00+ NtDxgkDisplayMuxSwitchPrepare
@ stub -version=0xA00+ NtDxgkDisplayPortOperation
@ stub -version=0xA00+ NtDxgkDuplicateHandle
@ stub -version=0xA00+ NtDxgkEnableProcessDebugBlobCollection
@ stub -version=0xA00+ NtDxgkEnumAdapters3
@ stub -version=0xA00+ NtDxgkEnumProcesses
@ stub -version=0xA00+ NtDxgkGetAvailableTrackedWorkloadIndex
@ stub -version=0xA00+ NtDxgkGetNativeFenceLogDetail
@ stub -version=0xA00+ NtDxgkGetProcessList
@ stub -version=0xA00+ NtDxgkGetProperties
@ stub -version=0xA00+ NtDxgkGetTrackedWorkloadStatistics
@ stub -version=0xA00+ NtDxgkIsFeatureEnabled
@ stub -version=0xA00+ NtDxgkMapProcessDebugBlob
@ stub -version=0xA00+ NtDxgkNotifyWorkSubmission
@ stub -version=0xA00+ NtDxgkOpenNativeFenceFromNtHandle
@ stub -version=0xA00+ NtDxgkOutputDuplPresentToHwQueue
@ stub -version=0xA00+ NtDxgkPinResources
@ stub -version=0xA00+ NtDxgkRegisterVailProcess
@ stub -version=0xA00+ NtDxgkResetTrackedWorkloadStatistics
@ stub -version=0xA00+ NtDxgkSetProperties
@ stub -version=0xA00+ NtDxgkSubmitPresentBltToHwQueue
@ stub -version=0xA00+ NtDxgkSubmitPresentToHwQueue
@ stub -version=0xA00+ NtDxgkUnmapProcessDebugBlob
@ stub -version=0xA00+ NtDxgkUnpinResources
@ stub -version=0xA00+ NtDxgkUpdateTrackedWorkload
@ stub -version=0xA00+ NtDxgkVailConnect
@ stub -version=0xA00+ NtDxgkVailDisconnect
@ stub -version=0xA00+ NtDxgkVailPromoteCompositionSurface
@ stub -version=0xA00+ NtEnableOneCoreTransformMode
@ stub -version=0xA00+ NtFlipObjectAddContent
@ stub -version=0xA00+ NtFlipObjectAddPoolBuffer
@ stub -version=0xA00+ NtFlipObjectConsumerAcquirePresent
@ stub -version=0xA00+ NtFlipObjectConsumerAdjustUsageReference
@ stub -version=0xA00+ NtFlipObjectConsumerBeginProcessPresent
@ stub -version=0xA00+ NtFlipObjectConsumerEndProcessPresent
@ stub -version=0xA00+ NtFlipObjectConsumerPostMessage
@ stub -version=0xA00+ NtFlipObjectConsumerQueryBufferInfo
@ stub -version=0xA00+ NtFlipObjectCreate
@ stub -version=0xA00+ NtFlipObjectDisconnectEndpoint
@ stub -version=0xA00+ NtFlipObjectEnablePresentStatisticsType
@ stub -version=0xA00+ NtFlipObjectOpen
@ stub -version=0xA00+ NtFlipObjectPresentCancel
@ stub -version=0xA00+ NtFlipObjectQueryBufferAvailableEvent
@ stub -version=0xA00+ NtFlipObjectQueryEndpointConnected
@ stub -version=0xA00+ NtFlipObjectQueryLostEvent
@ stub -version=0xA00+ NtFlipObjectQueryNextMessageToProducer
@ stub -version=0xA00+ NtFlipObjectReadNextMessageToProducer
@ stub -version=0xA00+ NtFlipObjectRemoveContent
@ stub -version=0xA00+ NtFlipObjectRemovePoolBuffer
@ stub -version=0xA00+ NtFlipObjectSetContent
@ stub -version=0xA00+ NtGdiAddInitialFonts
@ stub -version=0xA00+ NtGdiBeginGdiRendering
@ stub -version=0xA00+ NtGdiConfigureOPMProtectedOutput
@ stub -version=0xA00+ NtGdiCreateBitmapFromDxSurface
@ stub -version=0xA00+ NtGdiCreateBitmapFromDxSurface2
@ stub -version=0xA00+ NtGdiCreateOPMProtectedOutput
@ stub -version=0xA00+ NtGdiCreateOPMProtectedOutputs
@ stub -version=0xA00+ NtGdiCreateSessionMappedDIBSection
@ stub -version=0xA00+ NtGdiDDCCIGetCapabilitiesString
@ stub -version=0xA00+ NtGdiDDCCIGetCapabilitiesStringLength
@ stub -version=0xA00+ NtGdiDDCCIGetTimingReport
@ stub -version=0xA00+ NtGdiDDCCIGetVCPFeature
@ stub -version=0xA00+ NtGdiDDCCISaveCurrentSettings
@ stub -version=0xA00+ NtGdiDDCCISetVCPFeature
@ stub -version=0xA00+ NtGdiDdCreateFullscreenSprite
@ stub -version=0xA00+ NtGdiDdDDIAbandonSwapChain
@ stub -version=0xA00+ NtGdiDdDDIAcquireKeyedMutex
@ stub -version=0xA00+ NtGdiDdDDIAcquireKeyedMutex2
@ stub -version=0xA00+ NtGdiDdDDIAcquireSwapChain
@ stub -version=0xA00+ NtGdiDdDDIAddSurfaceToSwapChain
@ stub -version=0xA00+ NtGdiDdDDIAdjustFullscreenGamma
@ stub -version=0xA00+ NtGdiDdDDICacheHybridQueryValue
@ stub -version=0xA00+ NtGdiDdDDIChangeVideoMemoryReservation
@ stub -version=0xA00+ NtGdiDdDDICheckExclusiveOwnership
@ stub -version=0xA00+ NtGdiDdDDICheckMonitorPowerState
@ stub -version=0xA00+ NtGdiDdDDICheckMultiPlaneOverlaySupport
@ stub -version=0xA00+ NtGdiDdDDICheckMultiPlaneOverlaySupport2
@ stub -version=0xA00+ NtGdiDdDDICheckMultiPlaneOverlaySupport3
@ stub -version=0xA00+ NtGdiDdDDICheckOcclusion
@ stub -version=0xA00+ NtGdiDdDDICheckSharedResourceAccess
@ stub -version=0xA00+ NtGdiDdDDICheckVidPnExclusiveOwnership
@ stub -version=0xA00+ NtGdiDdDDICloseAdapter
@ stub -version=0xA00+ NtGdiDdDDIConfigureSharedResource
@ stub -version=0xA00+ NtGdiDdDDICreateAllocation
@ stub -version=0xA00+ NtGdiDdDDICreateBundleObject
@ stub -version=0xA00+ NtGdiDdDDICreateContext
@ stub -version=0xA00+ NtGdiDdDDICreateContextVirtual
@ stub -version=0xA00+ NtGdiDdDDICreateDCFromMemory
@ stub -version=0xA00+ NtGdiDdDDICreateDevice
@ stub -version=0xA00+ NtGdiDdDDICreateHwContext
@ stub -version=0xA00+ NtGdiDdDDICreateHwQueue
@ stub -version=0xA00+ NtGdiDdDDICreateKeyedMutex
@ stub -version=0xA00+ NtGdiDdDDICreateKeyedMutex2
@ stub -version=0xA00+ NtGdiDdDDICreateOutputDupl
@ stub -version=0xA00+ NtGdiDdDDICreateOverlay
@ stub -version=0xA00+ NtGdiDdDDICreatePagingQueue
@ stub -version=0xA00+ NtGdiDdDDICreateProtectedSession
@ stub -version=0xA00+ NtGdiDdDDICreateSwapChain
@ stub -version=0xA00+ NtGdiDdDDICreateSynchronizationObject
@ stub -version=0xA00+ NtGdiDdDDIDDisplayEnum
@ stub -version=0xA00+ NtGdiDdDDIDestroyAllocation
@ stub -version=0xA00+ NtGdiDdDDIDestroyAllocation2
@ stub -version=0xA00+ NtGdiDdDDIDestroyContext
@ stub -version=0xA00+ NtGdiDdDDIDestroyDCFromMemory
@ stub -version=0xA00+ NtGdiDdDDIDestroyDevice
@ stub -version=0xA00+ NtGdiDdDDIDestroyHwContext
@ stub -version=0xA00+ NtGdiDdDDIDestroyHwQueue
@ stub -version=0xA00+ NtGdiDdDDIDestroyKeyedMutex
@ stub -version=0xA00+ NtGdiDdDDIDestroyOutputDupl
@ stub -version=0xA00+ NtGdiDdDDIDestroyOverlay
@ stub -version=0xA00+ NtGdiDdDDIDestroyPagingQueue
@ stub -version=0xA00+ NtGdiDdDDIDestroyProtectedSession
@ stub -version=0xA00+ NtGdiDdDDIDestroySynchronizationObject
@ stub -version=0xA00+ NtGdiDdDDIDispMgrCreate
@ stub -version=0xA00+ NtGdiDdDDIDispMgrSourceOperation
@ stub -version=0xA00+ NtGdiDdDDIDispMgrTargetOperation
@ stub -version=0xA00+ NtGdiDdDDIEnumAdapters
@ stub -version=0xA00+ NtGdiDdDDIEnumAdapters2
@ stub -version=0xA00+ NtGdiDdDDIEscape
@ stub -version=0xA00+ NtGdiDdDDIEvict
@ stub -version=0xA00+ NtGdiDdDDIExtractBundleObject
@ stub -version=0xA00+ NtGdiDdDDIFlipOverlay
@ stub -version=0xA00+ NtGdiDdDDIFlushHeapTransitions
@ stub -version=0xA00+ NtGdiDdDDIFreeGpuVirtualAddress
@ stub -version=0xA00+ NtGdiDdDDIGetAllocationPriority
@ stub -version=0xA00+ NtGdiDdDDIGetCachedHybridQueryValue
@ stub -version=0xA00+ NtGdiDdDDIGetContextInProcessSchedulingPriority
@ stub -version=0xA00+ NtGdiDdDDIGetContextSchedulingPriority
@ stub -version=0xA00+ NtGdiDdDDIGetDWMVerticalBlankEvent
@ stub -version=0xA00+ NtGdiDdDDIGetDeviceState
@ stub -version=0xA00+ NtGdiDdDDIGetDisplayModeList
@ stub -version=0xA00+ NtGdiDdDDIGetMemoryBudgetTarget
@ stub -version=0xA00+ NtGdiDdDDIGetMultiPlaneOverlayCaps
@ stub -version=0xA00+ NtGdiDdDDIGetMultisampleMethodList
@ stub -version=0xA00+ NtGdiDdDDIGetOverlayState
@ stub -version=0xA00+ NtGdiDdDDIGetPostCompositionCaps
@ stub -version=0xA00+ NtGdiDdDDIGetPresentHistory
@ stub -version=0xA00+ NtGdiDdDDIGetPresentQueueEvent
@ stub -version=0xA00+ NtGdiDdDDIGetProcessDeviceRemovalSupport
@ stub -version=0xA00+ NtGdiDdDDIGetProcessSchedulingPriorityBand
@ stub -version=0xA00+ NtGdiDdDDIGetProcessSchedulingPriorityClass
@ stub -version=0xA00+ NtGdiDdDDIGetResourcePresentPrivateDriverData
@ stub -version=0xA00+ NtGdiDdDDIGetRuntimeData
@ stub -version=0xA00+ NtGdiDdDDIGetScanLine
@ stub -version=0xA00+ NtGdiDdDDIGetSetSwapChainMetadata
@ stub -version=0xA00+ NtGdiDdDDIGetSharedPrimaryHandle
@ stub -version=0xA00+ NtGdiDdDDIGetSharedResourceAdapterLuid
@ stub -version=0xA00+ NtGdiDdDDIGetSharedResourceAdapterLuidFlipManager
@ stub -version=0xA00+ NtGdiDdDDIGetSwapChainSurfacePhysicalAddress
@ stub -version=0xA00+ NtGdiDdDDIGetYieldPercentage
@ stub -version=0xA00+ NtGdiDdDDIInvalidateActiveVidPn
@ stub -version=0xA00+ NtGdiDdDDIInvalidateCache
@ stub -version=0xA00+ NtGdiDdDDILock
@ stub -version=0xA00+ NtGdiDdDDILock2
@ stub -version=0xA00+ NtGdiDdDDIMakeResident
@ stub -version=0xA00+ NtGdiDdDDIMapGpuVirtualAddress
@ stub -version=0xA00+ NtGdiDdDDIMarkDeviceAsError
@ stub -version=0xA00+ NtGdiDdDDINetDispGetNextChunkInfo
@ stub -version=0xA00+ NtGdiDdDDINetDispQueryMiracastDisplayDeviceStatus
@ stub -version=0xA00+ NtGdiDdDDINetDispQueryMiracastDisplayDeviceSupport
@ stub -version=0xA00+ NtGdiDdDDINetDispStartMiracastDisplayDevice
@ stub -version=0xA00+ NtGdiDdDDINetDispStopMiracastDisplayDevice
@ stub -version=0xA00+ NtGdiDdDDIOfferAllocations
@ stub -version=0xA00+ NtGdiDdDDIOpenAdapterFromDeviceName
@ stub -version=0xA00+ NtGdiDdDDIOpenAdapterFromHdc
@ stub -version=0xA00+ NtGdiDdDDIOpenAdapterFromLuid
@ stub -version=0xA00+ NtGdiDdDDIOpenBundleObjectNtHandleFromName
@ stub -version=0xA00+ NtGdiDdDDIOpenKeyedMutex
@ stub -version=0xA00+ NtGdiDdDDIOpenKeyedMutex2
@ stub -version=0xA00+ NtGdiDdDDIOpenKeyedMutexFromNtHandle
@ stub -version=0xA00+ NtGdiDdDDIOpenNtHandleFromName
@ stub -version=0xA00+ NtGdiDdDDIOpenProtectedSessionFromNtHandle
@ stub -version=0xA00+ NtGdiDdDDIOpenResource
@ stub -version=0xA00+ NtGdiDdDDIOpenResourceFromNtHandle
@ stub -version=0xA00+ NtGdiDdDDIOpenSwapChain
@ stub -version=0xA00+ NtGdiDdDDIOpenSyncObjectFromNtHandle
@ stub -version=0xA00+ NtGdiDdDDIOpenSyncObjectFromNtHandle2
@ stub -version=0xA00+ NtGdiDdDDIOpenSyncObjectNtHandleFromName
@ stub -version=0xA00+ NtGdiDdDDIOpenSynchronizationObject
@ stub -version=0xA00+ NtGdiDdDDIOutputDuplGetFrameInfo
@ stub -version=0xA00+ NtGdiDdDDIOutputDuplGetMetaData
@ stub -version=0xA00+ NtGdiDdDDIOutputDuplGetPointerShapeData
@ stub -version=0xA00+ NtGdiDdDDIOutputDuplPresent
@ stub -version=0xA00+ NtGdiDdDDIOutputDuplReleaseFrame
@ stub -version=0xA00+ NtGdiDdDDIPollDisplayChildren
@ stub -version=0xA00+ NtGdiDdDDIPresent
@ stub -version=0xA00+ NtGdiDdDDIPresentMultiPlaneOverlay
@ stub -version=0xA00+ NtGdiDdDDIPresentMultiPlaneOverlay2
@ stub -version=0xA00+ NtGdiDdDDIPresentMultiPlaneOverlay3
@ stub -version=0xA00+ NtGdiDdDDIPresentRedirected
@ stub -version=0xA00+ NtGdiDdDDIQueryAdapterInfo
@ stub -version=0xA00+ NtGdiDdDDIQueryAllocationResidency
@ stub -version=0xA00+ NtGdiDdDDIQueryClockCalibration
@ stub -version=0xA00+ NtGdiDdDDIQueryFSEBlock
@ stub -version=0xA00+ NtGdiDdDDIQueryProcessOfferInfo
@ stub -version=0xA00+ NtGdiDdDDIQueryProtectedSessionInfoFromNtHandle
@ stub -version=0xA00+ NtGdiDdDDIQueryProtectedSessionStatus
@ stub -version=0xA00+ NtGdiDdDDIQueryRemoteVidPnSourceFromGdiDisplayName
@ stub -version=0xA00+ NtGdiDdDDIQueryResourceInfo
@ stub -version=0xA00+ NtGdiDdDDIQueryResourceInfoFromNtHandle
@ stub -version=0xA00+ NtGdiDdDDIQueryStatistics
@ stub -version=0xA00+ NtGdiDdDDIQueryVidPnExclusiveOwnership
@ stub -version=0xA00+ NtGdiDdDDIQueryVideoMemoryInfo
@ stub -version=0xA00+ NtGdiDdDDIReclaimAllocations
@ stub -version=0xA00+ NtGdiDdDDIReclaimAllocations2
@ stub -version=0xA00+ NtGdiDdDDIReleaseKeyedMutex
@ stub -version=0xA00+ NtGdiDdDDIReleaseKeyedMutex2
@ stub -version=0xA00+ NtGdiDdDDIReleaseProcessVidPnSourceOwners
@ stub -version=0xA00+ NtGdiDdDDIReleaseSwapChain
@ stub -version=0xA00+ NtGdiDdDDIRemoveSurfaceFromSwapChain
@ stub -version=0xA00+ NtGdiDdDDIRender
@ stub -version=0xA00+ NtGdiDdDDIReserveGpuVirtualAddress
@ stub -version=0xA00+ NtGdiDdDDISetAllocationPriority
@ stub -version=0xA00+ NtGdiDdDDISetContextInProcessSchedulingPriority
@ stub -version=0xA00+ NtGdiDdDDISetContextSchedulingPriority
@ stub -version=0xA00+ NtGdiDdDDISetDisplayMode
@ stub -version=0xA00+ NtGdiDdDDISetDodIndirectSwapchain
@ stub -version=0xA00+ NtGdiDdDDISetFSEBlock
@ stub -version=0xA00+ NtGdiDdDDISetGammaRamp
@ stub -version=0xA00+ NtGdiDdDDISetHwProtectionTeardownRecovery
@ stub -version=0xA00+ NtGdiDdDDISetMemoryBudgetTarget
@ stub -version=0xA00+ NtGdiDdDDISetMonitorColorSpaceTransform
@ stub -version=0xA00+ NtGdiDdDDISetProcessDeviceRemovalSupport
@ stub -version=0xA00+ NtGdiDdDDISetProcessSchedulingPriorityBand
@ stub -version=0xA00+ NtGdiDdDDISetProcessSchedulingPriorityClass
@ stub -version=0xA00+ NtGdiDdDDISetQueuedLimit
@ stub -version=0xA00+ NtGdiDdDDISetStablePowerState
@ stub -version=0xA00+ NtGdiDdDDISetStereoEnabled
@ stub -version=0xA00+ NtGdiDdDDISetSyncRefreshCountWaitTarget
@ stub -version=0xA00+ NtGdiDdDDISetVidPnSourceHwProtection
@ stub -version=0xA00+ NtGdiDdDDISetVidPnSourceOwner
@ stub -version=0xA00+ NtGdiDdDDISetYieldPercentage
@ stub -version=0xA00+ NtGdiDdDDIShareObjects
@ stub -version=0xA00+ NtGdiDdDDISharedPrimaryLockNotification
@ stub -version=0xA00+ NtGdiDdDDISharedPrimaryUnLockNotification
@ stub -version=0xA00+ NtGdiDdDDISignalSynchronizationObject
@ stub -version=0xA00+ NtGdiDdDDISignalSynchronizationObjectFromCpu
@ stub -version=0xA00+ NtGdiDdDDISignalSynchronizationObjectFromGpu
@ stub -version=0xA00+ NtGdiDdDDISignalSynchronizationObjectFromGpu2
@ stub -version=0xA00+ NtGdiDdDDISubmitCommand
@ stub -version=0xA00+ NtGdiDdDDISubmitCommandToHwQueue
@ stub -version=0xA00+ NtGdiDdDDISubmitSignalSyncObjectsToHwQueue
@ stub -version=0xA00+ NtGdiDdDDISubmitWaitForSyncObjectsToHwQueue
@ stub -version=0xA00+ NtGdiDdDDITrimProcessCommitment
@ stub -version=0xA00+ NtGdiDdDDIUnOrderedPresentSwapChain
@ stub -version=0xA00+ NtGdiDdDDIUnlock
@ stub -version=0xA00+ NtGdiDdDDIUnlock2
@ stub -version=0xA00+ NtGdiDdDDIUpdateAllocationProperty
@ stub -version=0xA00+ NtGdiDdDDIUpdateGpuVirtualAddress
@ stub -version=0xA00+ NtGdiDdDDIUpdateOverlay
@ stub -version=0xA00+ NtGdiDdDDIWaitForIdle
@ stub -version=0xA00+ NtGdiDdDDIWaitForSynchronizationObject
@ stub -version=0xA00+ NtGdiDdDDIWaitForSynchronizationObjectFromCpu
@ stub -version=0xA00+ NtGdiDdDDIWaitForSynchronizationObjectFromGpu
@ stub -version=0xA00+ NtGdiDdDDIWaitForVerticalBlankEvent
@ stub -version=0xA00+ NtGdiDdDDIWaitForVerticalBlankEvent2
@ stub -version=0xA00+ NtGdiDdDestroyFullscreenSprite
@ stub -version=0xA00+ NtGdiDdNotifyFullscreenSpriteUpdate
@ stub -version=0xA00+ NtGdiDdQueryVisRgnUniqueness
@ stub -version=0xA00+ NtGdiDestroyOPMProtectedOutput
@ stub -version=0xA00+ NtGdiDestroyPhysicalMonitor
@ stub -version=0xA00+ NtGdiDisableUMPDSandboxing
@ stub -version=0xA00+ NtGdiDwmCreatedBitmapRemotingOutput
@ stub -version=0xA00+ NtGdiEndGdiRendering
@ stub -version=0xA00+ NtGdiEnsureDpiDepDefaultGuiFontForPlateau
@ stub -version=0xA00+ NtGdiEnumFonts
@ stub -version=0xA00+ NtGdiGetBitmapDpiScaleValue
@ stub -version=0xA00+ NtGdiGetCOPPCompatibleOPMInformation
@ stub -version=0xA00+ NtGdiGetCertificate
@ stub -version=0xA00+ NtGdiGetCertificateByHandle
@ stub -version=0xA00+ NtGdiGetCertificateSize
@ stub -version=0xA00+ NtGdiGetCertificateSizeByHandle
@ stub -version=0xA00+ NtGdiGetCurrentDpiInfo
@ stub -version=0xA00+ NtGdiGetDCDpiScaleValue
@ stub -version=0xA00+ NtGdiGetEntry
@ stub -version=0xA00+ NtGdiGetNumberOfPhysicalMonitors
@ stub -version=0xA00+ NtGdiGetOPMInformation
@ stub -version=0xA00+ NtGdiGetOPMRandomNumber
@ stub -version=0xA00+ NtGdiGetPhysicalMonitorDescription
@ stub -version=0xA00+ NtGdiGetPhysicalMonitorFromTarget
@ stub -version=0xA00+ NtGdiGetPhysicalMonitors
@ stub -version=0xA00+ NtGdiGetProcessSessionFonts
@ stub -version=0xA00+ NtGdiGetPublicFontTableChangeCookie
@ stub -version=0xA00+ NtGdiGetSuggestedOPMProtectedOutputArraySize
@ stub -version=0xA00+ NtGdiHLSurfGetInformation
@ stub -version=0xA00+ NtGdiHLSurfSetInformation
@ stub -version=0xA00+ NtGdiInit2
@ stub -version=0xA00+ NtGdiMakeObjectUnXferable
@ stub -version=0xA00+ NtGdiScaleRgn
@ stub -version=0xA00+ NtGdiScaleValues
@ stub -version=0xA00+ NtGdiSetOPMSigningKeyAndSequenceNumbers
@ stub -version=0xA00+ NtGdiSetUMPDSandboxState
@ stub -version=0xA00+ NtGdiWaitForTextReady
@ stub -version=0xA00+ NtHWCursorUpdatePointer
@ stub -version=0xA00+ NtInputSpaceRegionFromPoint
@ stub -version=0xA00+ NtIsOneCoreTransformMode
@ stub -version=0xA00+ NtKSTInitialize
@ stub -version=0xA00+ NtKSTWait
@ stub -version=0xA00+ NtMITAccessibilityTimerNotification
@ stub -version=0xA00+ NtMITActivateInputProcessing
@ stub -version=0xA00+ NtMITConfigureVirtualTouchpad
@ stub -version=0xA00+ NtMITCoreMsgKOpenConnectionTo
@ stub -version=0xA00+ NtMITDeactivateInputProcessing
@ stub -version=0xA00+ NtMITDisableMouseIntercept
@ stub -version=0xA00+ NtMITDispatchCompletion
@ stub -version=0xA00+ NtMITEnableMouseIntercept
@ stub -version=0xA00+ NtMITGetCursorUpdateHandle
@ stub -version=0xA00+ NtMITInitMinuserThread
@ stub -version=0xA00+ NtMITMinuserSetInputTransformOffset
@ stub -version=0xA00+ NtMITMinuserWindowDestroyed
@ stub -version=0xA00+ NtMITPostMouseInputMessage
@ stub -version=0xA00+ NtMITPostThreadEventMessage
@ stub -version=0xA00+ NtMITPostWindowEventMessage
@ stub -version=0xA00+ NtMITPrepareReceiveInputMessage
@ stub -version=0xA00+ NtMITPrepareSendInputMessage
@ stub -version=0xA00+ NtMITProcessDelegateCapturedPointers
@ stub -version=0xA00+ NtMITSetInputCallbacks
@ stub -version=0xA00+ NtMITSetInputDelegationMode
@ stub -version=0xA00+ NtMITSetInputObservationState
@ stub -version=0xA00+ NtMITSetKeyboardInputRoutingPolicy
@ stub -version=0xA00+ NtMITSetKeyboardOverriderState
@ stub -version=0xA00+ NtMITSetLastInputRecipient
@ stub -version=0xA00+ NtMITSynthesizeKeyboardInput
@ stub -version=0xA00+ NtMITSynthesizeMouseInput
@ stub -version=0xA00+ NtMITSynthesizeTouchInput
@ stub -version=0xA00+ NtMITUninitMinuserThread
@ stub -version=0xA00+ NtMITUpdateInputGlobals
@ stub -version=0xA00+ NtMapVisualRelativePoints
@ stub -version=0xA00+ NtMinGetInputTransform
@ stub -version=0xA00+ NtMinInteropCoreMessagingWithInput
@ stub -version=0xA00+ NtMinQPeekForInput
@ stub -version=0xA00+ NtMinQSuspendInputProcessing
@ stub -version=0xA00+ NtMinQUpdateWakeMask
@ stub -version=0xA00+ NtModerncoreBeginLayoutUpdate
@ stub -version=0xA00+ NtModerncoreCreateDCompositionHwndTarget
@ stub -version=0xA00+ NtModerncoreCreateGDIHwndTarget
@ stub -version=0xA00+ NtModerncoreDestroyDCompositionHwndTarget
@ stub -version=0xA00+ NtModerncoreDestroyGDIHwndTarget
@ stub -version=0xA00+ NtModerncoreEnableResizeLayoutSynchronization
@ stub -version=0xA00+ NtModerncoreGetNavigationWindowVisual
@ stub -version=0xA00+ NtModerncoreGetResizeDCompositionSynchronizationObject
@ stub -version=0xA00+ NtModerncoreGetWindowContentVisual
@ stub -version=0xA00+ NtModerncoreIdleTimerThread
@ stub -version=0xA00+ NtModerncoreIsResizeLayoutSynchronizationEnabled
@ stub -version=0xA00+ NtModerncoreProcessConnect
@ stub -version=0xA00+ NtModerncoreRegisterEnhancedNavigationWindowHandle
@ stub -version=0xA00+ NtModerncoreRegisterNavigationWindowHandle
@ stub -version=0xA00+ NtModerncoreSetNavigationServiceSid
@ stub -version=0xA00+ NtModerncoreUnregisterNavigationWindowHandle
@ stub -version=0xA00+ NtNotifyPresentToCompositionSurface
@ stub -version=0xA00+ NtOpenCompositionSurfaceDirtyRegion
@ stub -version=0xA00+ NtOpenCompositionSurfaceRealizationInfo
@ stub -version=0xA00+ NtOpenCompositionSurfaceSectionInfo
@ stub -version=0xA00+ NtQueryCompositionInputIsImplicit
@ stub -version=0xA00+ NtQueryCompositionInputQueueAndTransform
@ stub -version=0xA00+ NtQueryCompositionInputSink
@ stub -version=0xA00+ NtQueryCompositionInputSinkLuid
@ stub -version=0xA00+ NtQueryCompositionInputSinkViewId
@ stub -version=0xA00+ NtQueryCompositionSurfaceBinding
@ stub -version=0xA00+ NtQueryCompositionSurfaceFrameRate
@ stub -version=0xA00+ NtQueryCompositionSurfaceHDRMetaData
@ stub -version=0xA00+ NtQueryCompositionSurfaceRenderingRealization
@ stub -version=0xA00+ NtQueryCompositionSurfaceStatistics
@ stub -version=0xA00+ NtRIMAddInputObserver
@ stub -version=0xA00+ NtRIMAreSiblingDevices
@ stub -version=0xA00+ NtRIMDeviceIoControl
@ stub -version=0xA00+ NtRIMEnableMonitorMappingForDevice
@ stub -version=0xA00+ NtRIMFreeInputBuffer
@ stub -version=0xA00+ NtRIMGetDevicePreparsedData
@ stub -version=0xA00+ NtRIMGetDevicePreparsedDataLockfree
@ stub -version=0xA00+ NtRIMGetDeviceProperties
@ stub -version=0xA00+ NtRIMGetDevicePropertiesLockfree
@ stub -version=0xA00+ NtRIMGetPhysicalDeviceRect
@ stub -version=0xA00+ NtRIMGetSourceProcessId
@ stub -version=0xA00+ NtRIMObserveNextInput
@ stub -version=0xA00+ NtRIMOnAsyncPnpWorkNotification
@ stub -version=0xA00+ NtRIMOnPnpNotification
@ stub -version=0xA00+ NtRIMOnTimerNotification
@ stub -version=0xA00+ NtRIMQueryDevicePath
@ stub -version=0xA00+ NtRIMReadInput
@ stub -version=0xA00+ NtRIMRegisterForInput
@ stub -version=0xA00+ NtRIMRegisterForInputEx
@ stub -version=0xA00+ NtRIMRemoveInputObserver
@ stub -version=0xA00+ NtRIMSetDeadzoneRotation
@ stub -version=0xA00+ NtRIMSetExtendedDeviceProperty
@ stub -version=0xA00+ NtRIMSetTestModeStatus
@ stub -version=0xA00+ NtRIMUnregisterForInput
@ stub -version=0xA00+ NtRIMUpdateInputObserverRegistration
@ stub -version=0xA00+ NtSetCompositionSurfaceAnalogExclusive
@ stub -version=0xA00+ NtSetCompositionSurfaceBufferUsage
@ stub -version=0xA00+ NtSetCompositionSurfaceDirectFlipState
@ stub -version=0xA00+ NtSetCompositionSurfaceIndependentFlipInfo
@ stub -version=0xA00+ NtSetCompositionSurfaceStatistics
@ stub -version=0xA00+ NtSetCursorInputSpace
@ stub -version=0xA00+ NtSetPointerDeviceInputSpace
@ stub -version=0xA00+ NtSetShellCursorState
@ stub -version=0xA00+ NtTokenManagerConfirmOutstandingAnalogToken
@ stub -version=0xA00+ NtTokenManagerCreateCompositionTokenHandle
@ stub -version=0xA00+ NtTokenManagerCreateFlipObjectReturnTokenHandle
@ stub -version=0xA00+ NtTokenManagerCreateFlipObjectTokenHandle
@ stub -version=0xA00+ NtTokenManagerGetAnalogExclusiveSurfaceUpdates
@ stub -version=0xA00+ NtTokenManagerGetAnalogExclusiveTokenEvent
@ stub -version=0xA00+ NtTokenManagerOpenSectionAndEvents
@ stub -version=0xA00+ NtTokenManagerThread
@ stub -version=0xA00+ NtUnBindCompositionSurface
@ stub -version=0xA00+ NtUpdateInputSinkTransforms
@ stub -version=0xA00+ NtUserAcquireIAMKey
@ stub -version=0xA00+ NtUserAcquireInteractiveControlBackgroundAccess
@ stub -version=0xA00+ NtUserAddClipboardFormatListener
@ stub -version=0xA00+ NtUserAddVisualIdentifier
@ stub -version=0xA00+ NtUserAllowForegroundActivation
@ stub -version=0xA00+ NtUserAllowSetForegroundWindow
@ stub -version=0xA00+ NtUserApplyWindowAction
@ stub -version=0xA00+ NtUserArrangeIconicWindows
@ stub -version=0xA00+ NtUserAutoPromoteMouseInPointer
@ stub -version=0xA00+ NtUserAutoRotateScreen
@ stub -version=0xA00+ NtUserBeginDeferWindowPos
@ stub -version=0xA00+ NtUserBeginLayoutUpdate
@ stub -version=0xA00+ NtUserBroadcastImeShowStatusChange
@ stub -version=0xA00+ NtUserBroadcastThemeChangeEvent
@ stub -version=0xA00+ NtUserCalculatePopupWindowPosition
@ stub -version=0xA00+ NtUserCallHwndLockSafe
@ stub -version=0xA00+ NtUserCallHwndParamLockSafe
@ stub -version=0xA00+ NtUserCallHwndSafe
@ stub -version=0xA00+ NtUserCanBrokerForceForeground
@ stub -version=0xA00+ NtUserCanCurrentThreadChangeForeground
@ stub -version=0xA00+ NtUserCancelQueueEventCompletionPacket
@ stub -version=0xA00+ NtUserChangeWindowMessageFilter
@ stub -version=0xA00+ NtUserChangeWindowMessageFilterEx
@ stub -version=0xA00+ NtUserCheckAccessForIntegrityLevel
@ stub -version=0xA00+ NtUserCheckImeShowStatusInThread
@ stub -version=0xA00+ NtUserCheckProcessForClipboardAccess
@ stub -version=0xA00+ NtUserCheckProcessSession
@ stub -version=0xA00+ NtUserCheckWindowThreadDesktop
@ stub -version=0xA00+ NtUserCitSetInfo
@ stub -version=0xA00+ NtUserClearForeground
@ stub -version=0xA00+ NtUserClearWakeMask
@ stub -version=0xA00+ NtUserClearWindowState
@ stub -version=0xA00+ NtUserCompositionInputSinkLuidFromPoint
@ stub -version=0xA00+ NtUserCompositionInputSinkViewInstanceIdFromPoint
@ stub -version=0xA00+ NtUserConfigureActivationObject
@ stub -version=0xA00+ NtUserConfirmResizeCommit
@ stub -version=0xA00+ NtUserConvertPrimaryPointerToMouseDrag
@ stub -version=0xA00+ NtUserConvertToInterceptWindow
@ stub -version=0xA00+ NtUserCreateActivationObject
@ stub -version=0xA00+ NtUserCreateBaseWindow
@ stub -version=0xA00+ NtUserCreateDCompositionHwndTarget
@ stub -version=0xA00+ NtUserCreateDesktopEx
@ stub -version=0xA00+ NtUserCreateEmptyCursorObject
@ stub -version=0xA00+ NtUserCreateLayoutSyncForHwnd
@ stub -version=0xA00+ NtUserCreateMenu
@ stub -version=0xA00+ NtUserCreatePalmRejectionDelayZone
@ stub -version=0xA00+ NtUserCreatePopupMenu
@ stub -version=0xA00+ NtUserCreateSyntheticPointerDevice2
@ stub -version=0xA00+ NtUserCreateSystemThreads
@ stub -version=0xA00+ NtUserCreateWindowStationEx
@ stub -version=0xA00+ NtUserCsDdeUninitialize
@ stub -version=0xA00+ NtUserDWP_GetEnabledPopupOffset
@ stub -version=0xA00+ NtUserDeferWindowDpiChanges
@ stub -version=0xA00+ NtUserDeferWindowPosAndBand
@ stub -version=0xA00+ NtUserDeferredDesktopRotation
@ stub -version=0xA00+ NtUserDelegateCapturePointers
@ stub -version=0xA00+ NtUserDelegateInput
@ stub -version=0xA00+ NtUserDeregisterShellHookWindow
@ stub -version=0xA00+ NtUserDestroyActivationObject
@ stub -version=0xA00+ NtUserDestroyCaret
@ stub -version=0xA00+ NtUserDestroyDCompositionHwndTarget
@ stub -version=0xA00+ NtUserDestroyPalmRejectionDelayZone
@ stub -version=0xA00+ NtUserDirectedYield
@ stub -version=0xA00+ NtUserDisableImmersiveOwner
@ stub -version=0xA00+ NtUserDisableProcessWindowFiltering
@ stub -version=0xA00+ NtUserDisableProcessWindowsGhosting
@ stub -version=0xA00+ NtUserDiscardPointerFrameMessages
@ stub -version=0xA00+ NtUserDisplayConfigGetDeviceInfo
@ stub -version=0xA00+ NtUserDisplayConfigSetDeviceInfo
@ stub -version=0xA00+ NtUserDoInitMessagePumpHook
@ stub -version=0xA00+ NtUserDoMenuOperation
@ stub -version=0xA00+ NtUserDoSoundConnect
@ stub -version=0xA00+ NtUserDoSoundDisconnect
@ stub -version=0xA00+ NtUserDoUninitMessagePumpHook
@ stub -version=0xA00+ NtUserDownlevelTouchpad
@ stub -version=0xA00+ NtUserDrainThreadCoreMessagingCompletions
@ stub -version=0xA00+ NtUserDrainThreadCoreMessagingCompletions2
@ stub -version=0xA00+ NtUserDrawMenuBar
@ stub -version=0xA00+ NtUserDwmGetRemoteSessionOcclusionEvent
@ stub -version=0xA00+ NtUserDwmGetRemoteSessionOcclusionState
@ stub -version=0xA00+ NtUserDwmKernelShutdown
@ stub -version=0xA00+ NtUserDwmKernelStartup
@ stub -version=0xA00+ NtUserDwmLockScreenUpdates
@ stub -version=0xA00+ NtUserDwmValidateWindow
@ stub -version=0xA00+ NtUserDwmWindowNotificationsEnabled
@ stub -version=0xA00+ NtUserEnableChildWindowDpiMessage
@ stub -version=0xA00+ NtUserEnableIAMAccess
@ stub -version=0xA00+ NtUserEnableModernAppWindowKeyboardIntercept
@ stub -version=0xA00+ NtUserEnableMouseInPointer
@ stub -version=0xA00+ NtUserEnableMouseInPointerForThread
@ stub -version=0xA00+ NtUserEnableMouseInPointerForWindow
@ stub -version=0xA00+ NtUserEnableMouseInputForCursorSuppression
@ stub -version=0xA00+ NtUserEnableNonClientDpiScaling
@ stub -version=0xA00+ NtUserEnablePerMonitorMenuScaling
@ stub -version=0xA00+ NtUserEnableResizeLayoutSynchronization
@ stub -version=0xA00+ NtUserEnableSessionForMMCSS
@ stub -version=0xA00+ NtUserEnableShellWindowManagementBehavior
@ stub -version=0xA00+ NtUserEnableSoftwareCursorForScreenCapture
@ stub -version=0xA00+ NtUserEnableSynchronizedLayout
@ stub -version=0xA00+ NtUserEnableTouchPad
@ stub -version=0xA00+ NtUserEnableWindow
@ stub -version=0xA00+ NtUserEnableWindowGDIScaledDpiMessage
@ stub -version=0xA00+ NtUserEnableWindowResizeOptimization
@ stub -version=0xA00+ NtUserEnableWindowShellWindowManagementBehavior
@ stub -version=0xA00+ NtUserEnsureDpiDepSysMetCacheForPlateau
@ stub -version=0xA00+ NtUserEnsureDpiMetricsForDpi
@ stub -version=0xA00+ NtUserEnsureDpiServerInfoForDpi
@ stub -version=0xA00+ NtUserEnsureOemBitmapInfoForDpi
@ stub -version=0xA00+ NtUserEnterMoveSizeLoop
@ stub -version=0xA00+ NtUserEnumClipboardFormats
@ stub -version=0xA00+ NtUserForceEnableNumpadTranslation
@ stub -version=0xA00+ NtUserForceMaxTouchpadSensitivityForCurrentProcess
@ stub -version=0xA00+ NtUserForceWindowToDpiForTest
@ stub -version=0xA00+ NtUserFrostCrashedWindow
@ stub -version=0xA00+ NtUserFunctionalizeDisplayConfig
@ stub -version=0xA00+ NtUserGetActiveProcessesDpis
@ stub -version=0xA00+ NtUserGetAutoRotationState
@ stub -version=0xA00+ NtUserGetCIMSSM
@ stub -version=0xA00+ NtUserGetClassIcoCur
@ stub -version=0xA00+ NtUserGetClassInfoEx
@ stub -version=0xA00+ NtUserGetClipboardAccessToken
@ stub -version=0xA00+ NtUserGetClipboardMetadata
@ stub -version=0xA00+ NtUserGetCurrentDpiInfoForWindow
@ stub -version=0xA00+ NtUserGetCurrentInputMessageSource
@ stub -version=0xA00+ NtUserGetCursor
@ stub -version=0xA00+ NtUserGetCursorPos
@ stub -version=0xA00+ NtUserGetDCompositionHwndBitmap
@ stub -version=0xA00+ NtUserGetDManipHookInitFunction
@ stub -version=0xA00+ NtUserGetDesktopID
@ stub -version=0xA00+ NtUserGetDesktopVisualTransform
@ stub -version=0xA00+ NtUserGetDeviceChangeInfo
@ stub -version=0xA00+ NtUserGetDisplayAutoRotationPreferences
@ stub -version=0xA00+ NtUserGetDisplayAutoRotationPreferencesByProcessId
@ stub -version=0xA00+ NtUserGetDisplayConfigBufferSizes
@ stub -version=0xA00+ NtUserGetDpiForCurrentProcess
@ stub -version=0xA00+ NtUserGetDpiForMonitor
@ stub -version=0xA00+ NtUserGetDwmCursorShape
@ stub -version=0xA00+ NtUserGetExtendedPointerDeviceProperty
@ stub -version=0xA00+ NtUserGetGestureConfig
@ stub -version=0xA00+ NtUserGetGestureExtArgs
@ stub -version=0xA00+ NtUserGetGestureInfo
@ stub -version=0xA00+ NtUserGetHDevName
@ stub -version=0xA00+ NtUserGetHimetricScaleFactorFromPixelLocation
@ stub -version=0xA00+ NtUserGetIMEShowStatus
@ stub -version=0xA00+ NtUserGetInputContainerId
@ stub -version=0xA00+ NtUserGetInputDesktop
@ stub -version=0xA00+ NtUserGetInputEvent
@ stub -version=0xA00+ NtUserGetInputLocaleInfo
@ stub -version=0xA00+ NtUserGetInteractiveControlDeviceInfo
@ stub -version=0xA00+ NtUserGetInteractiveControlInfo
@ stub -version=0xA00+ NtUserGetInteractiveCtrlSupportedWaveforms
@ stub -version=0xA00+ NtUserGetKeyboardInputThreadId
@ stub -version=0xA00+ NtUserGetKeyboardLayout
@ stub -version=0xA00+ NtUserGetKeyboardType
@ stub -version=0xA00+ NtUserGetLatestInputMessageData
@ stub -version=0xA00+ NtUserGetMessagePos
@ stub -version=0xA00+ NtUserGetMinuserIdForBaseWindow
@ stub -version=0xA00+ NtUserGetModernAppWindow
@ stub -version=0xA00+ NtUserGetOemBitmapSize
@ stub -version=0xA00+ NtUserGetOwnerTransformedMonitorRect
@ stub -version=0xA00+ NtUserGetPhysicalDeviceRect
@ stub -version=0xA00+ NtUserGetPointerCursorId
@ stub -version=0xA00+ NtUserGetPointerDevice
@ stub -version=0xA00+ NtUserGetPointerDeviceCursors
@ stub -version=0xA00+ NtUserGetPointerDeviceInputSpace
@ stub -version=0xA00+ NtUserGetPointerDeviceOrientation
@ stub -version=0xA00+ NtUserGetPointerDeviceProperties
@ stub -version=0xA00+ NtUserGetPointerDeviceRects
@ stub -version=0xA00+ NtUserGetPointerDevices
@ stub -version=0xA00+ NtUserGetPointerFrameTimes
@ stub -version=0xA00+ NtUserGetPointerIdForPromotion
@ stub -version=0xA00+ NtUserGetPointerInfoList
@ stub -version=0xA00+ NtUserGetPointerInputTransform
@ stub -version=0xA00+ NtUserGetPointerProprietaryId
@ stub -version=0xA00+ NtUserGetPointerType
@ stub -version=0xA00+ NtUserGetPrecisionTouchPadConfiguration
@ stub -version=0xA00+ NtUserGetProcessDefaultLayout
@ stub -version=0xA00+ NtUserGetProcessDpiAwarenessContext
@ stub -version=0xA00+ NtUserGetProcessUIContextInformation
@ stub -version=0xA00+ NtUserGetProp
@ stub -version=0xA00+ NtUserGetProp2
@ stub -version=0xA00+ NtUserGetQueueIocp
@ stub -version=0xA00+ NtUserGetQueueStatus
@ stub -version=0xA00+ NtUserGetQueueStatusReadonly
@ stub -version=0xA00+ NtUserGetRawPointerDeviceData
@ stub -version=0xA00+ NtUserGetRequiredCursorSizes
@ stub -version=0xA00+ NtUserGetResizeDCompositionSynchronizationObject
@ stub -version=0xA00+ NtUserGetSendMessageReceiver
@ stub -version=0xA00+ NtUserGetSharedWindowData
@ stub -version=0xA00+ NtUserGetSuppressedWindowActions
@ stub -version=0xA00+ NtUserGetSysMenuOffset
@ stub -version=0xA00+ NtUserGetSystemContentRects
@ stub -version=0xA00+ NtUserGetSystemDpiForProcess
@ stub -version=0xA00+ NtUserGetTopLevelWindow
@ stub -version=0xA00+ NtUserGetTouchInputInfo
@ stub -version=0xA00+ NtUserGetTouchValidationStatus
@ stub -version=0xA00+ NtUserGetUniformSpaceMapping
@ stub -version=0xA00+ NtUserGetUnpredictedMessagePos
@ stub -version=0xA00+ NtUserGetUpdatedClipboardFormats
@ stub -version=0xA00+ NtUserGetWinStationInfo
@ stub -version=0xA00+ NtUserGetWindowBand
@ stub -version=0xA00+ NtUserGetWindowCompositionAttribute
@ stub -version=0xA00+ NtUserGetWindowCompositionInfo
@ stub -version=0xA00+ NtUserGetWindowContextHelpId
@ stub -version=0xA00+ NtUserGetWindowDisplayAffinity
@ stub -version=0xA00+ NtUserGetWindowFeedbackSetting
@ stub -version=0xA00+ NtUserGetWindowMinimizeRect
@ stub -version=0xA00+ NtUserGetWindowProcessHandle
@ stub -version=0xA00+ NtUserGetWindowRgnEx
@ stub -version=0xA00+ NtUserGetWindowThreadProcessId
@ stub -version=0xA00+ NtUserGetWindowTrackInfoAsync
@ stub -version=0xA00+ NtUserGhostWindowFromHungWindow
@ stub -version=0xA00+ NtUserGrantJobUIRestrictionException
@ stub -version=0xA00+ NtUserHandleDelegatedInput
@ stub -version=0xA00+ NtUserHandleSystemThreadCreationFailure
@ stub -version=0xA00+ NtUserHideCursorNoCapture
@ stub -version=0xA00+ NtUserHidePointerContactVisualization
@ stub -version=0xA00+ NtUserHungWindowFromGhostWindow
@ stub -version=0xA00+ NtUserHwndQueryRedirectionInfo
@ stub -version=0xA00+ NtUserHwndSetRedirectionInfo
@ stub -version=0xA00+ NtUserInheritWindowMonitor
@ stub -version=0xA00+ NtUserInitAnsiOem
@ stub -version=0xA00+ NtUserInitTask
@ stub -version=0xA00+ NtUserInitThreadCoreMessagingIocp
@ stub -version=0xA00+ NtUserInitThreadCoreMessagingIocp2
@ stub -version=0xA00+ NtUserInitializeGenericHidInjection
@ stub -version=0xA00+ NtUserInitializeInputDeviceInjection
@ stub -version=0xA00+ NtUserInitializePointerDeviceInjection
@ stub -version=0xA00+ NtUserInitializePointerDeviceInjectionEx
@ stub -version=0xA00+ NtUserInitializeTouchInjection
@ stub -version=0xA00+ NtUserInitializeUserModeMenus
@ stub -version=0xA00+ NtUserInjectDeviceInput
@ stub -version=0xA00+ NtUserInjectGenericHidInput
@ stub -version=0xA00+ NtUserInjectGesture
@ stub -version=0xA00+ NtUserInjectKeyboardInput
@ stub -version=0xA00+ NtUserInjectMouseInput
@ stub -version=0xA00+ NtUserInjectPointerInput
@ stub -version=0xA00+ NtUserInjectTouchInput
@ stub -version=0xA00+ NtUserInjectTouchpadAction
@ stub -version=0xA00+ NtUserInteractiveControlQueryUsage
@ stub -version=0xA00+ NtUserInternalGetWindowIcon
@ stub -version=0xA00+ NtUserInternalStartMoveSize
@ stub -version=0xA00+ NtUserInternalToUnicode
@ stub -version=0xA00+ NtUserIsChildWindowDpiMessageEnabled
@ stub -version=0xA00+ NtUserIsInterceptWindow
@ stub -version=0xA00+ NtUserIsMouseInPointerEnabled
@ stub -version=0xA00+ NtUserIsMouseInputEnabled
@ stub -version=0xA00+ NtUserIsNonClientDpiScalingEnabled
@ stub -version=0xA00+ NtUserIsQueueAttached
@ stub -version=0xA00+ NtUserIsResizeLayoutSynchronizationEnabled
@ stub -version=0xA00+ NtUserIsTopLevelWindow
@ stub -version=0xA00+ NtUserIsTouchWindow
@ stub -version=0xA00+ NtUserIsUtilitySession
@ stub -version=0xA00+ NtUserIsWindowBroadcastingDpiToChildren
@ stub -version=0xA00+ NtUserIsWindowDisplayChangeSuppressed
@ stub -version=0xA00+ NtUserIsWindowGDIScaledDpiMessageEnabled
@ stub -version=0xA00+ NtUserKillSystemTimer
@ stub -version=0xA00+ NtUserLW_LoadFonts
@ stub -version=0xA00+ NtUserLayoutCompleted
@ stub -version=0xA00+ NtUserLinkDpiCursor
@ stub -version=0xA00+ NtUserLoadCursorsAndIcons
@ stub -version=0xA00+ NtUserLoadUserApiHook
@ stub -version=0xA00+ NtUserLockCursor
@ stub -version=0xA00+ NtUserLockSetForegroundWindow
@ stub -version=0xA00+ NtUserLogicalToPerMonitorDPIPhysicalPoint
@ stub -version=0xA00+ NtUserLogicalToPhysicalDpiPointForWindow
@ stub -version=0xA00+ NtUserLogicalToPhysicalPoint
@ stub -version=0xA00+ NtUserMagControl
@ stub -version=0xA00+ NtUserMagGetContextInformation
@ stub -version=0xA00+ NtUserMagSetContextInformation
@ stub -version=0xA00+ NtUserMapDesktopObject
@ stub -version=0xA00+ NtUserMapPointsByVisualIdentifier
@ stub -version=0xA00+ NtUserMarkWindowForRawMouse
@ stub -version=0xA00+ NtUserMessageBeep
@ stub -version=0xA00+ NtUserMinInitialize
@ stub -version=0xA00+ NtUserModifyWindowTouchCapability
@ stub -version=0xA00+ NtUserMsgWaitForMultipleObjectsEx
@ stub -version=0xA00+ NtUserNavigateFocus
@ stub -version=0xA00+ NtUserNlsKbdSendIMENotification
@ stub -version=0xA00+ NtUserNotifyOverlayWindow
@ stub -version=0xA00+ NtUserOpenThreadDesktop
@ stub -version=0xA00+ NtUserPaintMonitor
@ stub -version=0xA00+ NtUserPerMonitorDPIPhysicalToLogicalPoint
@ stub -version=0xA00+ NtUserPhysicalToLogicalDpiPointForWindow
@ stub -version=0xA00+ NtUserPhysicalToLogicalPoint
@ stub -version=0xA00+ NtUserPlayEventSound
@ stub -version=0xA00+ NtUserPostKeyboardInputMessage
@ stub -version=0xA00+ NtUserPostQuitMessage
@ stub -version=0xA00+ NtUserPrepareForLogoff
@ stub -version=0xA00+ NtUserProcessInkFeedbackCommand
@ stub -version=0xA00+ NtUserPromoteMouseInPointer
@ stub -version=0xA00+ NtUserPromotePointer
@ stub -version=0xA00+ NtUserPromotePrimaryPointerToDrag
@ stub -version=0xA00+ NtUserQueryBSDRWindow
@ stub -version=0xA00+ NtUserQueryDisplayConfig
@ stub -version=0xA00+ NtUserRaiseLowerShellWindow
@ stub -version=0xA00+ NtUserRealizePalette
@ stub -version=0xA00+ NtUserReassociateQueueEventCompletionPacket
@ stub -version=0xA00+ NtUserRedrawFrame
@ stub -version=0xA00+ NtUserRedrawFrameAndHook
@ stub -version=0xA00+ NtUserRedrawTitle
@ stub -version=0xA00+ NtUserRegisterBSDRWindow
@ stub -version=0xA00+ NtUserRegisterCloakedNotification
@ stub -version=0xA00+ NtUserRegisterCoreMessagingEndPoint
@ stub -version=0xA00+ NtUserRegisterDManipHook
@ stub -version=0xA00+ NtUserRegisterEdgy
@ stub -version=0xA00+ NtUserRegisterErrorReportingDialog
@ stub -version=0xA00+ NtUserRegisterForCustomDockTargets
@ stub -version=0xA00+ NtUserRegisterForTooltipDismissNotification
@ stub -version=0xA00+ NtUserRegisterGhostWindow
@ stub -version=0xA00+ NtUserRegisterLPK
@ stub -version=0xA00+ NtUserRegisterLogonProcess
@ stub -version=0xA00+ NtUserRegisterManipulationThread
@ stub -version=0xA00+ NtUserRegisterPointerDeviceNotifications
@ stub -version=0xA00+ NtUserRegisterPointerInputTarget
@ stub -version=0xA00+ NtUserRegisterPrecisionTouchpadWindow
@ stub -version=0xA00+ NtUserRegisterServicesProcess
@ stub -version=0xA00+ NtUserRegisterSessionPort
@ stub -version=0xA00+ NtUserRegisterShellHookWindow
@ stub -version=0xA00+ NtUserRegisterShellPTPListener
@ stub -version=0xA00+ NtUserRegisterSiblingFrostWindow
@ stub -version=0xA00+ NtUserRegisterSystemThread
@ stub -version=0xA00+ NtUserRegisterTouchHitTestingWindow
@ stub -version=0xA00+ NtUserRegisterTouchPadCapable
@ stub -version=0xA00+ NtUserRegisterTouchpadCapableWindow
@ stub -version=0xA00+ NtUserRegisterUserHungAppHandlers
@ stub -version=0xA00+ NtUserRegisterWindowArrangementCallout
@ stub -version=0xA00+ NtUserReleaseCapture
@ stub -version=0xA00+ NtUserReleaseDC
@ stub -version=0xA00+ NtUserReleaseDwmHitTestWaiters
@ stub -version=0xA00+ NtUserRemoteConnectState
@ stub -version=0xA00+ NtUserRemoteConsoleShadowStop
@ stub -version=0xA00+ NtUserRemoteDisconnect
@ stub -version=0xA00+ NtUserRemoteNotify
@ stub -version=0xA00+ NtUserRemotePassthruDisable
@ stub -version=0xA00+ NtUserRemotePassthruEnable
@ stub -version=0xA00+ NtUserRemoteReconnect
@ stub -version=0xA00+ NtUserRemoteShadowCleanup
@ stub -version=0xA00+ NtUserRemoteShadowSetup
@ stub -version=0xA00+ NtUserRemoteShadowStart
@ stub -version=0xA00+ NtUserRemoteShadowStop
@ stub -version=0xA00+ NtUserRemoteThinwireStats
@ stub -version=0xA00+ NtUserRemoveClipboardFormatListener
@ stub -version=0xA00+ NtUserRemoveInjectionDevice
@ stub -version=0xA00+ NtUserRemoveQueueCompletion
@ stub -version=0xA00+ NtUserRemoveVisualIdentifier
@ stub -version=0xA00+ NtUserReplyMessage
@ stub -version=0xA00+ NtUserReportInertia
@ stub -version=0xA00+ NtUserRequestLowLatencyProfile
@ stub -version=0xA00+ NtUserResetDblClk
@ stub -version=0xA00+ NtUserRestoreWindowDpiChanges
@ stub -version=0xA00+ NtUserScaleSystemMetricForDPIWithoutCache
@ stub -version=0xA00+ NtUserScheduleDispatchNotification
@ stub -version=0xA00+ NtUserSendEventMessage
@ stub -version=0xA00+ NtUserSendInteractiveControlHapticsReport
@ stub -version=0xA00+ NtUserSetActivationFilter
@ stub -version=0xA00+ NtUserSetActiveProcessForMonitor
@ stub -version=0xA00+ NtUserSetAdditionalForegroundBoostProcesses
@ stub -version=0xA00+ NtUserSetAdditionalPowerThrottlingProcess
@ stub -version=0xA00+ NtUserSetAutoRotation
@ stub -version=0xA00+ NtUserSetBridgeWindowChild
@ stub -version=0xA00+ NtUserSetBrokeredForeground
@ stub -version=0xA00+ NtUserSetCalibrationData
@ stub -version=0xA00+ NtUserSetCancelRotationDelayHintWindow
@ stub -version=0xA00+ NtUserSetCaretBlinkTime
@ stub -version=0xA00+ NtUserSetCaretPos
@ stub -version=0xA00+ NtUserSetChildWindowNoActivate
@ stub -version=0xA00+ NtUserSetCoreWindow
@ stub -version=0xA00+ NtUserSetCoreWindowPartner
@ stub -version=0xA00+ NtUserSetCoveredWindowStates
@ stub -version=0xA00+ NtUserSetCursorIconDataEx
@ stub -version=0xA00+ NtUserSetCursorPos
@ stub -version=0xA00+ NtUserSetDesktopColorTransform
@ stub -version=0xA00+ NtUserSetDesktopVisualInputSink
@ stub -version=0xA00+ NtUserSetDialogControlDpiChangeBehavior
@ stub -version=0xA00+ NtUserSetDialogPointer
@ stub -version=0xA00+ NtUserSetDialogSystemMenu
@ stub -version=0xA00+ NtUserSetDisplayAutoRotationPreferences
@ stub -version=0xA00+ NtUserSetDisplayConfig
@ stub -version=0xA00+ NtUserSetDisplayMapping
@ stub -version=0xA00+ NtUserSetDoubleClickTime
@ stub -version=0xA00+ NtUserSetDpiForWindow
@ stub -version=0xA00+ NtUserSetFallbackForeground
@ stub -version=0xA00+ NtUserSetFeatureReportResponse
@ stub -version=0xA00+ NtUserSetForegroundRedirectionForActivationObject
@ stub -version=0xA00+ NtUserSetForegroundWindow
@ stub -version=0xA00+ NtUserSetForegroundWindowForApplication
@ stub -version=0xA00+ NtUserSetFullscreenMagnifierOffsetsDWMUpdated
@ stub -version=0xA00+ NtUserSetGUISystemInformation
@ stub -version=0xA00+ NtUserSetGestureConfig
@ stub -version=0xA00+ NtUserSetInputServiceState
@ stub -version=0xA00+ NtUserSetInteractiveControlFocus
@ stub -version=0xA00+ NtUserSetInteractiveCtrlRotationAngle
@ stub -version=0xA00+ NtUserSetJobUILimits
@ stub -version=0xA00+ NtUserSetMagnificationDesktopMagnifierOffsetsDWMUpdated
@ stub -version=0xA00+ NtUserSetManipulationInputTarget
@ stub -version=0xA00+ NtUserSetMaxTouchpadSensitivity
@ stub -version=0xA00+ NtUserSetMessageExtraInfo
@ stub -version=0xA00+ NtUserSetMirrorRendering
@ stub -version=0xA00+ NtUserSetModernAppWindow
@ stub -version=0xA00+ NtUserSetMonitorWorkArea
@ stub -version=0xA00+ NtUserSetMsgBox
@ stub -version=0xA00+ NtUserSetPrecisionTouchPadConfiguration
@ stub -version=0xA00+ NtUserSetProcessDefaultLayout
@ stub -version=0xA00+ NtUserSetProcessDpiAwarenessContext
@ stub -version=0xA00+ NtUserSetProcessInteractionFlags
@ stub -version=0xA00+ NtUserSetProcessLaunchForegroundPolicy
@ stub -version=0xA00+ NtUserSetProcessMousewheelRoutingMode
@ stub -version=0xA00+ NtUserSetProcessRestrictionExemption
@ stub -version=0xA00+ NtUserSetProcessUIAccessZorder
@ stub -version=0xA00+ NtUserSetProcessWin32Capabilities
@ stub -version=0xA00+ NtUserSetProgmanWindow
@ stub -version=0xA00+ NtUserSetProp2
@ stub -version=0xA00+ NtUserSetSensorPresence
@ stub -version=0xA00+ NtUserSetSharedWindowData
@ stub -version=0xA00+ NtUserSetShellChangeNotifyHWND
@ stub -version=0xA00+ NtUserSetShellSpecialWindow
@ stub -version=0xA00+ NtUserSetSysMenu
@ stub -version=0xA00+ NtUserSetSystemContentRects
@ stub -version=0xA00+ NtUserSetTSFEventState
@ stub -version=0xA00+ NtUserSetTargetForResourceBrokering
@ stub -version=0xA00+ NtUserSetTaskmanWindow
@ stub -version=0xA00+ NtUserSetThreadInputBlocked
@ stub -version=0xA00+ NtUserSetThreadQueueMergeSetting
@ stub -version=0xA00+ NtUserSetUserObjectCapability
@ stub -version=0xA00+ NtUserSetVisible
@ stub -version=0xA00+ NtUserSetWaitForQueueAttach
@ stub -version=0xA00+ NtUserSetWatermarkStrings
@ stub -version=0xA00+ NtUserSetWindowArrangement
@ stub -version=0xA00+ NtUserSetWindowBand
@ stub -version=0xA00+ NtUserSetWindowCompositionAttribute
@ stub -version=0xA00+ NtUserSetWindowCompositionTransition
@ stub -version=0xA00+ NtUserSetWindowContextHelpId
@ stub -version=0xA00+ NtUserSetWindowDisplayAffinity
@ stub -version=0xA00+ NtUserSetWindowFeedbackSetting
@ stub -version=0xA00+ NtUserSetWindowMessageCapability
@ stub -version=0xA00+ NtUserSetWindowRgnEx
@ stub -version=0xA00+ NtUserSetWindowShowState
@ stub -version=0xA00+ NtUserSetWindowState
@ stub -version=0xA00+ NtUserShellForegroundBoostProcess
@ stub -version=0xA00+ NtUserShellHandwritingDelegateInput
@ stub -version=0xA00+ NtUserShellHandwritingHandleDelegatedInput
@ stub -version=0xA00+ NtUserShellHandwritingUndelegateInput
@ stub -version=0xA00+ NtUserShellMigrateWindow
@ stub -version=0xA00+ NtUserShellRegisterHotKey
@ stub -version=0xA00+ NtUserShellSetWindowPos
@ stub -version=0xA00+ NtUserShowCursor
@ stub -version=0xA00+ NtUserShowOwnedPopups
@ stub -version=0xA00+ NtUserShowStartGlass
@ stub -version=0xA00+ NtUserShowSystemCursor
@ stub -version=0xA00+ NtUserShutdownBlockReasonCreate
@ stub -version=0xA00+ NtUserShutdownBlockReasonQuery
@ stub -version=0xA00+ NtUserShutdownReasonDestroy
@ stub -version=0xA00+ NtUserSignalRedirectionStartComplete
@ stub -version=0xA00+ NtUserSlicerControl
@ stub -version=0xA00+ NtUserStopAndEndInertia
@ stub -version=0xA00+ NtUserSuppressWindowActions
@ stub -version=0xA00+ NtUserSuppressWindowDisplayChange
@ stub -version=0xA00+ NtUserSwapMouseButton
@ stub -version=0xA00+ NtUserSwitchToThisWindow
@ stub -version=0xA00+ NtUserSystemParametersInfoForDpi
@ stub -version=0xA00+ NtUserThreadMessageQueueAttached
@ stub -version=0xA00+ NtUserTraceLoggingSendMixedModeTelemetry
@ stub -version=0xA00+ NtUserTransformPoint
@ stub -version=0xA00+ NtUserTransformRect
@ stub -version=0xA00+ NtUserUndelegateInput
@ stub -version=0xA00+ NtUserUnhookWindowsHook
@ stub -version=0xA00+ NtUserUnregisterSessionPort
@ stub -version=0xA00+ NtUserUpdateClientRect
@ stub -version=0xA00+ NtUserUpdateDefaultDesktopThumbnail
@ stub -version=0xA00+ NtUserUpdatePerUserImmEnabling
@ stub -version=0xA00+ NtUserUpdateWindow
@ stub -version=0xA00+ NtUserUpdateWindowInputSinkHints
@ stub -version=0xA00+ NtUserUpdateWindowTrackingInfo
@ stub -version=0xA00+ NtUserUpdateWindows
@ stub -version=0xA00+ NtUserUserPowerCalloutWorker
@ stub -version=0xA00+ NtUserValidateHandleSecure
@ stub -version=0xA00+ NtUserValidateRgn
@ stub -version=0xA00+ NtUserValidateTimerCallback
@ stub -version=0xA00+ NtUserWOWCleanup
@ stub -version=0xA00+ NtUserWOWModuleUnload
@ stub -version=0xA00+ NtUserWaitAvailableMessageEx
@ stub -version=0xA00+ NtUserWaitForRedirectionStartComplete
@ stub -version=0xA00+ NtUserWakeRITForShutdown
@ stub -version=0xA00+ NtUserWindowFromDC
@ stub -version=0xA00+ NtUserWindowFromPhysicalPoint
@ stub -version=0xA00+ NtUserZapActiveAndFocus
@ stub -version=0xA00+ NtValidateCompositionSurfaceHandle
@ stub -version=0xA00+ NtVisualCaptureBits
@ stub -arch=win32 -version=0xA00+ _ABI_Get_ForegroundWindow@4
@ stub -arch=win32 -version=0xA00+ _ABI_Get_apfnClientWorkerNtDll@4
@ stub -arch=win32 -version=0xA00+ _ABI_Get_currentMonitorTopologyId@4
@ stub -arch=win32 -version=0xA00+ _ABI_Get_tooltipStrings@4
@ stub -version=0xA00+ gDispatchTableValues
# Data exports (need an `extern`, not a code stub):
#   Wow64Transition
# END win10-surface stubs
