# Integrating a plugin

[Documentation](README.md) · [ABI](abi.md) · [Client services](client.md)

Depend on the `stalkore-sdk-api` Git repository at a reviewed commit. No private
SDK or adapter source is needed to compile an API consumer. Match the consumer's
architecture and use a C ABI at every DLL boundary.

The host invokes the canonical `stalkore_sdk_on_startup_v1() -> i32` callback outside
the loader lock. The SDK publishes service tables before invoking plugins.
`DllMain` remains a Windows loader notification, not the plugin startup contract.
The host configuration lists DLL paths only; it does not select arbitrary entry
point names.

Current SDKs prefer that name and accept the previous `kore_sdk_on_startup_v1`
only when the canonical symbol is absent. Plugins supporting older SDKs can
export both names into the same initialization state. Callback failure never
causes the host to invoke another alias. Windows attachment is still separate.

```rust,no_run
use stalkore_sdk_api::{SdkApi, Status, scheduler::*};

unsafe extern "C" fn on_event(
    _: *mut std::ffi::c_void,
    event: *const TickEventV1,
) -> Status {
    // The SDK supplies a valid borrowed event for the duration of this call.
    let event = unsafe { &*event };
    if event.kind == EVENT_TICK {
        if let Ok(client) = SdkApi::connect().and_then(|api| api.client()) {
            let _snapshot = client.snapshot();
        }
    }
    0
}

let api = SdkApi::connect()?;
let _client_info = api.client()?.probe()?;
// Null context has no allocation to retain. Non-null contexts require explicit
// ownership for every possible in-flight callback; see the ABI guide.
let subscription = unsafe {
    api.scheduler()?.subscribe(on_event, std::ptr::null_mut(), 50)?
};
# Ok::<(), stalkore_sdk_api::Error>(())
```

`connect()` resolves `StalkoreSdkQueryInterfaceV1` in the already loaded
`sdk.dll` and pins its module. It never loads a runtime or starts a process.
Use the returned tables for repeated operations rather than reconnecting on
every production tick. Zero from startup acknowledges submission; asynchronous
readiness or failure arrives through the scheduler.

C consumers resolve the same export with their platform loader and use
[`StalkoreQueryInterfaceV1`](../include/stalkore_sdk_api.h). Initialize each
output's header to `{sizeof(output), STALKORE_ABI_V1}` before calling a provider.
Keep the SDK module retained while using its tables. Rust bindings do this
automatically when connecting to the loaded runtime.
