# Lexic — Full Implementation Plan

A clean, modern, cross-platform dictionary and spaced-repetition flashcard desktop
application built in Rust (iced UI, SQLite storage).

## Context

The repository currently contains only `README.md` and `.gitignore`; there is no Cargo
workspace yet. One correction to the README: `iced 0.14.0` (latest, Dec 2025) requires
**Rust 1.88+ and edition 2024**, and `rusqlite` bundles C SQLite rather than being pure
Rust. This plan keeps `rusqlite` (battle-tested, FTS5 support) and recommends rewording
"pure-Rust" to "native Rust UI."

Locked decisions:

- Dictionary source: dictionaryapi.dev + permanent local cache
- SRS algorithm: SM-2 behind a swappable `Scheduler` trait
- V1 scope: core offline app + stats + JSON backup
- Visual direction: minimal monochrome

## 1. V1 Scope

### In scope

- Offline-first dictionary search with online lookup and permanent local cache
- Word detail: phonetics, audio, parts of speech, definitions, examples, synonyms and
  antonyms, source attribution
- Save words to decks, manual definitions and notes for missing words
- Flashcard review with SM-2, daily new/review limits, keyboard-first flow, session
  summary, undo
- Decks: create, rename, archive, delete; per-deck limits
- Stats: due forecast, review heatmap, retention, streaks, study time
- Settings: light/dark/system monochrome theme, limits, cache controls, reduced motion
- JSON export/import with merge/replace modes and pre-import backup
- Packaged builds for Linux, macOS, Windows

### Out of scope for v1

- Accounts, cloud sync, mobile
- Anki `.apkg` import (v1.1)
- Cloze and multiple card directions (schema supports them; UI only recognition in v1)
- Bundled WordNet dataset (v1.1 fallback option)
- Tags, shared decks, localization

### Definition of done

A user can install the app, search a word, save it, review due cards across sessions
with correct SM-2 scheduling, view stats, back up and restore data, and complete the
entire review flow using only the keyboard.

## 2. Architecture

```text
┌───────────────────── lexic-gui (iced 0.14) ─────────────────────┐
│ screens · widgets · theme · keyboard · toasts · app reducer      │
└───────────────┬──────────────────────────────┬──────────────────┘
                │ async service calls           │
┌───────────────▼──────────────┐  ┌────────────▼─────────────────┐
│ lexic-store                  │  │ lexic-dict                   │
│ SQLite · migrations · repos  │  │ reqwest · DTOs · cache policy │
│ queue queries · backup I/O   │  │ normalization · attribution  │
└───────────────┬──────────────┘  └────────────┬─────────────────┘
                └──────────► lexic-core ◄──────┘
                   domain models · SM-2 · stats · errors
```

### Workspace layout

```text
Lexic/
├── Cargo.toml
├── rust-toolchain.toml
├── crates/
│   ├── lexic-core/
│   │   └── src/{model,schedule,stats,time,validation,error}.rs
│   ├── lexic-store/
│   │   ├── src/{db,repos,queue,backup,search}.rs
│   │   └── migrations/0001_init.sql ...
│   ├── lexic-dict/
│   │   └── src/{client,model,normalize,cache,error}.rs
│   └── lexic-gui/
│       └── src/
│           ├── main.rs app.rs theme.rs services.rs keyboard.rs
│           ├── screens/{home,word,decks,deck_detail,review,stats,settings}.rs
│           └── widgets/{search_bar,rating_bar,flip_card,heatmap,bar_chart,modal,toast,empty_state}.rs
├── assets/{fonts,icons,seed_words.json}
├── docs/{architecture,schema,sm2,design-system}.md
└── .github/workflows/ci.yml release.yml
```

### Key dependencies

