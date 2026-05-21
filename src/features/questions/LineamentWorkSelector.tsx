import { Filter, FilterX, Search } from "lucide-react";
import { memo, useMemo, useState } from "react";

import { Button } from "@/components/ui/button";
import { Input } from "@/components/ui/input";
import {
  Select,
  SelectContent,
  SelectItem,
  SelectTrigger,
  SelectValue,
} from "@/components/ui/select";
import type { GuidelineAspect } from "@/lib/types";

import {
  characteristicOptionsFromLineaments,
  factorOptionsFromLineaments,
} from "./lineament";

type LineamentWorkSelectorProps = {
  selectedLineamentId: string;
  selectedLineament: GuidelineAspect | null;
  lineamentOptions: GuidelineAspect[];
  visibleCount: number;
  onChooseLineament: (aspectId: string) => void;
  onClearSelection: () => void;
};

export const LineamentWorkSelector = memo(function LineamentWorkSelector({
  selectedLineamentId,
  selectedLineament,
  lineamentOptions,
  visibleCount,
  onChooseLineament,
  onClearSelection,
}: LineamentWorkSelectorProps) {
  const [advancedOpen, setAdvancedOpen] = useState(false);
  const [query, setQuery] = useState("");
  const [factorCode, setFactorCode] = useState("");
  const [characteristicFilter, setCharacteristicFilter] = useState("");
  const factorOptions = useMemo(
    () => factorOptionsFromLineaments(lineamentOptions),
    [lineamentOptions],
  );
  const characteristicOptions = useMemo(
    () => characteristicOptionsFromLineaments(lineamentOptions, factorCode),
    [factorCode, lineamentOptions],
  );
  const filteredLineaments = useMemo(() => {
    const needle = query.trim().toLowerCase();
    const [characteristicFactorCode, characteristicCode] = characteristicFilter.split(":");
    return lineamentOptions.filter((aspect) => {
      if (factorCode && aspect.factorCode !== factorCode) return false;
      if (
        characteristicFactorCode &&
        characteristicCode &&
        (aspect.factorCode !== characteristicFactorCode ||
          aspect.characteristicCode !== characteristicCode)
      ) {
        return false;
      }
      if (!needle) return true;
      return [
        aspect.factorCode,
        aspect.factorName,
        aspect.characteristicCode,
        aspect.characteristicName,
        aspect.aspectCode,
        aspect.aspectDescription,
      ]
        .join(" ")
        .toLowerCase()
        .includes(needle);
    });
  }, [characteristicFilter, factorCode, lineamentOptions, query]);
  const hasFilters = Boolean(query.trim() || factorCode || characteristicFilter);

  function clearFilters() {
    setQuery("");
    setFactorCode("");
    setCharacteristicFilter("");
  }

  return (
    <section className="rounded-lg border bg-card/78 p-4 shadow-sm shadow-black/5 backdrop-blur-xl">
      <div className="grid gap-3 lg:grid-cols-[1fr_auto] lg:items-end">
        <div className="space-y-2">
          <p className="text-xs font-medium uppercase text-muted-foreground">
            Lineamiento de trabajo
          </p>
          <div className="grid gap-2 lg:grid-cols-[minmax(12rem,0.85fr)_minmax(16rem,1fr)_auto]">
            <div className="relative">
              <Search className="pointer-events-none absolute left-3 top-2.5 size-4 text-muted-foreground" />
              <Input
                value={query}
                onChange={(event) => setQuery(event.target.value)}
                placeholder="Buscar lineamiento"
                className="pl-9"
              />
            </div>
            <Select value={selectedLineamentId} onValueChange={onChooseLineament}>
              <SelectTrigger>
                <SelectValue placeholder={`Seleccionar lineamiento (${filteredLineaments.length})`} />
              </SelectTrigger>
              <SelectContent>
                {filteredLineaments.map((aspect) => (
                  <SelectItem key={aspect.id} value={aspect.id}>
                    {aspect.factorCode} / {aspect.characteristicCode} / {aspect.aspectCode}.{" "}
                    {aspect.aspectDescription}
                  </SelectItem>
                ))}
              </SelectContent>
            </Select>
            <Button
              type="button"
              variant="outline"
              size="icon"
              onClick={() => setAdvancedOpen((value) => !value)}
              aria-label="Filtros avanzados CNA"
              title="Filtros avanzados"
            >
              <Filter className="size-4" />
            </Button>
          </div>
          {advancedOpen ? (
            <div className="grid gap-2 rounded-lg border bg-background/60 p-3 md:grid-cols-[1fr_1fr_auto]">
              <Select
                value={factorCode || "all"}
                onValueChange={(value) => {
                  setFactorCode(value === "all" ? "" : value);
                  setCharacteristicFilter("");
                }}
              >
                <SelectTrigger>
                  <SelectValue placeholder="Factor" />
                </SelectTrigger>
                <SelectContent>
                  <SelectItem value="all">Todos los factores</SelectItem>
                  {factorOptions.map((factor) => (
                    <SelectItem key={factor.code} value={factor.code}>
                      {factor.code}. {factor.name}
                    </SelectItem>
                  ))}
                </SelectContent>
              </Select>

              <Select
                value={characteristicFilter || "all"}
                onValueChange={(value) =>
                  setCharacteristicFilter(value === "all" ? "" : value)
                }
              >
                <SelectTrigger>
                  <SelectValue placeholder="Característica" />
                </SelectTrigger>
                <SelectContent>
                  <SelectItem value="all">Todas las características</SelectItem>
                  {characteristicOptions.map((characteristic) => (
                    <SelectItem
                      key={`${characteristic.factorCode}:${characteristic.code}`}
                      value={`${characteristic.factorCode}:${characteristic.code}`}
                    >
                      {factorCode ? "" : `${characteristic.factorCode} / `}
                      {characteristic.code}. {characteristic.name}
                    </SelectItem>
                  ))}
                </SelectContent>
              </Select>

              <Button
                type="button"
                variant="outline"
                disabled={!hasFilters}
                onClick={clearFilters}
                aria-label="Limpiar filtros CNA"
                title="Limpiar filtros"
              >
                <FilterX className="size-4" />
              </Button>
            </div>
          ) : null}
        </div>
        <Button type="button" variant="outline" onClick={onClearSelection}>
          Ver todo
        </Button>
      </div>
      {selectedLineament ? (
        <div className="mt-3 rounded-lg border bg-background/55 p-3">
          <p className="break-words text-sm font-medium">
            {selectedLineament.aspectDescription}
          </p>
          <p className="mt-1 text-xs text-muted-foreground">
            {selectedLineament.scope === "institutional" ? "Institucional" : "Programa"} ·{" "}
            {visibleCount} preguntas asociadas visibles
          </p>
        </div>
      ) : null}
    </section>
  );
});
