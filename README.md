# data-scraper

## Build (zig + cargo-zigbuild)

To build the project with MUSL target for static linking, first install [Zig](https://ziglang.org/download/) and then install `cargo-zigbuild`:

```bash
cargo install cargo-zigbuild
```

Build the project using:

```bash
DATABASE_URL=postgres://admin:1234@localhost/vehicles cargo zigbuild --release --bin crawler --features postgres --target x86_64-unknown-linux-musl
```

## Run examples

The CLI is simplified. Here are some example commands:

- Run with default settings:

```bash
cargo run --release --bin crawler
```

- Run with a different source:

```bash
cargo run --release --bin crawler --source some_source
```

- Run with Chrome enabled:

```bash
cargo run --release --bin crawler --enable-chrome
```

## Note

Make sure to set the `DATABASE_URL` environment variable appropriately before running the commands, for example:

```bash
export DATABASE_URL=postgres://admin:1234@localhost/vehicles
```