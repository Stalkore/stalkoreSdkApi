use crate::{
    client::Client, presentation::Presentation, scheduler::Scheduler, status, ApiHeader, Error,
    QueryInterfaceV1, ABI_VERSION, CLIENT_SERVICE, PRESENTATION_SERVICE, SCHEDULER_SERVICE,
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
            let name: Vec<u16> = "sdk.dll".encode_utf16().chain([0]).collect();
            let module = unsafe { GetModuleHandleW(name.as_ptr()) };
            if module.is_null() {
                return Err(Error(status::SDK_UNAVAILABLE));
            }
            let symbol = unsafe { GetProcAddress(module, crate::QUERY_EXPORT.as_ptr().cast()) }
                .ok_or(Error(status::SDK_UNAVAILABLE))?;
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
}
