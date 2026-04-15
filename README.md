<div align="center">

<img src="./assets/art.png" width="120px" align="center">

# <img src="https://github.com/github/explore/blob/main/topics/rust/rust.png?raw=true" width="40px"> filthy-rich

**Tiny, ergonomic Discord Rich Presence library for your Rust apps.**<br>

![Crates.io MSRV](https://img.shields.io/crates/msrv/filthy-rich)
![Crates.io Size](https://img.shields.io/crates/size/filthy-rich)
![Crates.io License](https://img.shields.io/crates/l/filthy-rich)

</div>

```rust
// a sneak-peek into what you'll be working with
// this usually helps me personally when looking at libraries
use anyhow::Result;
use filthy_rich::{Activity, PresenceRunner};

#[tokio::main]
async fn main() -> Result<()> {
    let mut runner = PresenceRunner::new("1463450870480900160");

    let activity = Activity::new()
        .details("Playing a game")
        .state("In menu")
        .large_image("game_icon", Some("My Game"))
        .small_image("status", Some("Online"))
        .build();

    let client = runner.run(true).await?;
    client.set_activity(activity).await?;
    runner.wait().await?;

    Ok(())
}

```

> [!WARNING]
> All development work happens in the [Codeberg repository](https://codeberg.org/hitblast/filthy-rich); the GitHub mirror is primarily for *receiving* pull requests from the broader spectrum of the userbase.

> [!NOTE]
> Requires Rust 1.82.0 or later.

Add `filthy-rich` to your project with this command:

```bash
cargo add filthy-rich
```

## 🌺 Features

- Really easy to implement; just create a client ID at the [Discord Developer Portal](https://discord.com/developers) and you're good to go.
- Fruitful `Activity` builder guaranteed to make you fall in love with setting presences.
- Support for rich presence assets (large and small images with hover text).
- Clean whilst being easy; properly handles Discord's responses.
- Fully asynchronous, but can be easily wrapped upon for synchronous compatibility.
- Auto-reconnect on failure.

## Starter Snippets

Examples are included with the project. See these:

1. For an indefinitely running rich presence, [see this](./examples/indefinite.rs).
2. For an timed, changing rich presence, [see this](./examples/timed.rs).

To run these:

```bash
# First, clone and `cd` into the repository.
git clone https://github.com/hitblast/filthy-rich && cd filthy-rich

# Now, run any of the examples:
cargo run --example indefinite  # ex. 1
cargo run --example timed  # ex. 2
```

## API Reference (docs.rs)

https://docs.rs/crate/filthy-rich/latest

## Yet another library?

I don't want to bother myself with manually implementing Rich Presence everytime I start working on an app, so I created this library to make things much simpler; I just want a client that does its job in the background.

Also, other implementations felt much more complex to me and also felt like they lacked precise control. This is a more "spread-out" opinion and might hide the truth for some libraries, but yeah, nothing better than throwing your own luck into making yet another IPC RPC client.

## Contributors

Thanks to the amazing contributors for adding to this repository:

- [Sirokovsk](https://github.com/Sirokovsk)

## License

Licensed under [MIT](./LICENSE).
