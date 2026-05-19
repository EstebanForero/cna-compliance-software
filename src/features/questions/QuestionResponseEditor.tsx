import { Plus, Trash2 } from "lucide-react";
import { memo } from "react";

import { Button } from "@/components/ui/button";
import { Input } from "@/components/ui/input";
import {
  Select,
  SelectContent,
  SelectItem,
  SelectTrigger,
  SelectValue,
} from "@/components/ui/select";
import type { QuestionFormat } from "@/lib/types";

import {
  type ChoiceOption,
  responseConventionDescription,
  responseConventions,
  serializeChoiceOptions,
} from "./questionFormat";

type QuestionResponseEditorProps = {
  format: QuestionFormat;
  conventionCode?: string | null;
  onConventionChange: (value: string | undefined) => void;
  options: ChoiceOption[];
  onOptionsChange: (options: ChoiceOption[]) => void;
};

export const QuestionResponseEditor = memo(function QuestionResponseEditor({
  format,
  conventionCode,
  onConventionChange,
  options,
  onOptionsChange,
}: QuestionResponseEditorProps) {
  if (format === "open") {
    return (
      <p className="rounded-lg border bg-background/55 p-3 text-xs leading-5 text-muted-foreground">
        Las preguntas abiertas no usan convencion de respuesta.
      </p>
    );
  }

  if (format === "multipleChoice") {
    return (
      <MultipleChoiceOptionsEditor
        options={options}
        onOptionsChange={(nextOptions) => {
          onOptionsChange(nextOptions);
          onConventionChange(serializeChoiceOptions(nextOptions));
        }}
      />
    );
  }

  return (
    <div className="space-y-2">
      <Select
        value={conventionCode ?? ""}
        onValueChange={(value) => onConventionChange(value)}
      >
        <SelectTrigger>
          <SelectValue placeholder="Escala de respuesta" />
        </SelectTrigger>
        <SelectContent>
          {responseConventions.map((option) => (
            <SelectItem key={option.code} value={option.code}>
              {option.name} ({option.code})
            </SelectItem>
          ))}
        </SelectContent>
      </Select>
      {conventionCode ? (
        <p className="rounded-lg border bg-background/55 p-3 text-xs leading-5 text-muted-foreground">
          {responseConventionDescription(conventionCode)}
        </p>
      ) : null}
    </div>
  );
});

function MultipleChoiceOptionsEditor({
  options,
  onOptionsChange,
}: {
  options: ChoiceOption[];
  onOptionsChange: (options: ChoiceOption[]) => void;
}) {
  const updateOption = (id: string, label: string) => {
    onOptionsChange(
      options.map((option) => (option.id === id ? { ...option, label } : option)),
    );
  };

  const removeOption = (id: string) => {
    if (options.length <= 2) return;
    onOptionsChange(options.filter((option) => option.id !== id));
  };

  const addOption = () => {
    onOptionsChange([...options, { id: `option-${Date.now()}`, label: "" }]);
  };

  return (
    <div className="rounded-lg border bg-background/55 p-3">
      <div className="flex items-center justify-between gap-3">
        <div>
          <p className="text-xs font-medium uppercase text-muted-foreground">
            Opciones de respuesta
          </p>
          <p className="mt-1 text-xs text-muted-foreground">
            Agregue opciones estructuradas. Minimo dos opciones.
          </p>
        </div>
        <Button type="button" variant="outline" size="sm" onClick={addOption}>
          <Plus className="size-4" />
          Opcion
        </Button>
      </div>
      <div className="mt-3 space-y-2">
        {options.map((option, index) => (
          <div key={option.id} className="grid grid-cols-[auto_1fr_auto] items-center gap-2">
            <span className="flex size-7 items-center justify-center rounded-md border bg-card/70 text-xs font-medium text-muted-foreground">
              {index + 1}
            </span>
            <Input
              value={option.label}
              onChange={(event) => updateOption(option.id, event.target.value)}
              placeholder={`Opcion ${index + 1}`}
            />
            <Button
              type="button"
              variant="outline"
              size="icon"
              aria-label="Eliminar opcion"
              disabled={options.length <= 2}
              onClick={() => removeOption(option.id)}
            >
              <Trash2 className="size-4" />
            </Button>
          </div>
        ))}
      </div>
    </div>
  );
}
