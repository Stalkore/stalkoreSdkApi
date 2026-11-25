# Client update maintenance

[Home](../README.md) · [ABI and lifetime](abi.md) · [Tests](testing.md)

The SDK API contains logical types and versioned service tables. It contains no
client hashes, addresses, native methods or profiles. A client executable update
therefore does not require an API revision when the existing logical operations
still describe the required behavior.

Update this repository only for a deliberate contract change. Add fields by a
new compatible table/version, preserve existing ownership and status rules, run
contract/mock tests, then update consumer Git pins explicitly. Do not expose an
address or client-build detail to avoid updating the private adapter.

```powershell
Set-Location 'D:\GitHub\Stalkore\stalkoreSdkApi'
cargo fmt --all -- --check
cargo test --locked --jobs 1
cargo clippy --locked --all-targets --jobs 1 -- -D warnings
```

For an executable-only update, leave the API pin unchanged, update the private
adapter and SDK, and rerun plugin integration tests against the existing ABI.
