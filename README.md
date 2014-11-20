# ffto: Firefox tab opener

Small Rust daemon listening on localhost:7777 and spawning new tabs in Firefox
when receiving correctly formated URLs (one per line). Everything else is
discarded.

## How to build

This is developed against Rust nightly builds.

```
$ cargo build
```

## How to run

```
$ cargo run
```

## License

Licensed under the MIT license, see LICENSE.
