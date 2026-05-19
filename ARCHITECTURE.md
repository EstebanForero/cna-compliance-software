# Autoevaluacion CNA Architecture

## Core Boundaries

- **Workspace:** local configuration, editor profile, database location, OneDrive folder sync and Microsoft Graph app-folder sync.
- **Import:** reads Excel consolidated workbooks, merges compatible sheets, forward-fills CNA hierarchy cells, deduplicates questions by code and lineamientos by CNA hierarchy.
- **CNA Model:** Factor -> Caracteristica -> Aspecto hierarchy. CNA factors ship with known presets, but factor codes/names are data-driven so future lineamientos can add factors without a Rust release. Aspect codes are internal keys: imported values are normalized and manual aspects get deterministic generated keys.
- **Question Bank:** current editable source of truth for questions, audiences, operational status and justification.
- **Original Baseline:** immutable snapshot of the imported workbook selected as the cycle original. It is replaced only with reinforced confirmation.
- **Validation:** checks blocking readiness conditions before provider delivery or export.
- **Export:** generates consolidated and instrument Excel workbooks from the database, comparing current questions against the original baseline and applying change colors.
- **Provider:** tracks provider links and validation status after delivery.
- **Provider Review:** checklist grouped by instrument/audience. Reviewers mark each delivered question as correct, needs modification, or missing; they can attach evidence paths or local images and export a Word report for the selected instrument.
- **Persistence:** libSQL/Turso-compatible repository with modules by responsibility: schema, row parsing, helpers, history snapshots, provider reviews, questions and lineamientos.
- **Portable Database Package:** `.acna` files are complete Autoevaluacion CNA database packages. They are the same libSQL/SQLite database content under a domain-specific extension so the OS can associate them with the desktop app.

## Baseline Rules

- The user imports a consolidated Excel into the local libSQL database.
- The user may mark the current imported content as the original baseline.
- Marking or replacing the baseline requires:
  - confirmation text: `FIJAR ORIGINAL`
  - explicit replacement acknowledgement
  - backup acknowledgement
  - editor profile already saved
- The baseline stores immutable question snapshots with content hashes.
- Current questions are compared against the baseline by question code and content hash.

## Export Rules

- Exports are generated from the database, not from edited Excel files.
- The original baseline must exist before exporting color-coded workbooks.
- Colors:
  - red: removed questions
  - blue: modified questions
  - green: added questions
- Consolidated exports preserve the question, lineamiento, characteristic and status data needed to reopen the cycle.
- Instrument exports use the same diff engine and generate:
  - `Por lineamiento`: questions grouped by CNA hierarchy, with one column per audience/instrument.
  - `Por orden`: question-order view with one column per audience/instrument.
  - `Convención`: readable mapping for imported convention codes and question formats.
- Exported worksheets use wrapped text, frozen headers and row heights suitable for long CNA text.
- Provider review DOCX exports are scoped to one selected instrument/audience. Supported local evidence images (`png`, `jpg`, `jpeg`, `webp`, `bmp`, `gif`) are embedded in the document; unsupported files remain as evidence paths.

## History Rules

- Automatic history snapshots are created by app workflows and pruned to the latest 30 snapshots.
- Manual snapshots are created by the explicit sidebar save action and are persistent until the user deletes them with the reinforced confirmation flow.
- A snapshot stores the editable core state: questions and lineamientos. It is not a byte-for-byte backup of every auxiliary database table.
- Restoring a snapshot replaces the current question and lineamiento tables with the selected stored state.

## Desktop And OS Rules

- Runtime data uses Tauri platform directories:
  - Windows: per-user app config/data locations under the standard Windows app data profile.
  - Linux: XDG-compatible app config/data locations.
- The app can still work from a user-selected OneDrive/synced folder when the user wants the database file to live outside the default app data directory.
- `.acna` is the portable exchange format for a complete working database, including current questions, lineamientos, baseline snapshots, history snapshots, provider reviews, change logs and source document records.
- Opening a `.acna` file directly launches the app with that package as the active database.
- Installers register `.acna` as an Autoevaluacion CNA file association. Windows targets are configured for MSI/NSIS; Linux targets are configured for deb/rpm/AppImage.
- The app uses an explicit Content Security Policy instead of a null CSP. Frontend access stays limited to local Tauri IPC plus the Microsoft login and Graph endpoints required for sync.

## Duplicate Prevention

- Questions are unique by `code`.
- Lineamientos are unique by `scope + factor_code + characteristic_code + aspect_code`.
- `aspect_code` is not a manual workflow field. It is normalized from Excel `N° Aspecto` when available, or generated from factor, characteristic and aspect description.
- Factor and characteristic selection is data-driven from imported lineamientos plus built-in CNA presets. Characteristics can repeat across factors, so UI scopes characteristic choices by selected factor.
- Migrations clean pre-existing duplicates and create unique indexes.
- Import tests verify no repeated questions or lineamientos are produced from the sample consolidated workbook.

## Provider Review Rules

- Provider review is instrument-first, because each audience receives a different instrument assembled from the consolidated question bank.
- The default review filter shows pending items for the selected instrument.
- Resetting reviews can be scoped to one instrument/audience.
- DOCX reports include status, observation and evidence for the selected instrument only.
