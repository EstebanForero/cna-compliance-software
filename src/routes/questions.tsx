import { useMutation, useQuery, useQueryClient } from "@tanstack/react-query";
import { createFileRoute } from "@tanstack/react-router";
import { useCallback, useEffect, useMemo, useState } from "react";

import { useToast } from "@/components/ui/toast";
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
import type { CollaborationLock, NewQuestion } from "@/lib/types";

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
  const [blockedQuestionLocks, setBlockedQuestionLocks] = useState<
    Map<string, CollaborationLock>
  >(new Map());
  const queryClient = useQueryClient();
  const { toast } = useToast();
  const workspace = useQuery({ queryKey: ["workspace"], queryFn: api.workspace });
  const questions = useQuery({
    queryKey: ["questions"],
    queryFn: api.questions,
  });
  const aspects = useQuery({
    queryKey: ["guideline-aspects"],
    queryFn: api.guidelineAspects,
  });
  const instruments = useQuery({
    queryKey: ["instrument-definitions"],
    queryFn: api.instrumentDefinitions,
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
  const blockedQuestionIds = useMemo(
    () => Array.from(blockedQuestionLocks.keys()).sort(),
    [blockedQuestionLocks],
  );
  useQuery({
    queryKey: ["known-blocked-question-locks", blockedQuestionIds],
    queryFn: async () => {
      const locks = await api.collaborationLocksForResources({
        resourceType: "question",
        resourceIds: blockedQuestionIds,
      });
      setBlockedQuestionLocks((current) => {
        const next = new Map(current);
        const active = new Map(locks.map((lock) => [lock.resourceId, lock] as const));
        for (const id of blockedQuestionIds) {
          const lock = active.get(id);
          if (lock) {
            next.set(id, lock);
          } else {
            next.delete(id);
          }
        }
        return next;
      });
      return locks;
    },
    enabled: Boolean(workspace.data?.tursoConnected && blockedQuestionIds.length > 0),
    refetchInterval: 10000,
  });
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
      toast({
        title: "Pregunta registrada",
        description: "Revise validaciones antes de exportar.",
        tone: "success",
      });
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
        expectedUpdatedAt:
          questions.data?.find((item) => item.id === questionId)?.updatedAt ?? null,
      }),
    onSuccess: async () => {
      if (editingQuestionId) {
        releaseLock.mutate({
          resourceType: "question",
          resourceId: editingQuestionId,
        });
      }
      setEditingQuestionId(null);
      toast({
        title: "Pregunta actualizada",
        description: "La pregunta quedo marcada segun su impacto contra la linea base.",
        tone: "success",
      });
      await queryClient.invalidateQueries({ queryKey: ["questions"] });
      await queryClient.invalidateQueries({ queryKey: ["dashboard"] });
      await queryClient.invalidateQueries({ queryKey: ["baseline-status"] });
    },
    onError: (error) => {
      toast({
        title: "No se pudo guardar",
        description: error instanceof Error ? error.message : "No se pudo actualizar la pregunta.",
        tone: "error",
      });
    },
  });
  const acquireLock = useMutation({
    mutationFn: api.acquireCollaborationLock,
    onSuccess: async (_, request) => {
      setBlockedQuestionLocks((current) => {
        const next = new Map(current);
        next.delete(request.resourceId);
        return next;
      });
      setEditingQuestionId(request.resourceId);
    },
    onError: (error, request) => {
      const fallback = "Esta pregunta esta siendo editada por otro editor.";
      const description = error instanceof Error ? error.message : fallback;
      toast({
        title: "Pregunta bloqueada",
        description,
        tone: "warning",
      });
      void (async () => {
        if (!workspace.data?.tursoConnected) return;
        try {
          const locks = await api.collaborationLocksForResources({
            resourceType: "question",
            resourceIds: [request.resourceId],
          });
          const lock = locks.find((item) => item.resourceId === request.resourceId);
          if (!lock) return;
          setBlockedQuestionLocks((current) => {
            const next = new Map(current);
            next.set(request.resourceId, lock);
            return next;
          });
        } catch {
          // The lock conflict toast is already visible; the next edit attempt can retry.
        }
      })();
    },
  });
  const releaseLock = useMutation({
    mutationFn: api.releaseCollaborationLock,
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
  const currentEditorName = workspace.data?.editorProfile?.fullName ?? "";
  const handleEditQuestion = useCallback(
    (questionId: string) => {
      if (editingQuestionId === questionId) {
        releaseLock.mutate({ resourceType: "question", resourceId: questionId });
        setEditingQuestionId(null);
        return;
      }
      const blockedLock = blockedQuestionLocks.get(questionId);
      if (blockedLock) {
        toast({
          title: "Pregunta bloqueada",
          description: `Esta pregunta esta siendo editada por ${blockedLock.editorName}.`,
          tone: "warning",
        });
        return;
      }
      if (!workspace.data?.tursoConnected) {
        setEditingQuestionId(questionId);
        return;
      }
      acquireLock.mutate({ resourceType: "question", resourceId: questionId });
    },
    [
      acquireLock,
      blockedQuestionLocks,
      editingQuestionId,
      releaseLock,
      toast,
      workspace.data?.tursoConnected,
    ],
  );
  const handleCancelEdit = useCallback(() => {
    if (editingQuestionId && workspace.data?.tursoConnected) {
      releaseLock.mutate({ resourceType: "question", resourceId: editingQuestionId });
    }
    setEditingQuestionId(null);
  }, [editingQuestionId, releaseLock, workspace.data?.tursoConnected]);
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
          instruments={instruments.data ?? []}
          isUpdating={updateQuestion.isPending}
          editingQuestionLocked={Boolean(editingQuestionId && workspace.data?.tursoConnected)}
          blockedQuestionLocks={blockedQuestionLocks}
          currentEditorName={currentEditorName}
          onSearchChange={setSearch}
          onPageChange={setPage}
          onEditQuestion={handleEditQuestion}
          onCancelEdit={handleCancelEdit}
          onSaveQuestion={handleSaveQuestion}
        />
        <QuestionForm
          draft={draft}
          setDraft={setDraft}
          selectedLineamentId={selectedLineamentId}
          selectedLineament={selectedLineament}
          lineamentOptions={lineamentOptions}
          audienceOptions={audienceOptions}
          instruments={instruments.data ?? []}
          choiceOptions={choiceOptions}
          setChoiceOptions={setChoiceOptions}
          onChooseLineament={chooseLineament}
          createQuestion={createQuestion}
        />
      </section>
    </div>
  );
}
