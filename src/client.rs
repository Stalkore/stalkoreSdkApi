//! Logical client states and operations; tokens are not client memory addresses.
use crate::{status, ApiHeader, Error, Status};

pub const CAP_LOGIN: u64 = 1;
pub const CAP_CHARACTER: u64 = 2;
pub const CAP_PIN: u64 = 4;
pub const MODE_NONE: u32 = 0;
pub const MODE_LOGIN: u32 = 1;
pub const MODE_MAP: u32 = 2;
pub const MODE_OTHER: u32 = 3;
pub const PHASE_UNKNOWN: u32 = 0;
pub const PHASE_ACCOUNT_PROMPT: u32 = 1;
pub const PHASE_ACCOUNT_SELECTION: u32 = 2;
pub const PHASE_CREDENTIALS: u32 = 3;
pub const PHASE_AUTHENTICATING: u32 = 4;
pub const PHASE_SERVERS: u32 = 5;
pub const PHASE_CHARACTERS: u32 = 6;
pub const PHASE_MAP: u32 = 7;
pub const FLAG_LOGIN_READY: u32 = 1;
pub const FLAG_SERVER_COUNT: u32 = 2;
pub const FLAG_SSO_IDLE: u32 = 4;
pub const FLAG_MODAL: u32 = 8;
pub const FLAG_MAP_DISCONNECTED: u32 = 16;
pub const FLAG_PIN_PRESENT: u32 = 32;
pub const FLAG_PIN_READY: u32 = 64;
pub const FLAG_LOGIN_STAGE_VALID: u32 = 128;
pub const DIALOG_NONE: u32 = 0;
pub const DIALOG_DISCONNECTED: u32 = 1;
pub const DIALOG_CONNECTION_FAILURE: u32 = 2;
pub const PIN_CONFIRM: u32 = 10;

#[repr(C)]
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct ClientInfoV1 {
    pub header: ApiHeader,
    pub capabilities: u64,
    pub account_name_capacity: u32,
    pub reserved: u32,
}
impl Default for ClientInfoV1 {
    fn default() -> Self {
        Self {
            header: ApiHeader::of::<Self>(),
            capabilities: 0,
            account_name_capacity: 0,
            reserved: 0,
        }
    }
}

#[repr(C)]
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct ClientSnapshotV1 {
    pub header: ApiHeader,
    pub mode: u32,
    pub mode_token: u32,
    pub phase: u32,
    pub progress_revision: u32,
    pub flags: u32,
    pub server_count: i32,
    pub dialog_token: u32,
    pub dialog_kind: u32,
}
impl Default for ClientSnapshotV1 {
    fn default() -> Self {
        Self {
            header: ApiHeader::of::<Self>(),
            mode: MODE_NONE,
            mode_token: 0,
            phase: PHASE_UNKNOWN,
            progress_revision: 0,
            flags: 0,
            server_count: 0,
            dialog_token: 0,
            dialog_kind: DIALOG_NONE,
        }
    }
}
impl ClientSnapshotV1 {
    pub fn has(&self, flag: u32) -> bool {
        self.flags & flag != 0
    }
}

/// Borrowed bytes, readable only for the call. The SDK owns any data it retains.
#[repr(C)]
#[derive(Clone, Copy)]
pub struct BytesV1 {
    pub data: *const u8,
    pub len: u32,
}

#[repr(C)]
#[derive(Clone, Copy)]
pub struct ClientApiV1 {
    pub header: ApiHeader,
    pub probe: Option<unsafe extern "C" fn(*mut ClientInfoV1) -> Status>,
    pub snapshot: Option<unsafe extern "C" fn(*mut ClientSnapshotV1) -> Status>,
    pub open_account_selection: Option<unsafe extern "C" fn(u32) -> Status>,
    pub select_account: Option<unsafe extern "C" fn(u32) -> Status>,
    pub request_login: Option<unsafe extern "C" fn(u32, BytesV1, BytesV1) -> Status>,
    pub select_server: Option<unsafe extern "C" fn(u32, u32) -> Status>,
    pub select_character: Option<unsafe extern "C" fn(u32, u32) -> Status>,
    pub pin_ready: Option<unsafe extern "C" fn(u32, u32) -> Status>,
    pub pin_key: Option<unsafe extern "C" fn(u32, u32, u32) -> Status>,
    pub acknowledge_dialog: Option<unsafe extern "C" fn(u32, u32) -> Status>,
    pub return_to_login: Option<unsafe extern "C" fn(u32) -> Status>,
}

