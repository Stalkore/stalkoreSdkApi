//! Public, versioned C contracts and Rust bindings for SDK-owned client services.
//! This crate contains no client addresses, profiles, hooks or login policy.
mod api;
pub mod client;
pub mod presentation;
pub mod scheduler;
pub mod status;
pub use api::SdkApi;
pub use status::{Error, Status};

pub const ABI_VERSION: u32 = 1;
pub const CLIENT_SERVICE: u32 = 1;
pub const SCHEDULER_SERVICE: u32 = 2;
pub const PRESENTATION_SERVICE: u32 = 3;
pub const QUERY_EXPORT: &std::ffi::CStr = c"StalkoreSdkQueryInterfaceV1";

#[repr(C)]
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct ApiHeader {
    pub size: u32,
    pub version: u32,
}

impl ApiHeader {
    pub const fn of<T>() -> Self {
        Self {
            size: std::mem::size_of::<T>() as u32,
            version: ABI_VERSION,
        }
    }

    pub fn supports<T>(&self) -> bool {
        self.version == ABI_VERSION && self.size as usize >= std::mem::size_of::<T>()
    }
}

/// # Safety
/// `output` must be writable for one aligned pointer. A successful table and
/// its functions remain valid while the SDK is retained by the caller.
pub type QueryInterfaceV1 =
    unsafe extern "C" fn(service: u32, version: u32, output: *mut *const ApiHeader) -> Status;
