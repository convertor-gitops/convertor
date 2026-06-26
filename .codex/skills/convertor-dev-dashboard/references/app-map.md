# Dashboard App Map

## Main Structure

- `dashboard/src/app/service`: injectable services for metadata, env, encryption, ticker, URL construction, dashboard API, and status.
- `dashboard/src/app/common/model/core`: frontend domain models mirroring Rust core concepts.
- `dashboard/src/app/common/model/api`: backend API response models.
- `dashboard/src/app/common/response`: generic request/response/status wrappers.
- `dashboard/src/app/page/dashboard`: dashboard page and feature sections.
- `dashboard/src/app/page/shared`: reusable UI components.
- `dashboard/src/app/common/http`: captured exchange interceptor and HTTP exchange model.

## Build Metadata

- `dashboard/scripts/generate-metadata.js` generates dashboard metadata.
- `dashboard/src/version.ts` stores generated version information.
- `dashboard/METADATA.md` documents metadata behavior.
