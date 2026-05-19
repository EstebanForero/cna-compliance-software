import { memo, useEffect, useMemo, useState } from "react";

import {
  Select,
  SelectContent,
  SelectItem,
  SelectTrigger,
  SelectValue,
} from "@/components/ui/select";
import type { GuidelineAspect } from "@/lib/types";

import {
  aspectsForCharacteristic,
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
  const selectedAspect = useMemo(
    () => aspects.find((aspect) => aspect.id === selectedAspectId) ?? null,
    [aspects, selectedAspectId],
  );
  const [factorCode, setFactorCode] = useState(selectedAspect?.factorCode ?? "");
  const [characteristicCode, setCharacteristicCode] = useState(
    selectedAspect?.characteristicCode ?? "",
  );
  const factorOptions = useMemo(() => factorOptionsFromLineaments(aspects), [aspects]);
  const characteristicOptions = useMemo(
    () => characteristicOptionsFromLineaments(aspects, factorCode),
    [aspects, factorCode],
  );
  const aspectOptions = useMemo(
    () => aspectsForCharacteristic(aspects, factorCode, characteristicCode),
    [aspects, characteristicCode, factorCode],
  );

  useEffect(() => {
    setFactorCode(selectedAspect?.factorCode ?? "");
    setCharacteristicCode(selectedAspect?.characteristicCode ?? "");
  }, [selectedAspect]);

  return (
    <div className="grid gap-2">
      <Select
        value={factorCode}
        onValueChange={(value) => {
          setFactorCode(value);
          setCharacteristicCode("");
        }}
      >
        <SelectTrigger>
          <SelectValue placeholder="1. Seleccione factor" />
        </SelectTrigger>
        <SelectContent>
          {factorOptions.map((factor) => (
            <SelectItem key={factor.code} value={factor.code}>
              {factor.code}. {factor.name}
            </SelectItem>
          ))}
        </SelectContent>
      </Select>

      <Select
        value={characteristicCode}
        disabled={!factorCode}
        onValueChange={(value) => setCharacteristicCode(value)}
      >
        <SelectTrigger>
          <SelectValue placeholder="2. Seleccione caracteristica" />
        </SelectTrigger>
        <SelectContent>
          {characteristicOptions.map((characteristic) => (
            <SelectItem key={characteristic.code} value={characteristic.code}>
              {characteristic.code}. {characteristic.name}
            </SelectItem>
          ))}
        </SelectContent>
      </Select>

      <Select
        value={selectedAspectId}
        disabled={!factorCode || !characteristicCode}
        onValueChange={onSelectAspect}
      >
        <SelectTrigger>
          <SelectValue placeholder="3. Seleccione aspecto" />
        </SelectTrigger>
        <SelectContent>
          {aspectOptions.map((aspect) => (
            <SelectItem key={aspect.id} value={aspect.id}>
              {aspect.aspectCode}. {aspect.aspectDescription}
            </SelectItem>
          ))}
        </SelectContent>
      </Select>
    </div>
  );
});
