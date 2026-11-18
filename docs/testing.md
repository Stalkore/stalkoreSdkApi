# Contract checks

[Documentation](README.md)

Run compilers sequentially. These commands use synthetic providers and never
start a game, read account environments or send authentication requests.

```powershell
cargo fmt --all -- --check
cargo test --locked --jobs 1
cargo clippy --locked --all-targets --jobs 1 -- -D warnings
rustup target add i686-pc-windows-msvc
cargo test --locked --target i686-pc-windows-msvc --jobs 1
cargo clippy --locked --all-targets --target i686-pc-windows-msvc --jobs 1 -- -D warnings
cargo doc --locked --no-deps --jobs 1
```

Tests protect C structure sizes/offsets, table validation, status conversion,
logical client actions and scheduler callback/context forwarding. The private
SDK and adapter test their implementations separately. stalkoreAutoLogin additionally
uses a synthetic SDK DLL to verify that its production DLL needs only this
public contract. None of those checks establishes a live client login result.

Keep lockfiles, fixture source and the C header versioned. Exclude binaries,
logs, local instructions, environments, dumps and client material. Review the
staged source and outgoing commits before publication. New services require
documented IDs, version/layout tests and explicit lifetime/thread rules.

## Recorded checks — 2026-11-18

Three contract tests passed on Windows x86 and the Windows host target.
Formatting, Clippy with warnings denied and rustdoc passed. The C header also
passed freestanding C11 syntax and structure-size checks with GCC in 32-bit and
64-bit modes. These are contract checks; no game session was started.
