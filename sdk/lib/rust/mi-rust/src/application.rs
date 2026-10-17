//! MI_Application and mi.dll's two exports: `MI_Application_InitializeV1` and the data export
//! `mi_clientFT_V1` (a pointer to the client function table, through which a client such as
//! the CIM .NET layer reaches everything else; Docker's copy imports nothing but it).

use crate::instance::{Elem, Inst};
use crate::options::{self, DESTINATION_OPTIONS_FT, OPERATION_OPTIONS_FT};
use crate::session;
use crate::types::*;
use crate::value::*;
use crate::wstr::*;
use core::ptr::null_mut;

const APP_MAGIC: u64 = 0x4d49_4170_706c_6963; // "MIApplic"

unsafe fn valid(app: *const MI_Application) -> bool {
    !app.is_null() && (*app).reserved1 == APP_MAGIC
}

#[no_mangle]
pub unsafe extern "C" fn MI_Application_InitializeV1(
    flags: MI_Uint32,
    _application_id: *const MI_Char,
    extended_error: *mut *mut MI_Instance,
    application: *mut MI_Application,
) -> MI_Result {
    if !extended_error.is_null() {
        *extended_error = null_mut();
    }
    if application.is_null() || flags != 0 {
        return MI_RESULT_INVALID_PARAMETER;
    }
    (*application).reserved1 = APP_MAGIC;
    (*application).reserved2 = 0;
    (*application).ft = &APPLICATION_FT;
    MI_RESULT_OK
}

unsafe extern "system" fn a_close(app: *mut MI_Application) -> MI_Result {
    if !valid(app) {
        return MI_RESULT_INVALID_PARAMETER;
    }
    (*app).reserved1 = 0;
    MI_RESULT_OK
}

unsafe extern "system" fn a_new_session(
    app: *mut MI_Application,
    _protocol: *const MI_Char,
    destination: *const MI_Char,
    _options: *mut MI_DestinationOptions,
    _callbacks: *mut MI_SessionCallbacks,
    extended_error: *mut *mut MI_Instance,
    session: *mut MI_Session,
) -> MI_Result {
    if !extended_error.is_null() {
        *extended_error = null_mut();
    }
    if !valid(app) || session.is_null() {
        return MI_RESULT_INVALID_PARAMETER;
    }
    if !session::is_local(destination) {
        // The backend is this machine's wbemprox; there is no transport to a remote one.
        return MI_RESULT_NOT_SUPPORTED;
    }
    session::init(session, *app);
    MI_RESULT_OK
}

unsafe extern "system" fn a_new_hosted_provider(
    _app: *mut MI_Application,
    _ns: *const MI_Char,
    _provider: *const MI_Char,
    _main: MI_MainFunction,
    extended_error: *mut *mut MI_Instance,
    _out: *mut MI_HostedProvider,
) -> MI_Result {
    if !extended_error.is_null() {
        *extended_error = null_mut();
    }
    MI_RESULT_NOT_SUPPORTED
}

/// A new instance, with one null element per property when a class declaration is given.
unsafe fn new_instance(class: *const MI_Char, decl: *const MI_ClassDecl, out: *mut *mut MI_Instance) -> MI_Result {
    if out.is_null() {
        return MI_RESULT_INVALID_PARAMETER;
    }
    *out = null_mut();
    let name = match (wdup(class), decl.is_null()) {
        (Some(n), _) => n,
        (None, false) if !(*decl).name.is_null() => wdup((*decl).name).unwrap(),
        _ => return MI_RESULT_INVALID_PARAMETER,
    };
    let mut inst = Inst::new(name);
    if !decl.is_null() && !(*decl).properties.is_null() {
        for k in 0..(*decl).numProperties as usize {
            let p = *(*decl).properties.add(k);
            if p.is_null() {
                continue;
            }
            let Some(pname) = wdup((*p).name) else { continue };
            let ty = (*p).type_ as MI_Type;
            if !valid_type(ty) {
                return MI_RESULT_INVALID_PARAMETER;
            }
            inst.elems.push(Elem { name: pname, flags: (*p).flags & FLAG_KEY, value: Owned::null(ty) });
        }
    }
    inst.sync();
    *out = inst.into_mi();
    MI_RESULT_OK
}

unsafe extern "system" fn a_new_instance(
    app: *mut MI_Application,
    class: *const MI_Char,
    decl: *const MI_ClassDecl,
    out: *mut *mut MI_Instance,
) -> MI_Result {
    if !valid(app) {
        return MI_RESULT_INVALID_PARAMETER;
    }
    new_instance(class, decl, out)
}

unsafe extern "system" fn a_new_instance_from_class(
    app: *mut MI_Application,
    class: *const MI_Char,
    class_object: *const MI_Class,
    out: *mut *mut MI_Instance,
) -> MI_Result {
    if !valid(app) || class_object.is_null() {
        return MI_RESULT_INVALID_PARAMETER;
    }
    new_instance(class, (*class_object).classDecl, out)
}

