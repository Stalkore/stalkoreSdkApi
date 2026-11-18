pub type Status = i32;
pub const OK: Status = 0;
pub const NOT_READY: Status = 1;
pub const UNSUPPORTED_CLIENT: Status = 2;
pub const INVALID_ARGUMENT: Status = 3;
pub const WRONG_THREAD: Status = 4;
pub const STALE_HANDLE: Status = 5;
pub const INCOMPATIBLE_CLIENT: Status = 6;
pub const INTERNAL_ERROR: Status = 7;
pub const SDK_UNAVAILABLE: Status = 8;
pub const UNSUPPORTED_VERSION: Status = 9;
pub const UNKNOWN_SERVICE: Status = 10;
pub const BUSY: Status = 11;
pub const CHARACTER_EMPTY: Status = 20;
pub const CHARACTER_DELETING: Status = 21;
pub const CHARACTER_METHOD: Status = 22;
pub const CHARACTER_DISPATCH: Status = 23;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct Error(pub Status);

impl Error {
    pub const fn message(self) -> &'static str {
        match self.0 {
            NOT_READY => "client service is not ready",
            UNSUPPORTED_CLIENT => "unsupported client executable",
            INVALID_ARGUMENT => "invalid SDK API argument",
            WRONG_THREAD => "client operation requires the owning UI thread",
            STALE_HANDLE => "client object is no longer current",
            INCOMPATIBLE_CLIENT => "client identity or native method is incompatible",
            SDK_UNAVAILABLE => {
                "SDK API is unavailable; rebuild sdk.dll and start through the Forwarder"
            }
            UNSUPPORTED_VERSION => "SDK API version or table size is incompatible",
            UNKNOWN_SERVICE => "requested SDK service is unavailable",
            BUSY => "SDK service is busy",
            CHARACTER_EMPTY => "configured character slot is empty or unavailable",
            CHARACTER_DELETING => {
                "configured character is pending deletion or its record is unreadable"
            }
            CHARACTER_METHOD => "character window method or selected-slot storage is incompatible",
            CHARACTER_DISPATCH => "character window message could not be dispatched",
            _ => "SDK service failed",
        }
    }
}

impl std::fmt::Display for Error {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter.write_str(self.message())
    }
}
impl std::error::Error for Error {}

pub fn check(status: Status) -> Result<(), Error> {
    if status == OK {
        Ok(())
    } else {
        Err(Error(status))
    }
}
pub fn outcome(status: Status) -> Result<bool, Error> {
    match status {
        OK => Ok(true),
        NOT_READY => Ok(false),
        status => Err(Error(status)),
    }
}
