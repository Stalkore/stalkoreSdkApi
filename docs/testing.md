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
gcc -std=c11 -Wall -Wextra -Werror -m32 -fsyntax-only tests/c_layout.c
gcc -std=c11 -Wall -Wextra -Werror -m64 -fsyntax-only tests/c_layout.c
```

The C commands require a GCC installation supporting the selected architecture.
With a 32-bit-only GCC, run the 64-bit check from an MSVC x64 Developer Prompt
instead: `cl /nologo /TC /std:c11 /W4 /WX /Zs tests/c_layout.c`.

Tests protect C structure sizes/offsets, table validation, status conversion,
logical client actions and scheduler callback/context forwarding. The private
SDK and adapter test their implementations separately. stalkoreAutoLogin additionally
uses a synthetic SDK DLL to verify that its production DLL needs only this
public contract. None of those checks establishes a live client login result.

Keep lockfiles, fixture source and the C header versioned. Exclude binaries,
logs, local instructions, environments, dumps and client material. Review the
staged source and outgoing commits before publication. New services require
documented IDs, version/layout tests and explicit lifetime/thread rules.

The process-memory fixtures verify v1 counters/results, null and undersized
outputs, old-provider rejection, stale tokens, visible/unready contexts, and
wrong-thread failures. They make no paging calls and establish no memory-saving
result. The C fixture protects the new layouts alongside services 1–4.

## Recorded checks — 2026-11-18

Three contract tests passed on Windows x86 and the Windows host target.
Formatting, Clippy with warnings denied and rustdoc passed. The C header also
passed freestanding C11 syntax and structure-size checks with GCC in 32-bit and
64-bit modes. These are contract checks; no game session was started.

## Recorded checks — 2026-11-19

All 11 Rust contract tests passed on Windows host and x86, including six
process-memory fixtures. Both Clippy runs denied warnings; formatting, rustdoc,
and the new guide's Rust example passed. The C layout fixture passed with x86
GCC and x64 MSVC. No runtime paging operation was part of these checks.
