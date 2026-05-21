import { useMutation, useQuery, useQueryClient } from "@tanstack/react-query";
import { ArrowRight, ClipboardCheck, UserRound } from "lucide-react";
import type React from "react";
import { useState } from "react";

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

export function OnboardingGate({ children }: { children: React.ReactNode }) {
  const queryClient = useQueryClient();
  const workspace = useQuery({ queryKey: ["workspace"], queryFn: api.workspace });
  const [fullName, setFullName] = useState("");
  const saveEditor = useMutation({
    mutationFn: api.saveEditorProfile,
    onSuccess: async () => {
      await queryClient.invalidateQueries({ queryKey: ["workspace"] });
      await queryClient.invalidateQueries({ queryKey: ["dashboard"] });
    },
  });

  if (workspace.isLoading) {
    return (
      <div className="grid min-h-screen place-items-center px-6">
        <div className="h-2 w-40 overflow-hidden rounded-sm bg-muted">
          <div className="h-full w-1/2 animate-pulse rounded-sm bg-primary" />
        </div>
      </div>
    );
  }

  if (workspace.data?.editorProfile) {
    return children;
  }

  return (
    <main className="grid min-h-screen place-items-center px-5 py-8">
      <div className="w-full max-w-4xl">
        <div className="mb-8 flex items-center gap-3">
          <div className="glass-action flex size-11 items-center justify-center">
            <ClipboardCheck className="size-5" />
          </div>
          <div>
            <p className="text-sm font-semibold">Autoevaluacion CNA</p>
            <p className="text-xs text-muted-foreground">Fuente unica colaborativa</p>
          </div>
        </div>

        <Card className="overflow-hidden">
          <div className="grid lg:grid-cols-[1fr_0.8fr]">
            <div className="p-6 md:p-8">
              <CardHeader className="p-0">
                <CardTitle className="text-3xl leading-tight md:text-4xl">
                  Primero, identifique al editor responsable.
                </CardTitle>
                <CardDescription className="max-w-xl pt-3 text-base leading-7">
                  Cada importacion, pregunta nueva y ajuste de lineamiento queda
                  asociado a este nombre en el historial local de cambios.
                </CardDescription>
              </CardHeader>
              <CardContent className="mt-8 space-y-4 p-0">
                <div className="flex flex-col gap-3 sm:flex-row">
                  <Input
                    autoFocus
                    value={fullName}
                    onChange={(event) => setFullName(event.target.value)}
                    placeholder="Nombre completo"
                    onKeyDown={(event) => {
                      if (event.key === "Enter" && fullName.trim().length >= 3) {
                        saveEditor.mutate({ fullName });
                      }
                    }}
                  />
                  <Button
                    onClick={() => saveEditor.mutate({ fullName })}
                    disabled={fullName.trim().length < 3 || saveEditor.isPending}
                  >
                    Continuar
                    <ArrowRight className="size-4" />
                  </Button>
                </div>
                {saveEditor.isError ? (
                  <p className="text-sm text-destructive">
                    Escriba el nombre completo del responsable.
                  </p>
                ) : null}
              </CardContent>
            </div>
            <div className="border-t bg-muted/35 p-6 lg:border-l lg:border-t-0">
              <div className="space-y-3">
                {[
                  "Usar colaboración Turso configurada",
                  "Importar preguntas y lineamientos",
                  "Revisar lineamientos CNA",
                  "Usar OneDrive o Graph solo como copia",
                  "Validar y preparar entrega",
                ].map((step, index) => (
                  <div key={step} className="apple-tile flex items-center gap-3 p-3">
                    <div className="workflow-pill flex size-8 items-center justify-center text-sm font-semibold text-primary">
                      {index + 1}
                    </div>
                    <p className="text-sm font-medium">{step}</p>
                  </div>
                ))}
              </div>
              <div className="mt-6 flex items-center gap-2 text-xs text-muted-foreground">
                <UserRound className="size-4" />
                El perfil se guarda solo en esta instalacion.
              </div>
            </div>
          </div>
        </Card>
      </div>
    </main>
  );
}
