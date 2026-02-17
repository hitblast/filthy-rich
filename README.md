<div align="center">

<img src="./assets/art.png" width="120px" align="center">

# filthy-rich

Tiny Discord Rich Presence wrapper library for apps.<br>
**Only <500 LOC! Insanely tiny.**

</div>

> [!NOTE]
> Requires Rust 1.82.0 or later.

Add `filthy-rich` to your project with this command:

```bash
cargo add filthy-rich
```

## Features

- Really easy to implement; just create a client ID at the [Discord Developer Portal](https://discord.com/developers) and you're good to go.
- Asynchronous and enforces the tokio workflow.
- Persistent updates and convenient API.
- Auto-reconnect on failure.

## Starter Snippets

Examples are included with the project. To get, first clone the repository:

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
