use crate::prelude::*;
use core::ffi::c_void;

#[no_mangle]
pub unsafe extern "system" fn WinStationFreeMemory(A: PVOID) {
    // FIXME("WinStationFreeMemory %p not freed!\n",A);
}

#[no_mangle]
pub unsafe extern "system" fn WinStationAutoReconnect(A: PVOID) {
    // UNIMPLEMENTED;
}

#[no_mangle]
pub unsafe extern "system" fn WinStationBroadcastSystemMessage(
    A: PVOID,
    B: PVOID,
    C: PVOID,
    D: PVOID,
    E: PVOID,
    F: PVOID,
    G: PVOID,
    H: PVOID,
    I: PVOID,
    J: PVOID,
) {
    // UNIMPLEMENTED;
}

#[no_mangle]
pub unsafe extern "system" fn WinStationCheckAccess(A: PVOID, B: PVOID, C: PVOID) {
    // UNIMPLEMENTED;
}

#[no_mangle]
pub unsafe extern "system" fn WinStationCheckLoopBack(A: PVOID, B: PVOID, C: PVOID, D: PVOID) {
    // UNIMPLEMENTED;
}

#[no_mangle]
pub unsafe extern "system" fn WinStationConnectA(A: PVOID) {
    // UNIMPLEMENTED;
}

#[no_mangle]
pub unsafe extern "system" fn WinStationConnectCallback(
    A: PVOID,
    B: PVOID,
    C: PVOID,
    D: PVOID,
    E: PVOID,
) {
    // UNIMPLEMENTED;
}

#[no_mangle]
pub unsafe extern "system" fn WinStationConnectEx(A: PVOID, B: PVOID) {
    // UNIMPLEMENTED;
}

#[no_mangle]
pub unsafe extern "system" fn WinStationConnectW(A: PVOID, B: PVOID, C: PVOID, D: PVOID, E: PVOID) {
    // UNIMPLEMENTED;
}

#[no_mangle]
pub unsafe extern "system" fn WinStationFreeGAPMemory(A: PVOID, B: PVOID, C: PVOID) {
    // UNIMPLEMENTED;
}

#[no_mangle]
pub unsafe extern "system" fn WinStationIsHelpAssistantSession(A: PVOID, B: PVOID) {
    // UNIMPLEMENTED;
}

#[no_mangle]
pub unsafe extern "system" fn WinStationNtsdDebug(A: PVOID, B: PVOID, C: PVOID, D: PVOID, E: PVOID) {
    // UNIMPLEMENTED;
}

#[no_mangle]
pub unsafe extern "system" fn WinStationRenameA(A: PVOID, B: PVOID, C: PVOID) {
    // UNIMPLEMENTED;
}

#[no_mangle]
pub unsafe extern "system" fn WinStationRenameW(A: PVOID, B: PVOID, C: PVOID) {
    // UNIMPLEMENTED;
}

#[no_mangle]
pub unsafe extern "system" fn WinStationSendMessageA(
    A: PVOID,
    B: PVOID,
    C: PVOID,
    D: PVOID,
    E: PVOID,
    F: PVOID,
    G: PVOID,
    H: PVOID,
    I: PVOID,
    J: PVOID,
) {
    // UNIMPLEMENTED;
}

#[no_mangle]
pub unsafe extern "system" fn WinStationSendMessageW(
    A: PVOID,
    B: PVOID,
    C: PVOID,
    D: PVOID,
    E: PVOID,
    F: PVOID,
    G: PVOID,
    H: PVOID,
    I: PVOID,
    J: PVOID,
) {
    // UNIMPLEMENTED;
}

#[no_mangle]
pub unsafe extern "system" fn WinStationSendWindowMessage(
    A: PVOID,
    B: PVOID,
    C: PVOID,
    D: PVOID,
    E: PVOID,
    F: PVOID,
    G: PVOID,
    H: PVOID,
) {
    // UNIMPLEMENTED;
}

#[no_mangle]
pub unsafe extern "system" fn WinStationSetInformationA(
    A: PVOID,
    B: PVOID,
    C: PVOID,
    D: PVOID,
    E: PVOID,
) {
    // UNIMPLEMENTED;
}

#[no_mangle]
pub unsafe extern "system" fn WinStationSetInformationW(
    A: PVOID,
    B: PVOID,
    C: PVOID,
    D: PVOID,
    E: PVOID,
) {
    // UNIMPLEMENTED;
}

#[no_mangle]
pub unsafe extern "system" fn WinStationSetPoolCount(A: PVOID, B: PVOID, C: PVOID) {
    // UNIMPLEMENTED;
}

#[no_mangle]
pub unsafe extern "system" fn WinStationShadow(A: PVOID, B: PVOID, C: PVOID, D: PVOID, E: PVOID) {
    // UNIMPLEMENTED;
}

#[no_mangle]
pub unsafe extern "system" fn WinStationShadowStop(A: PVOID, B: PVOID, C: PVOID) {
    // UNIMPLEMENTED;
}

#[no_mangle]
pub unsafe extern "system" fn WinStationShutdownSystem(A: PVOID, B: PVOID) {
    // UNIMPLEMENTED;
}

#[no_mangle]
pub unsafe extern "system" fn WinStationSystemShutdownStarted() {
    // UNIMPLEMENTED;
}

#[no_mangle]
pub unsafe extern "system" fn WinStationSystemShutdownWait(A: PVOID, B: PVOID) {
    // UNIMPLEMENTED;
}

