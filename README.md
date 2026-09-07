# FC Squad Editor

An open-source editor for EA Sports FC 25 squad save files, written in Rust.

Browse and edit players, teams, leagues and formations, then save a file the
game actually loads.

![Rust](https://img.shields.io/badge/rust-2021-orange)
![License](https://img.shields.io/badge/license-MIT-blue)

---

## What it does

- **Squad view** — browse every team, see the full squad with ratings, click a
  player to edit any of their attributes.
- **Formation editor** — change a team's shape and swap players between pitch
  positions by clicking them.
- **League view** — edit league properties and see which teams belong to each.
- **Table view** — direct access to all 76 tables in the file, with search,
  paging and a description of what each table holds.
- **Export and import** — Excel (one sheet per table), CSV, TSV and JSON.
  Re-import edited CSV or TSV back into the save.
- **Undo and redo** with a change log for the session.
- **Safe saving** — automatic backups, and the save is refused outright if any
  checksum fails to verify, rather than writing a file the game would reject.

---

## Getting started

You need [Rust](https://rustup.rs/).

```sh
git clone https://github.com/RiccardoSilvestri/Fc-squad-editor
cd Fc-squad-editor
cargo run --release
```

To read a save you also need the game's schema file, `fifa_ng_db-meta.xml`. The
editor looks for it in `db/fifa_ng_db-meta.xml`, searching upwards from the
working directory and from the executable. You can also pass a path as the first
argument:

```sh
cargo run --release -- path/to/fifa_ng_db-meta.xml
```

Without the schema the file still opens, but every field is labelled with an
opaque four-character code instead of a name.

Optionally, load `fifa_ng_db.db` as a reference file from within the app to
resolve player names — squad saves store name IDs, not text.

Squad saves live in:

```
%LOCALAPPDATA%\EA SPORTS FC 25\settings\Squads<timestamp>
```

---

## A word of caution

Editing save files is at your own risk. Back up anything you care about.

The editor writes a backup into a `SquadEditorBackups` subfolder before
overwriting a save. It deliberately does not leave backups in the game's own
save folder, because FC 25 tries to read every file there and reports anything
it cannot parse as a damaged save.

---

## How the file format works

The binary layout is documented in full:

- **[docs/FORMAT.md](docs/FORMAT.md)** — byte-level structure: the container
  header, the table directory, record packing, and the two different checksum
  algorithms a save uses.
- **[docs/TABLE_MAP.md](docs/TABLE_MAP.md)** — how the 76 tables reference each
  other through domain IDs.

Everything documented there was reconstructed from real files and is covered by
the test suite.

---

## Project layout

```
crates/squad-core   parsing, editing and checksums, no UI
crates/squad-gui    the desktop application (iced)
docs/               file format documentation
```

---

## Running the tests

Most tests need a real squad save, which cannot be distributed. Point them at
your own files and they will run; leave the variables unset and they skip.

```sh
export SQUAD_EDITOR_TEST_META=/path/to/fifa_ng_db-meta.xml
export SQUAD_EDITOR_TEST_SAVE=/path/to/Squads20260101120000000
export SQUAD_EDITOR_TEST_REFERENCE_DB=/path/to/fifa_ng_db.db
cargo test --release
```

---

## Licence

MIT. This project is not affiliated with, endorsed by, or connected to
Electronic Arts. EA Sports FC is a trademark of Electronic Arts Inc. No game
data or assets are included in this repository.
