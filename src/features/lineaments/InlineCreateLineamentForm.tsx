import { Plus } from "lucide-react";
import { memo } from "react";

import { Button } from "@/components/ui/button";
import {
  Dialog,
  DialogContent,
  DialogDescription,
  DialogHeader,
  DialogTitle,
  DialogTrigger,
} from "@/components/ui/dialog";
import { Input } from "@/components/ui/input";
import {
  Select,
  SelectContent,
  SelectGroup,
  SelectItem,
  SelectLabel,
  SelectTrigger,
  SelectValue,
} from "@/components/ui/select";
import { Textarea } from "@/components/ui/textarea";
import type { CnaFactorCode, NewGuidelineAspect } from "@/lib/types";

import {
  type CharacteristicChoice,
  type CodeName,
  characteristicValue,
  nextNumericCode,
  parseCharacteristicValue,
} from "./hierarchy";

type InlineCreateLineamentFormProps = {
  draft: NewGuidelineAspect;
  setDraft: (value: NewGuidelineAspect) => void;
  createAspect: {
    mutate: (aspect: NewGuidelineAspect) => void;
    isPending: boolean;
  };
  factorChoices: CodeName[];
  characteristicOptions: CharacteristicChoice[];
  characteristicGroups: Array<{
    factorCode: string;
    factorName: string;
    options: CharacteristicChoice[];
  }>;
  factorDialogOpen: boolean;
  setFactorDialogOpen: (value: boolean) => void;
  characteristicDialogOpen: boolean;
  setCharacteristicDialogOpen: (value: boolean) => void;
  customFactorName: string;
  setCustomFactorName: (value: string) => void;
  customCharacteristicName: string;
  setCustomCharacteristicName: (value: string) => void;
};