#[no_mangle]
pub unsafe extern "system" fn WinStationTerminateProcess(A: PVOID, B: PVOID, C: PVOID) {
    // UNIMPLEMENTED;
}

#[no_mangle]
pub unsafe extern "system" fn WinStationUnRegisterConsoleNotification(A: PVOID, B: PVOID) {
    // UNIMPLEMENTED;
}

#[no_mangle]
pub unsafe extern "system" fn WinStationVirtualOpen(A: PVOID, B: PVOID, C: PVOID) {
    // UNIMPLEMENTED;
}

#[no_mangle]
pub unsafe extern "system" fn WinStationVirtualOpenEx(A: PVOID, B: PVOID, C: PVOID, D: PVOID) {
    // UNIMPLEMENTED;
}

#[no_mangle]
pub unsafe extern "system" fn WinStationWaitSystemEvent(A: PVOID, B: PVOID, C: PVOID) {
    // UNIMPLEMENTED;
}

#[no_mangle]
pub unsafe extern "system" fn WinStationDynVirtualChanRead(
    A: PVOID,
    B: PVOID,
    C: PVOID,
    D: PVOID,
    E: PVOID,
) {
    // UNIMPLEMENTED;
}

#[no_mangle]
pub unsafe extern "system" fn WinStationDynVirtualChanWrite(A: PVOID, B: PVOID, C: PVOID, D: PVOID) {
    // UNIMPLEMENTED;
}

#[no_mangle]
pub unsafe extern "system" fn WinStationRegisterConsoleNotificationEx(
    A: PVOID,
    B: PVOID,
    C: PVOID,
    D: PVOID,
) {
    // UNIMPLEMENTED;
}

#[no_mangle]
pub unsafe extern "system" fn WinStationRegisterConsoleNotification(A: PVOID, B: PVOID, C: PVOID) {
    // UNIMPLEMENTED;
}

#[no_mangle]
pub unsafe extern "system" fn WinStationRegisterNotificationEvent(
    A: PVOID,
    B: PVOID,
    C: PVOID,
    D: PVOID,
) {
    // UNIMPLEMENTED;
}

#[no_mangle]
pub unsafe extern "system" fn WinStationUnRegisterNotificationEvent(A: PVOID) {
    // UNIMPLEMENTED;
}

#[no_mangle]
pub unsafe extern "system" fn WinStationSwitchToServicesSession() {
    // UNIMPLEMENTED;
}

#[no_mangle]
pub unsafe extern "system" fn _WinStationAnnoyancePopup(A: PVOID, B: PVOID) {
    // UNIMPLEMENTED;
}

#[no_mangle]
pub unsafe extern "system" fn _WinStationBeepOpen(A: PVOID, B: PVOID, C: PVOID) {
    // UNIMPLEMENTED;
}

#[no_mangle]
pub unsafe extern "system" fn _WinStationBreakPoint(A: PVOID, B: PVOID, C: PVOID) {
    // UNIMPLEMENTED;
}

#[no_mangle]
pub unsafe extern "system" fn _WinStationSessionInitialized() {
    // UNIMPLEMENTED;
}

#[no_mangle]
pub unsafe extern "system" fn _WinStationCallback(A: PVOID, B: PVOID, C: PVOID) {
    // UNIMPLEMENTED;
}

#[no_mangle]
pub unsafe extern "system" fn _WinStationCheckForApplicationName(
    A: PVOID,
    B: PVOID,
    C: PVOID,
    D: PVOID,
    E: PVOID,
    F: PVOID,
    G: PVOID,
    H: PVOID,
    I: PVOID,
    J: PVOID,
    K: PVOID,
    M: PVOID,
) {
    // UNIMPLEMENTED;
}

#[no_mangle]
pub unsafe extern "system" fn _WinStationFUSCanRemoteUserDisconnect(A: PVOID, B: PVOID, C: PVOID) {
    // UNIMPLEMENTED;
}

#[no_mangle]
pub unsafe extern "system" fn _WinStationNotifyDisconnectPipe() {
    // UNIMPLEMENTED;
}

#[no_mangle]
pub unsafe extern "system" fn _WinStationNotifyNewSession(A: PVOID, B: PVOID) {
    // UNIMPLEMENTED;
}

#[no_mangle]
pub unsafe extern "system" fn _WinStationReInitializeSecurity(A: PVOID) {
    // UNIMPLEMENTED;
}

#[no_mangle]
pub unsafe extern "system" fn _WinStationReadRegistry(A: PVOID) {
    // UNIMPLEMENTED;
}

#[no_mangle]
pub unsafe extern "system" fn _WinStationShadowTargetSetup(A: PVOID, B: PVOID) {
    // UNIMPLEMENTED;
}

#[no_mangle]
pub unsafe extern "system" fn _WinStationUpdateSettings(A: PVOID, B: PVOID, C: PVOID) {
    // UNIMPLEMENTED;
}

#[no_mangle]
pub unsafe extern "system" fn _WinStationUpdateUserConfig(A: PVOID) {
    // UNIMPLEMENTED;
}

#[no_mangle]
pub unsafe extern "system" fn _WinStationUpdateClientCachedCredentials(
    A: PVOID,
    B: PVOID,
    C: PVOID,
    D: PVOID,
    E: PVOID,
    F: PVOID,
    G: PVOID,
) {
    // UNIMPLEMENTED;
}
