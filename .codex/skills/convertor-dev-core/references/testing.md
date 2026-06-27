# Core Validation

Choose the smallest useful validation set.

## Common Rust Checks

```bash
cargo test -p convertor
cargo test -p convd
cargo test -p confly
cargo test -p fetcher
```

## Compile-Only Checks

```bash
cargo check -p convertor
cargo check -p convd
cargo check -p confly
cargo check -p fetcher
```

## Snapshot Handling

- Use failing insta output to understand behavior changes.
- Review snapshots as contract changes.
- Do not accept snapshots as a mechanical cleanup step without checking the semantic diff.

## Cross-Frontend Checks

If Rust contract changes dashboard behavior, also run dashboard validation from `convertor-dev-dashboard`.
