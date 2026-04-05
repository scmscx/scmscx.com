#![allow(non_upper_case_globals)]
#![allow(non_camel_case_types)]
#![allow(non_snake_case)]
#![allow(improper_ctypes)]
#![allow(dead_code)]
#![allow(deref_nullptr)]
#![allow(clippy::ptr_offset_with_cast)]
#![allow(clippy::semicolon_if_nothing_returned)]
#![allow(clippy::missing_safety_doc)]
#![allow(unnecessary_transmutes)]

include!(concat!(env!("OUT_DIR"), "/bindings.rs"));
