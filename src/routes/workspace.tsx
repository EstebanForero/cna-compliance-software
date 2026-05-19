import { open, save } from "@tauri-apps/plugin-dialog";
import { useMutation, useQuery, useQueryClient } from "@tanstack/react-query";
import { createFileRoute } from "@tanstack/react-router";
import {
  CloudDownload,
  CloudUpload,
  Database,
  Download,
  FileSpreadsheet,
  FileUp,
  FolderOpen,
  ShieldAlert,
  Trash2,
  UserRound,
  AlertTriangle,
} from "lucide-react";
import type React from "react";
import { useState } from "react";

import { Badge } from "@/components/ui/badge";
import { Button } from "@/components/ui/button";
import {
  Card,
  CardContent,
  CardDescription,
  CardHeader,
  CardTitle,
} from "@/components/ui/card";
import { Input } from "@/components/ui/input";
import { api } from "@/lib/api";

export const Route = createFileRoute("/workspace")({
  component: WorkspacePage,
});

function WorkspacePage() {
  const queryClient = useQueryClient();
  const workspace = useQuery({ queryKey: ["workspace"], queryFn: api.workspace });
  const [clientId, setClientId] = useState("");
  const [tenantId, setTenantId] = useState("organizations");
  const [fullName, setFullName] = useState("");
  const [cycleName, setCycleName] = useState("Autoevaluacion CNA");
  const [pendingImportPath, setPendingImportPath] = useState("");
  const [resetConfirmation, setResetConfirmation] = useState("");
  const [acknowledgeBackup, setAcknowledgeBackup] = useState(false);
  const [acknowledgeIrreversible, setAcknowledgeIrreversible] = useState(false);

  const refresh = async () => {
    await queryClient.invalidateQueries({ queryKey: ["workspace"] });
    await queryClient.invalidateQueries({ queryKey: ["dashboard"] });
    await queryClient.invalidateQueries({ queryKey: ["questions"] });
    await queryClient.invalidateQueries({ queryKey: ["baseline-status"] });
  };

  const configure = useMutation({
    mutationFn: api.configureWorkspace,
    onSuccess: refresh,
  });
  const openDb = useMutation({
    mutationFn: api.openDatabase,
    onSuccess: refresh,
  });
  const importWorkbook = useMutation({
    mutationFn: api.importWorkbook,
    onSuccess: async () => {
      setPendingImportPath("");
      previewImport.reset();
      await refresh();
    },
  });
  const previewImport = useMutation({ mutationFn: api.previewImportWorkbook });
  const microsoftLogin = useMutation({
    mutationFn: api.loginWithMicrosoft,
    onSuccess: refresh,
  });
  const saveEditor = useMutation({
    mutationFn: api.saveEditorProfile,
    onSuccess: refresh,
  });
  const syncToGraph = useMutation({ mutationFn: api.syncToGraph });
  const syncFromGraph = useMutation({
    mutationFn: api.syncFromGraph,
    onSuccess: refresh,
  });
  const exportDatabasePackage = useMutation({ mutationFn: api.exportDatabasePackage });
  const openDatabasePackage = useMutation({
    mutationFn: api.openDatabasePackage,
    onSuccess: refresh,
  });
  const resetDatabase = useMutation({
    mutationFn: api.resetDatabaseData,
    onSuccess: async () => {
      setResetConfirmation("");
      setAcknowledgeBackup(false);
      setAcknowledgeIrreversible(false);
      await refresh();
      await queryClient.invalidateQueries({ queryKey: ["guideline-aspects"] });
      await queryClient.invalidateQueries({ queryKey: ["validations"] });
      await queryClient.invalidateQueries({ queryKey: ["provider-links"] });
    },
  });

  async function chooseOneDriveFolder() {
    const folderPath = await open({ directory: true, multiple: false });
    if (typeof folderPath !== "string") return;
    configure.mutate({ folderPath });
  }

  async function chooseExistingDatabase() {
    const databasePath = await open({
      directory: false,
      multiple: false,
      filters: [
        { name: "Autoevaluacion CNA", extensions: ["acna"] },
        { name: "libSQL database", extensions: ["db", "sqlite", "sqlite3"] },
      ],
    });
    if (typeof databasePath !== "string") return;
    openDb.mutate({ databasePath });
  }

  async function chooseDatabasePackage() {
    const path = await open({
      directory: false,
      multiple: false,
      filters: [{ name: "Autoevaluacion CNA", extensions: ["acna"] }],
    });
    if (typeof path !== "string") return;
    openDatabasePackage.mutate({ path });
  }

  async function exportCurrentDatabasePackage() {
    const path = await save({
      defaultPath: "autoevaluacion-cna.acna",
      filters: [{ name: "Autoevaluacion CNA", extensions: ["acna"] }],
    });
    if (!path) return;
    exportDatabasePackage.mutate({ path });
  }

  async function chooseWorkbook() {
    const path = await open({
      directory: false,
      multiple: false,
      filters: [{ name: "Excel workbook", extensions: ["xlsx", "xlsm", "xls"] }],
    });
    if (typeof path !== "string") return;
    setPendingImportPath(path);
    previewImport.mutate({ path, cycleName });
  }

  const resetReady =
    resetConfirmation.trim() === "BORRAR DATOS" &&
    acknowledgeBackup &&
    acknowledgeIrreversible;

  return (
    <div className="space-y-6">
      <section className="apple-hero p-6 md:p-8">
        <h1 className="text-3xl font-semibold md:text-4xl">Workspace</h1>
        <p className="mt-2 max-w-3xl text-sm leading-6 text-muted-foreground">
          Configure el archivo local, conecte Microsoft si necesita sincronizar
          y cargue el consolidado Excel inicial.
        </p>
      </section>

      <section className="grid gap-4 xl:grid-cols-[0.9fr_1.1fr]">
        <Card>
          <CardHeader>
            <CardTitle>Estado actual</CardTitle>
            <CardDescription>Resumen del archivo y conexiones activas.</CardDescription>
          </CardHeader>
          <CardContent className="space-y-4 text-sm">
            <div className="flex flex-wrap gap-2">
              <Badge variant={workspace.data?.hasQuestions ? "secondary" : "warning"}>
                {workspace.data?.hasQuestions ? "Banco cargado" : "Banco vacio"}
              </Badge>
              <Badge variant={workspace.data?.editorProfile ? "secondary" : "warning"}>
                {workspace.data?.editorProfile?.fullName ?? "Editor pendiente"}
              </Badge>
              <Badge variant={workspace.data?.microsoftAccount ? "secondary" : "outline"}>
                {workspace.data?.microsoftAccount?.email ?? "Microsoft no conectado"}
              </Badge>
            </div>
            <StatusRow
              icon={<Database />}
              label="Base de datos"
              value={workspace.data?.databasePath ?? "Cargando..."}
            />
            <StatusRow
              icon={<FolderOpen />}
              label="Carpeta OneDrive"
              value={workspace.data?.configuredOnedrivePath ?? "No configurada"}
            />
            <div className="grid gap-2 sm:grid-cols-2">
              <Button variant="outline" onClick={chooseExistingDatabase}>
                <Database className="size-4" />
                Abrir base
              </Button>
              <Button
                variant="outline"
                onClick={chooseOneDriveFolder}
                disabled={!workspace.data?.microsoftAccount || configure.isPending}
              >
                <FolderOpen className="size-4" />
                Elegir OneDrive
              </Button>
            </div>
            <div className="grid gap-2 sm:grid-cols-2">
              <Button
                variant="outline"
                onClick={chooseDatabasePackage}
                disabled={openDatabasePackage.isPending}
              >
                <FileUp className="size-4" />
                Abrir .acna
              </Button>
              <Button
                variant="outline"
                onClick={exportCurrentDatabasePackage}
                disabled={exportDatabasePackage.isPending}
              >
                <Download className="size-4" />
                Exportar .acna
              </Button>
            </div>
            {exportDatabasePackage.data ? (
              <p className="rounded-lg border bg-background/60 p-3 text-xs text-muted-foreground">
                {exportDatabasePackage.data.message} {exportDatabasePackage.data.path}
              </p>
            ) : null}
            {exportDatabasePackage.isError || openDatabasePackage.isError ? (
              <p className="text-sm text-destructive">
                No se pudo abrir/exportar el paquete. Use un archivo con extension .acna.
              </p>
            ) : null}
          </CardContent>
        </Card>

        <div className="grid gap-4">
          <Card>
            <CardHeader>
              <CardTitle>Responsable y Microsoft</CardTitle>
              <CardDescription>Datos minimos para trazabilidad y sincronizacion.</CardDescription>
            </CardHeader>
            <CardContent className="space-y-4">
              <div className="grid gap-3 md:grid-cols-[1fr_auto]">
                <Input
                  value={fullName}
                  onChange={(event) => setFullName(event.target.value)}
                  placeholder="Nombre completo del editor"
                />
                <Button
                  onClick={() => saveEditor.mutate({ fullName })}
                  disabled={fullName.trim().length < 3 || saveEditor.isPending}
                >
                  Guardar
                </Button>
              </div>
              <div className="grid gap-3 md:grid-cols-[1fr_0.7fr_auto]">
                <Input
                  value={clientId}
                  onChange={(event) => setClientId(event.target.value)}
                  placeholder="Application client id"
                />
                <Input
                  value={tenantId}
                  onChange={(event) => setTenantId(event.target.value)}
                  placeholder="Tenant id u organizations"
                />
                <Button
                  onClick={() =>
                    microsoftLogin.mutate({
                      clientId,
                      tenantId: tenantId || "organizations",
                    })
                  }
                  disabled={!clientId || microsoftLogin.isPending}
                >
                  <UserRound className="size-4" />
                  Iniciar sesion
                </Button>
              </div>
              {microsoftLogin.isError ? (
                <p className="text-sm text-destructive">
                  No se pudo completar el login. Revise el client id, redirect URI y
                  permisos de la app en Azure.
                </p>
              ) : null}
            </CardContent>
          </Card>

          <Card>
            <CardHeader>
              <CardTitle>Sincronizacion</CardTitle>
              <CardDescription>
                Carpeta local sincronizada o copia directa por Microsoft Graph.
              </CardDescription>
            </CardHeader>
            <CardContent className="flex flex-col gap-3 md:flex-row">
              <Button
                variant="outline"
                onClick={() => syncToGraph.mutate()}
                disabled={!workspace.data?.graphSyncAvailable || syncToGraph.isPending}
              >
                <CloudUpload className="size-4" />
                Subir por Graph
              </Button>
              <Button
                variant="outline"
                onClick={() => syncFromGraph.mutate()}
                disabled={!workspace.data?.graphSyncAvailable || syncFromGraph.isPending}
              >
                <CloudDownload className="size-4" />
                Descargar por Graph
              </Button>
            </CardContent>
          </Card>

          <Card>
            <CardHeader>
              <CardTitle>Importar preguntas y lineamientos</CardTitle>
              <CardDescription>
                Lee BASE/BASEvs, agrupa publicos por pregunta y deduplica la
                jerarquia CNA.
              </CardDescription>
            </CardHeader>
            <CardContent className="space-y-4">
              <div className="grid gap-3 md:grid-cols-[1fr_auto]">
                <Input
                  value={cycleName}
                  onChange={(event) => setCycleName(event.target.value)}
                  placeholder="Nombre del ciclo"
                />
                <Button
                  className="glass-action border-0"
                  onClick={chooseWorkbook}
                  disabled={previewImport.isPending || importWorkbook.isPending}
                >
                  <FileUp className="size-4" />
                  Previsualizar Excel
                </Button>
              </div>
              <p className="text-xs leading-5 text-muted-foreground">
                Metodo recomendado: revise el consolidado vigente, importe el
                archivo, valide cobertura, ajuste lineamientos nuevos y solo
                despues edite o agregue preguntas.
              </p>
              {previewImport.data ? (
                <div className="apple-tile space-y-3 p-3 text-sm">
                  <p className="flex items-center gap-2 font-medium">
                    <FileSpreadsheet className="size-4 text-primary" />
                    {previewImport.data.fileName}
                  </p>
                  <div className="grid gap-2 sm:grid-cols-3">
                    <PreviewMetric label="Preguntas" value={previewImport.data.detectedQuestions} />
                    <PreviewMetric
                      label="Lineamientos"
                      value={previewImport.data.detectedGuidelineAspects}
                    />
                    <PreviewMetric label="Filas omitidas" value={previewImport.data.skippedRows} />
                  </div>
                  <p className="text-xs text-muted-foreground">
                    Hoja base detectada: {previewImport.data.sheetName}. Publicos:{" "}
                    {previewImport.data.detectedAudiences.slice(0, 8).join(", ") || "N/A"}
                    {previewImport.data.detectedAudiences.length > 8 ? "..." : ""}
                  </p>
                  {previewImport.data.warnings.length > 0 ? (
                    <div className="space-y-1 rounded-lg border border-warning/30 bg-warning/10 p-3">
                      {previewImport.data.warnings.map((warning) => (
                        <p key={warning} className="flex gap-2 text-xs text-warning-foreground">
                          <AlertTriangle className="mt-0.5 size-3.5 shrink-0" />
                          {warning}
                        </p>
                      ))}
                    </div>
                  ) : null}
                  <div className="flex flex-wrap gap-2">
                    <Button
                      type="button"
                      onClick={() =>
                        importWorkbook.mutate({ path: pendingImportPath, cycleName })
                      }
                      disabled={!pendingImportPath || importWorkbook.isPending}
                    >
                      Confirmar importacion
                    </Button>
                    <Button
                      type="button"
                      variant="outline"
                      onClick={() => {
                        setPendingImportPath("");
                        previewImport.reset();
                      }}
                    >
                      Cancelar
                    </Button>
                  </div>
                </div>
              ) : null}
              {importWorkbook.data ? (
                <div className="apple-tile p-3 text-sm">
                  <p className="flex items-center gap-2 font-medium">
                    <FileSpreadsheet className="size-4 text-primary" />
                    {importWorkbook.data.fileName}
                  </p>
                  <p className="text-muted-foreground">
                    {importWorkbook.data.importedQuestions} preguntas importadas desde{" "}
                    {importWorkbook.data.sheetName};{" "}
                    {importWorkbook.data.importedGuidelineAspects} aspectos CNA detectados.{" "}
                    {importWorkbook.data.skippedRows} filas omitidas.
                  </p>
                </div>
              ) : null}
              {previewImport.isError || importWorkbook.isError ? (
                <p className="text-sm text-destructive">
                  No se pudo leer/importar el archivo. Verifique que sea un consolidado
                  con columnas de lineamiento, pregunta y publico.
                </p>
              ) : null}
            </CardContent>
          </Card>

          <Card>
            <CardHeader>
              <CardTitle className="flex items-center gap-2 text-destructive">
                <ShieldAlert className="size-4" />
                Zona peligrosa
              </CardTitle>
              <CardDescription>
                Borra preguntas, lineamientos, original, validaciones, proveedor
                y documentos fuente. Conserva configuracion local y editor.
              </CardDescription>
            </CardHeader>
            <CardContent className="space-y-3">
              <label className="flex gap-3 rounded-lg border bg-background/55 p-3 text-sm">
                <input
                  type="checkbox"
                  checked={acknowledgeBackup}
                  onChange={(event) => setAcknowledgeBackup(event.target.checked)}
                />
                Ya tengo respaldo o entiendo que debo sincronizar/exportar antes.
              </label>
              <label className="flex gap-3 rounded-lg border bg-background/55 p-3 text-sm">
                <input
                  type="checkbox"
                  checked={acknowledgeIrreversible}
                  onChange={(event) => setAcknowledgeIrreversible(event.target.checked)}
                />
                Entiendo que esta accion borra los datos de la base activa.
              </label>
              <Input
                value={resetConfirmation}
                onChange={(event) => setResetConfirmation(event.target.value)}
                placeholder="Escriba BORRAR DATOS"
              />
              <Button
                variant="destructive"
                disabled={!resetReady || resetDatabase.isPending}
                onClick={() =>
                  resetDatabase.mutate({
                    confirmationText: resetConfirmation,
                    acknowledgeBackup,
                    acknowledgeIrreversible,
                  })
                }
              >
                <Trash2 className="size-4" />
                Borrar datos
              </Button>
              {resetDatabase.data ? (
                <p className="text-sm text-muted-foreground">
                  {resetDatabase.data.message}
                </p>
              ) : null}
              {resetDatabase.isError ? (
                <p className="text-sm text-destructive">
                  No se pudo borrar. Complete las confirmaciones exactamente.
                </p>
              ) : null}
            </CardContent>
          </Card>
        </div>
      </section>
    </div>
  );
}

function StatusRow({
  icon,
  label,
  value,
}: {
  icon: React.ReactNode;
  label: string;
  value: string;
}) {
  return (
    <div className="flex gap-3 rounded-md border bg-background p-3">
      <div className="mt-0.5 text-primary [&_svg]:size-4">{icon}</div>
      <div className="min-w-0">
        <p className="text-xs text-muted-foreground">{label}</p>
        <p className="break-words font-medium">{value}</p>
      </div>
    </div>
  );
}

function PreviewMetric({ label, value }: { label: string; value: number }) {
  return (
    <div className="rounded-lg border bg-background/60 p-3">
      <p className="text-xs text-muted-foreground">{label}</p>
      <p className="mt-1 text-xl font-semibold">{value}</p>
    </div>
  );
}
