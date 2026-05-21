# Autoevaluacion CNA Architecture

## Core Boundaries

- **Workspace:** local configuration, editor profile, database location, OneDrive folder sync and Microsoft Graph app-folder sync.
- **Live Collaboration:** optional Turso Cloud workspace where all editors use the same libSQL database with short-lived edit locks and optimistic conflict detection.
- **Import:** reads Excel consolidated workbooks, merges compatible sheets, forward-fills CNA hierarchy cells, deduplicates questions by code and lineamientos by CNA hierarchy.
- **CNA Model:** Factor -> Caracteristica -> Aspecto hierarchy. CNA factors ship with known presets, but factor codes/names are data-driven so future lineamientos can add factors without a Rust release. Aspect codes are internal keys: imported values are normalized and manual aspects get deterministic generated keys.
- **Question Bank:** current editable source of truth for questions, audiences, operational status and justification.
- **Original Baseline:** immutable snapshot of the imported workbook selected as the cycle original. It is replaced only with reinforced confirmation.
- **Validation:** checks blocking readiness conditions before provider delivery or export.
- **Export:** generates consolidated and instrument Excel workbooks from the database, comparing current questions against the original baseline and applying change colors.
- **Provider:** tracks provider links and validation status after delivery.
- **Provider Review:** checklist grouped by the same exported instrument workbooks used by the export screen. Reviewers mark each delivered question as correct, needs modification, or missing; they can attach evidence paths or local images and export a Word report for the selected instrument.
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
- Operational status is part of export diff semantics: `Modificar`, `Agregar` and `Eliminar` must force blue, green and red export rows respectively, even when the textual content hash is otherwise unchanged.
- Questions that exist in the original baseline but no longer exist in the current bank are still emitted as removed rows during export so consolidated and instrument files keep the historical red deletion trace.

## Export Rules

- Exports are generated from the database, not from edited Excel files.
- The original baseline must exist before exporting color-coded workbooks.
- Colors:
  - red: removed questions
  - blue: modified questions
  - green: added questions
- The same color semantics apply to consolidated and instrument exports, including the `Por orden` and `Por lineamiento` instrument sheets.
- Consolidated exports preserve the question, lineamiento, characteristic and status data needed to reopen the cycle.
- Instrument exports use the same diff engine and generate one workbook per public/audience group, matching the sample files. Each workbook contains:
  - `Por lineamiento`: questions grouped by CNA hierarchy, with one column per subpublic.
  - `Por orden`: question-order view, with one column per subpublic.
  - `Convención`: readable mapping for imported convention codes and question formats.
- Instrument públicos are derived by the backend from the same grouping used by the exporter, not from independent frontend parsing. This prevents the UI from offering públicos that cannot be exported.
- Instruments are first-class definitions in the database. On import, the backend detects instrument definitions from the públicos present in the consolidated workbook and seeds the historical workbook groups seen in the examples: Administrativos, Directivos, Estudiantes, Profesores de cátedra and Profesores de planta when those públicos exist.
- Users can create or edit instrument definitions and assign públicos to them. Público assignment is exclusive: one público key can belong to only one instrument at a time, enforced by the database.
- Custom instruments can group multiple públicos into one exported workbook. The workbook still contains subpúblico columns, and question membership is calculated from the assigned público keys.
- The consolidated workbook stores público and tipo de público separately. Instruments collapse that into one workbook per main público and subpublic columns. Example: `0Estudiantes + 00Pregrado` becomes the `Estudiantes` workbook with an `Estudiantes Pregrado` column; `1Profesores_Planta + 10Pregrado` becomes the `Profesores de planta` workbook with a `Profesores Pregrado` column.
- Instrument display labels follow the historical templates: `Maestrías` is displayed as `Maestría`, `Maestrías virtuales` as `Maestría Virtual`, `Especializaciones MQ` as `EMQ`, and `Especializaciones virtuales/extensión` as `Especializaciones virtual / extensión`.
- Público/subpúblico normalization is centralized in the Rust audience module and shared by import, export and provider review. Frontend screens consume backend-derived options instead of reimplementing these rules.
- Exported worksheets use wrapped text, frozen headers and row heights suitable for long CNA text.
- Provider review DOCX exports are scoped to one selected exported instrument workbook. Supported local evidence images (`png`, `jpg`, `jpeg`, `webp`, `bmp`, `gif`) are embedded in the document; unsupported files remain as evidence paths.

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
- Installers register `.acna` as an Autoevaluacion CNA file association. Windows targets are configured for MSI/NSIS; Linux targets are configured for deb/rpm/AppImage. `bun run build:windows:msi` and `bun run build:windows:nsis` are the native Windows build commands. On Linux/Arch, NSIS cross-builds use `bun run build:windows:nsis:cross` with `cargo-xwin` and the `x86_64-pc-windows-msvc` target; MSI still requires a Windows host.
- The app uses an explicit Content Security Policy instead of a null CSP. Frontend access stays limited to local Tauri IPC plus the Microsoft login and Graph endpoints required for sync.