unsafe extern "system" fn a_new_destination_options(app: *mut MI_Application, out: *mut MI_DestinationOptions) -> MI_Result {
    if !valid(app) || out.is_null() {
        return MI_RESULT_INVALID_PARAMETER;
    }
    options::init(out as *mut options::Handle<MI_DestinationOptionsFT>, &DESTINATION_OPTIONS_FT, options::new_bag());
    MI_RESULT_OK
}

unsafe extern "system" fn a_new_operation_options(
    app: *mut MI_Application,
    _must_understand: MI_Boolean,
    out: *mut MI_OperationOptions,
) -> MI_Result {
    if !valid(app) || out.is_null() {
        return MI_RESULT_INVALID_PARAMETER;
    }
    options::init(out as *mut options::Handle<MI_OperationOptionsFT>, &OPERATION_OPTIONS_FT, options::new_bag());
    MI_RESULT_OK
}

unsafe extern "system" fn a_new_subscription_delivery_options(
    _app: *mut MI_Application,
    _ty: MI_SubscriptionDeliveryType,
    _out: *mut MI_SubscriptionDeliveryOptions,
) -> MI_Result {
    MI_RESULT_NOT_SUPPORTED
}

unsafe extern "system" fn a_new_serializer(_app: *mut MI_Application, _flags: MI_Uint32, _format: *mut MI_Char, _out: *mut MI_Serializer) -> MI_Result {
    MI_RESULT_NOT_SUPPORTED
}

unsafe extern "system" fn a_new_deserializer(_app: *mut MI_Application, _flags: MI_Uint32, _format: *mut MI_Char, _out: *mut MI_Deserializer) -> MI_Result {
    MI_RESULT_NOT_SUPPORTED
}

unsafe extern "system" fn a_new_class(
    _app: *mut MI_Application,
    _decl: *const MI_ClassDecl,
    _ns: *const MI_Char,
    _server: *const MI_Char,
    out: *mut *mut MI_Class,
) -> MI_Result {
    if !out.is_null() {
        *out = null_mut();
    }
    MI_RESULT_NOT_SUPPORTED
}

pub static APPLICATION_FT: MI_ApplicationFT = MI_ApplicationFT {
    Close: Some(a_close),
    NewSession: Some(a_new_session),
    NewHostedProvider: Some(a_new_hosted_provider),
    NewInstance: Some(a_new_instance),
    NewDestinationOptions: Some(a_new_destination_options),
    NewOperationOptions: Some(a_new_operation_options),
    NewSubscriptionDeliveryOptions: Some(a_new_subscription_delivery_options),
    NewSerializer: Some(a_new_serializer),
    NewDeserializer: Some(a_new_deserializer),
    NewInstanceFromClass: Some(a_new_instance_from_class),
    NewClass: Some(a_new_class),
};

unsafe extern "system" fn u_map_error(_ty: *const MI_Char, _error: MI_Uint32) -> MI_ErrorCategory {
    _MI_ErrorCategory_MI_ERRORCATEGORY_NOT_SPECIFIED
}

unsafe extern "system" fn u_cim_error(_e: MI_Uint32, _ty: *const MI_Char, _msg: *const MI_Char, out: *mut *mut MI_Instance) -> MI_Result {
    if !out.is_null() {
        *out = null_mut();
    }
    MI_RESULT_NOT_SUPPORTED
}

static UTILITIES_FT: MI_UtilitiesFT =
    MI_UtilitiesFT { MapErrorToMiErrorCategory: Some(u_map_error), CimErrorFromErrorCode: Some(u_cim_error) };

// Objects this backend never hands out (hosted providers, serializers); their tables exist
// so the client table has no dangling entry. Windows' own copy leaves the three options
// tables NULL in it too (measured): an options object carries its own table.
static HOSTED_PROVIDER_FT: MI_HostedProviderFT = unsafe { core::mem::zeroed() };
static SERIALIZER_FT: MI_SerializerFT = unsafe { core::mem::zeroed() };
static DESERIALIZER_FT: MI_DeserializerFT = unsafe { core::mem::zeroed() };

#[repr(transparent)]
pub struct ClientTable(MI_ClientFT_V1);
unsafe impl Sync for ClientTable {}
#[repr(transparent)]
pub struct ClientTablePtr(*const MI_ClientFT_V1);
unsafe impl Sync for ClientTablePtr {}

static CLIENT_FT: ClientTable = ClientTable(MI_ClientFT_V1 {
    applicationFT: &APPLICATION_FT,
    sessionFT: &session::SESSION_FT,
    operationFT: &crate::operation::OPERATION_FT,
    hostedProviderFT: &HOSTED_PROVIDER_FT,
    serializerFT: &SERIALIZER_FT,
    deserializerFT: &DESERIALIZER_FT,
    subscribeDeliveryOptionsFT: core::ptr::null(),
    destinationOptionsFT: &DESTINATION_OPTIONS_FT,
    operationOptionsFT: &OPERATION_OPTIONS_FT,
    utilitiesFT: &UTILITIES_FT,
});

/// `const MI_ClientFT_V1 *mi_clientFT_V1` — a data export holding the table's address.
#[no_mangle]
#[used]
pub static mi_clientFT_V1: ClientTablePtr = ClientTablePtr(&CLIENT_FT.0);
