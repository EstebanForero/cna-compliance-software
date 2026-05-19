import { ExternalLink, Pencil, Plus, Save, Trash2, X } from "lucide-react";
import { memo, useState } from "react";

import {
  Accordion,
  AccordionContent,
  AccordionItem,
  AccordionTrigger,
} from "@/components/ui/accordion";
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
  SelectItem,
  SelectTrigger,
  SelectValue,
} from "@/components/ui/select";
import { Textarea } from "@/components/ui/textarea";
import type { CnaFactorCode, GuidelineAspect, NewGuidelineAspect, Question } from "@/lib/types";

import {
  type CharacteristicChoice,
  type CodeName,
  aspectToDraft,
} from "./hierarchy";

type InlineLineamentPanelProps = {
  selected: GuidelineAspect;
  editDraft: NewGuidelineAspect | null;
  setEditDraft: (value: NewGuidelineAspect | null) => void;
  relatedQuestions: Question[];
  updateAspect: {
    mutate: (request: { aspectId: string; aspect: NewGuidelineAspect }) => void;
    isPending: boolean;
  };
  deleteAspect: {
    mutate: (request: {
      aspectId: string;
      confirmationText: string;
      acknowledgeRelatedQuestions: boolean;
    }) => void;
    isPending: boolean;
  };
  deleteText: string;
  setDeleteText: (value: string) => void;
  ackDeleteQuestions: boolean;
  setAckDeleteQuestions: (value: boolean) => void;
  factorChoices: CodeName[];
  editCharacteristicOptions: CharacteristicChoice[];
};

export const InlineLineamentPanel = memo(function InlineLineamentPanel({
  selected,
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
  editCharacteristicOptions,
}: InlineLineamentPanelProps) {
  const [deleteDialogOpen, setDeleteDialogOpen] = useState(false);

  return (
    <div className="border-t bg-muted/20 p-4">
      {editDraft ? (
        <form
          className="grid gap-3 lg:grid-cols-2"
          onSubmit={(event) => {
            event.preventDefault();
            updateAspect.mutate({ aspectId: selected.id, aspect: editDraft });
          }}
        >
          <Input
            className="lg:col-span-2"
            value={editDraft.guidelineTitle}
            onChange={(event) =>
              setEditDraft({ ...editDraft, guidelineTitle: event.target.value })
            }
            placeholder="Titulo del lineamiento"
          />
          <Select
            value={editDraft.factorCode}
            onValueChange={(value) => {
              const option = factorChoices.find((item) => item.code === value);
              setEditDraft({
                ...editDraft,
                factorCode: value as CnaFactorCode,
                factorName: option?.name ?? editDraft.factorName,
                characteristicCode: "",
                characteristicName: "",
              });
            }}
          >
            <SelectTrigger>
              <SelectValue placeholder="Factor" />
            </SelectTrigger>
            <SelectContent>
              {factorChoices.map((factor) => (
                <SelectItem key={factor.code} value={factor.code}>
                  {factor.code}. {factor.name}
                </SelectItem>
              ))}
            </SelectContent>
          </Select>
          <Select
            value={editDraft.characteristicCode}
            onValueChange={(value) => {
              const option = editCharacteristicOptions.find((item) => item.code === value);
              setEditDraft({
                ...editDraft,
                characteristicCode: value,
                characteristicName: option?.name ?? editDraft.characteristicName,
              });
            }}
          >
            <SelectTrigger>
              <SelectValue placeholder="Caracteristica" />
            </SelectTrigger>
            <SelectContent>
              {editCharacteristicOptions.map((option) => (
                <SelectItem key={option.code} value={option.code}>
                  {option.code}. {option.name}
                </SelectItem>
              ))}
            </SelectContent>
          </Select>
          <Textarea
            className="lg:col-span-2"
            value={editDraft.aspectDescription}
            onChange={(event) =>
              setEditDraft({ ...editDraft, aspectDescription: event.target.value })
            }
            placeholder="Descripcion del aspecto"
          />
          <Accordion type="single" collapsible className="lg:col-span-2">
            <AccordionItem value="save-impact">
              <AccordionTrigger>Impacto al guardar</AccordionTrigger>
              <AccordionContent className="text-xs leading-5 text-muted-foreground">
                Solo se marcaran preguntas como Modificar cuando cambie el factor, la
                caracteristica o el aspecto asociado. Guardar sin cambios en esa
                jerarquia no cambia el estado de las preguntas.
              </AccordionContent>
            </AccordionItem>
          </Accordion>
          <div className="flex flex-wrap gap-2 lg:col-span-2">
            <Button type="button" variant="outline" onClick={() => setEditDraft(null)}>
              <X className="size-4" />
              Cancelar
            </Button>
            <Button
              disabled={
                updateAspect.isPending ||
                !editDraft.factorCode.trim() ||
                !editDraft.characteristicCode.trim() ||
                !editDraft.aspectDescription.trim()
              }
            >
              <Save className="size-4" />
              Guardar cambios
            </Button>
          </div>
        </form>
      ) : (
        <LineamentDetails
          selected={selected}
          relatedQuestions={relatedQuestions}
          deleteAspect={deleteAspect}
          deleteText={deleteText}
          setDeleteText={setDeleteText}
          ackDeleteQuestions={ackDeleteQuestions}
          setAckDeleteQuestions={setAckDeleteQuestions}
          deleteDialogOpen={deleteDialogOpen}
          setDeleteDialogOpen={setDeleteDialogOpen}
          onEdit={() => setEditDraft(aspectToDraft(selected))}
        />
      )}
    </div>
  );
});

