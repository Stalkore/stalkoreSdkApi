# Presentation service

[Documentation](README.md) · [ABI and lifetime](abi.md)

Service 3 is an optional, separate v1 table. Connect with
`SdkApi::presentation()` and call its methods only in a READY/TICK SDK callback.
Older runtimes return `UNKNOWN_SERVICE`; client and scheduler tables are unchanged.

`snapshot()` returns supported capabilities and whether the main window is visible.
`CAP_WINDOW_VISIBILITY` permits `set_visible(false)` and `set_visible(true)`.
Showing does not request foreground activation. The SDK validates ownership and
performs the operation on the window thread. A missing/replaced window or wrong
thread returns an error rather than selecting an auxiliary window.

Hiding does not suspend the process or establish any reduction in rendering, CPU,
private allocations or working set. This version exposes no renderer suppression,
cache eviction, memory trimming or simulation throttling. Those require separate
capabilities and runtime acceptance; plugins must not infer them from visibility.

The table is process-local, retains SDK lifetime rules and exposes no native
window handles, addresses or client-specific structures. The caller initializes
the snapshot's header; providers validate size/version before writing it.
