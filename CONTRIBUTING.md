## Contribution Guidelines

So, you want to contribute to `filthy-rich`. It's very simple from a technical perspective, considering you have already shown a will to have a look at the guidelines to start adding your own features to it.

It is expected that you know the following things at a basic level:

- Rust concurrency:
  - `tokio` in general (`tokio::select!` loops, async I/O)
  - multithreaded-flavored `tokio` (e.g. sharing `oneshot` signals and data pointers between threads)
  - knowing when to stop using Rust concurrency
- Formatting:
  - `cargo clippy` and `cargo fmt` to keep up with the generic styling format
- Rust toolchains:
  - This crate is compatible with multiple platforms and toolchains (different versions of Rust), so, you may sometimes need to test your features via cross-compilation before you even hit the "Submit Pull Request" button.

### Getting Started

The MSRV (Minimum Supported Rust Version) for this crate is set to `v1.85.0`, and we have a convenient toolchain file to complement this. To get started, run these commands in order:

```bash
# clone the repo
git clone https://github.com/hitblast/filthy-rich.git

# cd into it
cd filthy-rich

# install the required toolchain
rustup toolchain install
```

### The Architecture

`filthy-rich` follows a **client-runner architecture** intead of a monolith struct to ease the complexity of implementing it in multithreaded apps. One such example is [QuantumLauncher](https://github.com/Mrmayman/quantumlauncher) which uses `iced` and it implements `filthy-rich` while following these principles:

- `PresenceRunner` is initialized (and dropped, partially) at the start of execution (if presence is toggled ON).
- The returned `PresenceClient` after executing `PresenceRunner::run` is stored in `Launcher` as a separate field.
- Later, `PresenceClient::set_activity` is called across different parts of the codebase which correlate to `Launcher`.
- Additionally `discord_presence_state` (a custom `PresenceConnectionState` enum) is used for reactive state in the UI outside of the core crate functions.

This is just an example and it could become simpler or harder depending on your use case.

In short:

- `PresenceRunner` hosts the core loop and its `JoinHandle` to provide a fixed interface to the IPC connection.
- `PresenceClient` is used to communicate with the core loop executed by `PresenceRunner` in the form of messages and oneshot signals.

### A Note on Undocumented Items

Sometimes I leave internal APIs (not re-exported to the outside world) undocumented only for the sake of spotting them easily. In that case, I often times write the code in such a way that it doesn't necessarily *need* any documentation to explain itsef.

Maybe have it in mind when contributing to this library. :3

### Checking the lints

A script is included with the project for you to check the project's linting status (syntax + docs):

```bash
chmod +x ./scripts/check-lint.sh

./scripts/check-lint.sh
```

For submitting pull requests, a GitHub Actions workflow can also be run for checking the lint status, but it is strongly recommended you check it before you make the PR available.
