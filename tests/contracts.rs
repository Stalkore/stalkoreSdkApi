use stalkore_sdk_api::{client::*, scheduler::*, *};
use std::{
    mem::{offset_of, size_of},
    sync::atomic::{AtomicU32, Ordering},
};

static CALLS: AtomicU32 = AtomicU32::new(0);
unsafe extern "C" fn probe(out: *mut ClientInfoV1) -> Status {
    unsafe {
        out.write(ClientInfoV1 {
            capabilities: CAP_LOGIN,
            account_name_capacity: 50,
            ..Default::default()
        });
    }
    status::OK
}
unsafe extern "C" fn snapshot(out: *mut ClientSnapshotV1) -> Status {
    unsafe {
        out.write(ClientSnapshotV1 {
            mode: MODE_LOGIN,
            mode_token: 17,
            phase: PHASE_CREDENTIALS,
            ..Default::default()
        });
    }
    status::OK
}
unsafe extern "C" fn action(token: u32) -> Status {
    if token == 17 {
        status::OK
    } else {
        status::STALE_HANDLE
    }
}
static CLIENT: ClientApiV1 = ClientApiV1 {
    header: ApiHeader::of::<ClientApiV1>(),
    probe: Some(probe),
    snapshot: Some(snapshot),
    open_account_selection: Some(action),
    select_account: None,
    request_login: None,
    select_server: None,
    select_character: None,
    pin_ready: None,
    pin_key: None,
    acknowledge_dialog: None,
    return_to_login: None,
};
unsafe extern "C" fn subscribe(
    callback: Option<TickCallbackV1>,
    context: *mut std::ffi::c_void,
    _: u32,
    id: *mut u32,
) -> Status {
    unsafe {
        id.write(42);
    }
    let Some(callback) = callback else {
        return status::INVALID_ARGUMENT;
    };
    unsafe {
        callback(
            context,
            &TickEventV1 {
                header: ApiHeader::of::<TickEventV1>(),
                kind: EVENT_READY,
                status: status::OK,
                subscription: 42,
                thread_id: 10,
            },
        )
    }
}
unsafe extern "C" fn unsubscribe(id: u32) -> Status {
    if id == 42 {
        status::OK
    } else {
        status::STALE_HANDLE
    }
}
static SCHEDULER: SchedulerApiV1 = SchedulerApiV1 {
    header: ApiHeader::of::<SchedulerApiV1>(),
    subscribe: Some(subscribe),
    unsubscribe: Some(unsubscribe),
};
unsafe extern "C" fn query(service: u32, version: u32, out: *mut *const ApiHeader) -> Status {
    if version != ABI_VERSION {
        return status::UNSUPPORTED_VERSION;
    }
    let table = match service {
        CLIENT_SERVICE => &CLIENT.header,
        SCHEDULER_SERVICE => &SCHEDULER.header,
        _ => return status::UNKNOWN_SERVICE,
    };
    unsafe {
        out.write(table);
    }
    status::OK
}
#[test]
fn logical_operations_and_scheduler_cross_the_versioned_c_boundary() {
    let api = unsafe { SdkApi::from_query(query) };
    let client = api.client().unwrap();
    assert_eq!(client.probe().unwrap().capabilities, CAP_LOGIN);
    let snapshot = client.snapshot().unwrap();
    assert_eq!(snapshot.mode_token, 17);
    assert_eq!(client.open_account_selection(snapshot.mode_token), Ok(true));
    assert_eq!(
        client.open_account_selection(18),
        Err(Error(status::STALE_HANDLE))
    );
    assert_eq!(
        client.select_account(17),
        Err(Error(status::UNKNOWN_SERVICE))
    );
    unsafe extern "C" fn ready(
        context: *mut std::ffi::c_void,
        event: *const TickEventV1,
    ) -> Status {
        assert_eq!(context as usize, 123);
        assert_eq!(unsafe { (*event).kind }, EVENT_READY);
        CALLS.fetch_add(1, Ordering::SeqCst);
        status::OK
    }
    let scheduler = api.scheduler().unwrap();
    let id = unsafe { scheduler.subscribe(ready, 123usize as *mut _, 50) }.unwrap();
    assert_eq!(id, 42);
    assert_eq!(CALLS.load(Ordering::SeqCst), 1);
    assert_eq!(scheduler.unsubscribe(id), Ok(()));
    assert_eq!(scheduler.unsubscribe(1), Err(Error(status::STALE_HANDLE)));
}
#[test]
fn invalid_provider_tables_are_rejected_before_function_pointer_access() {
    static SHORT: ApiHeader = ApiHeader {
        size: 8,
        version: ABI_VERSION,
    };
    unsafe extern "C" fn short(_: u32, _: u32, out: *mut *const ApiHeader) -> Status {
        unsafe {
            out.write(&SHORT);
        }
        status::OK
    }
    unsafe extern "C" fn null(_: u32, _: u32, _: *mut *const ApiHeader) -> Status {
        status::OK
    }
    assert!(matches!(
        unsafe { SdkApi::from_query(short) }.client(),
        Err(Error(status::UNSUPPORTED_VERSION))
    ));
    assert!(matches!(
        unsafe { SdkApi::from_query(null) }.client(),
        Err(Error(status::INVALID_ARGUMENT))
    ));
    assert_eq!(status::outcome(status::NOT_READY), Ok(false));
    assert_eq!(
        status::outcome(status::WRONG_THREAD),
        Err(Error(status::WRONG_THREAD))
    );
}
#[test]
fn abi_layout_is_fixed_for_c_consumers() {
    assert_eq!(size_of::<ApiHeader>(), 8);
    assert_eq!(size_of::<ClientInfoV1>(), 24);
    assert_eq!(size_of::<ClientSnapshotV1>(), 40);
    assert_eq!(size_of::<TickEventV1>(), 24);
    assert_eq!(offset_of!(ClientInfoV1, capabilities), 8);
    assert_eq!(offset_of!(ClientSnapshotV1, dialog_kind), 36);
    assert_eq!(offset_of!(ClientApiV1, probe), 8);
    assert_eq!(size_of::<ClientApiV1>(), 8 + 11 * size_of::<usize>());
    assert_eq!(size_of::<SchedulerApiV1>(), 8 + 2 * size_of::<usize>());
    assert!(!ApiHeader {
        size: u32::MAX,
        version: 2
    }
    .supports::<ClientApiV1>());
}