| Area | Crates |
|---|---|
| GUI | `iced = "0.14"` with `wgpu`, `tokio`, `svg`, `image`, `canvas`, `lazy`; optional `tester` for widget tests |
| DB | `rusqlite` (`bundled`), `rusqlite_migration` |
| Async | `tokio`, `reqwest` (`rustls-tls`, `json`) |
| Data | `serde`, `serde_json`, `uuid`, `chrono` |
| Platform | `directories`, `rfd`, `fs2` (single-instance lock) |
| Audio | `rodio` with `symphonia` codecs (optional, degrades gracefully) |
| Errors/logging | `thiserror` in libs, `anyhow` in the binary, `tracing`, `tracing-subscriber`, `tracing-appender` |
| Testing | `wiremock`, `tempfile`, `proptest`, `insta` |

Pin exact versions at scaffold time. Use `[workspace.dependencies]` and `[workspace.lints]`
(`clippy::all`, `-D warnings`).

### Concurrency model

- `lexic-store::Store` wraps `Arc<Mutex<rusqlite::Connection>>` behind async methods; every
  call runs in `tokio::task::spawn_blocking`, so the iced thread never blocks.
- Enable SQLite `WAL`, `foreign_keys=ON`, `busy_timeout=5000`, `synchronous=NORMAL`.
- Use a file lock in the data directory to prevent two instances from corrupting state.
- Dictionary lookups are async `reqwest` calls with 10s timeout, bounded concurrency
  (semaphore of 2), one in-flight request per normalized word, and stale-response
  discarding via request IDs.
- Import/export and audio downloads run off-thread and stream progress messages back to
  the UI.

## 3. Data Model and SQLite Schema

All IDs are UUIDv7 stored as `TEXT`. All timestamps are UTC epoch seconds; local-day
grouping is computed with `chrono::Local` and stored explicitly in `reviews.day` so stats
are timezone-stable.

```sql
PRAGMA journal_mode = WAL;
PRAGMA foreign_keys = ON;

CREATE TABLE entries (
  id               TEXT PRIMARY KEY,
  word             TEXT NOT NULL,
  normalized_word  TEXT NOT NULL,
  phonetic         TEXT,
  audio_url        TEXT,
  origin           TEXT,
  source           TEXT NOT NULL,
  source_url       TEXT,
  license          TEXT,
  fetched_at       INTEGER,
  created_at       INTEGER NOT NULL,
  updated_at       INTEGER NOT NULL,
  UNIQUE(normalized_word, source)
);

CREATE TABLE senses (
  id             TEXT PRIMARY KEY,
  entry_id       TEXT NOT NULL REFERENCES entries(id) ON DELETE CASCADE,
  part_of_speech TEXT,
  definition     TEXT NOT NULL,
  example        TEXT,
  synonyms_json   TEXT NOT NULL DEFAULT '[]',
  antonyms_json   TEXT NOT NULL DEFAULT '[]',
  sort_order     INTEGER NOT NULL
);

CREATE TABLE decks (
  id            TEXT PRIMARY KEY,
  name          TEXT NOT NULL UNIQUE COLLATE NOCASE,
  description   TEXT,
  new_per_day   INTEGER,
  reviews_per_day INTEGER,
  created_at    INTEGER NOT NULL,
  updated_at    INTEGER NOT NULL,
  archived_at   INTEGER
);

CREATE TABLE cards (
  id                TEXT PRIMARY KEY,
  entry_id          TEXT NOT NULL REFERENCES entries(id) ON DELETE CASCADE,
  deck_id           TEXT NOT NULL REFERENCES decks(id) ON DELETE CASCADE,
  direction         TEXT NOT NULL DEFAULT 'recognition',
  front_note        TEXT,
  back_note         TEXT,
  state             TEXT NOT NULL DEFAULT 'new',
  due_at            INTEGER,
  interval_days     REAL NOT NULL DEFAULT 0,
  ease_factor       REAL NOT NULL DEFAULT 2.5,
  repetitions       INTEGER NOT NULL DEFAULT 0,
  lapses            INTEGER NOT NULL DEFAULT 0,
  first_reviewed_at INTEGER,
  last_reviewed_at  INTEGER,
  created_at        INTEGER NOT NULL,
  updated_at        INTEGER NOT NULL,
  UNIQUE(entry_id, deck_id, direction)
);
CREATE INDEX cards_due_idx ON cards(state, due_at);
CREATE INDEX cards_deck_idx ON cards(deck_id, state);

CREATE TABLE reviews (
  id                 TEXT PRIMARY KEY,
  card_id            TEXT NOT NULL REFERENCES cards(id) ON DELETE CASCADE,
  reviewed_at        INTEGER NOT NULL,
  day                TEXT NOT NULL,
  rating             INTEGER NOT NULL CHECK(rating BETWEEN 1 AND 4),
  prev_state         TEXT NOT NULL,
  prev_interval_days REAL NOT NULL,
  prev_ease          REAL NOT NULL,
  new_state          TEXT NOT NULL,
  new_interval_days  REAL NOT NULL,
  new_ease           REAL NOT NULL,
  duration_ms        INTEGER
);
CREATE INDEX reviews_day_idx ON reviews(day);
CREATE INDEX reviews_card_idx ON reviews(card_id);

CREATE TABLE settings (
  key   TEXT PRIMARY KEY,
  value TEXT NOT NULL
);

CREATE VIRTUAL TABLE entries_fts USING fts5(
  word, definitions, content='', tokenize='unicode61 remove_diacritics 2'
);
```

