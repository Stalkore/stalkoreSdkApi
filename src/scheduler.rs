use crate::{status, ApiHeader, Error, Status};
pub const EVENT_READY: u32 = 1;
pub const EVENT_TICK: u32 = 2;
pub const EVENT_STOPPED: u32 = 3;
pub const EVENT_FAILED: u32 = 4;

#[repr(C)]
#[derive(Clone, Copy, Debug)]
pub struct TickEventV1 {
    pub header: ApiHeader,
    pub kind: u32,
    pub status: Status,
    pub subscription: u32,
    pub thread_id: u32,
}
/// Ready/tick run on the client UI thread; terminal failure/stop can run on the
/// SDK worker and must not invoke client operations. Nonzero disables delivery.
pub type TickCallbackV1 = unsafe extern "C" fn(*mut std::ffi::c_void, *const TickEventV1) -> Status;

#[repr(C)]
#[derive(Clone, Copy)]
pub struct SchedulerApiV1 {
    pub header: ApiHeader,
    pub subscribe: Option<
        unsafe extern "C" fn(
            Option<TickCallbackV1>,
            *mut std::ffi::c_void,
            u32,
            *mut u32,
        ) -> Status,
    >,
    pub unsubscribe: Option<unsafe extern "C" fn(u32) -> Status>,
}

#[derive(Clone, Copy)]
pub struct Scheduler(pub(crate) &'static SchedulerApiV1);

impl Scheduler {
    /// # Safety
    /// The callback/context must remain valid until all in-flight calls finish.
    /// The SDK pins callback code; context ownership stays with the plugin.
    pub unsafe fn subscribe(
        self,
        callback: TickCallbackV1,
        context: *mut std::ffi::c_void,
        interval_ms: u32,
    ) -> Result<u32, Error> {
        let mut id = 0;
        status::check(unsafe {
            self.0.subscribe.ok_or(Error(status::UNKNOWN_SERVICE))?(
                Some(callback),
                context,
                interval_ms,
                &mut id,
            )
        })?;
        if id == 0 {
            return Err(Error(status::INTERNAL_ERROR));
        }
        Ok(id)
    }
    pub fn unsubscribe(self, id: u32) -> Result<(), Error> {
        status::check(unsafe { self.0.unsubscribe.ok_or(Error(status::UNKNOWN_SERVICE))?(id) })
    }
}
