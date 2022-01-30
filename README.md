# dlpi-sys

This is a crate that provides [libdlpi](https://illumos.org/man/3LIB/libdlpi)
functionality.

- [Documentation](https://oxidecomputer.github.io/dlpi-sys/dlpi/index.html)

Idiomatic Rust interfaces are provided in [the top level module](src/lib.rs).
System-level interfaces are provided in [the sys sub-module](src/sys.rs).

For async clients there is a `recv_async` variant of `recv` that returns an
awaitable future.

## Contributing

### Basic Checks

```
cargo fmt -- --check
cargo clippy
```

### Testing

```
./run_tests.sh
```
