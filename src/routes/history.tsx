import { useQuery } from "@tanstack/react-query";
import { useMutation, useQueryClient } from "@tanstack/react-query";
import { createFileRoute } from "@tanstack/react-router";
import { Clock3, RotateCcw, Trash2 } from "lucide-react";
import { useMemo, useState } from "react";

import { Badge } from "@/components/ui/badge";
import {
  Card,
  CardContent,
  CardDescription,
  CardHeader,
  CardTitle,
} from "@/components/ui/card";
import { api } from "@/lib/api";
import { Button } from "@/components/ui/button";
import { Input } from "@/components/ui/input";
import type { HistorySnapshot } from "@/lib/types";

export const Route = createFileRoute("/history")({
  component: HistoryPage,
});

function HistoryPage() {
  const queryClient = useQueryClient();
  const [restoreText, setRestoreText] = useState("");
  const [deleteText, setDeleteText] = useState("");
  const logs = useQuery({ queryKey: ["change-logs"], queryFn: api.changeLogs });
  const snapshots = useQuery({
    queryKey: ["history-snapshots"],
    queryFn: api.historySnapshots,
  });
  const [selectedFixationId, setSelectedFixationId] = useState<string | null>(
    null,
  );
  const restore = useMutation({
    mutationFn: api.restoreHistorySnapshot,
    onSuccess: async () => {
      setRestoreText("");
      await queryClient.invalidateQueries();
      setSelectedFixationId(null);
    },
  });
  const deleteSnapshot = useMutation({
    mutationFn: api.deleteHistorySnapshot,
    onSuccess: async () => {
      setDeleteText("");
      await queryClient.invalidateQueries({ queryKey: ["history-snapshots"] });
      await queryClient.invalidateQueries({ queryKey: ["change-logs"] });
    },
  });
  const snapshotGroups = useMemo(
    () => groupSnapshotsByFixation(snapshots.data ?? [], selectedFixationId),
    [snapshots.data, selectedFixationId],
  );

  return (
    <div className="space-y-6">
      <section className="apple-hero p-6 md:p-8">
        <h1 className="text-3xl font-semibold md:text-4xl">Historial</h1>
        <p className="mt-3 max-w-3xl text-sm leading-6 text-muted-foreground">
          Ultimas 100 operaciones registradas en la base local. Sirve para
          auditar importaciones, altas, eliminaciones, exportaciones y cambios
          de linea base.
        </p>
      </section>

      <section className="grid gap-4 xl:grid-cols-[0.85fr_1.15fr]">
        <Card>
          <CardHeader>
            <CardTitle>Fijaciones de original</CardTitle>
            <CardDescription>
              Cada fijacion guarda una copia recuperable de la base y agrupa su
              propio historial.
            </CardDescription>
          </CardHeader>
          <CardContent className="space-y-3">
            <Input
              value={restoreText}
              onChange={(event) => setRestoreText(event.target.value)}
              placeholder="Escriba RESTAURAR HISTORIAL"
            />
            <Input
              value={deleteText}
              onChange={(event) => setDeleteText(event.target.value)}
              placeholder="Escriba ELIMINAR HISTORIAL para borrar manuales"
            />
            {snapshotGroups.fixations.map((fixation, index) => {
              const selected =
                fixation.id === snapshotGroups.selectedFixation?.id;

              return (
                <div
                  key={fixation.id}
                  className={`rounded-lg border p-4 ${
                    selected
                      ? "border-primary/40 bg-primary/5"
                      : "bg-background/60"
                  }`}
                >
                  <div className="flex items-start justify-between gap-3">
                    <div>
                      <p className="text-sm font-semibold">
                        {fixation.summary}
                      </p>
                      <p className="mt-1 text-xs text-muted-foreground">
                        {fixation.editorName} /{" "}
                        {new Date(fixation.createdAt).toLocaleString()}
                      </p>
                    </div>
                    <Badge variant={index === 0 ? "default" : "secondary"}>
                      {index === 0 ? "Actual" : "Fijacion"}
                    </Badge>
                  </div>
                  <div className="mt-3 flex flex-wrap gap-2">
                    <Button
                      variant={selected ? "secondary" : "outline"}
                      onClick={() => setSelectedFixationId(fixation.id)}
                    >
                      Ver historial
                    </Button>
                    <Button
                      variant="outline"
                      disabled={
                        restoreText.trim() !== "RESTAURAR HISTORIAL" ||
                        restore.isPending
                      }
                      onClick={() =>
                        restore.mutate({
                          snapshotId: fixation.id,
                          confirmationText: restoreText,
                        })
                      }
                    >
                      <RotateCcw className="size-4" />
                      Restaurar fijacion
                    </Button>
                  </div>
                </div>
              );
            })}
            {snapshots.isSuccess && snapshotGroups.fixations.length === 0 ? (
              <p className="rounded-lg border bg-background/60 p-6 text-sm text-muted-foreground">
                Aun no hay fijaciones de original. Use Fijar original para crear
                la primera.
              </p>
            ) : null}
          </CardContent>
        </Card>

        <Card>
          <CardHeader>
            <CardTitle>
              {snapshotGroups.selectedFixation
                ? snapshotGroups.selectedFixation.id ===
                  snapshotGroups.fixations[0]?.id
                  ? "Historial de la fijacion actual"
                  : "Historial de la fijacion"
                : "Historial sin fijacion"}
            </CardTitle>
            <CardDescription>
              {snapshotGroups.selectedFixation
                ? `Cambios guardados desde ${new Date(
                    snapshotGroups.selectedFixation.createdAt,
                  ).toLocaleString()}.`
                : "Cambios guardados antes de la primera fijacion."}
            </CardDescription>
          </CardHeader>
          <CardContent className="space-y-3">
            {snapshotGroups.selectedFixation ? (
              <SnapshotItem
                snapshot={snapshotGroups.selectedFixation}
                badge="Fijacion"
                restoreText={restoreText}
                restorePending={restore.isPending}
                onRestore={(snapshot) =>
                  restore.mutate({
                    snapshotId: snapshot.id,
                    confirmationText: restoreText,
                  })
                }
              />
            ) : null}
            {snapshotGroups.selectedHistory.map((snapshot) => (
              <SnapshotItem
                key={snapshot.id}
                snapshot={snapshot}
                badge={snapshotKindLabel(snapshot)}
                restoreText={restoreText}
                restorePending={restore.isPending}
                deleteText={deleteText}
                deletePending={deleteSnapshot.isPending}
                onRestore={(snapshot) =>
                  restore.mutate({
                    snapshotId: snapshot.id,
                    confirmationText: restoreText,
                  })
                }
                onDelete={
                  snapshot.snapshotKind === "manual"
                    ? (snapshot) =>
                        deleteSnapshot.mutate({
                          snapshotId: snapshot.id,
                          confirmationText: deleteText,
                        })
                    : undefined
                }
              />
            ))}
            {snapshots.isSuccess &&
            !snapshotGroups.selectedFixation &&
            snapshotGroups.selectedHistory.length === 0 ? (
              <p className="rounded-lg border bg-background/60 p-6 text-sm text-muted-foreground">
                No hay estados recuperables para mostrar.
              </p>
            ) : null}
            {snapshots.isSuccess &&
            snapshotGroups.selectedFixation &&
            snapshotGroups.selectedHistory.length === 0 ? (
              <p className="rounded-lg border bg-background/60 p-6 text-sm text-muted-foreground">
                Esta fijacion no tiene cambios posteriores guardados.
              </p>
            ) : null}
          </CardContent>
        </Card>
      </section>

      <section>
        <Card>
          <CardHeader>
            <CardTitle>Actividad reciente</CardTitle>
            <CardDescription>
              {logs.data?.length ?? 0} eventos visibles
            </CardDescription>
          </CardHeader>
          <CardContent className="space-y-3">
            {(logs.data ?? []).map((entry) => (
              <div
                key={entry.id}
                className="rounded-lg border bg-background/60 p-4"
              >
                <div className="flex flex-col gap-2 md:flex-row md:items-center md:justify-between">
                  <div>
                    <p className="text-sm font-semibold">{entry.action}</p>
                    <p className="mt-1 text-sm text-muted-foreground">
                      {entry.summary}
                    </p>
                  </div>
                  <div className="flex items-center gap-2 text-xs text-muted-foreground">
                    <Clock3 className="size-4" />
                    {new Date(entry.createdAt).toLocaleString()}
                  </div>
                </div>
                <p className="mt-2 text-xs text-muted-foreground">
                  {entry.editorName} / {entry.entity}
                </p>
              </div>
            ))}
            {logs.isSuccess && (logs.data?.length ?? 0) === 0 ? (
              <p className="rounded-lg border bg-background/60 p-6 text-sm text-muted-foreground">
                Aun no hay cambios registrados.
              </p>
            ) : null}
          </CardContent>
        </Card>
      </section>
    </div>
  );
}

