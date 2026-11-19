//! Optional frame-end pacing. Waiting also delays the caller's next loop cycle.
use crate::{status, ApiHeader, Error, Status};

pub const CAP_FRAME_RATE_LIMIT: u32 = 1;
pub const FLAG_FRAME_OBSERVER_ATTACHED: u32 = 1;

#[repr(C)]
#[derive(Clone, Copy, Debug)]
pub struct FramePacingSnapshotV1 {
    pub header: ApiHeader,
    pub capabilities: u32,
    pub minimum_fps: u32,
    pub maximum_fps: u32,
    /// Zero preserves the client's own frame timing.
    pub target_fps: u32,
    pub observed_millifps: u32,
    /// Zero until a complete observation window is available.
    pub observed_window_milliseconds: u32,
    /// Wraps after u32::MAX; this is an observation counter, not an identity.
    pub frame_count: u32,
    pub flags: u32,
}

impl Default for FramePacingSnapshotV1 {
    fn default() -> Self {
        Self {
            header: ApiHeader::of::<Self>(),
            capabilities: 0,
            minimum_fps: 0,
            maximum_fps: 0,
            target_fps: 0,
            observed_millifps: 0,
            observed_window_milliseconds: 0,
            frame_count: 0,
            flags: 0,
        }
    }
}

#[repr(C)]
#[derive(Clone, Copy)]
pub struct FramePacingApiV1 {
    pub header: ApiHeader,
    pub snapshot: Option<unsafe extern "C" fn(*mut FramePacingSnapshotV1) -> Status>,
    pub set_limit: Option<unsafe extern "C" fn(u32) -> Status>,
}

#[derive(Clone, Copy)]
pub struct FramePacing(pub(crate) &'static FramePacingApiV1);

impl FramePacing {
    pub fn snapshot(self) -> Result<FramePacingSnapshotV1, Error> {
        let mut value = FramePacingSnapshotV1::default();
        status::check(unsafe {
            self.0.snapshot.ok_or(Error(status::UNKNOWN_SERVICE))?(&mut value)
        })?;
        if !value.header.supports::<FramePacingSnapshotV1>() {
            return Err(Error(status::UNSUPPORTED_VERSION));
        }
        Ok(value)
    }

    /// False means the renderer is not ready yet. This never sleeps in a plugin
    /// callback; the provider applies the limit at the native frame boundary.
    pub fn set_limit(self, fps: u32) -> Result<bool, Error> {
        status::outcome(unsafe { self.0.set_limit.ok_or(Error(status::UNKNOWN_SERVICE))?(fps) })
    }
}
