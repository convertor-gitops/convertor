# Dashboard Validation

Run commands from `dashboard/`.

## Common Checks

```bash
npm test
npm run build
```

## Focused Checks

```bash
npm run build
```

Use build for TypeScript/template/style integration. Use tests when changing model serialization, services, or component behavior covered by specs.

## Cross-Rust Checks

If dashboard changes depend on Rust query or API contracts, run the relevant Rust tests from `convertor-dev-core`.
