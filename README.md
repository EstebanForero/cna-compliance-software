# Autoevaluacion CNA

Aplicacion de escritorio para gestionar el banco unico de preguntas de autoevaluacion CNA, importar el consolidado Excel historico, mantener trazabilidad por linea base, colaborar sobre una base Turso/libSQL y generar los consolidados, instrumentos y reportes de proveedor que usa el proceso actual.

## Alcance Funcional

- Importar consolidado Excel y extraer preguntas, lineamientos CNA, publicos, instrumentos, convenciones y estados operativos.
- Evitar duplicados de preguntas y lineamientos usando identidad estable por codigo de pregunta y jerarquia CNA.
- Gestionar lineamientos CNA como Factor -> Caracteristica -> Aspecto, con factores/caracteristicas data-driven.
- Crear, editar, marcar para modificar, agregar o eliminar preguntas sin perder comparacion contra la linea base original.
- Fijar o reemplazar linea base original solo con confirmacion reforzada.
- Exportar consolidado e instrumentos con colores de trazabilidad: rojo para eliminadas, azul para modificadas y verde para agregadas.
- Configurar instrumentos como grupos de publicos/subpublicos detectados desde el consolidado.
- Revisar entregas del proveedor por instrumento, registrar estado por pregunta, observaciones y evidencia.
- Exportar DOCX de revision de proveedor para el instrumento seleccionado.
- Guardar snapshots manuales persistentes y restaurar estados anteriores de preguntas/lineamientos.
- Usar Turso Cloud como fuente colaborativa por defecto, con bloqueos de edicion y deteccion optimista de conflictos.
- Usar OneDrive/Microsoft Graph como flujo secundario de respaldo/copia.
- Exportar y abrir paquetes `.acna` como bases portables con historial.

## Stack

- Desktop: Tauri 2.
- Frontend: React, TypeScript, TanStack Router, TanStack Query, Tailwind, componentes estilo shadcn.
- Backend: Rust, libSQL/Turso, `async-trait`, pruebas con `mockall`.
- Excel/DOCX: importadores y exportadores Rust para mantener formato y reglas de negocio cerca de la fuente de datos.

## Comandos

Instalar dependencias:

```bash
bun install
```

Desarrollo:

```bash
bun run dev
```

Build frontend:

```bash
bun run build
```

Build binario local para Arch/Linux sin empaquetar:

```bash
bun run build:arch
```

Build Windows NSIS desde Linux con `cargo-xwin`:

```bash
bun run build:windows:nsis:cross
```

Pruebas backend:

```bash
cd src-tauri
cargo test
```

## Variables De Build

El script `scripts/tauri-with-env.mjs` carga `.env.build` para desarrollo y builds. Use `.env.build.example` como plantilla.

```bash
AUTOCNA_TURSO_DATABASE_URL=libsql://...
AUTOCNA_TURSO_AUTH_TOKEN=...
```

Use siempre los scripts `bun run dev`, `bun run build:arch`, `bun run build:windows:*` o `bun run tauri -- ...` para que la configuracion de Turso sea inyectada. Ejecutar `bunx tauri build --no-bundle` directamente omite `.env.build`.

## Estructura Del Codigo

### Frontend

- `src/routes`: pantallas de la app. Deben coordinar queries, mutaciones y layout de pagina, no concentrar reglas de negocio complejas.
- `src/features`: componentes y logica por dominio visual.
  - `dashboard`: estado del banco, linea base y dialogo inicial.
  - `workspace`: configuracion, importacion del consolidado y resguardos.
  - `lineaments`: exploracion, creacion, edicion y borrado de lineamientos.
  - `questions`: banco, edicion de preguntas, selector CNA, publicos/instrumentos y formatos de respuesta.
  - `instruments`: definicion de instrumentos y asignacion de publicos.
- `src/components/ui`: primitives reutilizables tipo shadcn.
- `src/lib/api.ts`: frontera unica entre React y comandos Tauri.
- `src/lib/types.ts`: tipos compartidos del contrato frontend/backend.

### Backend

- `src-tauri/src/lib.rs`: composicion de Tauri, plugins, estado e invoke handler.
- `src-tauri/src/commands`: capa de comandos Tauri por caso de uso.
  - `workspace`: bases locales, Turso, Microsoft, paquetes `.acna`.
  - `bank`: preguntas, lineamientos, instrumentos, importacion, exportacion, validacion.
  - `collaboration`: presencia, bloqueos y lectura selectiva de locks.
  - `history`: snapshots manuales y restauracion.
  - `provider`: enlaces, revision, evidencia y DOCX.
- `src-tauri/src/service.rs` y `src-tauri/src/service/*`: capa de aplicacion. Orquesta reglas de negocio usando el repositorio.
- `src-tauri/src/repository.rs`: trait que define la frontera de persistencia.
- `src-tauri/src/db.rs` y `src-tauri/src/db/*`: implementacion libSQL/Turso del repositorio.
- `src-tauri/src/domain.rs`: DTOs, entidades y enums serializados.
- `src-tauri/src/importer.rs` y `src-tauri/src/importer/*`: lectura Excel, colores y normalizacion del consolidado.
- `src-tauri/src/audience.rs`: normalizacion central de publicos, subpublicos e instrumentos.
- `src-tauri/src/workspace_state.rs`: carga de configuracion local, rutas Tauri y seleccion local/Turso.

## Reglas De Arquitectura

- El frontend no reimplementa normalizacion de publicos ni reglas de instrumentos; consume opciones derivadas por backend.
- Las rutas React no deben crecer como controladores monoliticos; extraiga flujos completos a `src/features/<dominio>`.
- Los comandos Tauri no contienen reglas profundas; validan contexto, obtienen editor y llaman a `AutoEvaluationService`.
- El servicio no depende de libSQL directamente; solo del trait `AutoEvalRepository`.
- La base de datos no decide reglas de negocio de exportacion, importacion o validacion; persiste y consulta.
- Todo cambio destructivo o irreversible debe tener confirmacion reforzada en UI y backend.
- Las pruebas de reglas de negocio viven en la capa de servicio; las pruebas de persistencia viven en `db::tests`.

## Colaboracion

Turso es la fuente colaborativa recomendada. La app usa:

- presencia cada 30 segundos;
- locks por recurso para preguntas, lineamientos e instrumentos;
- adquisicion de lock al intentar editar;
- polling solo de locks conocidos/bloqueados, no de todas las filas visibles;
- `updated_at` esperado en guardado de preguntas para detectar cambios entre carga y escritura.

El trabajo offline no debe considerarse colaborativo. Si no hay conexion con Turso, los cambios son locales y requieren reconciliacion manual o un flujo futuro de borradores/conflictos.

## Documentacion Relacionada

- [ARCHITECTURE.md](./ARCHITECTURE.md): decisiones, reglas de dominio y mapa de capas.
- [contexto_autoevaluacion_propuesta.md](./contexto_autoevaluacion_propuesta.md): contexto funcional y requisitos del problema.