Card states: `new | learning | review | relearning | suspended`.

- Maintain `entries_fts` in the store layer on write; provide a rebuild command in Settings.
- Verify FTS5 is enabled in the bundled SQLite build during Phase 2. If not, fall back to
  indexed `LIKE` prefix search for v1.
- Search ranking: exact normalized match, then prefix match, then FTS rank. Suggestions
  query local FTS only; Enter triggers online lookup.
- Migrations are forward-only and tested from v0 to latest. Auto-backup `lexic.db` before a
  migration, then run integrity checks.

## 4. Dictionary Subsystem

Endpoint: `GET https://api.dictionaryapi.dev/api/v2/entries/en/{word}`

1. Normalize query: trim, collapse whitespace, NFKC, max 64 chars, reject control
   characters. Keep original casing for display; use lowercase key for lookup.
2. Check local cache:
   - Fresh (cache TTL, default 30 days): return immediately.
   - Stale: return cached immediately and revalidate in the background.
   - Missing: fetch if online.
3. On success, upsert entry plus senses in one transaction; never duplicate senses on
   refresh.
4. Map failures:
   - `404`: "No definition found"; offer manual entry.
   - `429`: "Dictionary is rate-limiting; try again shortly"; use cached entry if
     available.
   - Timeout/5xx/offline: serve stale cache with a "Cached" badge; otherwise offer manual
     entry.
5. Negative-cache misses for 24 hours.
6. Show source URL and license in word detail. Send an identifying `User-Agent`.

UX details:

- Local suggestions are debounced (250ms); remote lookup happens on Enter or "Look up
  online."
- Skeleton loading state; cancel and ignore out-of-order responses.
- "Add definition manually" creates `source='manual'` and supports full editing later.
- Audio: prefer the first `phonetics[].audio` URL, download to
  `cache/audio/<sha256>.mp3`, play via `rodio`; hide the button if unavailable; cap cache
  at 100MB LRU.

## 5. SM-2 Scheduler

### Domain types

```rust
pub enum Rating { Again, Hard, Good, Easy }
pub struct CardState { state, due_at, interval_days, ease_factor, repetitions, lapses }
pub struct Outcome { state, due_at, interval_days, ease_factor, repetitions, lapses }

pub trait Scheduler {
    fn new_card(&self, now: DateTime<Utc>) -> Outcome;
    fn grade(&self, card: &CardState, rating: Rating, now: DateTime<Utc>) -> Outcome;
}
```

