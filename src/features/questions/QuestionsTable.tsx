import { Lock, Pencil, Search } from "lucide-react";
import { Fragment, memo } from "react";

import { Badge } from "@/components/ui/badge";
import { Button } from "@/components/ui/button";
import {
  Card,
  CardContent,
  CardDescription,
  CardHeader,
  CardTitle,
} from "@/components/ui/card";
import { Input } from "@/components/ui/input";
import { Pagination } from "@/components/ui/pagination";
import type {
  CollaborationLock,
  GuidelineAspect,
  InstrumentDefinition,
  NewQuestion,
  Question,
} from "@/lib/types";

import { QuestionEditPanel } from "./QuestionEditPanel";
import { normalizeAudienceLabel } from "./audiences";
import type { ChoiceOption } from "./questionFormat";
import { questionFormatLabel, responseConventionLabel } from "./questionFormat";
import { StatusBadge } from "./StatusBadge";

type QuestionsTableProps = {
  questions: Question[];
  totalQuestions: number;
  search: string;
  page: number;
  pageCount: number;
  pageSize: number;
  editingQuestionId: string | null;
  lineamentOptions: GuidelineAspect[];
  audienceOptions: string[];
  instruments: InstrumentDefinition[];
  isUpdating: boolean;
  collaborationLocks: Map<string, CollaborationLock>;
  currentEditorName: string;
  onSearchChange: (value: string) => void;
  onPageChange: (page: number) => void;
  onEditQuestion: (questionId: string) => void;
  onCancelEdit: () => void;
  onSaveQuestion: (
    questionId: string,
    question: NewQuestion,
    choiceOptions: ChoiceOption[],
  ) => void;
};

