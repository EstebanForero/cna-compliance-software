import { useMutation, useQuery, useQueryClient } from "@tanstack/react-query";
import { createFileRoute } from "@tanstack/react-router";
import { useCallback, useEffect, useMemo, useState } from "react";

import { LineamentsHero } from "@/features/lineaments/LineamentsHero";
import { LineamentsList } from "@/features/lineaments/LineamentsList";
import {
  collectCharacteristicChoices,
  compareCharacteristicChoice,
  factorChoicesFromData,
  groupCharacteristicsByFactor,
  initialAspect,
  questionMatchesLineament,
} from "@/features/lineaments/hierarchy";
import { api } from "@/lib/api";
import type { GuidelineAspect, NewGuidelineAspect } from "@/lib/types";

export const Route = createFileRoute("/lineaments")({
  component: LineamentsPage,
});

const lineamentPageSize = 10;

function LineamentsPage() {
  const [search, setSearch] = useState("");
  const [draft, setDraft] = useState(initialAspect);
  const [createOpen, setCreateOpen] = useState(false);
  const [selected, setSelected] = useState<GuidelineAspect | null>(null);
  const [editDraft, setEditDraft] = useState<NewGuidelineAspect | null>(null);
  const [deleteText, setDeleteText] = useState("");
  const [ackDeleteQuestions, setAckDeleteQuestions] = useState(false);
  const [factorDialogOpen, setFactorDialogOpen] = useState(false);
  const [characteristicDialogOpen, setCharacteristicDialogOpen] = useState(false);
  const [customFactorName, setCustomFactorName] = useState("");
  const [customCharacteristicName, setCustomCharacteristicName] = useState("");
  const [page, setPage] = useState(1);
  const queryClient = useQueryClient();
  const aspects = useQuery({
    queryKey: ["guideline-aspects"],
    queryFn: api.guidelineAspects,
  });
  const questions = useQuery({ queryKey: ["questions"], queryFn: api.questions });

  const createAspect = useMutation({
    mutationFn: api.createGuidelineAspect,
    onSuccess: async () => {
      setDraft(initialAspect);
      setCreateOpen(false);
      await queryClient.invalidateQueries({ queryKey: ["guideline-aspects"] });
    },
  });
  const updateAspect = useMutation({
    mutationFn: api.updateGuidelineAspect,
    onSuccess: async (result) => {
      setSelected(result.aspect);
      setEditDraft(null);
      await queryClient.invalidateQueries({ queryKey: ["guideline-aspects"] });
      await queryClient.invalidateQueries({ queryKey: ["questions"] });
      await queryClient.invalidateQueries({ queryKey: ["dashboard"] });
      await queryClient.invalidateQueries({ queryKey: ["baseline-status"] });
    },
  });
  const deleteAspect = useMutation({
    mutationFn: api.deleteGuidelineAspect,
    onSuccess: async () => {
      setSelected(null);
      setDeleteText("");
      setAckDeleteQuestions(false);
      await queryClient.invalidateQueries({ queryKey: ["guideline-aspects"] });
      await queryClient.invalidateQueries({ queryKey: ["questions"] });
      await queryClient.invalidateQueries({ queryKey: ["dashboard"] });
    },
  });

  const filtered = useMemo(() => {
    const needle = search.trim().toLowerCase();
    const source = aspects.data ?? [];
    if (!needle) return source;
    return source.filter((aspect) =>
      [
        aspect.guidelineTitle,
        aspect.factorName,
        aspect.characteristicName,
        aspect.aspectDescription,
      ]
        .join(" ")
        .toLowerCase()
        .includes(needle),
    );
  }, [aspects.data, search]);
  const pageCount = Math.max(Math.ceil(filtered.length / lineamentPageSize), 1);
  const paginatedLineaments = useMemo(() => {
    const start = (page - 1) * lineamentPageSize;
    return filtered.slice(start, start + lineamentPageSize);
  }, [filtered, page]);

  const allCharacteristicOptions = useMemo(
    () => collectCharacteristicChoices(aspects.data ?? [], questions.data ?? []),
    [aspects.data, questions.data],
  );
  const characteristicOptions = useMemo(() => {
    const source = draft.factorCode
      ? allCharacteristicOptions.filter((option) => option.factorCode === draft.factorCode)
      : allCharacteristicOptions;
    return [...source].sort(compareCharacteristicChoice);
  }, [allCharacteristicOptions, draft.factorCode]);
  const editCharacteristicOptions = useMemo(() => {
    if (!editDraft) return [];
    return allCharacteristicOptions
      .filter((option) => option.factorCode === editDraft.factorCode)
      .sort(compareCharacteristicChoice);
  }, [allCharacteristicOptions, editDraft]);
  const factorChoices = useMemo(
    () => factorChoicesFromData(aspects.data ?? [], questions.data ?? []),
    [aspects.data, questions.data],
  );
  const characteristicGroups = useMemo(
    () => groupCharacteristicsByFactor(characteristicOptions),
    [characteristicOptions],
  );
  const relatedQuestions = useMemo(() => {
    if (!selected) return [];
    return (questions.data ?? []).filter((question) =>
      questionMatchesLineament(question, selected),
    );
  }, [questions.data, selected]);

  const handleSearchChange = useCallback((value: string) => setSearch(value), []);
  const handlePageChange = useCallback((nextPage: number) => setPage(nextPage), []);

  useEffect(() => {
    setPage(1);
  }, [search]);

  useEffect(() => {
    if (page > pageCount) setPage(pageCount);
  }, [page, pageCount]);

  return (
    <div className="space-y-6">
      <LineamentsHero />
      <LineamentsList
        filtered={paginatedLineaments}
        totalItems={filtered.length}
        page={page}
        pageCount={pageCount}
        pageSize={lineamentPageSize}
        hasLoaded={aspects.isSuccess}
        search={search}
        setSearch={handleSearchChange}
        onPageChange={handlePageChange}
        createOpen={createOpen}
        setCreateOpen={setCreateOpen}
        draft={draft}
        setDraft={setDraft}
        createAspect={createAspect}
        selected={selected}
        setSelected={setSelected}
        editDraft={editDraft}
        setEditDraft={setEditDraft}
        relatedQuestions={relatedQuestions}
        updateAspect={updateAspect}
        deleteAspect={deleteAspect}
        deleteText={deleteText}
        setDeleteText={setDeleteText}
        ackDeleteQuestions={ackDeleteQuestions}
        setAckDeleteQuestions={setAckDeleteQuestions}
        factorChoices={factorChoices}
        characteristicOptions={characteristicOptions}
        characteristicGroups={characteristicGroups}
        editCharacteristicOptions={editCharacteristicOptions}
        factorDialogOpen={factorDialogOpen}
        setFactorDialogOpen={setFactorDialogOpen}
        characteristicDialogOpen={characteristicDialogOpen}
        setCharacteristicDialogOpen={setCharacteristicDialogOpen}
        customFactorName={customFactorName}
        setCustomFactorName={setCustomFactorName}
        customCharacteristicName={customCharacteristicName}
        setCustomCharacteristicName={setCustomCharacteristicName}
      />
    </div>
  );
}
