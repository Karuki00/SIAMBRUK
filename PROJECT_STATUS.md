# Project Status

**Date:** 2026-11-26  
**Release target:** v0.1.1  
**Phase:** Complete — MASTER_PROJECT requirements implemented and verified

## Latest Fix

- Fixed navigation between Pembelian and Penjualan: the shared transaction view now recomputes its mode and clears stale form/result state on route changes.
- Verified and fixed the complete purchase/sales flow: records persist in SQLite, transaction history reloads into the control panel, and reports calculate the selected date range.
- Tightened transaction history and SMART period-control layouts; persisted history no longer shows its empty-state placeholder message.
- Added restrained depth and hover/press animation to primary and secondary buttons.
- Sidebar layout is now sticky to the app window height and no longer stretches or shifts with menu content.
- Increased UI typography across menus and data surfaces for clearer scanning and improved readability.
- Removed the workflow banner and improved the base hierarchy instead: compact panels, tighter page rhythm, and clearer grouping reduce unnecessary whitespace without adding instructional UI.
- Added pagination to categories, products, inventory variants, operational history, and SMART results/history panels.
- Paginated panels reserve the full page-size height and keep pagination anchored, preventing layout shrinkage on short final pages.
- All table panels now use a `540px` shared height so the next-page control remains visible; panel and navigation elevation is consistent across the UI.
- Pagination now occupies its own fixed-height footer section inside each table panel, while the table content area flexes independently and cannot clip the navigation buttons.
- Standard table bodies now render eight fixed-height row slots; missing slots are blank, non-interactive rows that do not affect record totals or page counts.
- Pembelian kini menggunakan jumlah bal, unit hasil dari bal, dan harga per bal; stok unit serta biaya per unit dihitung otomatis (contoh: Rp100.000 / 20 = Rp5.000 per unit).
- Pembelian kini menggunakan jumlah bal, unit hasil dari bal, dan harga per bal; istilah form menjelaskan hasil kemasan dan biaya per unit tetap dihitung otomatis.
- Penghapusan produk sudah mendukung produk yang memiliki varian tanpa stok aktif melalui soft-delete, sehingga riwayat transaksi tetap aman; produk dengan stok aktif masih dilindungi.
- SKU varian dapat dikosongkan di Inventori; aplikasi otomatis membuat kode `SKU-...` sebelum menyimpan varian.

## Current State

- `tauri-app/` now contains a standard Tauri 2 + Vue 3 + TypeScript + Vite desktop application shell.
- The Tauri product name is `SIAMBRUK` and its identifier is `siambruk`.
- Vue Router navigation is available for Dashboard, Products, Categories, Inventory, Purchases, Sales, Reports, SMART Ranking, and Settings.
- The dashboard foundation uses real empty states and does not fabricate operational data.
- SQLite opens in the Tauri app data directory as `siambruk.sqlite3` and applies migration `001_initial.sql` on first run.
- The backend exposes a `database_status` command for schema readiness and basic record counts.
- The initial schema preserves variant-level inventory, transaction, and SMART evaluation data needed by later modules.
- Categories and products can now be created and listed through typed Tauri commands and SQLite queries.
- The catalog screens use explicit validation, loading states, and empty states.
- Variant creation and stock adjustments write transactional inventory movements.
- Purchases increase stock; sales validate and decrease stock while preserving historical prices and profit.
- SMART calculations use the required four criteria and weights, persist evaluation details, and have Rust unit tests.
- Reports and dashboard summaries read actual SQLite records.
- The Three.js warehouse view provides a database-backed 3D shop with shelves, capped package visualization, checkout/storage scenery, camera exploration, object selection, and restock/sale actions through the same transaction services.
- 3D WASD movement now derives forward and strafe vectors from the camera orientation, projects them onto the floor plane, normalizes diagonal motion, and applies smooth bounded movement.
- The 3D environment now includes storefront and lighting details, a shopkeeper figure, visible rack identification, and product labels derived from the live variant records.
- The running Tauri scene was visually inspected; the CC0 Supermarket Operations GLB is bundled locally and loaded with `GLTFLoader`, while green cuboid product placeholders were replaced with shaped, color-varied snack packages.
- Ruang 3D now has a functional immersive-mode foundation, live `report_summary` HUD values, Escape exit behavior, and a registered shopkeeper target reachable by click or E-key proximity interaction.
- The GLB is now the rendered product source of truth: named prototypes are cloned per live variant with deterministic mapping, visible stock is capped by configured slots, source product duplicates are hidden, and rebuild cleanup disposes generated resources.
- The scene uses a centralized compact layout configuration and cleans up overlay intervals, DOM, renderer listeners, and immersive state when leaving the view.
- Categories and products support edit and guarded delete actions.
- Stock movement, purchase, and sales history are visible in the Riwayat module.
- Pembelian and Penjualan also show their own persisted recent history directly in the transaction control panel.
- SMART evaluation history and CSV export are available, as is report summary CSV export.
- The 3D interface has a keyboard-accessible variant list, camera reset, movement boundaries, and an explicit standard-interface fallback.
- Integration coverage verifies database initialization, catalog creation, purchase stock-in, sale stock deduction, and profit preservation.
- Settings provides database status, SMART weights, and transaction safety information.
- Frontend dependencies are installed and the generated Rust dependencies are locked in `tauri-app/src-tauri/Cargo.lock`.
- `DESIGN_RULES.md` is the authority for visual, interaction, and engineering quality standards.
- `MASTER_PROJECT.md` is the authority for product scope, technology stack, data rules, SMART methodology, and staged implementation order.
- `AGENTS.md` defines contributor workflow and repository conventions.

## Validation

- `npm.cmd run build` — passed.
- `npm.cmd install three @types/three` — passed; PixiJS was removed after the 3D migration.
- `cargo check` from `tauri-app/src-tauri/` — passed.
- `cargo fmt --all` from `tauri-app/src-tauri/` — passed.
- `cargo test` from `tauri-app/src-tauri/` — passed (3 unit/integration tests).
- `npm.cmd run tauri info` — environment detected; version lookups were unavailable due restricted network access.

## Next Recommended Stage

No requirements from points 1–14 of `MASTER_PROJECT.md` remain unimplemented in the current scope. Future work is optional enhancement, such as PDF export, richer historical drill-down, authentication, and broader end-to-end UI automation.

