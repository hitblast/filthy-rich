## filthy-rich

Tiny Discord Rich Presence wrapper for Unix.

**Only <300 LOC! Insanely tiny.**

### Starter Snippets

These can be found in `examples/`:

- For an indefinitely running rich presence, [see this](./examples/indefinite.rs).
- For an indefinite but changing rich presence, [see this](./examples/timed.rs).

### Add to your project

```bash
cargo add filthy-rich
```

### Yet another library?

I'm not a fan of too much unnecessary boilerplate hovering around the code I use for my primary projects,
so the primary goal for writing this library is to avoid Windows-specific binds.

### License

Licensed under [MIT](./LICENSE).
