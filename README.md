# Lexic (`lexic`)

A clean, modern, cross-platform Rust dictionary and spaced repetition flashcard desktop application built for fast, offline vocabulary acquisition and study.

![Rust](https://img.shields.io/badge/Language-Rust-orange.svg)
![GUI](https://img.shields.io/badge/GUI-iced-blue.svg)
![Storage](https://img.shields.io/badge/Storage-SQLite-lightgrey.svg)

---

## Key Features

- **Modern Pure-Rust GUI**: Powered by [`iced`](https://github.com/iced-rs/iced) using GPU acceleration (`wgpu`) for a native, lightweight, and fluid user interface across Linux, macOS, and Windows.
- **SM-2 Spaced Repetition**: Implements the SuperMemo-2 (SM-2) scheduling algorithm to optimize flashcard review intervals based on user confidence scoring.
- **Offline Dictionary Storage**: Fetches word definitions, parts of speech, phonetics, and examples via public dictionary sources and caches everything locally into a native SQLite database for instant offline access.
- **OS-Native Paths**: Fully compliant with platform conventions (`XDG` on Linux, `Application Support` on macOS, `%APPDATA%` on Windows) via the `directories` crate.
- **Import & Export**: Effortlessly back up and transfer your custom word vaults, notes, and learning histories using native OS file dialogs (`rfd`).

---

## Workspace Architecture

`lexic` is structured as a Cargo workspace, strictly separating core business logic, persistence, and scheduling algorithms from the graphical presentation layer.

---

## Getting Started

### Prerequisites

Ensure you have the Rust toolchain installed (stable 1.88+ recommended):

```bash
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
```

### Installation & Running

1. **Clone the repository:**
   ```bash
   git clone https://github.com/your-username/lexic.git
   cd lexic
   ```

2. **Run in development mode:**
   ```bash
   cargo run -p lexic-gui
   ```

3. **Build optimized release binary:**
   ```bash
   cargo build --release -p lexic-gui
   ```
   The output binary will be located under `target/release/lexic-gui`.

---

## Tech Stack & Dependencies

| Component | Library / Crate | Purpose |
| :--- | :--- | :--- |
| **GUI Framework** | [`iced`](https://crates.io/crates/iced) | Native cross-platform GUI architecture |
| **Database** | [`rusqlite`](https://crates.io/crates/rusqlite) | Embedded relational storage for entries and card states |
| **System Paths** | [`directories`](https://crates.io/crates/directories) | Standard OS configuration & data path resolution |
| **File Dialogs** | [`rfd`](https://crates.io/crates/rfd) | Native OS file picker for imports/exports |
| **Serialization** | [`serde`](https://crates.io/crates/serde) / [`serde_json`](https://crates.io/crates/serde_json) | Data serialization and backup handling |
| **HTTP Client** | [`reqwest`](https://crates.io/crates/reqwest) | Fetching online dictionary definitions |
