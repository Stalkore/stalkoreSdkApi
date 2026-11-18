# Client service

[Documentation](README.md) · [ABI](abi.md)

`probe` reports capabilities and the account-name capacity including its NUL
terminator. No supported capability means the caller must stop automation.
Snapshot/action access becomes ready only after the SDK validates its adapter
and installs the owning-thread dispatcher. No client memory addresses or native
message IDs are exposed by this API.

`ClientSnapshotV1` describes mode, normalized phase, progress revision, flags,
server count and a classified dialog. Tokens identify the observed mode/dialog
and expire when the SDK observes their replacement. Revisions change on native
phase progress even when several native stages normalize to the same logical
authentication phase. Revisions are wrapping change markers, not timestamps.
Consumers compare them for equality rather than ordering.

| Field/flag | Meaning |
| --- | --- |
| `mode` | None, login, map or another mode |
| `phase` | Logical account prompt, account selection, credentials, authentication, servers, characters or map |
| `LOGIN_STAGE_VALID` | A login stage was readable; unknown logical phase can still be valid progress |
| `LOGIN_READY` | Current login object and native method guards match |
| `SERVER_COUNT` | `server_count` was read; consumers must still validate index bounds |
| `SSO_IDLE` | No outstanding native SSO request; required before retry/reset |
| `MODAL` | A native modal window is present |
| `MAP_DISCONNECTED` | Connection state confirms a closed map connection |
| `PIN_PRESENT` / `PIN_READY` | Existing PIN UI present / eligible for empty four-digit entry |
| `dialog_kind` | Known disconnect or connection-failure class; independent of localized text |

Actions open/select the account, request native login, select a server/character,
enter a logical PIN key, acknowledge a classified dialog or return a stalled
login to its native start. They request the client's existing flow rather than
constructing a second authentication connection. Server and character indices
are zero-based. Character selection checks an occupied, non-deleting record.

`pin_ready(mode, count)` and `pin_key(mode, key, count)` validate the existing PIN
dialog, expected input count and enabled controls. Digits are 0–9; key 10 means
confirm and requires count 4. The adapter handles randomized keypad layout.
Plugins own attempt limits: a partially entered PIN must not be submitted again
automatically. PIN creation, change and edit flows are outside this contract.

`acknowledge_dialog(token, connection_failed)` requires a fresh classified token.
The context boolean is encoded as 0/1 on the C boundary. A generic connection
failure notice is recoverable only in a failed login/connection context; normal
map notices must remain open. Acknowledgment and return-to-login can retire
windows and modes. Login policy, retry budgets and delays remain in consumers.