export const QuestionsTable = memo(function QuestionsTable({
  questions,
  totalQuestions,
  search,
  page,
  pageCount,
  pageSize,
  editingQuestionId,
  lineamentOptions,
  audienceOptions,
  instruments,
  isUpdating,
  collaborationLocks,
  currentEditorName,
  onSearchChange,
  onPageChange,
  onEditQuestion,
  onCancelEdit,
  onSaveQuestion,
}: QuestionsTableProps) {
  return (
    <Card>
      <CardHeader className="gap-3 md:flex-row md:items-center md:justify-between md:space-y-0">
        <div>
          <CardTitle>Preguntas registradas</CardTitle>
          <CardDescription>{totalQuestions} resultados filtrados</CardDescription>
        </div>
        <div className="relative w-full md:w-72">
          <Search className="pointer-events-none absolute left-3 top-2.5 size-4 text-muted-foreground" />
          <Input
            className="pl-9"
            placeholder="Buscar por codigo, factor o texto"
            value={search}
            onChange={(event) => onSearchChange(event.target.value)}
          />
        </div>
      </CardHeader>
      <CardContent className="space-y-4">
        <div className="overflow-x-auto">
          <table className="w-full min-w-[760px] border-collapse text-sm">
            <thead>
              <tr className="border-b text-left text-xs uppercase text-muted-foreground">
                <th className="py-2 pr-3 font-medium">Codigo</th>
                <th className="px-3 py-2 font-medium">Pregunta</th>
                <th className="px-3 py-2 font-medium">Alineacion CNA</th>
                <th className="px-3 py-2 font-medium">Instrumentos</th>
                <th className="py-2 pl-3 font-medium">Estado</th>
                <th className="py-2 pl-3 font-medium">Editar</th>
              </tr>
            </thead>
            <tbody>
              {questions.map((question) => {
                const lock = collaborationLocks.get(question.id);
                const instrumentLabels = instrumentLabelsForQuestion(question, instruments);
                const lockedByOther = Boolean(lock && lock.editorName !== currentEditorName);
                const lockLabel = lockedByOther
                  ? `Editando: ${lock?.editorName}`
                  : lock
                    ? "Bloqueada para ti"
                    : "";
                return (
                  <Fragment key={question.id}>
                    <tr className="border-b last:border-b-0">
                      <td className="py-3 pr-3 font-medium">{question.code}</td>
                      <td className="max-w-md px-3 py-3">
                        <p className="line-clamp-2">{question.text}</p>
                        <p className="mt-1 text-xs text-muted-foreground">
                          {question.scope === "institutional" ? "Institucional" : "Programa"} ·{" "}
                          {questionFormatLabel(question.format)} ·{" "}
                          {question.format === "open"
                            ? "Sin convencion"
                            : responseConventionLabel(question.conventionCode)}
                        </p>
                        {lockLabel ? (
                          <p className="mt-2 inline-flex items-center gap-1 rounded-md border bg-muted/60 px-2 py-1 text-xs text-muted-foreground">
                            <Lock className="size-3" />
                            {lockLabel}
                          </p>
                        ) : null}
                      </td>
                      <td className="max-w-xs px-3 py-3 text-xs text-muted-foreground">
                        <p className="break-words">
                          <span className="font-medium text-foreground">Factor:</span>{" "}
                          {question.factor}
                        </p>
                        <p className="mt-1 break-words">
                          <span className="font-medium text-foreground">Caracteristica:</span>{" "}
                          {question.characteristic}
                        </p>
                        <p className="mt-1 break-words">
                          <span className="font-medium text-foreground">Aspecto:</span>{" "}
                          {question.aspect}
                        </p>
                      </td>
                      <td className="px-3 py-3">
                        <div className="max-w-56 space-y-1">
                          <div className="flex flex-wrap gap-1">
                            {instrumentLabels.map((label) => (
                              <Badge key={label} variant="outline" className="bg-background">
                                {label}
                              </Badge>
                            ))}
                          </div>
                          <p className="text-xs text-muted-foreground">
                            {instrumentLabels.length} instrumento
                            {instrumentLabels.length === 1 ? "" : "s"}{" "}
                            · {question.audiences.length} público
                            {question.audiences.length === 1 ? "" : "s"}
                          </p>
                        </div>
                      </td>
                      <td className="py-3 pl-3">
                        <StatusBadge status={question.status} />
                      </td>
                      <td className="py-3 pl-3">
                        <Button
                          type="button"
                          variant={lock ? "secondary" : "outline"}
                          size="icon"
                          aria-label={lockedByOther ? `Bloqueada por ${lock?.editorName}` : "Editar pregunta"}
                          onClick={() => onEditQuestion(question.id)}
                          title={lockLabel || "Editar pregunta"}
                          disabled={lockedByOther}
                        >
                          {lock ? <Lock className="size-4" /> : <Pencil className="size-4" />}
                        </Button>
                      </td>
                    </tr>
                    {editingQuestionId === question.id ? (
                      <tr>
                        <td colSpan={6} className="bg-muted/25 p-3">
                          <QuestionEditPanel
                            question={question}
                            lineamentOptions={lineamentOptions}
                            audienceOptions={audienceOptions}
                            instruments={instruments}
                            isSaving={isUpdating}
                            onCancel={onCancelEdit}
                            onSave={(draft, choiceOptions) =>
                              onSaveQuestion(question.id, draft, choiceOptions)
                            }
                          />
                        </td>
                      </tr>
                    ) : null}
                  </Fragment>
                );
              })}
              {questions.length === 0 ? (
                <tr>
                  <td colSpan={6} className="py-8 text-center text-muted-foreground">
                    No hay preguntas para los filtros actuales.
                  </td>
                </tr>
              ) : null}
            </tbody>
          </table>
        </div>
        <Pagination
          page={page}
          pageCount={pageCount}
          totalItems={totalQuestions}
          pageSize={pageSize}
          onPageChange={onPageChange}
        />
      </CardContent>
    </Card>
  );
});

function instrumentLabelsForQuestion(
  question: Question,
  instruments: InstrumentDefinition[],
) {
  const labels = new Set<string>();
  for (const audience of question.audiences) {
    const normalizedAudience = normalizeAudienceLabel(audience).toLowerCase();
    const instrument = instruments.find((candidate) =>
      candidate.publicKeys.some((publicKey) => {
        const normalizedPublic = normalizeAudienceLabel(publicKey).toLowerCase();
        return (
          normalizedAudience === normalizedPublic ||
          normalizedAudience.startsWith(`${normalizedPublic} `)
        );
      }),
    );
    labels.add(instrument?.label ?? audience);
  }
  return Array.from(labels).slice(0, 4);
}
