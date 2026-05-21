import { Save, X } from "lucide-react";
import { memo, useEffect, useMemo, useState } from "react";

import { Button } from "@/components/ui/button";
import { Input } from "@/components/ui/input";
import {
  Select,
  SelectContent,
  SelectItem,
  SelectTrigger,
  SelectValue,
} from "@/components/ui/select";
import { Textarea } from "@/components/ui/textarea";
import type { GuidelineAspect, InstrumentDefinition, NewQuestion, Question, QuestionFormat } from "@/lib/types";

import { InstrumentAudienceSelector } from "./InstrumentAudienceSelector";
import { LineamentHierarchyPicker } from "./LineamentHierarchyPicker";
import { QuestionResponseEditor } from "./QuestionResponseEditor";
import { fillQuestionFromLineament, findLineamentIdForQuestion } from "./lineament";
import {
  type ChoiceOption,
  choiceOptionsFromConvention,
  defaultConventionForFormat,
  normalizedChoiceOptions,
  prepareQuestionForSave,
  questionFormats,
} from "./questionFormat";

type QuestionEditPanelProps = {
  question: Question;
  lineamentOptions: GuidelineAspect[];
  audienceOptions: string[];
  instruments: InstrumentDefinition[];
  isSaving: boolean;
  onCancel: () => void;
  onSave: (question: NewQuestion, choiceOptions: ChoiceOption[]) => void;
};

export const QuestionEditPanel = memo(function QuestionEditPanel({
  question,
  lineamentOptions,
  audienceOptions,
  instruments,
  isSaving,
  onCancel,
  onSave,
}: QuestionEditPanelProps) {
  const [draft, setDraft] = useState<NewQuestion>(() => questionToDraft(question));
  const [choiceOptions, setChoiceOptions] = useState<ChoiceOption[]>(() =>
    choiceOptionsFromConvention(question.conventionCode),
  );
  const [lineamentId, setLineamentId] = useState(() =>
    findQuestionLineamentId(question, lineamentOptions),
  );
  const selectedFormatDescription =
    questionFormats.find((format) => format.value === draft.format)?.description ?? "";
  const canSave =
    !isSaving &&
    draft.text.trim() &&
    (draft.format === "open" || draft.conventionCode?.trim()) &&
    (draft.format !== "multipleChoice" ||
      normalizedChoiceOptions(choiceOptions).length >= 2) &&
    draft.factor.trim() &&
    draft.characteristic.trim() &&
    draft.aspect.trim() &&
    draft.audiences.length > 0;
  const isDirty = useMemo(
    () => JSON.stringify(prepareQuestionForSave(draft, choiceOptions)) !== JSON.stringify(questionToDraft(question)),
    [choiceOptions, draft, question],
  );
  const selectedLineament = useMemo(
    () => lineamentOptions.find((aspect) => aspect.id === lineamentId) ?? null,
    [lineamentOptions, lineamentId],
  );

  useEffect(() => {
    setDraft(questionToDraft(question));
    setChoiceOptions(choiceOptionsFromConvention(question.conventionCode));
    setLineamentId(findQuestionLineamentId(question, lineamentOptions));
  }, [lineamentOptions, question]);

  return (
    <form
      className="grid gap-3 rounded-lg border border-primary/20 bg-background/80 p-4 md:grid-cols-2"
      onSubmit={(event) => {
        event.preventDefault();
        onSave(prepareQuestionForSave(draft, choiceOptions), choiceOptions);
      }}
    >
      <Input
        value={draft.code}
        onChange={(event) => setDraft({ ...draft, code: event.target.value })}
        placeholder="Codigo"
      />
      <Select
        value={draft.format}
        onValueChange={(value) => {
          const format = value as QuestionFormat;
          setDraft({
            ...draft,
            format,
            conventionCode:
              format === "open"
                ? undefined
                : draft.conventionCode || defaultConventionForFormat(format),
          });
        }}
      >
        <SelectTrigger>
          <SelectValue placeholder="Tipo de pregunta" />
        </SelectTrigger>
        <SelectContent>
          {questionFormats.map((format) => (
            <SelectItem key={format.value} value={format.value}>
              {format.label}
            </SelectItem>
          ))}
        </SelectContent>
      </Select>
      <p className="text-xs leading-5 text-muted-foreground md:col-span-2">
        {selectedFormatDescription}
      </p>
      <Textarea
        className="md:col-span-2"
        value={draft.text}
        onChange={(event) => setDraft({ ...draft, text: event.target.value })}
        placeholder="Texto de la pregunta"
      />
      <div className="md:col-span-2">
        <QuestionResponseEditor
          format={draft.format}
          conventionCode={draft.conventionCode}
          onConventionChange={(conventionCode) =>
            setDraft({ ...draft, conventionCode })
          }
          options={choiceOptions}
          onOptionsChange={setChoiceOptions}
        />
      </div>
      <div className="space-y-2 md:col-span-2">
        <p className="text-xs font-medium text-muted-foreground">
          Lineamiento asociado
        </p>
        <LineamentHierarchyPicker
          aspects={lineamentOptions}
          selectedAspectId={lineamentId}
          onSelectAspect={(value) => {
            const aspect = lineamentOptions.find((item) => item.id === value);
            setLineamentId(value);
            if (aspect) setDraft((current) => fillQuestionFromLineament(current, aspect));
          }}
        />
        {selectedLineament ? (
          <p className="text-xs leading-5 text-muted-foreground">
            {selectedLineament.aspectDescription}
          </p>
        ) : null}
      </div>
      <div className="md:col-span-2">
        <InstrumentAudienceSelector
          selected={draft.audiences}
          audienceOptions={audienceOptions}
          instruments={instruments}
          onChange={(audiences) => setDraft({ ...draft, audiences })}
        />
      </div>
      <Textarea
        className="md:col-span-2"
        value={draft.justification ?? ""}
        onChange={(event) =>
          setDraft({ ...draft, justification: event.target.value })
        }
        placeholder="Justificacion del cambio"
      />
      <div className="flex flex-wrap gap-2 md:col-span-2">
        {isDirty ? (
          <p className="w-full text-xs text-warning-foreground">
            Hay cambios sin guardar. Al guardar, la pregunta se marcara como Modificar si corresponde.
          </p>
        ) : null}
        <Button type="button" variant="outline" onClick={onCancel}>
          <X className="size-4" />
          Cancelar
        </Button>
        <Button disabled={!canSave}>
          <Save className="size-4" />
          Guardar cambios
        </Button>
      </div>
    </form>
  );
});

function questionToDraft(question: Question): NewQuestion {
  return {
    code: question.code,
    text: question.text,
    scope: question.scope,
    format: question.format,
    conventionCode: question.conventionCode,
    status: question.status,
    factor: question.factor,
    characteristic: question.characteristic,
    aspect: question.aspect,
    audiences: question.audiences,
    justification: question.justification,
  };
}

function findQuestionLineamentId(question: Question, lineamentOptions: GuidelineAspect[]) {
  return findLineamentIdForQuestion(question, lineamentOptions);
}
