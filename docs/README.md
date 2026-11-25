# Documentation

[Home](../README.md)

## Learn the contract

Read [ABI and lifetime](abi.md) before following a service call. Then trace
[`SdkApi::connect`](../src/api.rs) → [`SdkApi::table`](../src/api.rs) →
one wrapper in [`client`](../src/client.rs) or
[`scheduler`](../src/scheduler.rs). Ask which side owns the table, callback
context, byte buffers and thread. The C header and Rust types must describe
the same ABI; a client executable update alone does not change it.

## Reference guides

1. [Integration](integration.md): plugin startup and a scheduled callback.
2. [ABI and lifetime](abi.md): versions, ownership and thread rules.
3. [Client services](client.md): observations and guarded actions.
4. [Tests](testing.md): reproducible contract checks and their limits.
5. [Glossary](../GLOSSARY.md): names used across the repositories.
6. [Presentation](presentation.md): optional window visibility without native handles.
7. [Frame pacing](frame-pacing.md): frame limits, measurements and shared-thread timing.
8. [Process memory](process-memory.md): counters, opaque context tokens and explicit residency requests.
9. [Client update maintenance](client-updates.md): keep build-specific details in the adapter and revise the ABI only deliberately.