`Sm2` is a pure implementation; `Fsrs` can be added later behind the same trait. No DB
or UI dependency.

### Rules

- Ease update on every grade:
  `EF' = clamp(EF + (0.1 - (5-q)*(0.08 + (5-q)*0.02)), 1.3, 3.0)`.
- In `Review` state with Hard/Good/Easy:
  - `repetitions == 0` -> 1 day; `repetitions == 1` -> 6 days; otherwise
    `round(interval * EF')`.
  - Increment repetitions, set due at `now + interval`, clamp to max 36500 days.
- In `Review` state with Again:
  - `lapses += 1`, `repetitions = 0`, state `Relearning`, due in 10 minutes.
- In `New`/`Learning`/`Relearning`:
  - Again/Hard: remain in learning, due in 10 minutes.
  - Good: graduate to `Review` with 1 day (first pass) or 6 days (second pass).
  - Easy: graduate to `Review` with 4 days.
- Optional ±5% interval fuzz, seeded deterministically per card ID, to avoid review
  pileups.
- 8 lapses -> mark as leech (surface in deck detail; auto-suspend is a setting).

### Queue building

- Reviews due now, oldest first.
- Learning/relearning due now, oldest first.
- New cards up to remaining `new_per_day`, interleaved after every N reviews (default 5).
- Review cap respects `reviews_per_day`; new-card count uses `first_reviewed_at >= local day
  start`.
- Session re-queue: Again adds the card to the in-memory queue after roughly 5 other cards
  without duplicating the persisted review.

### Session and undo

- Session keeps a one-level undo snapshot: previous card state plus review row ID.
- `Ctrl+Z` restores card state and deletes the review in one transaction.
- Rating buttons show next intervals from the pure scheduler
  (e.g., "Again 10m · Hard 3d · Good 7d · Easy 16d").

## 6. GUI Architecture and Design

### App structure

```rust
enum Screen { Onboarding, Home, Word(WordState), Decks, DeckDetail(DeckState),
              Review(Session), Stats, Settings }

enum Message {
    Navigate(Screen),
    Search(SearchMessage),
    Word(WordMessage),
    Decks(DeckMessage),
    Review(ReviewMessage),
    Store(StoreEvent),
    Dict(DictEvent),
    Toast(ToastMessage),
    Noop,
}
```

- iced Elm architecture: single `App` state, nested message enums with `.map()`, pure
  `update`, `view`, and `subscription`.
- Screens own transient state; global `App` owns navigation, theme, toasts, and service
  handles.
- Async results arrive as messages; no blocking in `update` or `view`.
- Modals (deck picker, delete confirmation, import summary) are an overlay layer, not
  separate windows.
- Subscriptions: keyboard events, window frames only during animations, and
  audio/fetch progress channels.

### Screens

1. Onboarding: create default "My Words" deck, theme choice, optional import, short
   offline-cache explanation.
2. Home/Search: large search field, due-today count, streak, recent searches, saved
   words, optional word of the day.
3. Word detail: large serif headword, phonetic plus audio, POS sections, definitions and
   examples, synonym chips, deck picker plus save, source attribution, edit notes.
4. Decks: deck cards with due/new counts and progress; create, rename, archive, delete.
5. Deck detail: card list (paginated/lazy), suspend, reset, remove, add word, leech
   indicators.
6. Review: centered card, reveal with Space, rating bar with interval previews, progress
   bar, session summary, undo, exit confirmation.
7. Stats: 30/90/365-day range with forecast bar chart, review heatmap, retention
   percentage, current and longest streak, total reviews and study time.
8. Settings: theme, reduced motion, daily limits, audio autoplay, cache TTL/clear/rebuild
   search index, export/import, data folder, about.

### Keyboard map

- `Ctrl/Cmd+K` or `/`: focus search
- `Space`: reveal answer; `1-4`: rate Again/Hard/Good/Easy
- `Ctrl/Cmd+Z`: undo last review
- `Esc`: close modal or leave screen with confirmation
- `Ctrl/Cmd+E`: export; `Ctrl/Cmd+I`: import

