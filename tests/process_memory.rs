use stalkore_sdk_api::{process_memory::*, *};
use std::{
    cell::Cell,
    mem::{offset_of, size_of},
    ptr,
};

const TOKEN: u32 = 19;
const READY_HIDDEN: u32 = FLAG_CONTEXT_READY | FLAG_WINDOW_HIDDEN;
const BEFORE: MemoryCountersV1 = MemoryCountersV1 {
    private_bytes: 900_000,
    working_set_bytes: 800_000,
    page_fault_count: 10,
};
const AFTER: MemoryCountersV1 = MemoryCountersV1 {
    private_bytes: BEFORE.private_bytes,
    working_set_bytes: 100_000,
    page_fault_count: 12,
};

thread_local! {
    static FLAGS: Cell<u32> = const { Cell::new(READY_HIDDEN) };
    static CALLS: Cell<u32> = const { Cell::new(0) };
    static OWNER_THREAD: Cell<bool> = const { Cell::new(true) };
}

unsafe fn output_status<T>(output: *mut T) -> Status {
    if output.is_null() || !(output as usize).is_multiple_of(std::mem::align_of::<T>()) {
        return status::INVALID_ARGUMENT;
    }
    if !unsafe { &*output.cast::<ApiHeader>() }.supports::<T>() {
        return status::UNSUPPORTED_VERSION;
    }
    status::OK
}

unsafe extern "C" fn snapshot(output: *mut ProcessMemorySnapshotV1) -> Status {
    let valid = unsafe { output_status(output) };
    if valid != status::OK {
        return valid;
    }
    if !OWNER_THREAD.get() {
        return status::WRONG_THREAD;
    }
    unsafe {
        output.write(ProcessMemorySnapshotV1 {
            capabilities: CAP_RESIDENCY_RECLAIM,
            flags: FLAGS.get(),
            context_token: TOKEN,
            counters: BEFORE,
            ..Default::default()
        });
    }
    status::OK
}

unsafe extern "C" fn reclaim(token: u32, output: *mut ResidencyReclaimResultV1) -> Status {
    let valid = unsafe { output_status(output) };
    if valid != status::OK {
        return valid;
    }
    if !OWNER_THREAD.get() {
        return status::WRONG_THREAD;
    }
    if token == 0 {
        return status::INVALID_ARGUMENT;
    }
    if token != TOKEN {
        return status::STALE_HANDLE;
    }
    if FLAGS.get() & READY_HIDDEN != READY_HIDDEN {
        return status::NOT_READY;
    }
    CALLS.set(CALLS.get() + 1);
    unsafe {
        output.write(ResidencyReclaimResultV1 {
            elapsed_microseconds: 125,
            before: BEFORE,
            after: AFTER,
            ..Default::default()
        });
    }
    status::OK
}

static TABLE: ProcessMemoryApiV1 = ProcessMemoryApiV1 {
    header: ApiHeader::of::<ProcessMemoryApiV1>(),
    snapshot: Some(snapshot),
    reclaim_residency: Some(reclaim),
};

unsafe extern "C" fn query(service: u32, version: u32, output: *mut *const ApiHeader) -> Status {
    if service != PROCESS_MEMORY_SERVICE {
        return status::UNKNOWN_SERVICE;
    }
    if version != ABI_VERSION {
        return status::UNSUPPORTED_VERSION;
    }
    if output.is_null() {
        return status::INVALID_ARGUMENT;
    }
    unsafe { output.write(&TABLE.header) };
    status::OK
}

#[test]
fn reports_counters_and_one_explicit_request_without_claiming_commit_release() {
    CALLS.set(0);
    let memory = unsafe { SdkApi::from_query(query) }
        .process_memory()
        .unwrap();
    let snapshot = memory.snapshot().unwrap();
    assert_eq!(snapshot.capabilities, CAP_RESIDENCY_RECLAIM);
    assert_eq!(snapshot.flags, READY_HIDDEN);
    assert_eq!(snapshot.counters, BEFORE);
    let result = memory.reclaim_residency(snapshot.context_token).unwrap();
    assert_eq!(result.elapsed_microseconds, 125);
    assert_eq!(result.before, BEFORE);
    assert_eq!(result.after, AFTER);
    assert_eq!(CALLS.get(), 1);
}

