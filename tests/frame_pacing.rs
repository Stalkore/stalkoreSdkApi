use stalkore_sdk_api::{frame_pacing::*, *};
use std::sync::atomic::{AtomicU32, Ordering};
static LIMIT: AtomicU32 = AtomicU32::new(0);
unsafe extern "C" fn snapshot(output: *mut FramePacingSnapshotV1) -> Status {
    assert!(unsafe { (*output).header.supports::<FramePacingSnapshotV1>() });
    unsafe {
        output.write(FramePacingSnapshotV1 {
            capabilities: CAP_FRAME_RATE_LIMIT,
            minimum_fps: 20,
            maximum_fps: 240,
            target_fps: LIMIT.load(Ordering::SeqCst),
            ..Default::default()
        });
    }
    status::OK
}
unsafe extern "C" fn set_limit(fps: u32) -> Status {
    match fps {
        0 | 20..=240 => {
            LIMIT.store(fps, Ordering::SeqCst);
            status::OK
        }
        10 => status::NOT_READY,
        _ => status::INVALID_ARGUMENT,
    }
}
static TABLE: FramePacingApiV1 = FramePacingApiV1 {
    header: ApiHeader::of::<FramePacingApiV1>(),
    snapshot: Some(snapshot),
    set_limit: Some(set_limit),
};
unsafe extern "C" fn query(service: u32, version: u32, output: *mut *const ApiHeader) -> Status {
    if service != FRAME_PACING_SERVICE {
        return status::UNKNOWN_SERVICE;
    }
    if version != ABI_VERSION {
        return status::UNSUPPORTED_VERSION;
    }
    unsafe {
        output.write(&TABLE.header);
    }
    status::OK
}
#[test]
fn limits_readiness_and_errors_cross_the_optional_service_boundary() {
    let service = unsafe { SdkApi::from_query(query) }.frame_pacing().unwrap();
    assert_eq!(std::mem::size_of::<FramePacingSnapshotV1>(), 40);
    assert_eq!(std::mem::offset_of!(FramePacingApiV1, snapshot), 8);
    assert_eq!(service.set_limit(20), Ok(true));
    assert_eq!(service.snapshot().unwrap().target_fps, 20);
    assert_eq!(service.set_limit(10), Ok(false));
    assert_eq!(service.set_limit(300), Err(Error(status::INVALID_ARGUMENT)));
    assert_eq!(service.set_limit(0), Ok(true));
    assert_eq!(service.snapshot().unwrap().target_fps, 0);
}
