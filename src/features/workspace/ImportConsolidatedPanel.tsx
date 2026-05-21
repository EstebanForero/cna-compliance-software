import { open } from "@tauri-apps/plugin-dialog";
import { useMutation } from "@tanstack/react-query";
import { AlertTriangle, FileSpreadsheet, FileUp } from "lucide-react";
import { useState } from "react";

import { Button } from "@/components/ui/button";
import {
  Card,
  CardContent,
  CardDescription,
  CardHeader,
  CardTitle,
} from "@/components/ui/card";
import {
  Dialog,
  DialogContent,
  DialogDescription,
  DialogHeader,
  DialogTitle,
} from "@/components/ui/dialog";
import { Input } from "@/components/ui/input";
import { Progress } from "@/components/ui/progress";
import { api } from "@/lib/api";

type ImportConsolidatedPanelProps = {
  hasQuestions: boolean;
  onImported: () => Promise<void>;
};

const replacementPhrase = "REEMPLAZAR CONSOLIDADO";

export function ImportConsolidatedPanel({
  hasQuestions,
  onImported,
}: ImportConsolidatedPanelProps) {
  const [cycleName, setCycleName] = useState("Autoevaluacion CNA");
  const [pendingImportPath, setPendingImportPath] = useState("");
  const [replacementDialogOpen, setReplacementDialogOpen] = useState(false);
  const [acknowledgeExistingImport, setAcknowledgeExistingImport] = useState(false);
  const [acknowledgeImportBackup, setAcknowledgeImportBackup] = useState(false);
  const [replacementConfirmation, setReplacementConfirmation] = useState("");

  const previewImport = useMutation({ mutationFn: api.previewImportWorkbook });
  const importWorkbook = useMutation({
    mutationFn: api.importWorkbook,
    onSuccess: async () => {
      resetImportState();
      previewImport.reset();
      await onImported();
    },
  });

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

  function confirmImportWorkbook() {
    if (!pendingImportPath) return;
    if (hasQuestions) {
      setReplacementDialogOpen(true);
      return;
    }
    importWorkbook.mutate({ path: pendingImportPath, cycleName });
  }

  function confirmReplacementImport() {
    importWorkbook.mutate({
      path: pendingImportPath,
      cycleName,
      acknowledgeExistingData: acknowledgeExistingImport,
      acknowledgeBackup: acknowledgeImportBackup,
      replacementConfirmation,
    });
  }

  function resetImportState() {
    setPendingImportPath("");
    setReplacementDialogOpen(false);
    setAcknowledgeExistingImport(false);
    setAcknowledgeImportBackup(false);
    setReplacementConfirmation("");
  }

  return (
    <>
      <Card>
        <CardHeader>
          <CardTitle>Importar preguntas y lineamientos</CardTitle>
          <CardDescription>
            Lee BASE/BASEvs, agrupa publicos por pregunta y deduplica la jerarquia CNA.
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
            Metodo recomendado: revise el consolidado vigente, importe el archivo,
            valide cobertura, ajuste lineamientos nuevos y solo despues edite o agregue
            preguntas.
          </p>
          {previewImport.isPending || importWorkbook.isPending ? (
            <ImportProgress
              title={importWorkbook.isPending ? "Importando consolidado" : "Previsualizando Excel"}
              detail={
                importWorkbook.isPending
                  ? "Guardando preguntas, lineamientos, publicos e instrumentos en la base."
                  : "Leyendo hojas compatibles, colores, publicos y jerarquia CNA."
              }
              value={importWorkbook.isPending ? 72 : 38}
            />
          ) : null}
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
                  onClick={confirmImportWorkbook}
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
              {importWorkbook.error instanceof Error
                ? importWorkbook.error.message
                : "No se pudo leer/importar el archivo. Verifique que sea un consolidado con columnas de lineamiento, pregunta y publico."}
            </p>
          ) : null}
        </CardContent>
      </Card>

      <Dialog open={replacementDialogOpen} onOpenChange={setReplacementDialogOpen}>
        <DialogContent className="w-[min(92vw,38rem)]">
          <DialogHeader>
            <DialogTitle>Confirmar importacion sobre datos existentes</DialogTitle>
            <DialogDescription>
              La base actual ya contiene informacion. Importar este consolidado puede
              actualizar preguntas, lineamientos, publicos, instrumentos e historial
              asociado al banco actual.
            </DialogDescription>
          </DialogHeader>
          <div className="space-y-4 pt-2">
            <div className="rounded-lg border border-amber-300 bg-amber-50 p-3 text-sm text-amber-950 dark:border-amber-400/40 dark:bg-amber-950/35 dark:text-amber-100">
              Use esta opcion solo si el archivo seleccionado es el consolidado vigente
              que debe reemplazar o actualizar la base colaborativa.
            </div>
            <label className="flex items-start gap-3 text-sm">
              <input
                type="checkbox"
                className="mt-1"
                checked={acknowledgeExistingImport}
                onChange={(event) => setAcknowledgeExistingImport(event.target.checked)}
              />
              <span>
                Entiendo que la base ya tiene datos y que esta importacion puede modificar
                registros existentes.
              </span>
            </label>
            <label className="flex items-start gap-3 text-sm">
              <input
                type="checkbox"
                className="mt-1"
                checked={acknowledgeImportBackup}
                onChange={(event) => setAcknowledgeImportBackup(event.target.checked)}
              />
              <span>Ya exporte una copia .acna o guarde un snapshot manual antes de continuar.</span>
            </label>
            <div className="space-y-2">
              <p className="text-sm font-medium">
                Escriba <span className="font-mono">{replacementPhrase}</span>
              </p>
              <Input
                value={replacementConfirmation}
                onChange={(event) => setReplacementConfirmation(event.target.value)}
                placeholder={replacementPhrase}
              />
            </div>
            <div className="flex flex-wrap justify-end gap-2">
              <Button type="button" variant="outline" onClick={() => setReplacementDialogOpen(false)}>
                Cancelar
              </Button>
              <Button
                type="button"
                onClick={confirmReplacementImport}
                disabled={
                  importWorkbook.isPending ||
                  !acknowledgeExistingImport ||
                  !acknowledgeImportBackup ||
                  replacementConfirmation.trim() !== replacementPhrase
                }
              >
                Importar y actualizar base
              </Button>
            </div>
          </div>
        </DialogContent>
      </Dialog>
    </>
  );
}

function ImportProgress({
  title,
  detail,
  value,
}: {
  title: string;
  detail: string;
  value: number;
}) {
  return (
    <div className="rounded-xl border border-blue-200 bg-blue-50 p-4 text-sm text-blue-950 dark:border-blue-400/30 dark:bg-blue-950/70 dark:text-blue-100">
      <div className="mb-3 flex items-start justify-between gap-3">
        <div>
          <p className="font-semibold">{title}</p>
          <p className="mt-1 text-xs leading-5 text-blue-800 dark:text-blue-200">{detail}</p>
        </div>
        <span className="rounded-full bg-white px-2 py-1 text-xs font-medium text-blue-800 shadow-sm dark:bg-blue-900 dark:text-blue-100">
          {value}%
        </span>
      </div>
      <Progress value={value} className="bg-blue-100 dark:bg-blue-900" />
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
