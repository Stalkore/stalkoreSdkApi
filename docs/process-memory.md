# Process memory and residency

[Documentation](README.md) · [ABI and lifetime](abi.md) · [Presentation](presentation.md)

Optional service 5 exposes process counters and one explicit residency-reclaim
request through `SdkApi::process_memory()`. It adds a separate v1 table;
services 1–4 retain their IDs and layouts. Both operations require an SDK
`READY` or `TICK` callback on the owning UI thread.

## Counters and current context

`snapshot()` returns capability bits, context flags, an opaque context token,
and three counters:

| Field | Meaning |
| --- | --- |
| `private_bytes` | Private committed process memory, in bytes |
| `working_set_bytes` | Currently resident process memory, in bytes; may include shared pages |
| `page_fault_count` | The operating system's cumulative process page-fault counter, widened to 64 bits |

The page-fault counter includes unrelated activity and can wrap at the operating
system counter's width. A working-set measurement is not exclusive physical
ownership. Compare actual counters rather than treating reserved virtual space,
resident pages, and private commit as the same quantity.

`CAP_RESIDENCY_RECLAIM` means the provider supports the operation. Context flags
have independent meanings:

- `FLAG_CONTEXT_READY`: the provider has a current, eligible client context.
- `FLAG_WINDOW_HIDDEN`: the supported main window is currently hidden.

A ready snapshot has a nonzero `context_token`. The token identifies the
provider's observed scene/player/map context; it contains no native address.
Changes or retirement invalidate old tokens, even if another context later
occupies the same native storage. Do not persist or interpret tokens. Reserved
fields are zero. Unknown capability and flag bits are reserved for extensions.

## One guarded paging request

`reclaim_residency(token)` revalidates the current nonzero token, context
readiness, and hidden-window state immediately before making one operating
system paging request for the current process. It preserves assets, allocations,
frame limits, and visibility. It performs no automatic retry.

Successful reclamation can reduce the working set without reducing private
commit. Resident pages may return when needed; subsequent faults and disk work
can add latency. Success promises neither a particular reduction nor that a
reduction will persist. The result reports actual before/after counters and
elapsed microseconds for this operation. It does not attribute every counter
change to reclamation.

The provider rejects requests explicitly:

| Condition | Status |
| --- | --- |
| Zero token or null/misaligned output | `INVALID_ARGUMENT` |
| Output version or size is incompatible | `UNSUPPORTED_VERSION` |
| Outside the owning SDK UI callback | `WRONG_THREAD` |
| Token no longer matches the current context | `STALE_HANDLE` |
| Context is unready or the window is visible | `NOT_READY` |
| Older provider has no service or optional function | `UNKNOWN_SERVICE` |

On failure there is no valid result. The Rust wrapper returns
`Result<ResidencyReclaimResultV1, Error>`; `NOT_READY` is an error value rather
than a successful reclamation. Snapshot errors are also explicit. The wrapper
initializes and checks output headers, rejects invalid ready tokens, and never
falls back to calling an operating system function itself.

## Rust use from an SDK callback

Resolve and retain the service when connecting to the SDK. The following helper
is for a single explicit request from an owning UI callback. A plugin decides
when to invoke it; calling it on every tick would request paging repeatedly.

```rust,no_run
use stalkore_sdk_api::{Error, process_memory::*};

fn reclaim_current(
    memory: ProcessMemory,
) -> Result<Option<ResidencyReclaimResultV1>, Error> {
    let current = memory.snapshot()?;
    let required = FLAG_CONTEXT_READY | FLAG_WINDOW_HIDDEN;
    if current.capabilities & CAP_RESIDENCY_RECLAIM == 0
        || current.flags & required != required
    {
        return Ok(None);
    }
    memory.reclaim_residency(current.context_token).map(Some)
}
```

A transition can still invalidate the token between snapshot and request. Treat
that failure as no action; keep retry or once-per-context policy in the plugin.
The SDK does not save a token for a future context or change the window to make
the request succeed.

C consumers query `STALKORE_PROCESS_MEMORY_SERVICE`, check the complete
`StalkoreProcessMemoryApiV1` header, and initialize each output header to
`{sizeof(output), STALKORE_ABI_V1}`. The [C layout fixture](../tests/c_layout.c)
and [Rust mock tests](../tests/process_memory.rs) cover layouts and failures.
These fixtures exercise the contract without a runtime process or paging call.