### Minimal monochrome design system

| Token | Light | Dark |
|---|---|---|
| Canvas | `#FAFAF9` | `#0C0C0D` |
| Surface | `#FFFFFF` | `#151516` |
| Text | `#18181B` | `#F4F4F5` |
| Muted | `#71717A` | `#A1A1AA` |
| Border | `#E4E4E7` | `#27272A` |
| Primary button | ink `#18181B` | paper `#FAFAFA` |
| Destructive/lapse | functional red only | functional red only |

- UI font: Inter; display font: Newsreader (serif) for headwords; both are OFL and
  embedded.
- 11px uppercase tracked labels, 14-16px body, 40-64px headwords.
- 1px borders, 12px radius, 8pt spacing scale, no gradients, subtle elevation only for
  modals.
- 240px sidebar collapsing to an icon rail under 900px; reading column max 720px.
- 160ms ease-out transitions; 240ms card flip; honor reduced-motion/system preference.
- Lucide SVG icons (ISC), embedded.
- Charts drawn with iced `canvas` in grayscale steps; functional red reserved for
  lapses/destructive actions.
- Accessibility: visible 2px focus rings, full keyboard navigation, AA contrast, text
  labels alongside icons.

### Performance targets

- Cold start to interactive < 1s with a small DB; cached search < 50ms; queue build for
  10k cards < 100ms; memory < 150MB.
- Use `iced::widget::lazy` for long lists, paginate deck/card queries, prepared
  statements, indexes, and WAL.
- Never load all entries into memory; stream import/export.

## 7. Import / Export

### Format

```json
{
  "format": "lexic.backup",
  "version": 1,
  "exported_at": "2026-11-03T12:00:00Z",
  "app_version": "0.1.0",
  "decks": [],
  "entries": [],
  "cards": [],
  "reviews": []
}
```

- Stable UUIDs preserve relationships. Optional "include review history" flag.
- Atomic writes: temp file in the target directory, then rename. Chosen path via `rfd`.
- Import validates schema and semantics (foreign keys, enum values, date ranges), shows a
  dry-run summary, and supports:
  - Merge: insert missing IDs, skip conflicts, report counts.
  - Replace: requires confirmation, auto-exports a safety backup first, then wipes in one
    transaction.
- Large files are read/streamed and committed in chunks with progress events.
- CSV word-list export is a small post-v1 addition.

## 8. Milestones and Schedule

Estimates assume one full-time developer. Two developers can overlap store/dict with GUI.

| Milestone | Work | Acceptance criteria | Est. |
|---|---|---|---|
| M0 Scaffold | Workspace, crates, lints, Rust 1.88 toolchain, CI skeleton, error/logging setup, README update | `cargo check/test/clippy/fmt` green; empty window runs | 1d |
| M1 Core + SM-2 | Models, IDs, clock/day helpers, `Scheduler` trait plus SM-2, queue ordering, stats calculations | Table-driven and proptest scheduler tests pass; no I/O in core | 3d |
| M2 Store | Schema, migrations, repos, FTS, queue queries, transactions, settings, single-instance lock | In-memory/file DB tests pass; migration v0 to latest tested; 10k-card query measured | 3d |
| M3 Dictionary | API DTOs, client, normalization, cache policy, dedupe/upsert, error mapping | Fixture tests for 200/404/429/offline/malformed; no live network in CI | 3d |
| M4 GUI shell | iced app, theme tokens/fonts, sidebar navigation, toasts, modals, keyboard plumbing, onboarding | Window launches; theme switch works; keyboard navigation works | 3d |
| M5 Search + word detail | Local suggestions, remote lookup, loading/error states, detail rendering, save-to-deck, manual entry | Search/save flow works online and offline | 4d |
| M6 Decks | Deck CRUD, card lists, add/remove/suspend/reset, per-deck limits | All deck actions persist; empty states handled | 4d |
| M7 Review | Session builder, card renderer, reveal, 4 ratings, interval previews, re-queue, summary, undo | Full keyboard review session; SM-2 state verified in DB | 5d |
| M8 Stats | Forecast, heatmap, retention, streak, study time, range selector | Values match seeded fixtures; DST boundaries tested | 3d |
| M9 Settings + backup + audio | Settings persistence, cache controls, JSON import/export, audio download/play/cache | Round-trip backup restores identical data; offline audio degrades gracefully | 4d |
| M10 Polish + packaging | Animations, accessibility pass, logging, icons, installers, docs, QA matrix | Signed-off QA checklist; installers build on 3 OSes | 4d |

