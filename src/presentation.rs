//! Optional presentation controls. All operations require an SDK UI callback.
use crate::{status, ApiHeader, Error, Status};

pub const CAP_WINDOW_VISIBILITY: u32 = 1;

#[repr(C)]
#[derive(Clone, Copy, Debug)]
pub struct PresentationSnapshotV1 {
    pub header: ApiHeader,
    pub capabilities: u32,
    /// Exactly zero or one. Hiding does not imply reduced rendering or memory.
    pub visible: u32,
}

impl Default for PresentationSnapshotV1 {
    fn default() -> Self {
        Self {
            header: ApiHeader::of::<Self>(),
            capabilities: 0,
            visible: 0,
        }
    }
}

#[repr(C)]
#[derive(Clone, Copy)]
pub struct PresentationApiV1 {
    pub header: ApiHeader,
    pub snapshot: Option<unsafe extern "C" fn(*mut PresentationSnapshotV1) -> Status>,
    /// Zero hides; one shows without requesting foreground activation.
    pub set_visible: Option<unsafe extern "C" fn(u32) -> Status>,
}

#[derive(Clone, Copy)]
pub struct Presentation(pub(crate) &'static PresentationApiV1);

impl Presentation {
    pub fn snapshot(self) -> Result<PresentationSnapshotV1, Error> {
        let mut value = PresentationSnapshotV1::default();
        status::check(unsafe {
            self.0.snapshot.ok_or(Error(status::UNKNOWN_SERVICE))?(&mut value)
        })?;
        if !value.header.supports::<PresentationSnapshotV1>() || value.visible > 1 {
            return Err(Error(status::INCOMPATIBLE_CLIENT));
        }
        Ok(value)
    }

    pub fn set_visible(self, visible: bool) -> Result<(), Error> {
        status::check(unsafe {
            self.0.set_visible.ok_or(Error(status::UNKNOWN_SERVICE))?(u32::from(visible))
        })
    }
}
