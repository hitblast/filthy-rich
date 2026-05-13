<div align="center">

<img src="./assets/art.png" width="120px" align="center">

# <img src="https://github.com/github/explore/blob/main/topics/rust/rust.png?raw=true" width="40px"> filthy-rich

**Tiny, ergonomic Discord Rich Presence library for your Rust apps.**<br>

![Crates.io Total Downloads](https://img.shields.io/crates/d/filthy-rich)
![Crates.io MSRV](https://img.shields.io/crates/msrv/filthy-rich)
![Crates.io Size](https://img.shields.io/crates/size/filthy-rich)
![Crates.io License](https://img.shields.io/crates/l/filthy-rich)

</div>

```rust
//! A sneak-peek into what you will be working with:
//!
use filthy_rich::{PresenceRunner, errors::PresenceError, types::Activity};

#[tokio::main]
async fn main() -> Result<(), PresenceError> {
    let mut runner = PresenceRunner::new("1463450870480900160");

    let activity = Activity::new()
        .name("cool app name")
        .details("Something?")
        .state("Probably~")
        .build()?;

    let client = runner.run(true).await?;
    client.set_activity(activity).await?;

    // indefinitely block here
    runner.wait().await?;

    Ok(())
}

```

### Bulletin

> [!WARNING]
> Even though this library follows most of Discord's spec-sheet, some features which are not even implemented in Discord itself but were included in the documentation have been skipped for a smoother experience.

> [!NOTE]
> Requires Rust 1.85.0 or later (MSRV).

## Getting Started

Add `filthy-rich` to your project with this command:

```bash
cargo add filthy-rich
```

Then, you can start writing code by reviewing one of these:

#### Starter Snippets

Examples are included with the project. See these:

1. For an indefinitely running, elaborate rich presence, [see this](./examples/indefinite.rs).
2. For an timed, changing rich presence, [see this](./examples/timed.rs).
3. For a very simple snippet, [see this](./examples/simple.rs).

To run these:

```bash
# First, clone and `cd` into the repository.
git clone https://github.com/hitblast/filthy-rich && cd filthy-rich

# Now, run any of the examples:
cargo run --example indefinite  # ex. 1
cargo run --example timed  # ex. 2
cargo run --example simple  # ex. 3
```

## 🌺 Features

- Really easy to implement; just create a client ID at the [Discord Developer Portal](https://discord.com/developers) and you're good to go.
- Fruitful `Activity` builder with a *type-state builder pattern* - guaranteed to make you fall in love with setting presences.
- Ergonomic `on_ready`, `on_activity_send` and other event registers to work with are *all included*.
- 100% coverage of the usable spec of Discord RPC (presence-related only).
- Fully asynchronous but easily wrappable for synchronous usage.
- *Client-runner architecture* for easy use in multithreaded apps or contexts.
- Automatically reconnects on disconnect, making your presence persist for prolonged periods of time.
- (TODO) Supports optional auth flow for elevated privileges.

## API Reference (docs.rs)

https://docs.rs/filthy-rich/latest/filthy_rich/

## TODO

- [x] Full Rich Presence coverage.
- [ ] Full RPC-over-IPC coverage (as provided by Discord).
- [ ] New commands for subscribing to events, sending channel/guild commands etc.

## Off-Topic Literature

#### Yet another library?

I did not want to bother myself with manually implementing Rich Presence everytime I start working on an app, so I created this library to make things much simpler; I just wanted a client that does its job in the background.

Also, other implementations felt much more complex to me and also felt like they lacked precise control. This is a more "spread-out" opinion and might hide the truth for some libraries, but yeah, nothing better than throwing your own luck into making yet another IPC RPC client.

#### Inspirations

An attempt to make an eye-candy syntax for this library was inspired by [discord.py](https://github.com/Rapptz/discord.py).

## Contributing

If you want to contribute to this project, be sure to follow the [contribution guidelines](./CONTRIBUTING.md) and prep your environment as mentioned before doing so. Thank you for showing your interest!

If you've found a bug, be sure to report it in the [GitHub Issues](https://github.com/hitblast/filthy-rich/issues/) section for this repository. Reporting actually makes it a lot easier to patch any issues.

#### Contributors

Amazing people adding amazing stuff to this library:

- [Sirokovsk](https://github.com/Sirokovsk)
- [Sreehari425](https://github.com/Sreehari425)

## Changelog

For a comprehensive release changelog of this library, please refer to [CHANGELOG.md](./CHANGELOG.md). The release changelogs are compiled from
the core changelog as a part of the release CI.

## License

Licensed under [MIT](./LICENSE).
