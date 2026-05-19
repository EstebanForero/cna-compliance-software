import { ShieldCheck } from "lucide-react";

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
import type { BaselineStatus } from "@/lib/types";

type BaselineCardProps = {
  baseline?: BaselineStatus;
  pendingChanges: number;
  confirmationText: string;
  acknowledgeReplacement: boolean;
  acknowledgeBackup: boolean;
  pending: boolean;
  error: boolean;
  onConfirmationTextChange: (value: string) => void;
  onReplacementChange: (value: boolean) => void;
  onBackupChange: (value: boolean) => void;
  onMarkOriginal: () => void;
};

export function BaselineCard({
  baseline,
  pendingChanges,
  confirmationText,
  acknowledgeReplacement,
  acknowledgeBackup,
  pending,
  error,
  onConfirmationTextChange,
  onReplacementChange,
  onBackupChange,
  onMarkOriginal,
}: BaselineCardProps) {
  const canMark =
    confirmationText.trim() === "FIJAR ORIGINAL" &&
    acknowledgeReplacement &&
    acknowledgeBackup;

  return (
    <Card>
      <CardHeader className="gap-3 md:flex-row md:items-start md:justify-between md:space-y-0">
        <div>
          <CardTitle>Linea base original</CardTitle>
          <CardDescription>
            Fije aqui el consolidado oficial que se usara para clasificar preguntas
            agregadas, modificadas y eliminadas.
          </CardDescription>
        </div>
        <Badge variant={baseline?.hasOriginal ? "secondary" : "warning"}>
          {baseline?.hasOriginal ? "Original fijado" : "Sin original"}
        </Badge>
      </CardHeader>
      <CardContent className="grid gap-4 lg:grid-cols-[0.85fr_1.15fr]">
        <div className="apple-tile p-3 text-sm">
          <p className="font-medium">
            {baseline?.sourceDocument?.fileName ?? "Ultimo consolidado importado"}
          </p>
          <p className="mt-1 break-words text-muted-foreground">
            {baseline?.sourceDocument?.path ??
              "La app usara el documento fuente mas reciente."}
          </p>
          <div className="mt-4 grid grid-cols-3 gap-2 text-center">
            <MiniMetric label="Original" value={baseline?.originalQuestions ?? 0} />
            <MiniMetric label="Actual" value={baseline?.currentQuestions ?? 0} />
            <MiniMetric label="Cambios" value={pendingChanges} />
          </div>
        </div>
        <div className="space-y-3">
          <label className="flex gap-3 rounded-lg border bg-background/55 p-3 text-sm">
            <input
              type="checkbox"
              checked={acknowledgeReplacement}
              onChange={(event) => onReplacementChange(event.target.checked)}
            />
            Entiendo que esto reemplaza la comparacion original del ciclo.
          </label>
          <label className="flex gap-3 rounded-lg border bg-background/55 p-3 text-sm">
            <input
              type="checkbox"
              checked={acknowledgeBackup}
              onChange={(event) => onBackupChange(event.target.checked)}
            />
            Confirmo que existe respaldo antes de fijar el original.
          </label>
          <div className="grid gap-2 md:grid-cols-[1fr_auto]">
            <Input
              value={confirmationText}
              onChange={(event) => onConfirmationTextChange(event.target.value)}
              placeholder="Escriba FIJAR ORIGINAL"
            />
            <Button
              className="glass-action border-0"
              disabled={!canMark || pending}
              onClick={onMarkOriginal}
            >
              <ShieldCheck className="size-4" />
              Fijar original
            </Button>
          </div>
          {error ? (
            <p className="text-sm text-destructive">
              No se pudo fijar el original. Verifique confirmaciones y que ya
              exista un consolidado importado.
            </p>
          ) : null}
        </div>
      </CardContent>
    </Card>
  );
}

function MiniMetric({ label, value }: { label: string; value: number }) {
  return (
    <div className="rounded-lg border bg-background/65 p-2">
      <p className="text-xs text-muted-foreground">{label}</p>
      <p className="mt-1 font-semibold">{value}</p>
    </div>
  );
}
