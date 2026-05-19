import { useMutation, useQuery, useQueryClient } from "@tanstack/react-query";
import { createFileRoute } from "@tanstack/react-router";
import { useCallback, useEffect, useMemo, useState } from "react";

import { LineamentWorkSelector } from "@/features/questions/LineamentWorkSelector";
import { QuestionForm } from "@/features/questions/QuestionForm";
import { QuestionsTable } from "@/features/questions/QuestionsTable";
import { audienceOptionsFromQuestions } from "@/features/questions/audiences";
import {
  compareGuidelineAspect,
  fillQuestionFromLineament,
  questionMatchesLineament,
} from "@/features/questions/lineament";
import {
  type ChoiceOption,
  prepareQuestionForSave,
} from "@/features/questions/questionFormat";
import { api } from "@/lib/api";
import type { NewQuestion } from "@/lib/types";

export const Route = createFileRoute("/questions")({
  component: QuestionsPage,
});

const initialQuestion: NewQuestion = {
  code: "",
  text: "",
  scope: "institutional",
  format: "likert",
  conventionCode: "A",
  status: "add",
  factor: "",
  characteristic: "",
  aspect: "",
  audiences: [],
  justification: "",
};

const initialChoiceOptions: ChoiceOption[] = [
  { id: "option-1", label: "" },
  { id: "option-2", label: "" },
];
const questionPageSize = 10;

