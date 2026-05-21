import { FilterX } from "lucide-react";
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

type LineamentHierarchyPickerProps = {
  aspects: GuidelineAspect[];
  selectedAspectId: string;
  onSelectAspect: (aspectId: string) => void;
};

export const LineamentHierarchyPicker = memo(function LineamentHierarchyPicker({
  aspects,
  selectedAspectId,
  onSelectAspect,
}: LineamentHierarchyPickerProps) {
  const [factorCode, setFactorCode] = useState("");
  const [characteristicFilter, setCharacteristicFilter] = useState("");
  const [query, setQuery] = useState("");
  const factorOptions = useMemo(() => factorOptionsFromLineaments(aspects), [aspects]);
  const characteristicOptions = useMemo(
    () => characteristicOptionsFromLineaments(aspects, factorCode),
    [aspects, factorCode],
  );
  const aspectOptions = useMemo(() => {
    const needle = query.trim().toLowerCase();
    const [filterFactorCode, filterCharacteristicCode] = characteristicFilter.split(":");
    return aspects.filter((aspect) => {
      if (factorCode && aspect.factorCode !== factorCode) return false;
      if (
        filterFactorCode &&
        filterCharacteristicCode &&
        (aspect.factorCode !== filterFactorCode ||
          aspect.characteristicCode !== filterCharacteristicCode)
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
  }, [aspects, characteristicFilter, factorCode, query]);
  const hasFilters = Boolean(factorCode || characteristicFilter || query.trim());

  return (
    <div className="grid gap-3">
      <div className="grid gap-2 sm:grid-cols-[1fr_1fr_auto]">
        <div className="grid gap-1.5">
          <span className="text-xs font-medium text-muted-foreground">Factor</span>
          <Select
            value={factorCode || "all"}
            onValueChange={(value) => {
              setFactorCode(value === "all" ? "" : value);
              setCharacteristicFilter("");
            }}
          >
            <SelectTrigger>
              <SelectValue placeholder="Filtrar por factor" />
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
        </div>

        <div className="grid gap-1.5">
          <span className="text-xs font-medium text-muted-foreground">
            Caracteristica
          </span>
          <Select
            value={characteristicFilter || "all"}
            onValueChange={(value) =>
              setCharacteristicFilter(value === "all" ? "" : value)
            }
          >
            <SelectTrigger>
              <SelectValue placeholder="Filtrar por caracteristica" />
            </SelectTrigger>
            <SelectContent>
              <SelectItem value="all">Todas las caracteristicas</SelectItem>
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
        </div>

        <Button
          type="button"
          variant="outline"
          className="self-end"
          disabled={!hasFilters}
          onClick={() => {
            setFactorCode("");
            setCharacteristicFilter("");
            setQuery("");
          }}
          aria-label="Limpiar filtros CNA"
          title="Limpiar filtros"
        >
          <FilterX className="size-4" />
        </Button>
      </div>

      <Input
        value={query}
        onChange={(event) => setQuery(event.target.value)}
        placeholder="Buscar dentro de los lineamientos filtrados"
      />

      <Select value={selectedAspectId} onValueChange={onSelectAspect}>
        <SelectTrigger>
          <SelectValue placeholder={`Seleccione lineamiento CNA (${aspectOptions.length})`} />
        </SelectTrigger>
        <SelectContent>
          {aspectOptions.map((aspect) => (
            <SelectItem key={aspect.id} value={aspect.id}>
              {aspect.factorCode} / {aspect.characteristicCode} / {aspect.aspectCode}.{" "}
              {aspect.aspectDescription}
            </SelectItem>
          ))}
        </SelectContent>
      </Select>
    </div>
  );
});
