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
  const [filterFactorCode, setFilterFactorCode] = useState("");
  const [filterCharacteristicKey, setFilterCharacteristicKey] = useState("");
  const [page, setPage] = useState(1);
  const queryClient = useQueryClient();
  const workspace = useQuery({ queryKey: ["workspace"], queryFn: api.workspace });
  const liveRefetchInterval = workspace.data?.tursoConnected ? 5000 : false;
  const aspects = useQuery({
    queryKey: ["guideline-aspects"],
    queryFn: api.guidelineAspects,
    refetchInterval: liveRefetchInterval,
  });
  const questions = useQuery({
    queryKey: ["questions"],
    queryFn: api.questions,
    refetchInterval: liveRefetchInterval,
  });

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
  const filterCharacteristicOptions = useMemo(() => {
    const source = filterFactorCode
      ? allCharacteristicOptions.filter((option) => option.factorCode === filterFactorCode)
      : allCharacteristicOptions;
    return [...source].sort(compareCharacteristicChoice);
  }, [allCharacteristicOptions, filterFactorCode]);
  const characteristicGroups = useMemo(
    () => groupCharacteristicsByFactor(characteristicOptions),
    [characteristicOptions],
  );
  const filtered = useMemo(() => {
    const needle = search.trim().toLowerCase();
    const [characteristicFactorCode, characteristicCode] = filterCharacteristicKey.split(":");
    return (aspects.data ?? []).filter((aspect) => {
      if (filterFactorCode && aspect.factorCode !== filterFactorCode) return false;
      if (
        characteristicFactorCode &&
        characteristicCode &&
        (aspect.factorCode !== characteristicFactorCode ||
          aspect.characteristicCode !== characteristicCode)
      ) {
        return false;
      }
      if (!needle) return true;
      return [
        aspect.guidelineTitle,
        aspect.factorCode,
        aspect.factorName,
        aspect.characteristicCode,
        aspect.characteristicName,
        aspect.aspectCode,
        aspect.aspectDescription,
      ]
        .join(" ")
        .toLowerCase()
        .includes(needle);
    });
  }, [aspects.data, filterCharacteristicKey, filterFactorCode, search]);
  const pageCount = Math.max(Math.ceil(filtered.length / lineamentPageSize), 1);
  const paginatedLineaments = useMemo(() => {
    const start = (page - 1) * lineamentPageSize;
    return filtered.slice(start, start + lineamentPageSize);
  }, [filtered, page]);
  const relatedQuestions = useMemo(() => {
    if (!selected) return [];
    return (questions.data ?? []).filter((question) =>
      questionMatchesLineament(question, selected),
    );
  }, [questions.data, selected]);

  const handleSearchChange = useCallback((value: string) => setSearch(value), []);
  const handlePageChange = useCallback((nextPage: number) => setPage(nextPage), []);
  const handleFilterFactorChange = useCallback((value: string) => {
    setFilterFactorCode(value);
    setFilterCharacteristicKey("");
  }, []);
  const clearLineamentFilters = useCallback(() => {
    setFilterFactorCode("");
    setFilterCharacteristicKey("");
    setSearch("");
  }, []);

  useEffect(() => {
    setPage(1);
  }, [filterCharacteristicKey, filterFactorCode, search]);

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
        filterFactorCode={filterFactorCode}
        setFilterFactorCode={handleFilterFactorChange}
        filterCharacteristicKey={filterCharacteristicKey}
        setFilterCharacteristicKey={setFilterCharacteristicKey}
        filterCharacteristicOptions={filterCharacteristicOptions}
        clearLineamentFilters={clearLineamentFilters}
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
