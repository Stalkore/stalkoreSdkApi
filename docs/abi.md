# ABI, threads and lifetime

[Documentation](README.md) · [Client services](client.md)

All exported functions and callbacks use `extern "C"` (cdecl on Windows x86).
Structures use C layout, fixed-width fields and explicit `size`/`version`
headers. Rust containers, strings, references, panics and allocator ownership
never cross the DLL boundary. Nullable function pointers are represented with
`Option<extern "C" fn>` in Rust. A provider validates null arguments and headers;
callers must still supply valid, aligned, readable/writable memory of the stated
size. This ABI is an in-process cooperation contract, not a memory sandbox.

`StalkoreSdkQueryInterfaceV1(service, version, output)` returns a process-lifetime
immutable table. IDs 1 and 2 select client and scheduler services; both currently
use version 1. Unknown service/version and an SDK not yet initialized fail
explicitly. No function from an incompatible or truncated table may be called.
An incompatible table change requires a new version. Optional new services use
new IDs; existing IDs and fields retain their meanings.

| Event/operation | Thread and lifetime |
| --- | --- |
| Client `probe` | Any thread after SDK initialization; file capability check only |
| Scheduler `subscribe` / `unsubscribe` | Any thread, outside the loader lock |
| `READY`, `TICK` | Client owning UI thread; client operations are allowed |
| `FAILED`, `STOPPED` | May run on the SDK worker; do not call client operations |
| Snapshot and client actions | Verified owning UI thread; wrong-thread calls fail |

The SDK owns one dispatcher and multiplexes subscriptions. Each subscription
receives `READY` before its first `TICK`; requested intervals are clamped to
10–1000 ms. Intervals are scheduling targets, not real-time guarantees. Plugin
callbacks must be short, catch their own panics and return a nonzero status to
disable further delivery. A callback failure does not disable other subscribers.

The SDK pins its carrier and callback code. Pinning prevents code unload; it
does not own callback context memory. `unsubscribe` disables new delivery
admission. A delivery admitted before it returns may still begin or finish
afterward; admission occurs before the callback body starts. V1 provides no
quiescence notification or wait operation. Retain context for process lifetime
unless an external UI-thread/lifecycle protocol proves all admitted calls ended.
An empty callback counter alone does not prove that no delivery was admitted.
No teardown callback is provided. Terminal dispatcher failure requires a new
process; subscriptions are not silently restarted.

Snapshot structures are value copies. Mode and dialog tokens are opaque,
ephemeral identities, never addresses. Native actions revalidate current
objects and expected phases. After an operation that can retire objects,
obtain a fresh snapshot before another logical action. PIN digits are the
exceptional multi-call sequence: each step revalidates its exact input count.

Credential byte spans are borrowed only for the call, exclude the trailing NUL
and cannot contain embedded NULs. A provider owns any asynchronous storage it
retains; a plugin may release its input after the function returns. Never log
credential bytes or callback context contents.

Status 0 means success, 1 means not ready/no action, and other codes indicate a
specific failure. Rust `Result<bool, Error>` action wrappers preserve that
distinction. [`status`](../src/status.rs) and the C header define the numeric
contract. A successful startup or operation does not imply authentication or map
entry; inspect subsequent logical state.
