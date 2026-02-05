<img src="./assets/art.png" width="150px" align="right">

# filthy-rich

Tiny Discord Rich Presence wrapper library for apps.<br>
**Only <300 LOC! Insanely tiny.**

```bash
# Add to project!
cargo add filthy-rich
```

## Starter Snippets

Examples are included with the project. To get, first clone the repository:

1. For an indefinitely running rich presence, [see this](./examples/indefinite.rs).
2. For an timed, changing rich presence, [see this](./examples/timed.rs).

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

## TODO

- [ ] **(New)** Windows support
- [x] Timestamp-preserving code
- [ ] Global `Activity` struct for a well-defined Rich Presence schema

## License

Licensed under [MIT](./LICENSE).
