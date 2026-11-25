use crate::{
    client::Client, frame_pacing::FramePacing, presentation::Presentation,
    process_memory::ProcessMemory, scheduler::Scheduler, status, ApiHeader, Error,
    QueryInterfaceV1, ABI_VERSION, CLIENT_SERVICE, FRAME_PACING_SERVICE, PRESENTATION_SERVICE,
    PROCESS_MEMORY_SERVICE, SCHEDULER_SERVICE,
};

#[derive(Clone, Copy)]
pub struct SdkApi {
    query: QueryInterfaceV1,
}

impl SdkApi {
    /// Resolve the already loaded SDK and pin its query implementation.
    pub fn connect() -> Result<Self, Error> {
        #[cfg(windows)]
        {
            use windows_sys::Win32::System::LibraryLoader::{
                GetModuleHandleExW, GetModuleHandleW, GetProcAddress,
                GET_MODULE_HANDLE_EX_FLAG_FROM_ADDRESS, GET_MODULE_HANDLE_EX_FLAG_PIN,
            };
            use windows_sys::Win32::System::{
                ProcessStatus::K32EnumProcessModules, Threading::GetCurrentProcess,
            };
            let name: Vec<u16> = "sdk.dll".encode_utf16().chain([0]).collect();
            let module = unsafe { GetModuleHandleW(name.as_ptr()) };
            let legacy = if module.is_null() {
                None
            } else {
                unsafe { GetProcAddress(module, crate::QUERY_EXPORT.as_ptr().cast()) }
            };
            let symbol = if let Some(symbol) = legacy {
                symbol
            } else {
                // The generated DLL has a client-selected filename. Find its
                // unique versioned SDK export instead of guessing that name.
                let mut modules = [std::ptr::null_mut(); 1024];
                let capacity = std::mem::size_of_val(&modules) as u32;
                let mut required = 0u32;
                if unsafe {
                    K32EnumProcessModules(
                        GetCurrentProcess(),
                        modules.as_mut_ptr(),
                        capacity,
                        &mut required,
                    )
                } == 0
                    || required > capacity
                {
                    return Err(Error(status::SDK_UNAVAILABLE));
                }
                let mut matches = modules[..required as usize / std::mem::size_of_val(&module)]
                    .iter()
                    .filter_map(|module| unsafe {
                        GetProcAddress(*module, crate::QUERY_EXPORT.as_ptr().cast())
                    });
                let first = matches.next().ok_or(Error(status::SDK_UNAVAILABLE))?;
                if matches.next().is_some() {
                    return Err(Error(status::SDK_UNAVAILABLE));
                }
                first
            };
            let mut pinned = std::ptr::null_mut();
            if unsafe {
                GetModuleHandleExW(
                    GET_MODULE_HANDLE_EX_FLAG_FROM_ADDRESS | GET_MODULE_HANDLE_EX_FLAG_PIN,
                    symbol as *const () as *const u16,
                    &mut pinned,
                )
            } == 0
            {
                return Err(Error(status::SDK_UNAVAILABLE));
            }
            Ok(Self {
                query: unsafe {
                    std::mem::transmute::<unsafe extern "system" fn() -> isize, QueryInterfaceV1>(
                        symbol,
                    )
                },
            })
        }
        #[cfg(not(windows))]
        Err(Error(status::SDK_UNAVAILABLE))
    }

    /// # Safety
    /// Query and returned tables must honor the ABI and remain valid for the
    /// entire process. This constructor also supports owned provider fixtures.
    pub const unsafe fn from_query(query: QueryInterfaceV1) -> Self {
        Self { query }
    }

    fn table<T>(&self, service: u32) -> Result<&'static T, Error> {
        let mut pointer: *const ApiHeader = std::ptr::null();
        status::check(unsafe { (self.query)(service, ABI_VERSION, &mut pointer) })?;
        if pointer.is_null() || !(pointer as usize).is_multiple_of(std::mem::align_of::<T>()) {
            return Err(Error(status::INVALID_ARGUMENT));
        }
        let header = unsafe { &*pointer };
        if !header.supports::<T>() {
            return Err(Error(status::UNSUPPORTED_VERSION));
        }
        // The query contract guarantees a readable complete table and lifetime.
        Ok(unsafe { &*pointer.cast::<T>() })
    }
    pub fn client(self) -> Result<Client, Error> {
        self.table(CLIENT_SERVICE).map(Client)
    }
    pub fn scheduler(self) -> Result<Scheduler, Error> {
        self.table(SCHEDULER_SERVICE).map(Scheduler)
    }
    pub fn presentation(self) -> Result<Presentation, Error> {
        self.table(PRESENTATION_SERVICE).map(Presentation)
    }
    pub fn frame_pacing(self) -> Result<FramePacing, Error> {
        self.table(FRAME_PACING_SERVICE).map(FramePacing)
    }
    pub fn process_memory(self) -> Result<ProcessMemory, Error> {
        self.table(PROCESS_MEMORY_SERVICE).map(ProcessMemory)
    }
}
