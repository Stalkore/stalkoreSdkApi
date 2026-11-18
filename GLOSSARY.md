# Stalkore client services

The shared language of plugins and the client integration supplied by the SDK.

## Language

**Client adapter**: The implementation that translates supported client behavior
into the services available to plugins.
_Avoid_: Plugin, SDK host.

**Client snapshot**: An observation of the client's current connection and
interaction state at one point in time.

**Client token**: An opaque identity for a current client object. It expires when
that object is replaced.
_Avoid_: Address, pointer.

**Owning UI thread**: The client's thread responsible for its windows and native
interactions.

**Login policy**: The plugin's decisions about attempts, waiting, character
selection, PIN attempts and recovery.
