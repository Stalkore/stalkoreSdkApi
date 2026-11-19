# Frame pacing

[Documentation](README.md) · [ABI and lifetime](abi.md)

Optional service 4 controls frame-end waiting through `SdkApi::frame_pacing()`.
It has its own v1 table; existing login, scheduler and visibility tables retain
their layouts. All calls require an SDK READY/TICK on the owning UI thread.

`snapshot()` reports capabilities, the backend's supported FPS range, applied
target, observed frames per second multiplied by 1000, observation duration and
a wrapping native frame-end counter. This is a call rate, not a measurement of
GPU-completed or displayed frames. A zero observation duration means no complete sample
is available. The attached flag indicates a current native observer.

`set_limit(0)` preserves native timing. Other limits must be within the range
reported by the backend. A successful request returns true; false means the
renderer is not ready and the caller can retry on a later SDK tick. Incompatible
native interfaces and invalid limits fail explicitly.

Pacing is performed after the original frame method returns, outside plugin
callbacks. It preserves that method and its result. Sleeping delays the calling
thread's next cycle; it does not create an independent simulation or networking
thread. A supported limit therefore needs native client acceptance, including map
transitions and responsiveness. This service does not evict textures, unload
resources or promise a reduction in private memory.