## Collaboration Rules

- Turso Cloud is the recommended/default sync mode. It connects the repository directly to a remote `libsql://` database; OneDrive and Microsoft Graph are secondary backup/copy flows.
- Turso URL/token can be provided in the Workspace screen, runtime environment variables, or build-time environment variables embedded into the installer (`AUTOCNA_TURSO_DATABASE_URL` / `AUTOCNA_TURSO_AUTH_TOKEN`, with `TURSO_DATABASE_URL` / `TURSO_AUTH_TOKEN` also accepted). Desktop dev and installer builds all go through `scripts/tauri-with-env.mjs`, which loads local `.env.build`; `.env.build` is ignored by git and `.env.build.example` documents the required keys. `bun run dev`, `bun run dev:tauri`, `bun run build:windows:msi` and `bun run build:windows:nsis` therefore use the same Turso defaults.
- The token must be rotated if it is pasted into chat, committed, or shared in an installer package.
- In Turso mode, question and lineamiento screens poll for fresh data every few seconds so edits from other users appear without manual reload.
- Editing a question first checks for an active `question` collaboration lock. If another editor owns it, the table shows that editor's name and the edit action is disabled. If no lock exists, the app acquires a five-minute lock before opening the editor.
- Backend saves also reject attempts to update a question while another editor owns its lock, so bypassing the UI still cannot overwrite a locked edit.
- Saving a question sends the `updated_at` value that the editor originally loaded. If another editor saved the question first, the backend rejects the save and asks the user to refresh before trying again.
- Locks are coordination hints, not permanent permissions. Expired locks are pruned automatically so abandoned sessions do not block the team.

## Duplicate Prevention

- Questions are unique by `code`.
- Lineamientos are unique by `scope + factor_code + characteristic_code + aspect_code`.
- `aspect_code` is not a manual workflow field. It is normalized from Excel `N° Aspecto` when available, or generated from factor, characteristic and aspect description.
- Factor and characteristic selection is data-driven from imported lineamientos plus built-in CNA presets. Characteristics can repeat across factors, so UI scopes characteristic choices by selected factor.
- Migrations clean pre-existing duplicates and create unique indexes.
- Import tests verify no repeated questions or lineamientos are produced from the sample consolidated workbook.

## Provider Review Rules

- Provider review is exported-instrument-first and must show the same main instrument groups as the export screen: `Administrativos`, `Directivos`, `Estudiantes`, `Profesores de cátedra` and `Profesores de planta` when those groups exist in the imported data.
- Público/subpúblico columns are not independent provider review scopes. They stay inside the exported workbook, so `Estudiantes Pregrado` and `Estudiantes Maestría Virtual` both review under the `Estudiantes` instrument.
- Provider review stores a stable exported-instrument key and a display label. UI cards, filters and DOCX reports show the display label, while save/reset operations use the stable key.
- The default review filter shows pending items for the selected exported instrument.
- Resetting reviews is scoped to the selected exported instrument.
- DOCX reports include status, observation and evidence for the selected exported instrument only.
