## Changelog

Active since v0.8.5.

### v0.10.0

New / breaking features:

- Add: `details_url` and `state_url` fields for `Activity`, `ActivityBuilder`, and other internal structs.
- Add these new methods to `ActivityBuilder`:
  - `details_url()`
  - `state_url()`
  - `large_text()`
  - `large_url()`
  - `small_text()`
  - `small_url()`
- Modified the signatures of these builder functions within `ActivityBuilder` to adapt with the new methods added above:
  - `details()`
  - `state()`
  - `large_image()`
  - `small_image()`

Bug fixes:

- Fixed incorrect serialization length for the `AssetsPayload` struct.

Internal changes:

- Documented more of the previously undocumented parts of the API.
- Implemented the `Default` trait for `Activity` for creating empty activities (previously known as `Activity::build_empty`).

### v0.9.0

New features:

- Added: `ActivityBuilder::set_as_instance`
- (Experimental) Added: `PresenceRunner::on_activity_send` which receives a closure to execute whenever an activity is sent through the connection. Returns `ActivityResponseData` within the closure.

Internal changes:

- Changed `do_verbose_errors` to `show_errors` inside `PresenceRunner`.
- Removed unnecessary code from `DiscordSock::close` which was there for nothing in Windows-based targets.

Bug fixes:

- Fixed a classic "compute but forget to store" bug with `session_start` that led to new `start` timestamps every time.

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