export const InlineCreateLineamentForm = memo(function InlineCreateLineamentForm({
  draft,
  setDraft,
  createAspect,
  factorChoices,
  characteristicOptions,
  characteristicGroups,
  factorDialogOpen,
  setFactorDialogOpen,
  characteristicDialogOpen,
  setCharacteristicDialogOpen,
  customFactorName,
  setCustomFactorName,
  customCharacteristicName,
  setCustomCharacteristicName,
}: InlineCreateLineamentFormProps) {
  return (
    <form
      className="grid gap-3 lg:grid-cols-2"
      onSubmit={(event) => {
        event.preventDefault();
        createAspect.mutate(draft);
      }}
    >
      <Input
        className="lg:col-span-2"
        value={draft.guidelineTitle}
        onChange={(event) => setDraft({ ...draft, guidelineTitle: event.target.value })}
        placeholder="Titulo del lineamiento"
      />
      <div className="rounded-lg border bg-background/45 p-3">
        <p className="text-xs font-semibold uppercase text-muted-foreground">Factor</p>
        <div className="mt-2 grid grid-cols-[1fr_auto] gap-2">
          <Select
            value={draft.factorCode}
            onValueChange={(value) => {
              const option = factorChoices.find((item) => item.code === value);
              setDraft({
                ...draft,
                factorCode: value as CnaFactorCode,
                factorName: option?.name ?? draft.factorName,
                characteristicCode: "",
                characteristicName: "",
              });
            }}
          >
            <SelectTrigger>
              <SelectValue placeholder="Seleccione factor" />
            </SelectTrigger>
            <SelectContent>
              {factorChoices.map((factor) => (
                <SelectItem key={factor.code} value={factor.code}>
                  {factor.code}. {factor.name}
                </SelectItem>
              ))}
            </SelectContent>
          </Select>
          <Dialog open={factorDialogOpen} onOpenChange={setFactorDialogOpen}>
            <DialogTrigger asChild>
              <Button type="button" variant="outline" size="icon" aria-label="Agregar factor">
                <Plus className="size-4" />
              </Button>
            </DialogTrigger>
            <DialogContent>
              <DialogHeader>
                <DialogTitle>Nuevo factor</DialogTitle>
                <DialogDescription>El codigo se asigna automaticamente.</DialogDescription>
              </DialogHeader>
              <div className="space-y-3">
                <Input
                  value={customFactorName}
                  onChange={(event) => setCustomFactorName(event.target.value)}
                  placeholder="Nombre del factor"
                />
                <Button
                  type="button"
                  className="w-full"
                  disabled={!customFactorName.trim()}
                  onClick={() => {
                    const factorCode = nextNumericCode(factorChoices);
                    setDraft({
                      ...draft,
                      factorCode,
                      factorName: customFactorName.trim(),
                      characteristicCode: "",
                      characteristicName: "",
                    });
                    setCustomFactorName("");
                    setFactorDialogOpen(false);
                  }}
                >
                  Usar factor
                </Button>
              </div>
            </DialogContent>
          </Dialog>
        </div>
      </div>
      <div className="rounded-lg border bg-background/45 p-3">
        <p className="text-xs font-semibold uppercase text-muted-foreground">
          Caracteristica
        </p>
        <div className="mt-2 grid grid-cols-[1fr_auto] gap-2">
          <Select
            value={
              draft.factorCode && draft.characteristicCode
                ? characteristicValue(draft.factorCode, draft.characteristicCode)
                : ""
            }
            onValueChange={(value) => {
              const { factorCode, characteristicCode } = parseCharacteristicValue(value);
              const option = characteristicOptions.find(
                (item) => item.factorCode === factorCode && item.code === characteristicCode,
              );
              setDraft({
                ...draft,
                factorCode,
                factorName: option?.factorName ?? draft.factorName,
                characteristicCode,
                characteristicName: option?.name ?? draft.characteristicName,
              });
            }}
          >
            <SelectTrigger>
              <SelectValue placeholder="Seleccione caracteristica" />
            </SelectTrigger>
            <SelectContent>
              {draft.factorCode
                ? characteristicOptions.map((option) => (
                    <SelectItem
                      key={`${option.factorCode}:${option.code}`}
                      value={characteristicValue(option.factorCode, option.code)}
                    >
                      {option.code}. {option.name}
                    </SelectItem>
                  ))
                : characteristicGroups.map((group) => (
                    <SelectGroup key={group.factorCode}>
                      <SelectLabel className="px-2 py-1.5 text-xs font-semibold text-muted-foreground">
                        {group.factorCode}. {group.factorName}
                      </SelectLabel>
                      {group.options.map((option) => (
                        <SelectItem
                          key={`${option.factorCode}:${option.code}`}
                          value={characteristicValue(option.factorCode, option.code)}
                        >
                          {option.code}. {option.name}
                        </SelectItem>
                      ))}
                    </SelectGroup>
                  ))}
            </SelectContent>
          </Select>
          <Dialog open={characteristicDialogOpen} onOpenChange={setCharacteristicDialogOpen}>
            <DialogTrigger asChild>
              <Button
                type="button"
                variant="outline"
                size="icon"
                aria-label="Agregar caracteristica"
                disabled={!draft.factorCode}
              >
                <Plus className="size-4" />
              </Button>
            </DialogTrigger>
            <DialogContent>
              <DialogHeader>
                <DialogTitle>Nueva caracteristica</DialogTitle>
                <DialogDescription>El codigo se asigna automaticamente.</DialogDescription>
              </DialogHeader>
              <div className="space-y-3">
                <Input
                  value={customCharacteristicName}
                  onChange={(event) => setCustomCharacteristicName(event.target.value)}
                  placeholder="Nombre de la caracteristica"
                />
                <Button
                  type="button"
                  className="w-full"
                  disabled={!customCharacteristicName.trim()}
                  onClick={() => {
                    const characteristicCode = nextNumericCode(characteristicOptions);
                    setDraft({
                      ...draft,
                      characteristicCode,
                      characteristicName: customCharacteristicName.trim(),
                    });
                    setCustomCharacteristicName("");
                    setCharacteristicDialogOpen(false);
                  }}
                >
                  Usar caracteristica
                </Button>
              </div>
            </DialogContent>
          </Dialog>
        </div>
      </div>
      <Textarea
        className="lg:col-span-2"
        value={draft.aspectDescription}
        onChange={(event) => setDraft({ ...draft, aspectDescription: event.target.value })}
        placeholder="Descripcion del aspecto por evaluar"
      />
      <div className="flex flex-wrap gap-2 lg:col-span-2">
        <Button
          type="button"
          variant={draft.scope === "institutional" ? "default" : "outline"}
          onClick={() => setDraft({ ...draft, scope: "institutional" })}
        >
          Institucional
        </Button>
        <Button
          type="button"
          variant={draft.scope === "program" ? "default" : "outline"}
          onClick={() => setDraft({ ...draft, scope: "program" })}
        >
          Programa
        </Button>
        <Button
          disabled={
            createAspect.isPending ||
            !draft.factorCode.trim() ||
            !draft.characteristicCode.trim() ||
            !draft.aspectDescription.trim()
          }
        >
          <Plus className="size-4" />
          Agregar lineamiento
        </Button>
      </div>
    </form>
  );
});