#[test]
fn stale_unready_visible_and_wrong_thread_requests_remain_explicit_failures() {
    CALLS.set(0);
    let memory = unsafe { SdkApi::from_query(query) }
        .process_memory()
        .unwrap();
    assert_eq!(
        memory.reclaim_residency(0).unwrap_err(),
        Error(status::INVALID_ARGUMENT)
    );
    assert_eq!(
        memory.reclaim_residency(TOKEN + 1).unwrap_err(),
        Error(status::STALE_HANDLE)
    );
    for flags in [0, FLAG_CONTEXT_READY, FLAG_WINDOW_HIDDEN] {
        FLAGS.set(flags);
        assert_eq!(
            memory.reclaim_residency(TOKEN).unwrap_err(),
            Error(status::NOT_READY)
        );
    }
    FLAGS.set(READY_HIDDEN);
    OWNER_THREAD.set(false);
    assert_eq!(memory.snapshot().unwrap_err(), Error(status::WRONG_THREAD));
    assert_eq!(
        memory.reclaim_residency(TOKEN).unwrap_err(),
        Error(status::WRONG_THREAD)
    );
    OWNER_THREAD.set(true);
    assert_eq!(CALLS.get(), 0);
}

#[test]
fn provider_rejects_null_and_short_outputs_without_writing_their_body() {
    assert_eq!(
        unsafe { snapshot(ptr::null_mut()) },
        status::INVALID_ARGUMENT
    );
    assert_eq!(
        unsafe { reclaim(TOKEN, ptr::null_mut()) },
        status::INVALID_ARGUMENT
    );
    let mut snapshot_output = ProcessMemorySnapshotV1::default();
    snapshot_output.header.size = size_of::<ApiHeader>() as u32;
    snapshot_output.counters = AFTER;
    assert_eq!(
        unsafe { snapshot(&mut snapshot_output) },
        status::UNSUPPORTED_VERSION
    );
    assert_eq!(snapshot_output.counters, AFTER);
    let mut reclaim_output = ResidencyReclaimResultV1::default();
    reclaim_output.header.version = ABI_VERSION + 1;
    reclaim_output.elapsed_microseconds = 99;
    assert_eq!(
        unsafe { reclaim(TOKEN, &mut reclaim_output) },
        status::UNSUPPORTED_VERSION
    );
    assert_eq!(reclaim_output.elapsed_microseconds, 99);
}

#[test]
fn old_providers_and_missing_optional_functions_fail_without_fallback() {
    unsafe extern "C" fn old(_: u32, _: u32, _: *mut *const ApiHeader) -> Status {
        status::UNKNOWN_SERVICE
    }
    assert!(matches!(
        unsafe { SdkApi::from_query(old) }.process_memory(),
        Err(Error(status::UNKNOWN_SERVICE))
    ));
    static EMPTY: ProcessMemoryApiV1 = ProcessMemoryApiV1 {
        header: ApiHeader::of::<ProcessMemoryApiV1>(),
        snapshot: None,
        reclaim_residency: None,
    };
    unsafe extern "C" fn missing(_: u32, _: u32, output: *mut *const ApiHeader) -> Status {
        unsafe { output.write(&EMPTY.header) };
        status::OK
    }
    let memory = unsafe { SdkApi::from_query(missing) }
        .process_memory()
        .unwrap();
    assert_eq!(
        memory.snapshot().unwrap_err(),
        Error(status::UNKNOWN_SERVICE)
    );
    assert_eq!(
        memory.reclaim_residency(TOKEN).unwrap_err(),
        Error(status::UNKNOWN_SERVICE)
    );
}

