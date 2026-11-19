use stalkore_sdk_api::{presentation::*, *};
use std::sync::atomic::{AtomicU32, Ordering};

static VISIBLE: AtomicU32 = AtomicU32::new(1);
unsafe extern "C" fn snapshot(out: *mut PresentationSnapshotV1) -> Status {
    assert!(unsafe { (*out).header.supports::<PresentationSnapshotV1>() });
    unsafe {
        out.write(PresentationSnapshotV1 {
            capabilities: CAP_WINDOW_VISIBILITY,
            visible: VISIBLE.load(Ordering::SeqCst),
            ..Default::default()
        })
    };
    status::OK
}
unsafe extern "C" fn set_visible(value: u32) -> Status {
    VISIBLE.store(value, Ordering::SeqCst);
    status::OK
}
static TABLE: PresentationApiV1 = PresentationApiV1 {
    header: ApiHeader::of::<PresentationApiV1>(),
    snapshot: Some(snapshot),
    set_visible: Some(set_visible),
};
unsafe extern "C" fn query(service: u32, version: u32, out: *mut *const ApiHeader) -> Status {
    if service != PRESENTATION_SERVICE {
        return status::UNKNOWN_SERVICE;
    }
    if version != ABI_VERSION {
        return status::UNSUPPORTED_VERSION;
    }
    unsafe { out.write(&TABLE.header) };
    status::OK
}
#[test]
fn visibility_round_trip_and_invalid_provider_state_cross_the_c_boundary() {
    let service = unsafe { SdkApi::from_query(query) }.presentation().unwrap();
    assert_eq!(std::mem::size_of::<PresentationSnapshotV1>(), 16);
    assert_eq!(std::mem::offset_of!(PresentationApiV1, snapshot), 8);
    assert_eq!(
        service.snapshot().unwrap().capabilities,
        CAP_WINDOW_VISIBILITY
    );
    service.set_visible(false).unwrap();
    assert_eq!(service.snapshot().unwrap().visible, 0);
    service.set_visible(true).unwrap();
    assert_eq!(service.snapshot().unwrap().visible, 1);
    VISIBLE.store(2, Ordering::SeqCst);
    assert_eq!(
        service.snapshot().unwrap_err(),
        Error(status::INCOMPATIBLE_CLIENT)
    );
}