function SnapshotItem({
  snapshot,
  badge,
  restoreText,
  restorePending,
  deleteText,
  deletePending,
  onRestore,
  onDelete,
}: {
  snapshot: HistorySnapshot;
  badge: string;
  restoreText: string;
  restorePending: boolean;
  deleteText?: string;
  deletePending?: boolean;
  onRestore: (snapshot: HistorySnapshot) => void;
  onDelete?: (snapshot: HistorySnapshot) => void;
}) {
  return (
    <div className="rounded-lg border bg-background/60 p-4">
      <div className="flex items-start justify-between gap-3">
        <p className="text-sm font-semibold">{snapshot.summary}</p>
        <Badge
          variant={snapshot.snapshotKind === "auto" ? "outline" : "secondary"}
        >
          {badge}
        </Badge>
      </div>
      <p className="mt-1 text-xs text-muted-foreground">
        {snapshot.editorName} / {new Date(snapshot.createdAt).toLocaleString()}
      </p>
      <div className="mt-3 flex flex-wrap gap-2">
        <Button
          variant="outline"
          disabled={
            restoreText.trim() !== "RESTAURAR HISTORIAL" || restorePending
          }
          onClick={() => onRestore(snapshot)}
        >
          <RotateCcw className="size-4" />
          Restaurar
        </Button>
        {onDelete ? (
          <Button
            variant="destructive"
            disabled={
              deleteText?.trim() !== "ELIMINAR HISTORIAL" || deletePending
            }
            onClick={() => onDelete(snapshot)}
          >
            <Trash2 className="size-4" />
            Eliminar
          </Button>
        ) : null}
      </div>
    </div>
  );
}

