import { Plus } from "lucide-react";
import { memo, type Dispatch, type SetStateAction } from "react";

import { Button } from "@/components/ui/button";
import {
  Card,
  CardContent,
  CardDescription,
  CardHeader,
  CardTitle,
} from "@/components/ui/card";
import { Input } from "@/components/ui/input";
import {
  Select,
  SelectContent,
  SelectItem,
  SelectTrigger,
  SelectValue,
} from "@/components/ui/select";
import { Textarea } from "@/components/ui/textarea";
import type { GuidelineAspect, NewQuestion, QuestionFormat } from "@/lib/types";

import { AudienceSelector } from "./AudienceSelector";
import { LineamentHierarchyPicker } from "./LineamentHierarchyPicker";
import { QuestionResponseEditor } from "./QuestionResponseEditor";
import {
  type ChoiceOption,
  defaultConventionForFormat,
  normalizedChoiceOptions,
  questionFormats,
} from "./questionFormat";

type CreateQuestionMutation = {
  mutate: (question: NewQuestion) => void;
  isPending: boolean;
  isError: boolean;
};

type QuestionFormProps = {
  draft: NewQuestion;
  setDraft: Dispatch<SetStateAction<NewQuestion>>;
  selectedLineamentId: string;
  selectedLineament: GuidelineAspect | null;
  lineamentOptions: GuidelineAspect[];
  audienceOptions: string[];
  choiceOptions: ChoiceOption[];
  setChoiceOptions: Dispatch<SetStateAction<ChoiceOption[]>>;
  onChooseLineament: (aspectId: string) => void;
  createQuestion: CreateQuestionMutation;
};

export const QuestionForm = memo(function QuestionForm({
  draft,
  setDraft,
  selectedLineamentId,
  selectedLineament,
  lineamentOptions,
  audienceOptions,
  choiceOptions,
  setChoiceOptions,
  onChooseLineament,
  createQuestion,
}: QuestionFormProps) {
  const selectedFormatDescription =
    questionFormats.find((format) => format.value === draft.format)?.description ?? "";
  const isSubmitDisabled =
    createQuestion.isPending ||
    !draft.text.trim() ||
    (draft.format !== "open" && !draft.conventionCode?.trim()) ||
    (draft.format === "multipleChoice" &&
      normalizedChoiceOptions(choiceOptions).length < 2) ||
    !draft.factor.trim() ||
    !draft.characteristic.trim() ||
    !draft.aspect.trim() ||
    draft.audiences.length === 0;

  return (
    <Card>
      <CardHeader>
        <CardTitle>Nueva pregunta</CardTitle>
        <CardDescription>
          Alta guiada con escala de respuesta semantica y codigo Excel automatico.
        </CardDescription>
      </CardHeader>
      <CardContent>
        <form
          className="space-y-3"
          onSubmit={(event) => {
            event.preventDefault();
            createQuestion.mutate(draft);
          }}
        >
          <Input
            placeholder="Codigo opcional; la app lo genera si queda vacio"
            value={draft.code}
            onChange={(event) => setDraft({ ...draft, code: event.target.value })}
          />
          <Textarea
            placeholder="Texto de la pregunta"
            value={draft.text}
            onChange={(event) => setDraft({ ...draft, text: event.target.value })}
          />
          <div className="rounded-lg border bg-background/55 p-3">
            <p className="text-xs font-medium uppercase text-muted-foreground">
              Tipo de pregunta
            </p>
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
              <SelectTrigger className="mt-2">
                <SelectValue placeholder="Seleccione tipo" />
              </SelectTrigger>
              <SelectContent>
                {questionFormats.map((format) => (
                  <SelectItem key={format.value} value={format.value}>
                    {format.label}
                  </SelectItem>
                ))}
              </SelectContent>
            </Select>
            <p className="mt-2 text-xs leading-5 text-muted-foreground">
              {selectedFormatDescription}
            </p>
          </div>
          <QuestionResponseEditor
            format={draft.format}
            conventionCode={draft.conventionCode}
            onConventionChange={(conventionCode) =>
              setDraft({ ...draft, conventionCode })
            }
            options={choiceOptions}
            onOptionsChange={setChoiceOptions}
          />
          <div className="space-y-2">
            <p className="text-xs font-medium text-muted-foreground">
              Lineamiento asociado
            </p>
            <LineamentHierarchyPicker
              aspects={lineamentOptions}
              selectedAspectId={selectedLineamentId}
              onSelectAspect={onChooseLineament}
            />
            {selectedLineament ? (
              <div className="rounded-lg border bg-background/60 p-3 text-xs leading-5 text-muted-foreground">
                <p className="break-words">{draft.factor}</p>
                <p className="break-words">{draft.characteristic}</p>
                <p className="break-words">{draft.aspect}</p>
              </div>
            ) : (
              <p className="rounded-lg border bg-background/60 p-3 text-xs leading-5 text-muted-foreground">
                Primero seleccione un lineamiento. Asi la pregunta queda vinculada sin
                digitar factor, caracteristica o aspecto a mano.
              </p>
            )}
          </div>
          <AudienceSelector
            selected={draft.audiences}
            options={audienceOptions}
            onChange={(audiences) => setDraft({ ...draft, audiences })}
          />
          <Textarea
            placeholder="Justificacion"
            value={draft.justification ?? ""}
            onChange={(event) =>
              setDraft({ ...draft, justification: event.target.value })
            }
          />
          {createQuestion.isError ? (
            <p className="text-sm text-destructive">
              No se pudo registrar la pregunta. Revise los campos obligatorios.
            </p>
          ) : null}
          <Button className="w-full" disabled={isSubmitDisabled}>
            <Plus className="size-4" />
            Registrar pregunta
          </Button>
        </form>
      </CardContent>
    </Card>
  );
});