function LineamentDetails({
  selected,
  relatedQuestions,
  deleteAspect,
  deleteText,
  setDeleteText,
  ackDeleteQuestions,
  setAckDeleteQuestions,
  deleteDialogOpen,
  setDeleteDialogOpen,
  onEdit,
}: {
  selected: GuidelineAspect;
  relatedQuestions: Question[];
  deleteAspect: InlineLineamentPanelProps["deleteAspect"];
  deleteText: string;
  setDeleteText: (value: string) => void;
  ackDeleteQuestions: boolean;
  setAckDeleteQuestions: (value: boolean) => void;
  deleteDialogOpen: boolean;
  setDeleteDialogOpen: (value: boolean) => void;
  onEdit: () => void;
}) {
  return (
    <div className="space-y-4">
      <div>
        <div className="flex flex-wrap gap-2">
          <Button type="button" variant="outline" size="sm" onClick={onEdit}>
            <Pencil className="size-4" />
            Editar aqui
          </Button>
          <Button asChild variant="outline" size="sm">
            <a href={`/questions?lineamentId=${encodeURIComponent(selected.id)}`}>
              <ExternalLink className="size-3" />
              Ver preguntas
            </a>
          </Button>
          <Button asChild size="sm">
            <a href={`/questions?lineamentId=${encodeURIComponent(selected.id)}`}>
              <Plus className="size-3" />
              Nueva pregunta
            </a>
          </Button>
          <Dialog
            open={deleteDialogOpen}
            onOpenChange={(open) => {
              setDeleteDialogOpen(open);
              if (!open) {
                setDeleteText("");
                setAckDeleteQuestions(false);
              }
            }}
          >
            <DialogTrigger asChild>
              <Button
                type="button"
                variant="destructive"
                size="icon"
                aria-label="Eliminar lineamiento"
              >
                <Trash2 className="size-4" />
              </Button>
            </DialogTrigger>
            <DialogContent>
              <DialogHeader>
                <DialogTitle>Eliminar lineamiento</DialogTitle>
                <DialogDescription>
                  Esta accion tambien elimina las preguntas asociadas. Confirme
                  solo si ya reviso la cobertura.
                </DialogDescription>
              </DialogHeader>
              <div className="space-y-3">
                <div className="rounded-lg border bg-background/60 p-3 text-sm">
                  <p className="line-clamp-4">{selected.aspectDescription}</p>
                  <p className="mt-2 text-xs text-muted-foreground">
                    {relatedQuestions.length} preguntas relacionadas
                  </p>
                </div>
                <label className="flex gap-3 rounded-lg border bg-background/60 p-3 text-sm">
                  <input
                    type="checkbox"
                    checked={ackDeleteQuestions}
                    onChange={(event) => setAckDeleteQuestions(event.target.checked)}
                  />
                  Entiendo que tambien se borraran las preguntas asociadas.
                </label>
                <Input
                  value={deleteText}
                  onChange={(event) => setDeleteText(event.target.value)}
                  placeholder="Escriba ELIMINAR LINEAMIENTO"
                />
                <div className="grid grid-cols-2 gap-2">
                  <Button
                    type="button"
                    variant="outline"
                    onClick={() => setDeleteDialogOpen(false)}
                  >
                    Cancelar
                  </Button>
                  <Button
                    variant="destructive"
                    disabled={
                      deleteText.trim() !== "ELIMINAR LINEAMIENTO" ||
                      !ackDeleteQuestions ||
                      deleteAspect.isPending
                    }
                    onClick={() =>
                      deleteAspect.mutate({
                        aspectId: selected.id,
                        confirmationText: deleteText,
                        acknowledgeRelatedQuestions: ackDeleteQuestions,
                      })
                    }
                  >
                    <Trash2 className="size-4" />
                    Eliminar
                  </Button>
                </div>
              </div>
            </DialogContent>
          </Dialog>
        </div>
        <p className="mt-4 text-sm font-medium">
          Preguntas relacionadas ({relatedQuestions.length})
        </p>
        <div className="mt-2 max-h-72 space-y-2 overflow-auto">
          {relatedQuestions.map((question) => (
            <div key={question.id} className="rounded-lg border bg-background/70 p-3">
              <p className="text-xs font-semibold text-muted-foreground">
                {question.code}
              </p>
              <p className="mt-1 line-clamp-3 text-sm">{question.text}</p>
            </div>
          ))}
          {relatedQuestions.length === 0 ? (
            <p className="rounded-lg border bg-background/70 p-3 text-sm text-muted-foreground">
              No hay preguntas asociadas.
            </p>
          ) : null}
        </div>
      </div>
    </div>
  );
}
