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

#[tokio::main]
async fn main() -> Result<()> {
    let mut client = DiscordIPC::new("1463450870480900160")
        .on_ready(|data| println!("Connected to: {}", data.user.username));

    client.run(true).await?;

    let activity = Activity::build_empty();

    client.set_activity(activity).await?;
    client.wait().await?;

    Ok(())
}
```

> [!WARNING]
> This library is still a work-in-progress, so expect sudden breaking changes.

> [!NOTE]
> Requires Rust 1.82.0 or later.

Add `filthy-rich` to your project with this command:

```bash
cargo add filthy-rich
```

## 🌺 Features

- Really easy to implement; just create a client ID at the [Discord Developer Portal](https://discord.com/developers) and you're good to go.
- Fruitful `Activity` builder guaranteed to make you fall in love with setting presences.
- Clean whilst being easy; properly handles Discord's responses.
- Asynchronous and synchronous backends available according to your use case.
- Auto-reconnect on failure.

## Starter Snippets

Examples are included with the project. See these:

1. For an indefinitely running rich presence, [see this](./examples/indefinite.rs) (or see the [blocking/sync version](./examples/indefinite-sync.rs)).
2. For an timed, changing rich presence, [see this](./examples/timed.rs) (or see the [blocking/sync version](./examples/timed-sync.rs)).

You can also run them to see how they work.

```bash
# First, clone and `cd` into the repository.
git clone https://github.com/hitblast/filthy-rich && cd filthy-rich

# Run any of the examples:
cargo run --example indefinite  # ex. 1
cargo run --example timed  # ex. 2
```

## API Reference (docs.rs)

https://docs.rs/crate/filthy-rich/latest

## Yet another library?

I don't want to bother myself with manually implementing Rich Presence everytime I start working on an app, so I created this library to make things much simpler; I just want a client that does its job in the background.

Also, other implementations felt much more complex to me and also felt like they lacked precise control. This is a more "spread-out" opinion and might hide the truth for some libraries, but yeah, nothing better than throwing your own luck into making yet another IPC client.

## Contributors

Thanks to the amazing contributors for adding to this repository:

- [ParkBlake](https://github.com/ParkBlake)

## License

Licensed under [MIT](./LICENSE).
