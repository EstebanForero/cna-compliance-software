import { useQuery } from "@tanstack/react-query";
import { useMutation, useQueryClient } from "@tanstack/react-query";
import { createFileRoute } from "@tanstack/react-router";
import { Clock3, Trash2 } from "lucide-react";

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
import { useState } from "react";

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
  const restore = useMutation({
    mutationFn: api.restoreHistorySnapshot,
    onSuccess: async () => {
      setRestoreText("");
      await queryClient.invalidateQueries();
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

      <section className="grid gap-4 xl:grid-cols-[0.8fr_1.2fr]">
        <Card>
          <CardHeader>
            <CardTitle>Estados recuperables</CardTitle>
            <CardDescription>
              Ultimos 30 puntos persistentes antes de operaciones destructivas.
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
            {(snapshots.data ?? []).map((snapshot) => (
              <div key={snapshot.id} className="rounded-lg border bg-background/60 p-4">
                <div className="flex items-start justify-between gap-3">
                  <p className="text-sm font-semibold">{snapshot.summary}</p>
                  <Badge variant={snapshot.snapshotKind === "manual" ? "secondary" : "outline"}>
                    {snapshot.snapshotKind === "manual" ? "Manual" : "Auto"}
                  </Badge>
                </div>
                <p className="mt-1 text-xs text-muted-foreground">
                  {snapshot.editorName} / {new Date(snapshot.createdAt).toLocaleString()}
                </p>
                <div className="mt-3 flex flex-wrap gap-2">
                  <Button
                    variant="outline"
                    disabled={
                      restoreText.trim() !== "RESTAURAR HISTORIAL" || restore.isPending
                    }
                    onClick={() =>
                      restore.mutate({
                        snapshotId: snapshot.id,
                        confirmationText: restoreText,
                      })
                    }
                  >
                    Restaurar
                  </Button>
                  {snapshot.snapshotKind === "manual" ? (
                    <Button
                      variant="destructive"
                      disabled={
                        deleteText.trim() !== "ELIMINAR HISTORIAL" ||
                        deleteSnapshot.isPending
                      }
                      onClick={() =>
                        deleteSnapshot.mutate({
                          snapshotId: snapshot.id,
                          confirmationText: deleteText,
                        })
                      }
                    >
                      <Trash2 className="size-4" />
                      Eliminar
                    </Button>
                  ) : null}
                </div>
              </div>
            ))}
          </CardContent>
        </Card>

      <Card>
        <CardHeader>
          <CardTitle>Actividad reciente</CardTitle>
          <CardDescription>{logs.data?.length ?? 0} eventos visibles</CardDescription>
        </CardHeader>
        <CardContent className="space-y-3">
          {(logs.data ?? []).map((entry) => (
            <div key={entry.id} className="rounded-lg border bg-background/60 p-4">
              <div className="flex flex-col gap-2 md:flex-row md:items-center md:justify-between">
                <div>
                  <p className="text-sm font-semibold">{entry.action}</p>
                  <p className="mt-1 text-sm text-muted-foreground">{entry.summary}</p>
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
