import { Plus, Search, X } from "lucide-react";
import { memo } from "react";

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
import type { GuidelineAspect, NewGuidelineAspect, Question } from "@/lib/types";

import { InlineCreateLineamentForm } from "./InlineCreateLineamentForm";
import { InlineLineamentPanel } from "./InlineLineamentPanel";
import type { CharacteristicChoice, CodeName } from "./hierarchy";

type LineamentsListProps = {
  filtered: GuidelineAspect[];
  totalItems: number;
  page: number;
  pageCount: number;
  pageSize: number;
  hasLoaded: boolean;
  search: string;
  setSearch: (value: string) => void;
  onPageChange: (page: number) => void;
  createOpen: boolean;
  setCreateOpen: (value: boolean | ((value: boolean) => boolean)) => void;
  draft: NewGuidelineAspect;
  setDraft: (value: NewGuidelineAspect) => void;
  createAspect: {
    mutate: (aspect: NewGuidelineAspect) => void;
    isPending: boolean;
  };
  selected: GuidelineAspect | null;
  setSelected: (value: GuidelineAspect | null | ((value: GuidelineAspect | null) => GuidelineAspect | null)) => void;
  editDraft: NewGuidelineAspect | null;
  setEditDraft: (value: NewGuidelineAspect | null) => void;
  relatedQuestions: Question[];
  updateAspect: InlineLineamentPanelPropsUpdate;
  deleteAspect: InlineLineamentPanelPropsDelete;
  deleteText: string;
  setDeleteText: (value: string) => void;
  ackDeleteQuestions: boolean;
  setAckDeleteQuestions: (value: boolean) => void;
  factorChoices: CodeName[];
  characteristicOptions: CharacteristicChoice[];
  characteristicGroups: Array<{
    factorCode: string;
    factorName: string;
    options: CharacteristicChoice[];
  }>;
  editCharacteristicOptions: CharacteristicChoice[];
  factorDialogOpen: boolean;
  setFactorDialogOpen: (value: boolean) => void;
  characteristicDialogOpen: boolean;
  setCharacteristicDialogOpen: (value: boolean) => void;
  customFactorName: string;
  setCustomFactorName: (value: string) => void;
  customCharacteristicName: string;
  setCustomCharacteristicName: (value: string) => void;
};

type InlineLineamentPanelPropsUpdate = {
  mutate: (request: { aspectId: string; aspect: NewGuidelineAspect }) => void;
  isPending: boolean;
};

type InlineLineamentPanelPropsDelete = {
  mutate: (request: {
    aspectId: string;
    confirmationText: string;
    acknowledgeRelatedQuestions: boolean;
  }) => void;
  isPending: boolean;
};

export const LineamentsList = memo(function LineamentsList({
  filtered,
  totalItems,
  page,
  pageCount,
  pageSize,
  hasLoaded,
  search,
  setSearch,
  onPageChange,
  createOpen,
  setCreateOpen,
  draft,
  setDraft,
  createAspect,
  selected,
  setSelected,
  editDraft,
  setEditDraft,
  relatedQuestions,
  updateAspect,
  deleteAspect,
  deleteText,
  setDeleteText,
  ackDeleteQuestions,
  setAckDeleteQuestions,
  factorChoices,
  characteristicOptions,
  characteristicGroups,
  editCharacteristicOptions,
  factorDialogOpen,
  setFactorDialogOpen,
  characteristicDialogOpen,
  setCharacteristicDialogOpen,
  customFactorName,
  setCustomFactorName,
  customCharacteristicName,
  setCustomCharacteristicName,
}: LineamentsListProps) {
  return (
    <section className="space-y-4">
      <Card>
        <CardHeader className="gap-3 md:flex-row md:items-center md:justify-between md:space-y-0">
          <div>
            <CardTitle>Aspectos registrados</CardTitle>
            <CardDescription>{totalItems} aspectos filtrados</CardDescription>
          </div>
          <div className="flex w-full flex-col gap-2 md:w-auto md:flex-row">
            <Button
              type="button"
              onClick={() => {
                setCreateOpen((value) => !value);
                setSelected(null);
                setEditDraft(null);
              }}
            >
              <Plus className="size-4" />
              Agregar
            </Button>
            <div className="relative w-full md:w-80">
              <Search className="pointer-events-none absolute left-3 top-2.5 size-4 text-muted-foreground" />
              <Input
                className="pl-9"
                placeholder="Buscar factor, caracteristica o aspecto"
                value={search}
                onChange={(event) => setSearch(event.target.value)}
              />
            </div>
          </div>
        </CardHeader>
        <CardContent className="space-y-3">
          {createOpen ? (
            <div className="rounded-lg border border-primary/20 bg-background/70 p-4">
              <div className="mb-4 flex items-start justify-between gap-3">
                <div>
                  <p className="font-semibold">Nuevo lineamiento</p>
                  <p className="mt-1 text-xs text-muted-foreground">
                    Se crea dentro de la lista para mantener el contexto de cobertura.
                  </p>
                </div>
                <Button
                  type="button"
                  variant="ghost"
                  size="icon"
                  aria-label="Cerrar formulario"
                  onClick={() => setCreateOpen(false)}
                >
                  <X className="size-4" />
                </Button>
              </div>
              <InlineCreateLineamentForm
                draft={draft}
                setDraft={setDraft}
                createAspect={createAspect}
                factorChoices={factorChoices}
                characteristicOptions={characteristicOptions}
                characteristicGroups={characteristicGroups}
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
          ) : null}
          {filtered.map((aspect) => (
            <div key={aspect.id} className="rounded-lg border bg-background/70">
              <button
                type="button"
                onClick={() => {
                  setSelected((current) => (current?.id === aspect.id ? null : aspect));
                  setCreateOpen(false);
                  setEditDraft(null);
                  setDeleteText("");
                  setAckDeleteQuestions(false);
                }}
                className="w-full p-4 text-left transition-colors hover:bg-muted/45"
              >
                <div className="flex flex-col gap-2 md:flex-row md:items-start md:justify-between">
                  <div>
                    <p className="mt-1 break-words text-sm leading-6">
                      {aspect.aspectDescription}
                    </p>
                  </div>
                  <Badge variant={aspect.scope === "program" ? "warning" : "secondary"}>
                    {aspect.scope === "program" ? "Programa" : "Institucional"}
                  </Badge>
                </div>
                <p className="mt-3 break-words text-xs text-muted-foreground">
                  {aspect.factorCode} {aspect.factorName} / {aspect.characteristicCode}{" "}
                  {aspect.characteristicName}
                </p>
              </button>
              {selected?.id === aspect.id ? (
                <InlineLineamentPanel
                  selected={selected}
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
                  editCharacteristicOptions={editCharacteristicOptions}
                />
              ) : null}
            </div>
          ))}
          {hasLoaded && filtered.length === 0 ? (
            <div className="rounded-lg border bg-background/70 p-6 text-sm text-muted-foreground">
              Aun no hay lineamientos registrados. Agregue el primer aspecto CNA.
            </div>
          ) : null}
          <Pagination
            page={page}
            pageCount={pageCount}
            totalItems={totalItems}
            pageSize={pageSize}
            onPageChange={onPageChange}
          />
        </CardContent>
      </Card>
    </section>
  );
});
