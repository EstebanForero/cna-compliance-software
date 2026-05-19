import { Check, ChevronDown, Plus, Search, X } from "lucide-react";
import { memo, useMemo, useState } from "react";

import { Badge } from "@/components/ui/badge";
import { Button } from "@/components/ui/button";
import { Input } from "@/components/ui/input";

import { normalizeAudienceLabel, normalizeAudienceList } from "./audiences";

type AudienceSelectorProps = {
  selected: string[];
  options: string[];
  onChange: (audiences: string[]) => void;
};

export const AudienceSelector = memo(function AudienceSelector({
  selected,
  options,
  onChange,
}: AudienceSelectorProps) {
  const [customAudience, setCustomAudience] = useState("");
  const [open, setOpen] = useState(false);
  const [query, setQuery] = useState("");
  const normalizedSelected = useMemo(() => normalizeAudienceList(selected), [selected]);
  const availableOptions = useMemo(
    () => normalizeAudienceList([...options, ...selected]),
    [options, selected],
  );
  const filteredOptions = useMemo(() => {
    const needle = query.trim().toLowerCase();
    if (!needle) return availableOptions;
    return availableOptions.filter((audience) => audience.toLowerCase().includes(needle));
  }, [availableOptions, query]);

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

  return (
    <div className="rounded-lg border bg-background/55 p-3">
      <div className="flex items-start justify-between gap-3">
        <div>
          <p className="text-xs font-medium uppercase text-muted-foreground">
            Publicos que reciben la pregunta
          </p>
          <p className="mt-1 text-xs leading-5 text-muted-foreground">
            Una pregunta puede aplicar a varios publicos o subpublicos. Estos valores
            salen del consolidado importado.
          </p>
        </div>
        <Badge variant="outline">{normalizedSelected.length}</Badge>
      </div>

      {normalizedSelected.length > 0 ? (
        <div className="mt-3 flex flex-wrap gap-2">
          {normalizedSelected.map((audience) => (
            <Button
              key={audience}
              type="button"
              variant="secondary"
              size="sm"
              className="h-8 rounded-full"
              onClick={() => toggleAudience(audience)}
            >
              {audience}
              <X className="size-3.5" />
            </Button>
          ))}
        </div>
      ) : null}

      <div className="relative mt-3">
        <Button
          type="button"
          variant="outline"
          className="h-auto min-h-10 w-full justify-between px-3 py-2 text-left"
          onClick={() => setOpen((value) => !value)}
        >
          <span className="min-w-0 truncate">
            {normalizedSelected.length
              ? `${normalizedSelected.length} publicos seleccionados`
              : "Seleccionar publicos"}
          </span>
          <ChevronDown className="size-4 shrink-0 text-muted-foreground" />
        </Button>

        {open ? (
          <div className="apple-glass absolute left-0 right-0 top-12 z-30 rounded-lg p-2 shadow-xl">
            <div className="relative">
              <Search className="pointer-events-none absolute left-3 top-2.5 size-4 text-muted-foreground" />
              <Input
                value={query}
                onChange={(event) => setQuery(event.target.value)}
                placeholder="Buscar publico"
                className="pl-9"
              />
            </div>
            <div className="mt-2 max-h-72 space-y-1 overflow-y-auto pr-1">
              {filteredOptions.length ? (
                filteredOptions.map((audience) => {
                  const selectedOption = normalizedSelected.includes(audience);
                  return (
                    <button
                      key={audience}
                      type="button"
                      className="flex w-full items-start gap-2 rounded-md px-2 py-2 text-left text-sm hover:bg-muted"
                      onClick={() => toggleAudience(audience)}
                    >
                      <span className="mt-0.5 flex size-4 shrink-0 items-center justify-center rounded border bg-background">
                        {selectedOption ? <Check className="size-3" /> : null}
                      </span>
                      <span className="min-w-0 break-words leading-5">{audience}</span>
                    </button>
                  );
                })
              ) : (
                <p className="px-2 py-4 text-center text-sm text-muted-foreground">
                  No hay publicos con ese filtro.
                </p>
              )}
            </div>
          </div>
        ) : null}
      </div>

      <div className="mt-3 grid gap-2 sm:grid-cols-[1fr_auto]">
        <Input
          value={customAudience}
          placeholder="Agregar publico si no existe"
          onChange={(event) => setCustomAudience(event.target.value)}
        />
        <Button type="button" variant="outline" onClick={addCustomAudience}>
          <Plus className="size-4" />
          Agregar
        </Button>
      </div>
    </div>
  );
});