Total: approximately 5-6 weeks solo.

## 9. Testing, CI, and Quality

- Unit: SM-2 invariants (monotonic intervals, EF bounds, lapse resets, due-in-future),
  normalization, day/streak math, backup serialization.
- Store: every repository against in-memory SQLite; FK cascades; transaction rollback;
  queue limits; undo; import conflicts; migration upgrade.
- Dictionary: `wiremock` fixtures; cache TTL and stale-while-revalidate; negative cache.
- GUI logic: extract and test reducers for search, session, and import state. Use iced
  `tester` for headless widget tests where stable; otherwise maintain a manual QA checklist.
- E2E without UI: seed DB, search, save, review, stats, export, import into a fresh DB,
  assert equality.
- CI: `cargo fmt --check`, `cargo clippy -D warnings`, tests on Ubuntu/macOS/Windows,
  `cargo-deny` for licenses/advisories, release builds.
- Manual QA matrix: HiDPI, offline/online transitions, 10k cards, malformed import, clock
  changes, DST, missing audio device, small window, keyboard-only use.

## 10. Packaging and Distribution

- Use `dist`/`cargo-dist` in GitHub Actions for `.msi`/`.exe` (Windows), `.dmg`
  (macOS), `.AppImage`/`.deb` (Linux); optional Flatpak later.
- Add app icons, category metadata, and optional `.lexic.json` file association for backups.
- Unsigned builds initially; document macOS notarization and Windows signing as release
  requirements.
- Data locations via `directories`: DB in platform data dir, audio cache in cache dir, logs
  in state dir; document them in Settings and README.
- No telemetry. Network only for dictionary lookups and audio downloads.

## 11. Risks and Mitigations

| Risk | Mitigation |
|---|---|
| dictionaryapi.dev has no SLA and rate limits | Aggressive local cache, negative caching, stale-while-offline, manual definitions; abstract `DictSource` trait for a future provider |
| iced 0.14 API churn | Pin version, isolate widget wrappers and theme behind local modules, follow official examples |
| SQLite blocking the UI thread | All store calls via `spawn_blocking`; measure with large decks; actor pattern available if contention appears |
| Audio platform deps (ALSA/PipeWire) | Make audio optional, feature-gate rodio, graceful fallback with clear error toast |
| Unicode/CJK search quality | `unicode61 remove_diacritics 2`; test multilingual words; document English-first scope |
| Import data loss | Validation, dry-run summary, automatic pre-import backup, atomic writes, transactions |
| Legal/licensing of dictionary text | Display source/license, do not claim ownership, review API terms before packaging, avoid shipping API data in bundled assets |
| Scope creep (Anki, sync) | Explicit v1 scope; keep schema and `Scheduler` trait extensible |

## 12. Immediate Next Actions

1. Approve this plan and the deferred items: tags, multiple directions, Anki import.
2. Scaffold M0: workspace, four crates, toolchain, CI, README corrections.
3. Implement M1 (SM-2 plus tests) and M2 (schema plus store) before any GUI work.
4. Build M4 as a thin vertical slice with a hardcoded word, then wire M3 and M5.

If this plan is approved, the first implementation step is M0 scaffolding and the core
scheduler.
