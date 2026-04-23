## Changelog

Active since v0.8.5.

### v0.8.5

New features:

- Add `PresenceRunner::show_errors()` which can be used to enable verbose error logging for the runner.
- `ActivityBuilder::small_image` and `ActivityBuilder::large_image` now have new `url` fields for them.

Improvements of existing features:

- Removed even more unwraps.
- When sending an `Activity` over an `IPCCommand` message, it is now wrapped in a `Box` first.
- The internal `AssetsPayload` now has a custom `Serialize` implementation which ensures that `*_url` and `*_text` fields aren't serialized if the `*_image` fields are missing in the first place. This is more of an extra, redundant layer of safety to protect against invalid objects being passed down the IPC RPC pipeline.
- Fixed more `clippy` errors.

Bug fixes:

- Added a new internal `DynamicRPCFrame` type for the generic frame read loop, which fixes s bug that lead to invalid deserialization of frames.
