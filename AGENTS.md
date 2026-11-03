# AGENTS.md

Rust Cargo workspace for `lexic`, an offline-first dictionary + SM-2 flashcard desktop app (`iced` GUI, SQLite).

## Commands

- Toolchain is pinned by `rust-toolchain.toml` (stable + `rustfmt` + `clippy`).
- Build: `cargo build --workspace` (or `make build`)
- Run app: `cargo run -p lexic-gui` (or `make run`); release: `make run-release`
- Test all: `cargo test --workspace` (or `make test`)
- Single crate: `cargo test -p lexic-core`, `-p lexic-store`, `-p lexic-dict`, `-p lexic-gui`
- Single test: `cargo test -p lexic-core new_cards_respect_limit`; integration tests: `cargo test -p lexic-dict --test lookup`, `cargo test -p lexic-store --test schema`
- Lint/format gate: `make check` (runs `cargo fmt --all -- --check` then `cargo clippy --workspace --all-targets -- -D warnings`)
- Apply formatting: `make fmt`
- Docs: `make docs`
- CI env sets `RUSTFLAGS=-Dwarnings`; clippy config is `all = deny`, plus workspace `unsafe_code = forbid`, `unused_must_use = deny`.
- Linux builds need the system packages from `.github/workflows/ci.yml` (`libx11-dev`, `libxkbcommon-dev`, `libxkbcommon-x11-dev`, `libwayland-dev`, `libasound2-dev`, `libgl-dev`).

## Crate map

- `lexic-core`: domain models, SM-2 `Scheduler` trait + `Sm2`, review queue, stats, time. No I/O.
- `lexic-store`: SQLite (`rusqlite` bundled), schema/migrations, repositories, backup/import.
- `lexic-dict`: Free Dictionary API client, query normalization, cache helpers.
- `lexic-gui`: `iced` app reducer and screens in `src/app.rs`; entrypoint `src/main.rs`. Optional `ui` feature (default on); without it `main` just prints that UI is disabled.

## Storage / database

- Data file: `directories::ProjectDirs::from("dev", "Lexic", "Lexic").data_dir()/lexic.db`. `prepare()` silently returns no store if that path cannot be created, so the app must work with `store: Option<Arc<Store>>`.
- `Store::open` sets WAL, foreign keys, `busy_timeout=5000`, then runs the custom migration runner.
- Migrations are hand-rolled: `MIGRATIONS` in `crates/lexic-store/src/db.rs` pairs embedded SQL from `migrations/` with a `PRAGMA user_version` check. `Store::in_memory` skips WAL/busy_timeout. Add migrations by appending both the SQL file and a tuple.
- Search uses exact match, FTS5 (`entries_fts`), then `LIKE` fallback; `upsert_entry` fully rebuilds the FTS row for an entry.
- `entries_fts` is a contentless FTS5 table with `contentless_delete=1` so deletes/rebuilds work; migration `0002` recreates it for older databases.
- `dict::cache::is_fresh` exists, but `upsert_entry` stores `fetched_at = NULL`, so cache freshness is not currently enforced.
- GSET is vendored as a root submodule in `GSET/`; use `git submodule update --init --recursive` before builds/CI. Lexic invokes the `gset` CLI and falls back to `GSET/target/.../gset` or `cargo run --manifest-path GSET/Cargo.toml`.
- A new project/transpilation screen calls `crate::gset::transpile`; capture stdout as generated output and stderr as diagnostics.

## Testing notes

- `lexic-dict` tests use `wiremock` (no live network required).
- Store tests use `Store::in_memory()`.
- GUI has no automated tests.
- Schema changes must keep `lexic-store/tests/schema.rs` and migration runner consistent.

## Notes

- `README.md` and `docs/implementation-plan.md` describe intended/aspirational architecture (assets, screens modules, `rusqlite_migration`) that does not match the current tree. Trust `Cargo.toml` and source over docs.
- No existing `AGENTS.md`/`CLAUDE.md`/cursor rules were present.