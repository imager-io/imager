pub use napi_sys_dev::{
    napi_throw_type_error,
    napi_typeof,
    napi_get_value_string_utf16,
    napi_get_value_uint32,
    napi_get_value_int64,
    napi_get_value_double,
    napi_get_global,
    napi_get_named_property,
    napi_call_function,
    napi_get_null,
    napi_get_boolean,
    napi_create_double,
    napi_create_int64,
    napi_create_uint32,
    napi_create_string_utf8,
    napi_create_array,
    napi_create_object,
    napi_set_property,
    NAPI_AUTO_LENGTH,
    napi_set_element,
    napi_get_cb_info,
    napi_throw_error,
    napi_create_external,
    napi_get_value_external,
    napi_adjust_external_memory,
    napi_has_own_property,
    napi_get_property,
    napi_create_async_work,
    napi_create_promise,
    napi_resolve_deferred,
    napi_reject_deferred,
    napi_is_promise,
    napi_queue_async_work,
    napi_open_handle_scope,
    napi_close_handle_scope,
    napi_delete_async_work,
    napi_create_reference,
    napi_async_init,
    napi_open_callback_scope,
    napi_close_callback_scope,
    napi_create_threadsafe_function,
};


pub type NapiModule = napi_sys_dev::napi_module;
pub type NapiEnv = napi_sys_dev::napi_env;
pub type NapiValue = napi_sys_dev::napi_value;
pub type NapiValueType = napi_sys_dev::napi_valuetype;
pub static NAPI_UNDEFINED: NapiValueType = napi_sys_dev::napi_valuetype_napi_undefined;
pub static NAPI_NULL: NapiValueType = napi_sys_dev::napi_valuetype_napi_null;
pub static NAPI_BOOLEAN: NapiValueType = napi_sys_dev::napi_valuetype_napi_boolean;
pub static NAPI_NUMBER: NapiValueType = napi_sys_dev::napi_valuetype_napi_number;
pub static NAPI_STRING: NapiValueType = napi_sys_dev::napi_valuetype_napi_string;
pub static NAPI_SYMBOL: NapiValueType = napi_sys_dev::napi_valuetype_napi_symbol;
pub static NAPI_OBJECT: NapiValueType = napi_sys_dev::napi_valuetype_napi_object;
pub static NAPI_FUNCTION: NapiValueType = napi_sys_dev::napi_valuetype_napi_function;
pub static NAPI_EXTERNAL: NapiValueType = napi_sys_dev::napi_valuetype_napi_external;
pub static NAPI_BIGINT: NapiValueType = napi_sys_dev::napi_valuetype_napi_bigint;

pub type NapiStatus = napi_sys_dev::napi_status;
pub static NAPI_OK: NapiStatus = napi_sys_dev::napi_status_napi_ok;
pub static NAPI_INVALID_ARG: NapiStatus = napi_sys_dev::napi_status_napi_invalid_arg;
pub static NAPI_OBJECT_EXPECTED: NapiStatus = napi_sys_dev::napi_status_napi_object_expected;
pub static NAPI_STRING_EXPECTED: NapiStatus = napi_sys_dev::napi_status_napi_string_expected;
pub static NAPI_NAME_EXPECTED: NapiStatus = napi_sys_dev::napi_status_napi_name_expected;
pub static NAPI_FUNCTION_EXPECTED: NapiStatus = napi_sys_dev::napi_status_napi_function_expected;
pub static NAPI_NUMBER_EXPECTED: NapiStatus = napi_sys_dev::napi_status_napi_number_expected;
pub static NAPI_BOOLEAN_EXPECTED: NapiStatus = napi_sys_dev::napi_status_napi_boolean_expected;
pub static NAPI_ARRAY_EXPECTED: NapiStatus = napi_sys_dev::napi_status_napi_array_expected;
pub static NAPI_GENERIC_FAILURE: NapiStatus = napi_sys_dev::napi_status_napi_generic_failure;
pub static NAPI_PENDING_EXCEPTION: NapiStatus = napi_sys_dev::napi_status_napi_pending_exception;
pub static NAPI_CANCELLED: NapiStatus = napi_sys_dev::napi_status_napi_cancelled;
pub static NAPI_ESCAPE_CALLED_TWICE: NapiStatus = napi_sys_dev::napi_status_napi_escape_called_twice;
pub static NAPI_HANDLE_SCOPE_MISMATCH: NapiStatus = napi_sys_dev::napi_status_napi_handle_scope_mismatch;
pub static NAPI_CALLBACK_SCOPE_MISMATCH: NapiStatus = napi_sys_dev::napi_status_napi_callback_scope_mismatch;
pub static NAPI_QUEUE_FULL: NapiStatus = napi_sys_dev::napi_status_napi_queue_full;
pub static NAPI_CLOSING: NapiStatus = napi_sys_dev::napi_status_napi_closing;
pub static NAPI_BIGINT_EXPECTED: NapiStatus = napi_sys_dev::napi_status_napi_bigint_expected;
pub static NAPI_DATE_EXPECTED: NapiStatus = napi_sys_dev::napi_status_napi_date_expected;

pub type NapiCallbackInfo = napi_sys_dev::napi_callback_info;
pub type NapiFinalize = napi_sys_dev::napi_finalize;
pub type NapiDeferred = napi_sys_dev::napi_deferred;
pub type NapiAsyncWork = napi_sys_dev::napi_async_work;
pub type NapiAsyncExecuteCallback = napi_sys_dev::napi_async_execute_callback;
pub type NapiAsyncCompleteCallback = napi_sys_dev::napi_async_complete_callback;
pub type NapiHandleScope = napi_sys_dev::napi_handle_scope;
pub type NapiRef = napi_sys_dev::napi_ref;
pub type NapiAsyncContext = napi_sys_dev::napi_async_context;
pub type NapiCallbackScope = napi_sys_dev::napi_callback_scope;
pub type NapiThreadsafeFunction = napi_sys_dev::napi_threadsafe_function;
pub type NapiThreadsafeFunctionCallJs = napi_sys_dev::napi_threadsafe_function_call_js;


///////////////////////////////////////////////////////////////////////////////
// EXTRA
///////////////////////////////////////////////////////////////////////////////
pub type NapiCallback = extern "C" fn(env: NapiEnv, info: NapiCallbackInfo) -> NapiValue;
