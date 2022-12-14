# dlpi-sys

This pair of crates provides [libdlpi](https://illumos.org/man/3LIB/libdlpi)
functionality.

- [Documentation](https://oxidecomputer.github.io/dlpi-sys/dlpi/index.html)

Idiomatic Rust interfaces are provided in [the `dlpi` crate](dlpi/src/lib.rs).
System-level interfaces are provided in [the `libdlpi-sys`
crate](libdlpi-sys/src/lib.rs).

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
