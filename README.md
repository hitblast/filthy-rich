<div align="center">

<img src="./assets/art.png" width="120px" align="center">

# <img src="https://github.com/github/explore/blob/main/topics/rust/rust.png?raw=true" width="40px"> filthy-rich

**Tiny, ergonomic Discord Rich Presence library for your apps.**<br>

</div>

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

## API Reference

docs.rs: https://docs.rs/crate/filthy-rich/latest

## Yet another library?

I'm not a fan of too much unnecessary boilerplate hovering around the code I use for my primary projects,
so the primary goal for writing this library is to avoid Windows-specific binds.

The project is also **only intended for actual binary apps**, unless you have a matching stack to go with the library itself.

## License

Licensed under [MIT](./LICENSE).
