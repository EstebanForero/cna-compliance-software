# Autoevaluacion CNA Architecture

## Architecture Goals

- Keep Excel as an import/export artifact, not the source of truth.
- Keep domain rules in Rust service modules where they are testable without the UI.
- Keep the frontend focused on workflow and interaction, not data normalization.
- Keep persistence interchangeable through `AutoEvalRepository`.
- Keep collaboration predictable by using Turso as one shared database plus short-lived locks.
- Make destructive workflow steps explicit in both UI and backend.

## Layered Architecture

```text
React Routes
  -> Feature Components
    -> src/lib/api.ts
      -> Tauri Commands
        -> AutoEvaluationService
          -> AutoEvalRepository trait
            -> LibSqlAutoEvalRepository
              -> libSQL local file or Turso Cloud
```

### Frontend Layers

- **Routes (`src/routes`)**
  - Own page layout, route params, query wiring and high-level orchestration.
  - Should not contain complete workflows when those workflows have modal state, several mutations, or domain-specific UI. Those move to `src/features`.
  - Example: `workspace.tsx` owns configuration layout; `ImportConsolidatedPanel` owns Excel import preview, confirmation and import mutations.

- **Features (`src/features`)**
  - Own reusable domain experiences: question editing, lineamiento management, instrument configuration, import workflow, dashboard cards.
  - May compose UI primitives and call `api`.
  - May contain frontend-only derived presentation helpers, but not canonical business normalization.

- **UI Primitives (`src/components/ui`)**
  - Small reusable controls following the shadcn-style component boundary.
  - No domain knowledge.

- **API Contract (`src/lib/api.ts`, `src/lib/types.ts`)**
  - Single frontend boundary to Tauri commands.
  - TypeScript shapes mirror Rust `serde(rename_all = "camelCase")` DTOs.

### Backend Layers

- **Tauri composition (`src-tauri/src/lib.rs`)**
  - Initializes plugins, app state and command registration only.
  - Does not contain command bodies or business rules.

- **Commands (`src-tauri/src/commands`)**
  - Thin controllers grouped by application capability:
    - `workspace`: local/Turso/Microsoft configuration and `.acna` packages.
    - `bank`: dashboard, questions, lineamientos, instruments, import/export, validation.
    - `collaboration`: presence and locks.
    - `history`: manual snapshots and restore.
    - `provider`: provider links, question review, evidence and DOCX.
  - Commands may validate transport-level input, resolve current editor, and record changes after service calls.
  - Commands do not parse Excel, calculate diffs, or decide export structure.

- **Application Service (`src-tauri/src/service.rs`, `src-tauri/src/service/*`)**
  - Owns workflow rules and orchestration across repository methods.
  - Enforces destructive confirmations such as baseline replacement, database cleanup and import over existing data.
  - Owns validation, baseline diff, provider review and export orchestration.
  - Depends only on `AutoEvalRepository`, not on libSQL.

- **Repository Contract (`src-tauri/src/repository.rs`)**
  - Defines persistence operations required by services.
  - Enables mock-based unit tests with `mockall`.

- **Persistence (`src-tauri/src/db.rs`, `src-tauri/src/db/*`)**
  - Implements `AutoEvalRepository` for libSQL/Turso.
  - Owns schema, row mapping, migrations, deduplication constraints and database-specific queries.
  - Does not own UX workflows or export decisions.

- **Import/Export Support**
  - `importer.rs` and `importer/*` parse Excel, detect cell color marks and normalize incoming workbook rows into domain DTOs.
  - `audience.rs` centralizes public/subpublic/instrument label normalization shared by import, export and provider review.
  - `service/export.rs` writes consolidated and instrument workbooks using domain diffs.

## Module Map

| Path | Responsibility |
| --- | --- |
| `src-tauri/src/domain.rs` | Serializable domain DTOs and enums shared with frontend. |
| `src-tauri/src/workspace_state.rs` | Runtime workspace, config file, app data paths, Turso default resolution. |
| `src-tauri/src/commands/*` | Tauri command layer split by capability. |
| `src-tauri/src/service.rs` | Core application service facade. |
| `src-tauri/src/service/baseline.rs` | Original baseline snapshots and question diff semantics. |
| `src-tauri/src/service/export.rs` | Consolidated/instrument Excel generation. |
| `src-tauri/src/service/provider.rs` | Provider review grouping, evidence and DOCX report generation. |
| `src-tauri/src/service/validation.rs` | Blocking/warning validation rules. |
| `src-tauri/src/db/*` | libSQL persistence implementation, schema, rows and per-aggregate queries. |
| `src-tauri/src/importer/*` | Excel workbook extraction and mark/color detection. |
| `src/features/*` | Frontend domain workflows and reusable page sections. |
| `src/routes/*` | Route-level layouts and page orchestration. |

## Architecture Decision Records

### ADR-001: Tauri Desktop Instead Of Server App

Decision: Ship as a Tauri desktop app with local OS integration.

Reasoning: The workflow is document-heavy, used by a small internal team, and needs file dialogs, local Excel import/export, local evidence images and Windows/Linux packaging. A server-side app would add hosting and authentication complexity without improving the primary workflow.

### ADR-002: libSQL/Turso Repository Boundary

Decision: Use libSQL locally and Turso remotely behind `AutoEvalRepository`.

Reasoning: The same repository implementation can connect to a local database file or a Turso Cloud database. The service layer remains testable with mocks and can later support another persistence backend if needed.

### ADR-003: Excel Is Not The Source Of Truth

Decision: Excel files are imported as source documents and exported as delivery artifacts. The database is the source of truth after import.

Reasoning: The current manual process depends on Excel formatting conventions and color marks, but long-term traceability, deduplication, collaboration and validation require structured storage.

### ADR-004: Backend Owns Public/Instrument Normalization

Decision: Publico/subpublico and instrument grouping rules live in Rust (`audience.rs`) and are consumed by frontend APIs.

Reasoning: These rules affect import, export, provider review and UI options. Reimplementing them in React would create mismatches and the exact bugs previously seen with duplicated/incorrect publics.

### ADR-005: Reinforced Confirmation In UI And Backend

Decision: Irreversible or high-risk actions require confirmation in the frontend and are also enforced in the backend.

Reasoning: UI-only protection can be bypassed by a stale screen, command call or future route. Backend enforcement protects imports over existing data, baseline replacement and database cleanup.

### ADR-006: Collaboration Locks Are Acquired On Edit Attempt

Decision: Do not poll locks for every visible question. Acquire a question lock when the user clicks edit, then poll only known blocked locks.

Reasoning: This reduces Turso reads and makes lock checks proportional to actual editing conflicts. Backend writes still enforce locks and optimistic `updated_at` checks.

### ADR-007: `.acna` Is A Domain Package Extension

Decision: `.acna` files are complete libSQL/SQLite database packages with a domain-specific file association.

Reasoning: Users need a portable handoff/backup file that contains current data, baseline, history and provider review state. A custom extension communicates app ownership while preserving database portability.

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
- In Turso mode, screens refresh workspace/presence periodically. Lock checks are not run for every visible question.
- Editing a question attempts to acquire a `question` collaboration lock. If another editor owns it, the app shows a top-right toast with that editor's name and caches that question as known blocked.
- Known blocked question locks are rechecked every 10 seconds until they disappear. This keeps Turso reads proportional to conflicts instead of table size.
- If no lock exists, the app acquires a five-minute lock before opening the editor.
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
