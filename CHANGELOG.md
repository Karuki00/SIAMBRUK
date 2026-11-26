# Changelog

All notable project changes are recorded here.

## Unreleased

Reserved for changes planned after `0.1.0`. Add new entries here first, then move them into the next numbered release section when publishing.

### Added

<!-- Add future additions here. -->

### Changed

<!-- Add future changes here. -->

### Fixed

<!-- Add future fixes here. -->

## 0.1.0 - 2026-11-26

### Added

- Initialized the Tauri 2 + Vue 3 + TypeScript + Vite application in `tauri-app/`.
- Installed frontend dependencies and generated the Rust dependency lockfile.
- Added the Vue Router application shell with navigation for all planned business modules.
- Added a restrained SIAMBRUK dashboard foundation with explicit empty states and the fixed SMART criteria weights.
- Added Tailwind CSS, Pinia, Vue Router, and Lucide icon dependencies.
- Added SQLite initialization in the Tauri backend with an ordered initial migration.
- Added normalized tables for categories, products, variants, stock movements, purchases, sales, settings, and SMART evaluation snapshots.
- Added the `database_status` command and connected dashboard counts to the local database.
- Added category and product list/create commands with validated form screens.
- Added catalog repository queries with product/category counts and duplicate-name handling.
- Added variant creation, stock adjustment, and inventory movement recording.
- Added transactional purchase and sale services with stock validation, historical prices, and profit calculation.
- Added the tested SMART calculation engine and persisted evaluation snapshots/details.
- Added operational screens for inventory, purchases, sales, reports, and SMART ranking.
- Added a PixiJS 2D warehouse interface that uses the same inventory services for restock and sale actions.
- Added safe category/product edit and delete workflows with confirmation and dependency safeguards.
- Added operational history screens for stock movements, purchases, and sales.
- Added SMART evaluation history browsing and CSV export.
- Added report summary CSV export.
- Added database integration coverage for purchase/sale stock and profit invariants.
- Added keyboard-accessible variant selection and reduced-motion-safe 2D interface guidance.
- Replaced the settings placeholder with database, SMART methodology, and transaction safety information.

### Changed

- Made the GLB the single source for rendered product prototypes: live variants now clone named GLB products by deterministic type mapping, cap visible units by shelf slots, hide source duplicates, and dispose rebuilt product resources.
- Added a centralized 10×8m layout configuration and corrected clone-aware hover/raycast selection for grouped GLB products.
- Fixed the 3D overlay lifecycle by storing and clearing its HUD interval, removing listeners/overlay DOM, and clearing immersive mode on unmount.
- Added a functional immersive-mode foundation for Ruang 3D: interface chrome can be hidden, a live database HUD is shown, Escape exits the mode, and the shopkeeper is registered as a clickable/E-key interaction target with Indonesian dialogue.
- Integrated the locally stored CC0 Supermarket Operations GLB through Three.js `GLTFLoader` after inspecting the live Tauri render; the fixture asset now supplies the primary supermarket shelving and checkout composition.
- Replaced the remaining green cuboid product placeholders with shaped, color-varied snack-pouch meshes carrying live product and variant labels.
- Polished the 3D shop scene with storefront windows, signage, ceiling lamps, plant decoration, a cashier/shopkeeper figure, rack labels, and database-derived package labels.
- Fixed 3D WASD movement to follow the camera's horizontal facing direction, including correct strafing and normalized diagonal movement with smooth acceleration.
- Replaced the PixiJS 2D warehouse with a Three.js 3D shop environment containing a room, shelves, packages, checkout counter, storage area, lighting, camera controls, hover selection, and database-backed operations.
- Added Three.js dependencies and removed PixiJS so the visual inventory environment follows the updated project specification.
- Standardized data tables to eight row slots per page with real empty placeholder rows; placeholders stay outside record counts and pagination calculations.
- Made paginated table panels use a dedicated fixed footer for pagination; the table area now flexes above it so `Berikutnya` remains visible on short and full pages.
- Set the Tauri product name and identifier to `SIAMBRUK` and `siambruk`.
- Made the purchase/sales transaction view reactive to route changes and reset its form state when switching menus.
- Fixed purchase/sales persistence visibility: the transaction panel now reloads saved history from SQLite after submit and when switching menus.
- Fixed report period calculation by adding explicit date-range filtering for units sold, revenue, and profit.
- Reduced excess whitespace in transaction history and SMART evaluation controls.
- Removed the persisted-history placeholder copy once transaction records exist.
- Added restrained button depth, hover lift, and press feedback while respecting the existing restrained visual language.
- Made the sidebar sticky at the application viewport height with independent overflow, keeping navigation stable across menus and long content pages.
- Increased the application type scale across navigation, controls, tables, labels, and helper text for easier scanning.
- Removed the workflow banner and tightened the underlying page/container spacing so navigation is self-explanatory through hierarchy and content grouping.
- Added reusable pagination controls to catalog, inventory, history, and SMART result panels to keep long lists readable.
- Reserved a stable full-page table height for paginated panels so short final pages do not shrink the surrounding container.
- Standardized all table panels to the Riwayat reference height of `533.1px` while keeping 8 records per page.
- Increased the shared table-panel height to `540px` so pagination controls remain fully visible.
- Added consistent depth to panels, cards, containers, sidebar navigation, and pagination buttons.

### Validation

- `npm.cmd run build` passes.
- `cargo check` passes from `tauri-app/src-tauri/`.
- `cargo fmt --all` passes from `tauri-app/src-tauri/`.
- Category/product catalog code compiles with `cargo check` and `npm.cmd run build`.
- `cargo test` passes with SMART normalization and ranking coverage.
- PixiJS warehouse frontend compiles with `npm.cmd run build`.
- Frontend and backend checks pass after the hardening work.
- Integration test confirms purchases and sales are saved to history and included in period reports.
- UI build passes after the spacing and interaction polish.
- UI build passes after the fixed sidebar layout change.
- UI build passes after the workflow and typography update.
- UI build passes after the compact layout refinement.
- UI build passes after pagination was added to long-list panels.
- UI build passes after stable paginated-panel sizing was added.
- UI build passes after the shared `533.1px` table-panel standard was applied.
- UI build passes after the `540px` panel and elevation refinement.

### Documentation

- Added repository contribution guidance in `AGENTS.md`.
- Reviewed `DESIGN_RULES.md` and `MASTER_PROJECT.md` as the governing design and implementation references.
- Recorded the initial project baseline and implementation status.

