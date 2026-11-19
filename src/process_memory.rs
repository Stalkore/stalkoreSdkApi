//! Optional process residency control. Paging does not release assets or commit.
use crate::{status, ApiHeader, Error, Status};

pub const CAP_RESIDENCY_RECLAIM: u32 = 1;
pub const FLAG_CONTEXT_READY: u32 = 1;
pub const FLAG_WINDOW_HIDDEN: u32 = 2;

#[repr(C)]
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct MemoryCountersV1 {
    pub private_bytes: u64,
    pub working_set_bytes: u64,
    /// Cumulative process page faults, including faults unrelated to reclamation.
    pub page_fault_count: u64,
}

#[repr(C)]
#[derive(Clone, Copy, Debug)]
pub struct ProcessMemorySnapshotV1 {
    pub header: ApiHeader,
    pub capabilities: u32,
    pub flags: u32,
    /// Opaque current-context identity; never a native address.
    pub context_token: u32,
    pub reserved: u32,
    pub counters: MemoryCountersV1,
}

impl Default for ProcessMemorySnapshotV1 {
    fn default() -> Self {
        Self {
            header: ApiHeader::of::<Self>(),
            capabilities: 0,
            flags: 0,
            context_token: 0,
            reserved: 0,
            counters: MemoryCountersV1::default(),
        }
    }
}

#[repr(C)]
#[derive(Clone, Copy, Debug)]
pub struct ResidencyReclaimResultV1 {
    pub header: ApiHeader,
    pub elapsed_microseconds: u64,
    pub before: MemoryCountersV1,
    pub after: MemoryCountersV1,
}

impl Default for ResidencyReclaimResultV1 {
    fn default() -> Self {
        Self {
            header: ApiHeader::of::<Self>(),
            elapsed_microseconds: 0,
            before: MemoryCountersV1::default(),
            after: MemoryCountersV1::default(),
        }
    }
}

/// Both operations require an SDK READY/TICK callback on the owning UI thread.
#[repr(C)]
#[derive(Clone, Copy)]
pub struct ProcessMemoryApiV1 {
    pub header: ApiHeader,
    pub snapshot: Option<unsafe extern "C" fn(*mut ProcessMemorySnapshotV1) -> Status>,
    pub reclaim_residency:
        Option<unsafe extern "C" fn(u32, *mut ResidencyReclaimResultV1) -> Status>,
}

#[derive(Clone, Copy)]
pub struct ProcessMemory(pub(crate) &'static ProcessMemoryApiV1);

impl ProcessMemory {
    pub fn snapshot(self) -> Result<ProcessMemorySnapshotV1, Error> {
        let mut value = ProcessMemorySnapshotV1::default();
        status::check(unsafe {
            self.0.snapshot.ok_or(Error(status::UNKNOWN_SERVICE))?(&mut value)
        })?;
        if !value.header.supports::<ProcessMemorySnapshotV1>() {
            return Err(Error(status::UNSUPPORTED_VERSION));
        }
        if value.reserved != 0
            || (value.flags & FLAG_CONTEXT_READY != 0 && value.context_token == 0)
        {
            return Err(Error(status::INCOMPATIBLE_CLIENT));
        }
        Ok(value)
    }

    /// Make one paging request for the current ready, hidden context. The SDK
    /// revalidates the token and conditions immediately before the operation.
    /// This never frees resources, changes limits, or retries automatically.
    pub fn reclaim_residency(self, context_token: u32) -> Result<ResidencyReclaimResultV1, Error> {
        if context_token == 0 {
            return Err(Error(status::INVALID_ARGUMENT));
        }
        let mut value = ResidencyReclaimResultV1::default();
        status::check(unsafe {
            self.0
                .reclaim_residency
                .ok_or(Error(status::UNKNOWN_SERVICE))?(context_token, &mut value)
        })?;
        if !value.header.supports::<ResidencyReclaimResultV1>() {
            return Err(Error(status::UNSUPPORTED_VERSION));
        }
        Ok(value)
    }
}
