import { useMutation } from "@tanstack/react-query";
import { createFileRoute } from "@tanstack/react-router";
import { AlertTriangle, CheckCircle2, RefreshCw } from "lucide-react";

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

export const Route = createFileRoute("/validation")({
  component: ValidationPage,
});

function ValidationPage() {
  const validations = useMutation({ mutationFn: api.validations });
  const issues = validations.data ?? [];
  const blocking = issues.filter((issue) => issue.severity === "blocking").length;

  return (
    <div className="space-y-6">
      <section className="flex flex-col justify-between gap-4 md:flex-row md:items-end">
        <div>
          <h1 className="text-2xl font-semibold">Validaciones previas</h1>
          <p className="mt-2 max-w-3xl text-sm leading-6 text-muted-foreground">
            Reglas de cobertura minima antes de generar instrumentos Excel o
            paquetes de entrega al proveedor.
          </p>
        </div>
        <Button onClick={() => validations.mutate()} disabled={validations.isPending}>
          <RefreshCw className="size-4" />
          Ejecutar validacion
        </Button>
      </section>

      <section className="grid gap-4 lg:grid-cols-[0.8fr_1.2fr]">
        <Card>
          <CardHeader>
            <CardTitle>Resultado</CardTitle>
            <CardDescription>Resumen de la ultima corrida local.</CardDescription>
          </CardHeader>
          <CardContent className="space-y-4">
            <div className="flex items-center gap-3 rounded-md border bg-background p-4">
              <div className="flex size-11 items-center justify-center rounded-md bg-muted text-primary">
                {blocking === 0 ? (
                  <CheckCircle2 className="size-5" />
                ) : (
                  <AlertTriangle className="size-5 text-destructive" />
                )}
              </div>
              <div>
                <p className="text-2xl font-semibold">{issues.length}</p>
                <p className="text-sm text-muted-foreground">hallazgos registrados</p>
              </div>
            </div>
            <div className="grid grid-cols-2 gap-3">
              <div className="rounded-md border bg-background p-3">
                <p className="text-xs text-muted-foreground">Bloqueantes</p>
                <p className="mt-2 text-xl font-semibold">{blocking}</p>
              </div>
              <div className="rounded-md border bg-background p-3">
                <p className="text-xs text-muted-foreground">Advertencias</p>
                <p className="mt-2 text-xl font-semibold">{issues.length - blocking}</p>
              </div>
            </div>
          </CardContent>
        </Card>

        <Card>
          <CardHeader>
            <CardTitle>Hallazgos</CardTitle>
            <CardDescription>
              Preguntas sin audiencia, convenciones faltantes y eliminaciones sin
              justificacion.
            </CardDescription>
          </CardHeader>
          <CardContent className="space-y-3">
            {validations.isIdle ? (
              <p className="text-sm text-muted-foreground">
                Ejecute la validacion para guardar una corrida y revisar pendientes.
              </p>
            ) : null}
            {issues.map((issue) => (
              <div
                key={issue.id}
                className="flex flex-col gap-2 rounded-md border bg-background p-3 sm:flex-row sm:items-center sm:justify-between"
              >
                <p className="text-sm">{issue.message}</p>
                <Badge
                  variant={issue.severity === "blocking" ? "destructive" : "warning"}
                >
                  {issue.severity === "blocking" ? "Bloqueante" : "Advertencia"}
                </Badge>
              </div>
            ))}
            {validations.isSuccess && issues.length === 0 ? (
              <div className="rounded-md border bg-background p-4 text-sm">
                No hay hallazgos bloqueantes para la exportacion.
              </div>
            ) : null}
          </CardContent>
        </Card>
      </section>
    </div>
  );
}
