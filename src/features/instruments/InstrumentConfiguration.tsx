import { useMutation, useQuery, useQueryClient } from "@tanstack/react-query";
import { Lock, Pencil, Plus, X } from "lucide-react";
import { useMemo, useState } from "react";

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
import type { AvailableInstrumentPublic } from "@/lib/types";

export function InstrumentConfiguration() {
  const queryClient = useQueryClient();
  const [instrumentLabel, setInstrumentLabel] = useState("");
  const [selectedPublicKeys, setSelectedPublicKeys] = useState<string[]>([]);
  const [editingInstrumentId, setEditingInstrumentId] = useState<string | null>(null);
  const [showAvailablePublics, setShowAvailablePublics] = useState(false);
  const [customPublic, setCustomPublic] = useState("");
  const [notice, setNotice] = useState("");
  const workspace = useQuery({ queryKey: ["workspace"], queryFn: api.workspace });
  const instruments = useQuery({
    queryKey: ["instrument-definitions"],
    queryFn: api.instrumentDefinitions,
  });
  const instrumentIds = useMemo(
    () => (instruments.data ?? []).map((instrument) => instrument.id),
    [instruments.data],
  );
  const locks = useQuery({
    queryKey: ["collaboration-locks", "instrumentDefinition", instrumentIds],
    queryFn: () =>
      api.collaborationLocksForResources({
        resourceType: "instrumentDefinition",
        resourceIds: instrumentIds,
      }),
    enabled: Boolean(workspace.data?.tursoConnected && instrumentIds.length > 0),
    refetchInterval: workspace.data?.tursoConnected ? 10000 : false,
  });
  const availablePublics = useQuery({
    queryKey: ["available-instrument-publics"],
    queryFn: api.availableInstrumentPublics,
  });
  const releaseInstrumentLock = useMutation({
    mutationFn: api.releaseCollaborationLock,
    onSuccess: async () => {
      await queryClient.invalidateQueries({
        queryKey: ["collaboration-locks", "instrumentDefinition"],
      });
    },
  });
  const saveInstrument = useMutation({
    mutationFn: api.saveInstrumentDefinition,
    onSuccess: async () => {
      if (editingInstrumentId && workspace.data?.tursoConnected) {
        releaseInstrumentLock.mutate({
          resourceType: "instrumentDefinition",
          resourceId: editingInstrumentId,
        });
      }
      clearInstrumentDraft();
      await queryClient.invalidateQueries({ queryKey: ["instrument-definitions"] });
      await queryClient.invalidateQueries({ queryKey: ["instrument-public-options"] });
      await queryClient.invalidateQueries({ queryKey: ["available-instrument-publics"] });
      await queryClient.invalidateQueries({ queryKey: ["provider-review-items"] });
    },
  });
  const acquireInstrumentLock = useMutation({
    mutationFn: api.acquireCollaborationLock,
    onSuccess: async (_, request) => {
      loadInstrumentDraft(request.resourceId);
      setNotice("Edición bloqueada para ti. Otros editores no podrán modificar este instrumento.");
      await queryClient.invalidateQueries({
        queryKey: ["collaboration-locks", "instrumentDefinition"],
      });
    },
    onError: (error) => {
      setNotice(
        error instanceof Error
          ? error.message
          : "Otro editor esta modificando este instrumento.",
      );
    },
  });
  const assignablePublics = (availablePublics.data ?? []).filter(
    (option) =>
      !option.assignedInstrumentId || option.assignedInstrumentId === editingInstrumentId,
  );
  const selectedPublics = assignablePublics.filter((option) =>
    selectedPublicKeys.includes(option.key),
  );
  const availableToAddPublics = assignablePublics.filter(
    (option) => !selectedPublicKeys.includes(option.key),
  );
  const lockByInstrument = useMemo(
    () => new Map((locks.data ?? []).map((lock) => [lock.resourceId, lock])),
    [locks.data],
  );
  const currentEditorName = workspace.data?.editorProfile?.fullName ?? "";

  function togglePublic(publicKey: string) {
    setSelectedPublicKeys((current) =>
      current.includes(publicKey)
        ? current.filter((item) => item !== publicKey)
        : [...current, publicKey],
    );
  }

  function loadInstrumentDraft(instrumentId: string) {
    const instrument = instruments.data?.find((item) => item.id === instrumentId);
    if (!instrument) return;
    setEditingInstrumentId(instrument.id);
    setInstrumentLabel(instrument.label);
    setSelectedPublicKeys(
      expandInstrumentPublicKeys(instrument.publicKeys, availablePublics.data ?? []),
    );
  }

  function editInstrument(instrumentId: string) {
    const lock = lockByInstrument.get(instrumentId);
    if (lock && lock.editorName !== currentEditorName) {
      setNotice(`Este instrumento esta siendo editado por ${lock.editorName}.`);
      return;
    }
    if (!workspace.data?.tursoConnected) {
      loadInstrumentDraft(instrumentId);
      setNotice("");
      return;
    }
    acquireInstrumentLock.mutate({
      resourceType: "instrumentDefinition",
      resourceId: instrumentId,
    });
  }

  function cancelInstrumentEdit() {
    if (editingInstrumentId && workspace.data?.tursoConnected) {
      releaseInstrumentLock.mutate({
        resourceType: "instrumentDefinition",
        resourceId: editingInstrumentId,
      });
    }
    clearInstrumentDraft();
  }

  function clearInstrumentDraft() {
    setEditingInstrumentId(null);
    setInstrumentLabel("");
    setSelectedPublicKeys([]);
    setShowAvailablePublics(false);
    setCustomPublic("");
    setNotice("");
  }

  function addCustomPublic() {
    const value = customPublic.trim().replace(/\s+/g, " ");
    if (!value) return;
    setSelectedPublicKeys((current) =>
      current.includes(value) ? current : [...current, value].sort(),
    );
    setCustomPublic("");
  }

  return (
    <section className="grid gap-4 xl:grid-cols-[1fr_1fr]">
      <Card>
        <CardHeader>
          <CardTitle>Instrumentos</CardTitle>
          <CardDescription>
            Defina los libros de aplicación que se exportarán y revisarán por proveedor.
          </CardDescription>
        </CardHeader>
        <CardContent className="space-y-3">
          {(instruments.data ?? []).map((instrument) => (
            <div key={instrument.id} className="rounded-lg border bg-background/55 p-3">
              <div className="flex items-center justify-between gap-3">
                <div>
                  <p className="font-medium">{instrument.label}</p>
                  <p className="mt-1 text-xs text-muted-foreground">
                    {instrument.publicKeys.length} público
                    {instrument.publicKeys.length === 1 ? "" : "s"} configurado
                    {instrument.publicKeys.length === 1 ? "" : "s"}
                  </p>
                </div>
                <div className="flex items-center gap-2">
                  <Badge variant={instrument.isSystem ? "secondary" : "outline"}>
                    {instrument.isSystem ? "Detectado" : "Manual"}
                  </Badge>
                  {lockByInstrument.has(instrument.id) ? (
                    <Badge variant="outline" className="bg-primary/10 text-primary">
                      <Lock className="mr-1 size-3" />
                      {lockByInstrument.get(instrument.id)?.editorName === currentEditorName
                        ? "Tu edición"
                        : "Bloqueado"}
                    </Badge>
                  ) : null}
                  <Button
                    type="button"
                    variant="outline"
                    size="icon"
                    aria-label={
                      lockByInstrument.get(instrument.id)?.editorName &&
                      lockByInstrument.get(instrument.id)?.editorName !== currentEditorName
                        ? `Bloqueado por ${lockByInstrument.get(instrument.id)?.editorName}`
                        : "Editar instrumento"
                    }
                    onClick={() => editInstrument(instrument.id)}
                    disabled={
                      Boolean(lockByInstrument.get(instrument.id)) &&
                      lockByInstrument.get(instrument.id)?.editorName !== currentEditorName
                    }
                  >
                    {lockByInstrument.has(instrument.id) ? (
                      <Lock className="size-4" />
                    ) : (
                      <Pencil className="size-4" />
                    )}
                  </Button>
                </div>
              </div>
            </div>
          ))}
          {instruments.data?.length === 0 ? (
            <p className="text-sm text-muted-foreground">
              Importe un consolidado para detectar instrumentos.
            </p>
          ) : null}
        </CardContent>
      </Card>

      <Card>
        <CardHeader>
          <CardTitle>{editingInstrumentId ? "Editar instrumento" : "Crear instrumento"}</CardTitle>
          <CardDescription>
            Un público solo puede pertenecer a un instrumento. Esto controla la exportación,
            la revisión de proveedor y los reportes.
          </CardDescription>
        </CardHeader>
        <CardContent className="space-y-4">
          {notice ? (
            <p className="rounded-lg border border-primary/20 bg-primary/10 p-3 text-sm text-primary">
              {notice}
            </p>
          ) : null}
          {editingInstrumentId ? (
            <div className="flex items-center gap-2 rounded-lg border bg-background/70 p-3 text-sm">
              <Lock className="size-4 text-primary" />
              <span>
                Bloqueado para edición por{" "}
                <strong>{currentEditorName || "este usuario"}</strong>.
              </span>
            </div>
          ) : null}
          <Input
            value={instrumentLabel}
            onChange={(event) => setInstrumentLabel(event.target.value)}
            placeholder="Nombre del instrumento"
          />
          <div className="space-y-3">
            <div>
              <p className="mb-2 text-xs font-medium text-muted-foreground">
                {editingInstrumentId ? "Públicos del instrumento" : "Públicos disponibles"}
              </p>
              <div className="max-h-64 space-y-2 overflow-y-auto rounded-lg border bg-background/55 p-3">
                {(editingInstrumentId ? selectedPublics : assignablePublics).map((option) => (
                  <label
                    key={option.key}
                    className="flex items-start gap-3 rounded-md p-2 text-sm hover:bg-muted/60"
                  >
                    <input
                      className="mt-1"
                      type="checkbox"
                      checked={selectedPublicKeys.includes(option.key)}
                      onChange={() => togglePublic(option.key)}
                    />
                    <span>
                      <span className="font-medium text-foreground">{option.label}</span>
                      {option.subpublics.length ? (
                        <span className="mt-1 flex max-h-20 flex-wrap gap-1 overflow-y-auto pr-1">
                          {option.subpublics.map((subpublic) => (
                            <Badge
                              key={subpublic}
                              variant="outline"
                              className="bg-background text-[11px]"
                            >
                              {subpublic}
                            </Badge>
                          ))}
                        </span>
                      ) : null}
                    </span>
                  </label>
                ))}
                {selectedPublicKeys
                  .filter((key) => !assignablePublics.some((option) => option.key === key))
                  .map((key) => (
                    <label
                      key={key}
                      className="flex items-start gap-3 rounded-md p-2 text-sm hover:bg-muted/60"
                    >
                      <input
                        className="mt-1"
                        type="checkbox"
                        checked
                        onChange={() => togglePublic(key)}
                      />
                      <span className="font-medium text-foreground">{key}</span>
                    </label>
                  ))}
                {(editingInstrumentId ? selectedPublics : assignablePublics).length === 0 ? (
                  <p className="rounded-md bg-muted/50 p-3 text-sm text-muted-foreground">
                    {editingInstrumentId
                      ? "Este instrumento no tiene públicos seleccionados."
                      : "No hay públicos disponibles."}
                  </p>
                ) : null}
              </div>
              {editingInstrumentId && availableToAddPublics.length > 0 ? (
                <div className="mt-3">
                  <Button
                    type="button"
                    variant="outline"
                    size="sm"
                    onClick={() => setShowAvailablePublics((value) => !value)}
                  >
                    <Plus className="size-4" />
                    {showAvailablePublics ? "Ocultar disponibles" : "Agregar públicos"}
                  </Button>
                  {showAvailablePublics ? (
                    <div className="mt-2 max-h-48 space-y-2 overflow-y-auto rounded-lg border bg-background/55 p-3">
                      {availableToAddPublics.map((option) => (
                        <label
                          key={option.key}
                          className="flex items-start gap-3 rounded-md p-2 text-sm hover:bg-muted/60"
                        >
                          <input
                            className="mt-1"
                            type="checkbox"
                            checked={false}
                            onChange={() => togglePublic(option.key)}
                          />
                          <span>
                            <span className="font-medium text-foreground">{option.label}</span>
                            {option.subpublics.length ? (
                              <span className="mt-1 flex max-h-20 flex-wrap gap-1 overflow-y-auto pr-1">
                                {option.subpublics.map((subpublic) => (
                                  <Badge
                                    key={subpublic}
                                    variant="outline"
                                    className="bg-background text-[11px]"
                                  >
                                    {subpublic}
                                  </Badge>
                                ))}
                              </span>
                            ) : null}
                          </span>
                        </label>
                      ))}
                    </div>
                  ) : null}
                </div>
              ) : null}
              <div className="mt-3 grid gap-2 sm:grid-cols-[1fr_auto]">
                <Input
                  value={customPublic}
                  onChange={(event) => setCustomPublic(event.target.value)}
                  placeholder="Agregar público nuevo"
                />
                <Button type="button" variant="outline" onClick={addCustomPublic}>
                  <Plus className="size-4" />
                  Agregar
                </Button>
              </div>
            </div>
          </div>
          <div className="flex flex-wrap gap-2">
            <Button
              type="button"
              disabled={
                instrumentLabel.trim().length < 3 ||
                selectedPublicKeys.length === 0 ||
                saveInstrument.isPending
              }
              onClick={() =>
                saveInstrument.mutate({
                  id: editingInstrumentId,
                  label: instrumentLabel,
                  publicKeys: selectedPublicKeys,
                })
              }
            >
              {editingInstrumentId ? <Pencil className="size-4" /> : <Plus className="size-4" />}
              {editingInstrumentId ? "Guardar instrumento" : "Crear instrumento"}
            </Button>
            {editingInstrumentId ? (
              <Button type="button" variant="outline" onClick={cancelInstrumentEdit}>
                <X className="size-4" />
                Cancelar edición
              </Button>
            ) : null}
          </div>
          {saveInstrument.isError ? (
            <p className="text-sm text-destructive">
              No se pudo guardar el instrumento. Revise que los públicos no estén asignados a otro
              instrumento.
            </p>
          ) : null}
        </CardContent>
      </Card>
    </section>
  );
}

function expandInstrumentPublicKeys(
  publicKeys: string[],
  options: AvailableInstrumentPublic[],
) {
  const optionKeys = new Set(options.map((option) => option.key));
  const expanded = publicKeys.flatMap((key) => {
    if (optionKeys.has(key)) return [key];
    const matchingOptions = options.filter(
      (option) => option.key === key || option.key.startsWith(`${key} `),
    );
    return matchingOptions.length > 0 ? matchingOptions.map((option) => option.key) : [key];
  });
  return Array.from(new Set(expanded)).sort();
}