#[test]
fn wrappers_reject_incompatible_tables_outputs_and_contexts() {
    static SHORT: ApiHeader = ApiHeader {
        size: 8,
        version: ABI_VERSION,
    };
    unsafe extern "C" fn short(_: u32, _: u32, output: *mut *const ApiHeader) -> Status {
        unsafe { output.write(&SHORT) };
        status::OK
    }
    assert!(matches!(
        unsafe { SdkApi::from_query(short) }.process_memory(),
        Err(Error(status::UNSUPPORTED_VERSION))
    ));
    unsafe extern "C" fn invalid_snapshot(output: *mut ProcessMemorySnapshotV1) -> Status {
        unsafe {
            output.write(ProcessMemorySnapshotV1 {
                flags: FLAG_CONTEXT_READY,
                ..Default::default()
            })
        };
        status::OK
    }
    unsafe extern "C" fn invalid_result(output: *mut ProcessMemorySnapshotV1) -> Status {
        unsafe { (*output).header.version += 1 };
        status::OK
    }
    unsafe extern "C" fn invalid_reclaim(_: u32, output: *mut ResidencyReclaimResultV1) -> Status {
        unsafe { (*output).header.size = 8 };
        status::OK
    }
    static BAD_CONTEXT: ProcessMemoryApiV1 = ProcessMemoryApiV1 {
        header: ApiHeader::of::<ProcessMemoryApiV1>(),
        snapshot: Some(invalid_snapshot),
        reclaim_residency: Some(invalid_reclaim),
    };
    static BAD_HEADER: ProcessMemoryApiV1 = ProcessMemoryApiV1 {
        header: ApiHeader::of::<ProcessMemoryApiV1>(),
        snapshot: Some(invalid_result),
        reclaim_residency: Some(invalid_reclaim),
    };
    unsafe extern "C" fn bad_context(_: u32, _: u32, output: *mut *const ApiHeader) -> Status {
        unsafe { output.write(&BAD_CONTEXT.header) };
        status::OK
    }
    unsafe extern "C" fn bad_header(_: u32, _: u32, output: *mut *const ApiHeader) -> Status {
        unsafe { output.write(&BAD_HEADER.header) };
        status::OK
    }
    let memory = unsafe { SdkApi::from_query(bad_context) }
        .process_memory()
        .unwrap();
    assert_eq!(
        memory.snapshot().unwrap_err(),
        Error(status::INCOMPATIBLE_CLIENT)
    );
    assert_eq!(
        memory.reclaim_residency(TOKEN).unwrap_err(),
        Error(status::UNSUPPORTED_VERSION)
    );
    let memory = unsafe { SdkApi::from_query(bad_header) }
        .process_memory()
        .unwrap();
    assert_eq!(
        memory.snapshot().unwrap_err(),
        Error(status::UNSUPPORTED_VERSION)
    );
}

#[test]
fn process_memory_c_layout_and_existing_service_ids_remain_fixed() {
    assert_eq!(
        [
            CLIENT_SERVICE,
            SCHEDULER_SERVICE,
            PRESENTATION_SERVICE,
            FRAME_PACING_SERVICE,
            PROCESS_MEMORY_SERVICE
        ],
        [1, 2, 3, 4, 5]
    );
    assert_eq!(size_of::<MemoryCountersV1>(), 24);
    assert_eq!(offset_of!(MemoryCountersV1, page_fault_count), 16);
    assert_eq!(size_of::<ProcessMemorySnapshotV1>(), 48);
    assert_eq!(offset_of!(ProcessMemorySnapshotV1, context_token), 16);
    assert_eq!(offset_of!(ProcessMemorySnapshotV1, counters), 24);
    assert_eq!(size_of::<ResidencyReclaimResultV1>(), 64);
    assert_eq!(
        offset_of!(ResidencyReclaimResultV1, elapsed_microseconds),
        8
    );
    assert_eq!(offset_of!(ResidencyReclaimResultV1, before), 16);
    assert_eq!(offset_of!(ResidencyReclaimResultV1, after), 40);
    assert_eq!(offset_of!(ProcessMemoryApiV1, snapshot), 8);
    assert_eq!(size_of::<ProcessMemoryApiV1>(), 8 + 2 * size_of::<usize>());
}