#[derive(Clone, Copy)]
pub struct Client(pub(crate) &'static ClientApiV1);

impl Client {
    pub fn probe(self) -> Result<ClientInfoV1, Error> {
        let mut info = ClientInfoV1::default();
        status::check(unsafe { self.0.probe.ok_or(Error(status::UNKNOWN_SERVICE))?(&mut info) })?;
        if !info.header.supports::<ClientInfoV1>() {
            return Err(Error(status::UNSUPPORTED_VERSION));
        }
        Ok(info)
    }
    pub fn snapshot(self) -> Result<ClientSnapshotV1, Error> {
        let mut state = ClientSnapshotV1::default();
        status::check(unsafe {
            self.0.snapshot.ok_or(Error(status::UNKNOWN_SERVICE))?(&mut state)
        })?;
        if !state.header.supports::<ClientSnapshotV1>() {
            return Err(Error(status::UNSUPPORTED_VERSION));
        }
        Ok(state)
    }
    pub fn open_account_selection(self, mode: u32) -> Result<bool, Error> {
        status::outcome(unsafe {
            self.0
                .open_account_selection
                .ok_or(Error(status::UNKNOWN_SERVICE))?(mode)
        })
    }
    pub fn select_account(self, mode: u32) -> Result<bool, Error> {
        status::outcome(unsafe {
            self.0
                .select_account
                .ok_or(Error(status::UNKNOWN_SERVICE))?(mode)
        })
    }
    pub fn request_login(
        self,
        mode: u32,
        user: &std::ffi::CStr,
        password: &std::ffi::CStr,
    ) -> Result<bool, Error> {
        let bytes = |value: &std::ffi::CStr| -> Result<BytesV1, Error> {
            Ok(BytesV1 {
                data: value.to_bytes().as_ptr(),
                len: value
                    .to_bytes()
                    .len()
                    .try_into()
                    .map_err(|_| Error(status::INVALID_ARGUMENT))?,
            })
        };
        status::outcome(unsafe {
            self.0.request_login.ok_or(Error(status::UNKNOWN_SERVICE))?(
                mode,
                bytes(user)?,
                bytes(password)?,
            )
        })
    }
    pub fn select_server(self, mode: u32, server: u32) -> Result<bool, Error> {
        status::outcome(unsafe {
            self.0.select_server.ok_or(Error(status::UNKNOWN_SERVICE))?(mode, server)
        })
    }
    pub fn select_character(self, mode: u32, slot: u8) -> Result<bool, Error> {
        status::outcome(unsafe {
            self.0
                .select_character
                .ok_or(Error(status::UNKNOWN_SERVICE))?(mode, slot.into())
        })
    }
    pub fn pin_ready(self, mode: u32, count: u32) -> Result<bool, Error> {
        status::outcome(unsafe {
            self.0.pin_ready.ok_or(Error(status::UNKNOWN_SERVICE))?(mode, count)
        })
    }
    pub fn pin_key(self, mode: u32, key: u32, count: u32) -> Result<bool, Error> {
        status::outcome(unsafe {
            self.0.pin_key.ok_or(Error(status::UNKNOWN_SERVICE))?(mode, key, count)
        })
    }
    pub fn acknowledge_dialog(self, dialog: u32, failed: bool) -> Result<bool, Error> {
        status::outcome(unsafe {
            self.0
                .acknowledge_dialog
                .ok_or(Error(status::UNKNOWN_SERVICE))?(dialog, failed.into())
        })
    }
    pub fn return_to_login(self, mode: u32) -> Result<bool, Error> {
        status::outcome(unsafe {
            self.0
                .return_to_login
                .ok_or(Error(status::UNKNOWN_SERVICE))?(mode)
        })
    }
}