function QuestionsPage() {
  const initialSearch =
    typeof window === "undefined"
      ? ""
      : new URLSearchParams(window.location.search).get("lineament") ?? "";
  const initialLineamentId =
    typeof window === "undefined"
      ? ""
      : new URLSearchParams(window.location.search).get("lineamentId") ?? "";
  const [search, setSearch] = useState(initialSearch);
  const [draft, setDraft] = useState(initialQuestion);
  const [choiceOptions, setChoiceOptions] = useState<ChoiceOption[]>(initialChoiceOptions);
  const [selectedLineamentId, setSelectedLineamentId] = useState(initialLineamentId);
  const [page, setPage] = useState(1);
  const [editingQuestionId, setEditingQuestionId] = useState<string | null>(null);
  const [notice, setNotice] = useState("");
  const queryClient = useQueryClient();
  const questions = useQuery({ queryKey: ["questions"], queryFn: api.questions });
  const aspects = useQuery({
    queryKey: ["guideline-aspects"],
    queryFn: api.guidelineAspects,
  });

  const selectedLineament = useMemo(
    () => (aspects.data ?? []).find((aspect) => aspect.id === selectedLineamentId) ?? null,
    [aspects.data, selectedLineamentId],
  );
  const lineamentOptions = useMemo(
    () => [...(aspects.data ?? [])].sort(compareGuidelineAspect),
    [aspects.data],
  );
  const audienceOptions = useMemo(
    () => audienceOptionsFromQuestions(questions.data ?? []),
    [questions.data],
  );
  const filtered = useMemo(() => {
    const needle = search.trim().toLowerCase();
    const source = selectedLineament
      ? (questions.data ?? []).filter((question) =>
          questionMatchesLineament(question, selectedLineament),
        )
      : (questions.data ?? []);

    if (!needle) return source;

    return source.filter((question) =>
      [question.code, question.text, question.factor, question.characteristic, question.aspect]
        .join(" ")
        .toLowerCase()
        .includes(needle),
    );
  }, [questions.data, search, selectedLineament]);
  const pageCount = Math.max(Math.ceil(filtered.length / questionPageSize), 1);
  const paginatedQuestions = useMemo(() => {
    const start = (page - 1) * questionPageSize;
    return filtered.slice(start, start + questionPageSize);
  }, [filtered, page]);

  const createQuestion = useMutation({
    mutationFn: (question: NewQuestion) =>
      api.createQuestion(prepareQuestionForSave(question, choiceOptions)),
    onSuccess: async () => {
      setDraft(
        selectedLineament
          ? fillQuestionFromLineament(initialQuestion, selectedLineament)
          : initialQuestion,
      );
      setChoiceOptions(initialChoiceOptions);
      setNotice("Pregunta registrada. Revise validaciones antes de exportar.");
      await queryClient.invalidateQueries({ queryKey: ["questions"] });
      await queryClient.invalidateQueries({ queryKey: ["dashboard"] });
    },
  });
  const updateQuestion = useMutation({
    mutationFn: ({
      questionId,
      question,
      choiceOptions,
    }: {
      questionId: string;
      question: NewQuestion;
      choiceOptions: ChoiceOption[];
    }) =>
      api.updateQuestion({
        questionId,
        question: prepareQuestionForSave(question, choiceOptions),
      }),
    onSuccess: async () => {
      setEditingQuestionId(null);
      setNotice("Pregunta actualizada y marcada segun impacto contra la linea base.");
      await queryClient.invalidateQueries({ queryKey: ["questions"] });
      await queryClient.invalidateQueries({ queryKey: ["dashboard"] });
      await queryClient.invalidateQueries({ queryKey: ["baseline-status"] });
    },
  });

  useEffect(() => {
    if (selectedLineament) {
      setDraft((current) => fillQuestionFromLineament(current, selectedLineament));
    }
  }, [selectedLineament]);

  useEffect(() => {
    setPage(1);
  }, [search, selectedLineamentId]);

  useEffect(() => {
    if (page > pageCount) {
      setPage(pageCount);
    }
  }, [page, pageCount]);

  const chooseLineament = useCallback(
    (aspectId: string) => {
      const aspect = (aspects.data ?? []).find((item) => item.id === aspectId);
      setSelectedLineamentId(aspectId);
      if (typeof window !== "undefined") {
        const url = new URL(window.location.href);
        url.searchParams.set("lineamentId", aspectId);
        url.searchParams.delete("lineament");
        window.history.replaceState(null, "", `${url.pathname}${url.search}`);
      }
      if (aspect) {
        setDraft((current) => fillQuestionFromLineament(current, aspect));
      }
    },
    [aspects.data],
  );

  const clearLineamentSelection = useCallback(() => {
    setSelectedLineamentId("");
    setSearch("");
    if (typeof window !== "undefined") {
      window.history.replaceState(null, "", window.location.pathname);
    }
  }, []);
  const handleEditQuestion = useCallback((questionId: string) => {
    setEditingQuestionId((current) => (current === questionId ? null : questionId));
  }, []);
  const handleSaveQuestion = useCallback(
    (questionId: string, question: NewQuestion, choiceOptions: ChoiceOption[]) => {
      updateQuestion.mutate({ questionId, question, choiceOptions });
    },
    [updateQuestion],
  );

  return (
    <div className="space-y-6">
      <section>
        <h1 className="text-2xl font-semibold">Banco unico de preguntas</h1>
        <p className="mt-2 max-w-3xl text-sm leading-6 text-muted-foreground">
          Registro central con estado operativo, trazabilidad de justificacion,
          convencion de respuesta, aspecto CNA y subpublicos asignados.
        </p>
      </section>

      <LineamentWorkSelector
        selectedLineamentId={selectedLineamentId}
        selectedLineament={selectedLineament}
        lineamentOptions={lineamentOptions}
        visibleCount={filtered.length}
        onChooseLineament={chooseLineament}
        onClearSelection={clearLineamentSelection}
      />
      {notice ? (
        <div className="rounded-lg border border-primary/20 bg-primary/10 p-3 text-sm text-primary">
          {notice}
        </div>
      ) : null}

      <section className="grid gap-4 xl:grid-cols-[1fr_25rem]">
        <QuestionsTable
          questions={paginatedQuestions}
          totalQuestions={filtered.length}
          search={search}
          page={page}
          pageCount={pageCount}
          pageSize={questionPageSize}
          editingQuestionId={editingQuestionId}
          lineamentOptions={lineamentOptions}
          audienceOptions={audienceOptions}
          isUpdating={updateQuestion.isPending}
          onSearchChange={setSearch}
          onPageChange={setPage}
          onEditQuestion={handleEditQuestion}
          onCancelEdit={() => setEditingQuestionId(null)}
          onSaveQuestion={handleSaveQuestion}
        />
        <QuestionForm
          draft={draft}
          setDraft={setDraft}
          selectedLineamentId={selectedLineamentId}
          selectedLineament={selectedLineament}
          lineamentOptions={lineamentOptions}
          audienceOptions={audienceOptions}
          choiceOptions={choiceOptions}
          setChoiceOptions={setChoiceOptions}
          onChooseLineament={chooseLineament}
          createQuestion={createQuestion}
        />
      </section>
    </div>
  );
}
