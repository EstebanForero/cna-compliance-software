import { memo } from "react";

import { Button } from "@/components/ui/button";
import {
  Select,
  SelectContent,
  SelectItem,
  SelectTrigger,
  SelectValue,
} from "@/components/ui/select";
import type { GuidelineAspect } from "@/lib/types";

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
  return (
    <section className="rounded-lg border bg-card/78 p-4 shadow-sm shadow-black/5 backdrop-blur-xl">
      <div className="grid gap-3 lg:grid-cols-[1fr_auto] lg:items-end">
        <div className="space-y-2">
          <p className="text-xs font-medium uppercase text-muted-foreground">
            Lineamiento de trabajo
          </p>
          <Select value={selectedLineamentId} onValueChange={onChooseLineament}>
            <SelectTrigger>
              <SelectValue placeholder="Seleccione un lineamiento para ver y agregar preguntas" />
            </SelectTrigger>
            <SelectContent>
              {lineamentOptions.map((aspect) => (
                <SelectItem key={aspect.id} value={aspect.id}>
                  {aspect.factorCode}. {aspect.factorName} / {aspect.characteristicCode}.{" "}
                  {aspect.characteristicName}
                </SelectItem>
              ))}
            </SelectContent>
          </Select>
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
