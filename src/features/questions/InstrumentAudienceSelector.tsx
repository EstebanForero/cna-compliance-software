import { Check, Plus, Search, X } from "lucide-react";
import { memo, useMemo, useState } from "react";

import { Badge } from "@/components/ui/badge";
import { Button } from "@/components/ui/button";
import {
  Dialog,
  DialogContent,
  DialogDescription,
  DialogHeader,
  DialogTitle,
} from "@/components/ui/dialog";
import { Input } from "@/components/ui/input";
import { cn } from "@/lib/utils";
import type { InstrumentDefinition } from "@/lib/types";

import { normalizeAudienceList, normalizeAudienceLabel } from "./audiences";

type InstrumentAudienceSelectorProps = {
  selected: string[];
  audienceOptions: string[];
  instruments: InstrumentDefinition[];
  onChange: (audiences: string[]) => void;
};

export const InstrumentAudienceSelector = memo(function InstrumentAudienceSelector({
  selected,
  audienceOptions,
  instruments,
  onChange,
}: InstrumentAudienceSelectorProps) {
  const [activeInstrumentId, setActiveInstrumentId] = useState<string | null>(null);
  const [query, setQuery] = useState("");
  const [customAudience, setCustomAudience] = useState("");
  const normalizedSelected = useMemo(() => normalizeAudienceList(selected), [selected]);

  function audiencesForInstrument(instrument: InstrumentDefinition) {
    return normalizeAudienceList(
      [...audienceOptions, ...normalizedSelected].filter((audience) =>
        instrument.publicKeys.some((publicKey) => audienceBelongsToPublic(audience, publicKey)),
      ),
    );
  }

  function openInstrument(instrument: InstrumentDefinition) {
    const instrumentAudiences = audiencesForInstrument(instrument);
    if (instrumentAudiences.length === 0) return;
    const selectedSet = new Set(normalizedSelected);
    const hasSelection = instrumentAudiences.some((audience) => selectedSet.has(audience));
    if (!hasSelection) {
      onChange(normalizeAudienceList([...normalizedSelected, ...instrumentAudiences]));
    }
    setActiveInstrumentId(instrument.id);
    setQuery("");
  }

  function selectAllForInstrument(instrument: InstrumentDefinition) {
    const instrumentAudiences = audiencesForInstrument(instrument);
    onChange(normalizeAudienceList([...normalizedSelected, ...instrumentAudiences]));
  }

  function clearInstrument(instrument: InstrumentDefinition) {
    const instrumentAudiences = audiencesForInstrument(instrument);
    onChange(
      normalizeAudienceList(
        normalizedSelected.filter((audience) => !instrumentAudiences.includes(audience)),
      ),
    );
  }

  function toggleAudience(audience: string) {
    const normalized = normalizeAudienceLabel(audience);
    if (!normalized) return;
    const next = normalizedSelected.includes(normalized)
      ? normalizedSelected.filter((item) => item !== normalized)
      : [...normalizedSelected, normalized];
    onChange(normalizeAudienceList(next));
  }

  function addCustomAudience() {
    const normalized = normalizeAudienceLabel(customAudience);
    if (!normalized) return;
    onChange(normalizeAudienceList([...normalizedSelected, normalized]));
    setCustomAudience("");
  }

  const instrumentStates = instruments.map((instrument) => {
    const audiences = audiencesForInstrument(instrument);
    const selectedCount = audiences.filter((audience) => normalizedSelected.includes(audience)).length;
    return {
      instrument,
      audiences,
      selectedCount,
      active: selectedCount > 0,
      complete: audiences.length > 0 && selectedCount === audiences.length,
    };
  });
  const activeInstrument =
    instrumentStates.find(({ instrument }) => instrument.id === activeInstrumentId) ?? null;
  const filteredActiveAudiences = activeInstrument
    ? activeInstrument.audiences.filter((audience) =>
        audience.toLowerCase().includes(query.trim().toLowerCase()),
      )
    : [];

  return (
    <div className="rounded-lg border bg-background/80 p-3 shadow-sm">
      <div className="flex items-start justify-between gap-3">
        <div>
          <p className="text-xs font-medium uppercase text-muted-foreground">
            Instrumentos y públicos
          </p>
          <p className="mt-1 text-xs leading-5 text-muted-foreground">
            Abra un instrumento para ajustar los públicos específicos de esa pregunta.
          </p>
        </div>
        <Badge variant="outline">{normalizedSelected.length}</Badge>
      </div>

      <div className="mt-3 grid gap-2">
        {instrumentStates.length > 0 ? (
          instrumentStates.map(({ instrument, audiences, selectedCount, active, complete }) => (
            <Button
              key={instrument.id}
              type="button"
              variant="outline"
              className={cn(
                "h-auto justify-between gap-3 rounded-lg bg-background px-3 py-2 text-left shadow-sm hover:bg-muted/45",
                active && "border-primary/45 bg-primary/5 text-foreground",
              )}
              disabled={audiences.length === 0}
              onClick={() => openInstrument(instrument)}
            >
              <span className="min-w-0">
                <span className="block truncate font-medium">{instrument.label}</span>
                <span className="block text-xs text-muted-foreground">
                  {selectedCount}/{audiences.length} públicos
                </span>
              </span>
              <Badge
                variant="outline"
                className={cn(
                  "bg-background text-muted-foreground",
                  active && "border-primary/30 bg-primary/10 text-primary",
                )}
              >
                {complete ? "Todos" : active ? "Parcial" : "Configurar"}
              </Badge>
            </Button>
          ))
        ) : (
          <p className="rounded-lg border bg-background/60 p-3 text-sm text-muted-foreground">
            Importe un consolidado para detectar instrumentos. Mientras tanto puede elegir
            públicos manualmente.
          </p>
        )}
      </div>

      <Dialog open={Boolean(activeInstrument)} onOpenChange={(open) => !open && setActiveInstrumentId(null)}>
        <DialogContent className="w-[min(94vw,40rem)]">
          {activeInstrument ? (
            <>
              <DialogHeader>
                <DialogTitle>{activeInstrument.instrument.label}</DialogTitle>
                <DialogDescription>
                  Ajuste los públicos de este instrumento. Al abrirlo por primera vez se
                  seleccionan todos por defecto.
                </DialogDescription>
              </DialogHeader>

              <div className="mt-4 flex flex-wrap gap-2">
                <Button
                  type="button"
                  variant="outline"
                  size="sm"
                  onClick={() => selectAllForInstrument(activeInstrument.instrument)}
                >
                  <Check className="size-4" />
                  Seleccionar todos
                </Button>
                <Button
                  type="button"
                  variant="outline"
                  size="sm"
                  onClick={() => clearInstrument(activeInstrument.instrument)}
                >
                  <X className="size-4" />
                  Limpiar instrumento
                </Button>
              </div>

              <div className="relative mt-4">
                <Search className="pointer-events-none absolute left-3 top-2.5 size-4 text-muted-foreground" />
                <Input
                  value={query}
                  onChange={(event) => setQuery(event.target.value)}
                  placeholder="Buscar público dentro del instrumento"
                  className="pl-9"
                />
              </div>

              <div className="mt-3 max-h-80 space-y-1 overflow-y-auto pr-1">
                {filteredActiveAudiences.map((audience) => {
                  const selectedAudience = normalizedSelected.includes(audience);
                  return (
                    <button
                      key={audience}
                      type="button"
                      className="flex w-full items-start gap-2 rounded-md px-2 py-2 text-left text-sm hover:bg-muted"
                      onClick={() => toggleAudience(audience)}
                    >
                      <span className="mt-0.5 flex size-4 shrink-0 items-center justify-center rounded border bg-background">
                        {selectedAudience ? <Check className="size-3" /> : null}
                      </span>
                      <span className="min-w-0 break-words leading-5">{audience}</span>
                    </button>
                  );
                })}
                {filteredActiveAudiences.length === 0 ? (
                  <p className="rounded-lg border bg-background/70 p-4 text-center text-sm text-muted-foreground">
                    No hay públicos con ese filtro.
                  </p>
                ) : null}
              </div>

              <div className="mt-4 grid gap-2 sm:grid-cols-[1fr_auto]">
                <Input
                  value={customAudience}
                  placeholder="Agregar público específico"
                  onChange={(event) => setCustomAudience(event.target.value)}
                />
                <Button type="button" variant="outline" onClick={addCustomAudience}>
                  <Plus className="size-4" />
                  Agregar
                </Button>
              </div>
            </>
          ) : null}
        </DialogContent>
      </Dialog>
    </div>
  );
});

function audienceBelongsToPublic(audience: string, publicKey: string) {
  const normalizedAudience = normalizeAudienceLabel(audience).toLowerCase();
  const normalizedPublic = normalizeAudienceLabel(publicKey).toLowerCase();
  return normalizedAudience === normalizedPublic || normalizedAudience.startsWith(`${normalizedPublic} `);
}