function groupSnapshotsByFixation(
  snapshots: HistorySnapshot[],
  selectedFixationId: string | null,
) {
  const fixations = snapshots
    .filter((snapshot) => snapshot.snapshotKind === "baseline")
    .sort(compareNewestFirst);
  const selectedFixation =
    fixations.find((snapshot) => snapshot.id === selectedFixationId) ??
    fixations[0] ??
    null;

  if (!selectedFixation) {
    return {
      fixations,
      selectedFixation,
      selectedHistory: snapshots
        .filter((snapshot) => snapshot.snapshotKind !== "baseline")
        .sort(compareNewestFirst),
    };
  }

  const selectedCreatedAt = new Date(selectedFixation.createdAt).getTime();
  const nextFixation = fixations
    .filter(
      (fixation) => new Date(fixation.createdAt).getTime() > selectedCreatedAt,
    )
    .sort(compareOldestFirst)[0];
  const nextCreatedAt = nextFixation
    ? new Date(nextFixation.createdAt).getTime()
    : Number.POSITIVE_INFINITY;

  return {
    fixations,
    selectedFixation,
    selectedHistory: snapshots
      .filter((snapshot) => {
        if (snapshot.snapshotKind === "baseline") {
          return false;
        }
        const createdAt = new Date(snapshot.createdAt).getTime();
        return createdAt >= selectedCreatedAt && createdAt < nextCreatedAt;
      })
      .sort(compareNewestFirst),
  };
}

function snapshotKindLabel(snapshot: HistorySnapshot) {
  if (snapshot.snapshotKind === "manual") {
    return "Manual";
  }
  if (snapshot.snapshotKind === "baseline") {
    return "Fijacion";
  }
  return "Auto";
}

function compareNewestFirst(a: HistorySnapshot, b: HistorySnapshot) {
  return new Date(b.createdAt).getTime() - new Date(a.createdAt).getTime();
}

function compareOldestFirst(a: HistorySnapshot, b: HistorySnapshot) {
  return new Date(a.createdAt).getTime() - new Date(b.createdAt).getTime();
}
