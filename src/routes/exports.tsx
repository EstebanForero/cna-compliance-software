import { save } from "@tauri-apps/plugin-dialog";
import { useMutation, useQuery } from "@tanstack/react-query";
import { createFileRoute } from "@tanstack/react-router";
import { Download, FileCheck2 } from "lucide-react";

import { Badge } from "@/components/ui/badge";
import { Button } from "@/components/ui/button";
import {
  Card,
  CardContent,
  CardDescription,
  CardHeader,
  CardTitle,
} from "@/components/ui/card";
import { api } from "@/lib/api";

export const Route = createFileRoute("/exports")({
  component: ExportsPage,
});

function ExportsPage() {
  const baseline = useQuery({
    queryKey: ["baseline-status"],
    queryFn: api.baselineStatus,
  });
  const exportWorkbook = useMutation({ mutationFn: api.exportWorkbook });

  async function chooseExportPath() {
    const path = await save({
      defaultPath: "consolidado-autoevaluacion-cna.xlsx",
      filters: [{ name: "Excel workbook", extensions: ["xlsx"] }],
    });
    if (!path) return;
    exportWorkbook.mutate({ path, kind: "consolidated" });
  }

  async function chooseInstrumentPath() {
    const path = await save({
      defaultPath: "instrumentos-autoevaluacion-cna.xlsx",
      filters: [{ name: "Excel workbook", extensions: ["xlsx"] }],
    });
    if (!path) return;
    exportWorkbook.mutate({ path, kind: "instruments" });
  }

  return (
    <div className="space-y-6">
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
              <p className="rounded-lg border bg-background/55 p-3 text-muted-foreground">
                Vaya al panel para fijar la linea base original antes de exportar.
              </p>
            ) : null}
          </CardContent>
        </Card>

        <Card>
          <CardHeader>
            <CardTitle>Exportar Excel</CardTitle>
            <CardDescription>
              Genera el consolidado coloreado: rojo eliminadas, azul modificadas,
              verde agregadas.
            </CardDescription>
          </CardHeader>
          <CardContent className="space-y-4">
            <div className="grid gap-3">
              <Legend color="bg-destructive/25" label="Eliminadas" />
              <Legend color="bg-primary/20" label="Modificadas" />
              <Legend color="bg-secondary/20" label="Agregadas" />
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
              disabled={!baseline.data?.hasOriginal || exportWorkbook.isPending}
              onClick={chooseInstrumentPath}
            >
              <Download className="size-4" />
              Exportar instrumentos
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
