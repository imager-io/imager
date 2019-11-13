#![allow(unused)]

#[macro_export]
pub mod napi;
pub mod api;


library_exports!{
    version => crate::api::version,
    buffer_open => crate::api::buffer_open,
    buffer_save => crate::api::buffer_save,
    buffer_opt => crate::api::buffer_opt,
}

