# Agents.md

This repository is a Rust `egui`/`eframe` desktop-and-web application named `eprice`.
The codebase is UI-first: keep changes aligned with the existing `egui` architecture and the in-memory service layer unless the user explicitly asks for persistence or backend work.

## Project Summary

- App entry points:
  - Native: [`src/main.rs`](src/main.rs)
  - Shared app/controller: [`src/app.rs`](src/app.rs)
  - Public module surface: [`src/lib.rs`](src/lib.rs)
- Main domain areas:
  - Stores and products
  - Search and filters
  - Authentication UI
  - Alerts and notifications
  - Scanner UI and product matching
  - Settings, verification, OCR, and async ops

## Working Rules

- Do not change existing crate versions in `Cargo.toml`.
- Do not introduce backend/server work unless the user explicitly requests it.
- Prefer small, local edits that fit the current module boundaries.
- After every code change, run `cargo check` until it passes.
- Do not revert unrelated user changes.
- Keep edits ASCII unless the file already uses non-ASCII text.

## UI Rules For `egui`

- Preserve the existing `egui`/`eframe` style and structure in this repo.
- Keep UI logic inside the relevant tab/component instead of spreading it across unrelated modules.
- Favor `egui`-native widgets, panels, windows, and tables over custom abstractions unless needed.
- If you add a new screen or tab, wire it through `TemplateApp` in [`src/app.rs`](src/app.rs).
- If you change desktop/web behavior, verify `#[cfg(target_arch = "wasm32")]` and native-only code paths separately.

## Architecture Map

- `src/app.rs`
  - Owns the `TemplateApp` controller.
  - Holds tab state, sample data, map state, auth UI, alert UI, and service aggregation.
  - Routes rendering for Stores, Products, Scanner, Alerts, Trends, Community, and Settings.
- `src/models.rs`
  - Core shared data models such as `User`, `Product`, `Store`, `PriceRecord`, `PriceAlert`, and review/OCR types.
- `src/services/`
  - In-memory business logic services for users, products, stores, prices, and reviews.
- `src/auth/`
  - Authentication state, session handling, and auth dialogs.
- `src/alerts/`
  - Price alert monitoring and notifications.
- `src/search/`
  - Search engine, filters, and search UI.
- `src/scanner/`
  - Native-only camera, barcode decoding, product matching, and scanner UI.
- `src/settings/`
  - App configuration and settings UI.
- `src/database/`
  - SQLite connection and repository scaffolding for native builds.
- `src/verification/`
  - Price verification workflow built on top of the price service.
- `src/utils/`
  - Validation, crypto, file helpers, notification helpers, and formatting utilities.

## Implementation Notes

- Sample stores/products are currently seeded into the app and services during initialization.
- The database path is native-only and initialization should remain guarded by `#[cfg(not(target_arch = "wasm32"))]`.
- The scanner feature is native-only; keep the wasm fallback message intact.
- Some parts of the app are scaffolded or partially implemented, so confirm whether a requested change should preserve sample-data behavior.

## Suggested Workflow

1. Inspect the relevant module(s) first.
2. Make the smallest consistent change.
3. Update any wiring in `src/app.rs` if the feature is visible in the main UI.
4. Run `cargo check`.
5. If the change affects UI flow, inspect both native and wasm code paths when applicable.

## Notes For Future Agents

- Prefer existing service and model types instead of creating duplicate structures.
- Keep public APIs stable unless the task explicitly asks for a refactor.
- If a request touches a feature that is currently sample-data driven, call that out in the response.

