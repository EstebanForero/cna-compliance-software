import { open, save } from "@tauri-apps/plugin-dialog";
import { useMutation, useQuery } from "@tanstack/react-query";
import { createFileRoute } from "@tanstack/react-router";
import { CheckCircle2, Download, FileCheck2 } from "lucide-react";
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
import {
  Dialog,
  DialogContent,
  DialogDescription,
  DialogHeader,
  DialogTitle,
} from "@/components/ui/dialog";
import {
  Select,
  SelectContent,
  SelectItem,
  SelectTrigger,
  SelectValue,
} from "@/components/ui/select";
import { api } from "@/lib/api";

export const Route = createFileRoute("/exports")({
  component: ExportsPage,
});

function ExportsPage() {
  const [selectedPublic, setSelectedPublic] = useState("all");
  const [successDialogOpen, setSuccessDialogOpen] = useState(false);
  const baseline = useQuery({
    queryKey: ["baseline-status"],
    queryFn: api.baselineStatus,
  });
  const instrumentPublics = useQuery({
    queryKey: ["instrument-public-options"],
    queryFn: api.instrumentPublicOptions,
  });
  const exportWorkbook = useMutation({
    mutationFn: api.exportWorkbook,
    onSuccess: () => {
      setSuccessDialogOpen(true);
    },
  });
  const publicOptions = instrumentPublics.data ?? [];

  async function chooseExportPath() {
    const path = await save({
      defaultPath: "consolidado-autoevaluacion-cna.xlsx",
      filters: [{ name: "Excel workbook", extensions: ["xlsx"] }],
    });
    if (!path) return;
    exportWorkbook.mutate({ path, kind: "consolidated", instrumentPublic: null });
  }

  async function chooseInstrumentPath() {
    const path = await open({
      directory: true,
      multiple: false,
      title: "Seleccione la carpeta para guardar los instrumentos por público",
    });
    if (typeof path !== "string") return;
    exportWorkbook.mutate({
      path,
      kind: "instruments",
      instrumentPublic: selectedPublic === "all" ? null : selectedPublic,
    });
  }

  return (
    <div className="space-y-6">
      <Dialog open={successDialogOpen} onOpenChange={setSuccessDialogOpen}>
        <DialogContent>
          <DialogHeader>
            <DialogTitle className="flex items-center gap-2">
              <CheckCircle2 className="size-5 text-primary" />
              Exportación completada
            </DialogTitle>
            <DialogDescription>
              El archivo o carpeta se generó correctamente en la ruta seleccionada.
            </DialogDescription>
          </DialogHeader>
          <div className="apple-tile p-3 text-sm">
            <p className="font-medium">
              {exportWorkbook.data?.kind === "instruments"
                ? "Instrumentos exportados"
                : "Consolidado exportado"}
            </p>
            <p className="mt-1 break-words text-muted-foreground">
              {exportWorkbook.data?.path}
            </p>
            {exportWorkbook.data ? (
              <div className="mt-3 grid gap-2 text-xs sm:grid-cols-3">
                <ExportStat label="Agregadas" value={exportWorkbook.data.addedQuestions} />
                <ExportStat label="Modificadas" value={exportWorkbook.data.modifiedQuestions} />
                <ExportStat label="Eliminadas" value={exportWorkbook.data.removedQuestions} />
              </div>
            ) : null}
          </div>
          <Button type="button" onClick={() => setSuccessDialogOpen(false)}>
            Listo
          </Button>
        </DialogContent>
      </Dialog>
      <section className="apple-hero p-6 md:p-8">
        <div className="flex flex-col justify-between gap-4 md:flex-row md:items-end">
          <div>
            <h1 className="text-3xl font-semibold md:text-4xl">Exportaciones</h1>
            <p className="mt-3 max-w-3xl text-sm leading-6 text-muted-foreground">
              Compare los cambios del banco y genere Excel con colores de
              trazabilidad. La linea base original se fija desde el panel.
            </p>
          </div>
          <Badge variant={baseline.data?.hasOriginal ? "secondary" : "warning"}>
            {baseline.data?.hasOriginal ? "Original fijado" : "Sin original"}
          </Badge>
        </div>
      </section>

      <section className="grid gap-4 lg:grid-cols-4">
        <Metric title="Original" value={baseline.data?.originalQuestions ?? 0} />
        <Metric title="Actual" value={baseline.data?.currentQuestions ?? 0} />
        <Metric title="Modificadas" value={baseline.data?.modifiedQuestions ?? 0} />
        <Metric title="Agregadas" value={baseline.data?.addedQuestions ?? 0} />
      </section>

      <section className="grid gap-4 xl:grid-cols-[0.8fr_1.2fr]">
        <Card>
          <CardHeader>
            <CardTitle>Linea base</CardTitle>
            <CardDescription>
              El exportador usa el original fijado en el panel para calcular
              eliminadas, modificadas y agregadas.
            </CardDescription>
          </CardHeader>
          <CardContent className="space-y-3 text-sm">
            <div className="apple-tile p-3">
              <p className="font-medium">
                {baseline.data?.sourceDocument?.fileName ?? "Sin consolidado fuente"}
              </p>
              <p className="mt-1 break-words text-muted-foreground">
                {baseline.data?.sourceDocument?.path ??
                  "Importe un consolidado y fije el original desde el panel."}
              </p>
            </div>
            {!baseline.data?.hasOriginal ? (
              <div className="rounded-lg border border-warning/30 bg-warning/10 p-3 text-warning-foreground">
                <p className="font-medium">No hay línea base original fijada.</p>
                <p className="mt-1 text-muted-foreground">
                  El banco actual tiene preguntas, pero todavía no existe una copia
                  original para comparar cambios. Por eso aquí aparecen como
                  agregadas. Fije el consolidado original desde el panel antes de
                  exportar con colores de trazabilidad.
                </p>
              </div>
            ) : null}
          </CardContent>
        </Card>

        <Card>
          <CardHeader>
            <CardTitle>Exportar Excel</CardTitle>
            <CardDescription>
              Genera el consolidado coloreado y un archivo de instrumento por
              público, cada uno con sus columnas de subpúblicos.
            </CardDescription>
          </CardHeader>
          <CardContent className="space-y-4">
            <div className="grid gap-3">
              <Legend color="bg-destructive/25" label="Eliminadas" />
              <Legend color="bg-primary/20" label="Modificadas" />
              <Legend color="bg-secondary/20" label="Agregadas" />
            </div>
            <div className="grid gap-2 rounded-lg border bg-background/55 p-3">
              <label className="text-sm font-medium">Público para instrumentos</label>
              <Select value={selectedPublic} onValueChange={setSelectedPublic}>
                <SelectTrigger>
                  <SelectValue placeholder="Seleccione el público a exportar" />
                </SelectTrigger>
                <SelectContent>
                  <SelectItem value="all">Todos los públicos</SelectItem>
                  {publicOptions.map((option) => (
                    <SelectItem key={option.public} value={option.public}>
                      {option.label} · {option.questionCount} preguntas
                    </SelectItem>
                  ))}
                </SelectContent>
              </Select>
              <p className="text-xs leading-5 text-muted-foreground">
                Cada público se exporta como un libro independiente. Sus columnas
                corresponden a los subpúblicos detectados, por ejemplo Pregrado,
                Maestría, Doctorado o Especializaciones.
              </p>
              {selectedPublic !== "all" ? (
                <div className="flex flex-wrap gap-2">
                  {publicOptions
                    .find((option) => option.public === selectedPublic)
                    ?.subpublics.map((subpublic) => (
                      <Badge key={subpublic} variant="outline">
                        {subpublic}
                      </Badge>
                    ))}
                </div>
              ) : null}
            </div>
            <Button
              variant="outline"
              disabled={!baseline.data?.hasOriginal || exportWorkbook.isPending}
              onClick={chooseExportPath}
            >
              <Download className="size-4" />
              Exportar consolidado
            </Button>
            <Button
              disabled={
                !baseline.data?.hasOriginal ||
                publicOptions.length === 0 ||
                exportWorkbook.isPending
              }
              onClick={chooseInstrumentPath}
            >
              <Download className="size-4" />
              Exportar instrumentos por público
            </Button>
            {exportWorkbook.data ? (
              <div className="apple-tile p-3 text-sm">
                <p className="font-medium">Exportado</p>
                <p className="mt-1 break-words text-muted-foreground">
                  {exportWorkbook.data.path}
                </p>
              </div>
            ) : null}
            {exportWorkbook.isError ? (
              <p className="text-sm text-destructive">
                No se pudo exportar. Fije primero el original y elija una ruta
                con permisos de escritura.
              </p>
            ) : null}
          </CardContent>
        </Card>
      </section>
    </div>
  );
}

function Metric({ title, value }: { title: string; value: number }) {
  return (
    <Card>
      <CardContent className="flex items-center justify-between p-4">
        <div>
          <p className="text-sm text-muted-foreground">{title}</p>
          <p className="mt-2 text-2xl font-semibold">{value}</p>
        </div>
        <FileCheck2 className="size-5 text-primary" />
      </CardContent>
    </Card>
  );
}

function Legend({ color, label }: { color: string; label: string }) {
  return (
    <div className="flex items-center gap-3 rounded-lg border bg-background/55 p-3 text-sm">
      <span className={`size-4 rounded-sm ${color}`} />
      {label}
    </div>
  );
}

function ExportStat({ label, value }: { label: string; value: number }) {
  return (
    <div className="rounded-lg border bg-background/60 p-2">
      <p className="text-muted-foreground">{label}</p>
      <p className="mt-1 text-base font-semibold">{value}</p>
    </div>
  );
}
