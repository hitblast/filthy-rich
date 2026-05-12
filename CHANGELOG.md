## Changelog

Active since v0.8.5.

### v1.0.4

New features:

- Added `PresenceRunner::can_connect` for checking if a Discord client is available for connection. This works without the initialization of `PresenceRunner`.
- Added two new type aliases:
  - `RPCRunner` (alias for `PresenceRunner`)
  - `RPCClient` (alias for `PresenceClient`)

### v1.0.3

Internal changes:

- Added a new `retry!` macro for minimizing some duplicate code related to incrementing the retries counter and executing the passed closure from `on_retry`.
- Added a new `str!` macro globally which aids with generating functions returning `&str` from `String` fields (yeah, I'm lazy).

### v1.0.2

Changes:

- Removed exponential backoff as it was largely unused. It has been replaced with a `RETRY_DELAY` constant with the value being 1 second.

Internal changes:

- `Opcode` is now `PartialEq`-compatible and is directly parsed with `read_frame()` as a part of the `Frame` object.
- Made `DiscordSock::write` private as `send_frame` is used throughout most of the code, and bare-bones writes are risky.
- Removed the internal `connect` flag.

### v1.0.1

This release primarily focuses on optimizations.

Changes:

- Closures passed in with `on_*` event functions now spawn those functions in a separate thread for leaving no obstructions in the inner loop.

Internal changes:

- The `tokio::select!` for receiving commands and reading frames is now unbiased again, giving equal priority to both of them.

### v1.0.0

Finally, a stabilized API for `filthy-rich`!

New features:

- Added a new `ActivitySpec` type which basically replaces the current architecture of a mutable `Activity` to a post-built, immutable object which can be sent around.
- Added `Activity::empty_spec` which gives an empty but usable `ActivitySpec` (basically executes `ActivitySpec::default` internally).
- `ActivityResponseData` now includes an `ActivityPayload` object (flattened internally) instance for you to directly see which activity details were sent.
- Completed the `ReadyData` struct with these changes:
  1. New fields in `ReadyData`:
    - `v` - RPC version (`u8`)
    - `config` - server configuration (a new `ServerConfigurationData` object)
  2. The inner `user` field (with type `DiscordUser`) now has the complete specification of struct fields documented in the official Discord RPC documentation.

Changes:

- `ActivityBuilder::build` now returns `Result<ActivitySpec, ActivitySpecBuildError>` with all of its niceties (guardrails for you to bring your Rich Presence to life with safety).
- All of the asset keys/values/URLs which can be passed through the builder functions are now `None`-filtered.
- `PresenceRunner::new` now accepts any type of value for `client_id` as long as it implements `Into<String>`.
- `ActivityResponseData` now only gives out borrowed read-only data through functions (e.g. `ActivityResponseData::name` now returns `Option<&str>`).
- The change above also has been done to `ReadyData`.
- Removed unused `derive` macros from `Activity` since its now just a placeholder struct for accessing `ActivityBuilder` with no values attached to it.
- A lot of improvements to the documentation for various items in the library!

Bug fixes:

- Fixed a fatal bug in ping-pong logic which led to the sending of frames with `PING` and not `PONG` in response to `PING`.

Internal changes:

- Added an idiomatic `Opcode` enum and bettered code quality for matching opcodes in the `socket.read_frame()` loop inside of `PresenceRunner`.
- Removed the use of `Arc` from the inner `readhalf` and `writehalf` fields of `DiscordSock` since the main loop doesn't cross thread boundaries.
- Added a new `DisconnectReason::OldRelicComputer` error.
- `ActivityPayload`, `TimestampPayload`, `ButtonPayload` and `AssetsPayload` are all public structs now. Their fields can be accessed using functions which point to borrowed data.
- The `types` modules is now separated into many other modules internally.
- Optimized some reusable syntax for `PresenceRunner` with macros for a better developer experience.
- Removed the `acquire!()` macro from `DiscordSock`.

### v0.14.1

Internal changes:

- Removed unsafe buffer init from the internal `DiscordSock::read_frame` function and implement a `MAX_FRAME_SIZE`, which, if crossed, returns a `DiscordSockError::PayloadTooLarge` error from the function (thanks to @Sreehari425) [(see pull request)](https://github.com/hitblast/filthy-rich/pull/6).

### v0.14.0

New features:

- Added `PresenceRunner::on_retry` for accepting a closure to execute during socket creation / handshake retries. The returned `usize` value (through the closure parameter) indicates the amount of total retries done at the time of the closure's execution.
- Added `PresenceRunner::set_max_retries` for enabling developers to set a custom retry count for socket creation / handshakes after which the runner instance should give up on connecting to Discord. By default this is set to `0` (indefinite) internally.

Bug fixes:

- Fixed a bug which had prevented the closure passed through `on_activity_send` from firing due to improper fields in arRPC.

API changes:

- `PresenceClient::client_id` now returns an `&str` instead of a cloned `String`.
- All fields for the `ActivityResponseData` struct have been made optional.

Internal changes / improvements:

- Use `BytesMut`/`Bytes` instead of standard `Vec<u8>` in places (added a new `bytes` crate to the dependencies list).
- Removed multiple writes from `send_frame` and instead merged them into a single logical write.
- Removed the `json!()` macro call from `DiscordSock::do_handshake` and replaced it with `PresenceHandshake`.
- Removed unused derive macros from several structs in the `types` module.
- Minor optimizations have been done to `ActivityCommand`.

### v0.13.2

Improvements:

- Much smaller `tokio` bundle (removed the `full` feature set in favor of a more intentional list of features).

### v0.13.1

Bug fixes:

- Fixed fatal flaw in `socket` module leading to compilation failures due to old code related to `anyhow` instead of adopting the new `DiscordSockError` for Windows-based builds.

### v0.13.0

I don't know for how long I would be able to trust GitHub with my personal projects since its becoming so unstable with each passing day. Its quite uncertain - but for now, I guess you can enjoy these releases here. :3

New features:

- Explicit error enums have been added with the new `errors` module - featuring `PresenceError` (unified), `PresenceClientError` and `PresenceRunnerError`, as well as some other internal error types for sockets.

Internal changes:

- Replaced `anyhow` in the dependencies list with `thiserror`.

### v0.12.0

New features:

- Added `on_disconnect` hook with a new `DisconnectReason` enum to compliment it (thanks to @Sreehari425) [(see pull request)](https://github.com/hitblast/filthy-rich/pull/5).

### v0.11.3

Improvements:

- Added support for `snapd` environments (thanks to @Sreehari425) [(see pull request)](https://github.com/hitblast/filthy-rich/pull/4).

Bug fixes:

- More MSRV-specific (1.85.0) syntax fixes.

### v0.11.2

Bug fixes:

- Fixed critical compilation errors related to unstable features being pushed despite the MSRV being `1.85.0`.

### v0.11.1

Internal changes:

- Updated the MSRV for the crate to v1.85.0.
- Basic internal syntax changes.
- Fix documentation for some crate identities.

### v0.11.0

Breaking changes:

- Removed `PresenceClient::running` (correlates to the first change listed in "Internal changes").

Improvements of existing features:

- `PresenceClient::close` now blocks execution till the runner thread responds with a message.

Internal changes:

- Removed the `running` field from `PresenceClient` entirely; `mpsc` is now being used as the only source of truth for all inter-thread communications and executions.

### v0.10.0

New / breaking features:

- Added: `details_url` and `state_url` fields for `Activity`, `ActivityBuilder`, and other internal structs.
- Added these new methods to `ActivityBuilder`:
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

- Added `PresenceRunner::show_errors()` which can be used to enable verbose error logging for the runner.
- `ActivityBuilder::small_image` and `ActivityBuilder::large_image` now have new `url` fields for them.

Improvements of existing features:

- Removed even more unwraps.
- When sending an `Activity` over an `IPCCommand` message, it is now wrapped in a `Box` first.
- The internal `AssetsPayload` now has a custom `Serialize` implementation which ensures that `*_url` and `*_text` fields aren't serialized if the `*_image` fields are missing in the first place. This is more of an extra, redundant layer of safety to protect against invalid objects being passed down the IPC RPC pipeline.
- Fixed more `clippy` errors.

Bug fixes:

- Added a new internal `DynamicRPCFrame` type for the generic frame read loop, which fixes s bug that lead to invalid deserialization of frames.
