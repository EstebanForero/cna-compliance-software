import { open, save } from "@tauri-apps/plugin-dialog";
import { useMutation, useQuery, useQueryClient } from "@tanstack/react-query";
import { createFileRoute } from "@tanstack/react-router";
import { Clipboard, FileText, RotateCcw, Search } from "lucide-react";
import { useEffect, useMemo, useState, type ClipboardEvent } from "react";

import { Badge } from "@/components/ui/badge";
import { Button } from "@/components/ui/button";
import { Card, CardContent, CardDescription, CardHeader, CardTitle } from "@/components/ui/card";
import {
  Dialog,
  DialogContent,
  DialogDescription,
  DialogHeader,
  DialogTitle,
  DialogTrigger,
} from "@/components/ui/dialog";
import { Input } from "@/components/ui/input";
import { Pagination } from "@/components/ui/pagination";
import {
  Select,
  SelectContent,
  SelectItem,
  SelectTrigger,
  SelectValue,
} from "@/components/ui/select";
import { Textarea } from "@/components/ui/textarea";
import { api } from "@/lib/api";
import type { ProviderQuestionReviewItem, ProviderQuestionReviewStatus } from "@/lib/types";

export const Route = createFileRoute("/provider")({
  component: ProviderPage,
});

const providerPageSize = 10;
const resetConfirmation = "REINICIAR REVISION";
type ReviewFilter = "pending" | "all";
type InstrumentStats = {
  instrument: string;
  total: number;
  pending: number;
  approved: number;
  issues: number;
};

function ProviderPage() {
  const queryClient = useQueryClient();
  const [search, setSearch] = useState("");
  const [selectedInstrument, setSelectedInstrument] = useState("");
  const [reviewFilter, setReviewFilter] = useState<ReviewFilter>("pending");
  const [selected, setSelected] = useState<ProviderQuestionReviewItem | null>(null);
  const [status, setStatus] = useState<ProviderQuestionReviewStatus>("correct");
  const [observation, setObservation] = useState("");
  const [evidencePath, setEvidencePath] = useState("");
  const [notice, setNotice] = useState("");
  const [page, setPage] = useState(1);
  const [resetOpen, setResetOpen] = useState(false);
  const [resetInput, setResetInput] = useState("");
  const items = useQuery({
    queryKey: ["provider-question-review-items"],
    queryFn: api.providerQuestionReviewItems,
  });
  const saveReview = useMutation({
    mutationFn: api.saveProviderQuestionReview,
    onSuccess: async () => {
      setNotice("Revision guardada.");
      await queryClient.invalidateQueries({ queryKey: ["provider-question-review-items"] });
      if (reviewFilter === "pending") setSelected(null);
    },
  });
  const resetReviews = useMutation({
    mutationFn: api.resetProviderQuestionReviews,
    onSuccess: async (result) => {
      setNotice(`Revision reiniciada. ${result.deletedReviews} marcas eliminadas.`);
      setSelected(null);
      setResetInput("");
      setResetOpen(false);
      await queryClient.invalidateQueries({ queryKey: ["provider-question-review-items"] });
    },
  });
  const saveEvidence = useMutation({
    mutationFn: api.saveEvidenceAttachment,
    onSuccess: (result) => {
      setEvidencePath(result.path);
      setNotice("Evidencia pegada y adjuntada.");
    },
  });
  const exportDocx = useMutation({ mutationFn: api.exportProviderReviewDocx });
  const instruments = useMemo(() => {
    const values = (items.data ?? []).map((item) => item.instrumentAudience);
    return Array.from(new Set(values)).sort((left, right) =>
      left.localeCompare(right, "es", { sensitivity: "base" }),
    );
  }, [items.data]);
  const instrumentStats = useMemo(() => {
    const stats = new Map<string, InstrumentStats>();
    for (const item of items.data ?? []) {
      const current =
        stats.get(item.instrumentAudience) ??
        {
          instrument: item.instrumentAudience,
          total: 0,
          pending: 0,
          approved: 0,
          issues: 0,
        };
      current.total += 1;
      const itemStatus = item.review?.status ?? "pending";
      if (itemStatus === "pending") current.pending += 1;
      else if (itemStatus === "correct") current.approved += 1;
      else current.issues += 1;
      stats.set(item.instrumentAudience, current);
    }
    return Array.from(stats.values()).sort((left, right) =>
      left.instrument.localeCompare(right.instrument, "es", { sensitivity: "base" }),
    );
  }, [items.data]);

  useEffect(() => {
    if (!selectedInstrument && instruments.length > 0) {
      setSelectedInstrument(instruments[0]);
      return;
    }
    if (selectedInstrument && instruments.length > 0 && !instruments.includes(selectedInstrument)) {
      setSelectedInstrument(instruments[0]);
    }
  }, [instruments, selectedInstrument]);

  const filtered = useMemo(() => {
    const needle = search.trim().toLowerCase();
    return (items.data ?? []).filter((item) => {
      if (selectedInstrument && item.instrumentAudience !== selectedInstrument) return false;
      const status = item.review?.status ?? "pending";
      if (reviewFilter === "pending" && status !== "pending") return false;
      if (!needle) return true;
      return [
        item.question.code,
        item.question.text,
        item.question.factor,
        item.question.aspect,
        item.instrumentAudience,
      ]
        .join(" ")
        .toLowerCase()
        .includes(needle);
    });
  }, [items.data, reviewFilter, search, selectedInstrument]);
  const selectedInstrumentItems = useMemo(
    () =>
      (items.data ?? []).filter((item) =>
        selectedInstrument ? item.instrumentAudience === selectedInstrument : true,
      ),
    [items.data, selectedInstrument],
  );
  const reviewedCount = useMemo(
    () =>
      selectedInstrumentItems.filter((item) => (item.review?.status ?? "pending") !== "pending")
        .length,
    [selectedInstrumentItems],
  );
  const pendingCount = Math.max(selectedInstrumentItems.length - reviewedCount, 0);
  const pageCount = Math.max(Math.ceil(filtered.length / providerPageSize), 1);
  const paginatedItems = useMemo(() => {
    const start = (page - 1) * providerPageSize;
    return filtered.slice(start, start + providerPageSize);
  }, [filtered, page]);

  useEffect(() => {
    setPage(1);
  }, [reviewFilter, search, selectedInstrument]);

  useEffect(() => {
    if (page > pageCount) setPage(pageCount);
  }, [page, pageCount]);

  function selectItem(item: ProviderQuestionReviewItem) {
    setSelected(item);
    setStatus(item.review?.status ?? "correct");
    setObservation(item.review?.observation ?? "");
    setEvidencePath(item.review?.evidencePath ?? "");
  }

  function chooseInstrument(instrument: string) {
    setSelectedInstrument(instrument);
    setSelected(null);
  }

  async function chooseEvidence() {
    const path = await open({
      directory: false,
      multiple: false,
      filters: [{ name: "Evidence", extensions: ["png", "jpg", "jpeg", "pdf"] }],
    });
    if (typeof path === "string") setEvidencePath(path);
  }

  function handleEvidencePaste(event: ClipboardEvent<HTMLTextAreaElement>) {
    if (!selected) return;
    const imageItem = Array.from(event.clipboardData.items).find((item) =>
      item.type.startsWith("image/"),
    );
    const file = imageItem?.getAsFile();
    if (!file) return;
    event.preventDefault();
    const reader = new FileReader();
    reader.onload = () => {
      if (typeof reader.result !== "string") return;
      saveEvidence.mutate({
        questionId: selected.question.id,
        fileName: file.name || "captura.png",
        dataUrl: reader.result,
      });
    };
    reader.readAsDataURL(file);
  }

  async function chooseDocxPath() {
    const path = await save({
      defaultPath: selectedInstrument
        ? `revision-proveedor-${selectedInstrument}.docx`
        : "revision-proveedor.docx",
      filters: [{ name: "Word document", extensions: ["docx"] }],
    });
    if (path) exportDocx.mutate({ path, instrumentAudience: selectedInstrument || null });
  }

  return (
    <div className="space-y-6">
      <section className="apple-hero p-6 md:p-8">
        <h1 className="text-3xl font-semibold md:text-4xl">Revision del proveedor</h1>
        <p className="mt-3 max-w-3xl text-sm leading-6 text-muted-foreground">
          Busque preguntas en los enlaces del proveedor, marque si estan correctas,
          requieren modificacion o no aparecen, y genere el documento Word de revision.
        </p>
      </section>
      {notice ? (
        <div className="rounded-lg border border-primary/20 bg-primary/10 p-3 text-sm text-primary">
          {notice}
        </div>
      ) : null}

      <section className="grid gap-3 md:grid-cols-2 xl:grid-cols-4">
        {instrumentStats.map((instrument) => {
          const completed = instrument.total - instrument.pending;
          const progress =
            instrument.total > 0 ? Math.round((completed / instrument.total) * 100) : 0;
          const active = selectedInstrument === instrument.instrument;
          return (
            <button
              key={instrument.instrument}
              type="button"
              onClick={() => chooseInstrument(instrument.instrument)}
              className={`rounded-lg border bg-card/70 p-4 text-left shadow-sm transition-all hover:border-primary/35 hover:shadow-md ${
                active ? "border-primary/45 ring-2 ring-ring" : ""
              }`}
            >
              <div className="flex items-start justify-between gap-3">
                <p className="min-w-0 break-words text-sm font-semibold leading-5">
                  {instrument.instrument}
                </p>
                <Badge variant={instrument.issues ? "warning" : "secondary"}>{progress}%</Badge>
              </div>
              <div className="mt-3 h-2 overflow-hidden rounded-full bg-muted">
                <div className="h-full rounded-full bg-primary" style={{ width: `${progress}%` }} />
              </div>
              <div className="mt-3 grid grid-cols-3 gap-2 text-xs text-muted-foreground">
                <span>{instrument.pending} pendientes</span>
                <span>{instrument.approved} OK</span>
                <span>{instrument.issues} alertas</span>
              </div>
            </button>
          );
        })}
      </section>

      <section className="grid gap-4 xl:grid-cols-[1fr_26rem]">
        <Card>
          <CardHeader className="gap-3 md:flex-row md:items-center md:justify-between md:space-y-0">
            <div>
              <CardTitle>Checklist de preguntas</CardTitle>
              <CardDescription>
                {selectedInstrument || "Seleccione un instrumento"} · {pendingCount} pendientes ·{" "}
                {reviewedCount} revisadas
              </CardDescription>
            </div>
            <div className="flex w-full flex-col gap-2 md:w-auto md:flex-row md:items-center">
              <Select value={selectedInstrument} onValueChange={chooseInstrument}>
                <SelectTrigger className="md:w-64">
                  <SelectValue placeholder="Instrumento" />
                </SelectTrigger>
                <SelectContent>
                  {instruments.map((instrument) => (
                    <SelectItem key={instrument} value={instrument}>
                      {instrument}
                    </SelectItem>
                  ))}
                </SelectContent>
              </Select>
              <div className="grid grid-cols-2 rounded-lg border bg-background/60 p-1 shadow-sm md:w-52">
                {(["pending", "all"] as const).map((value) => (
                  <Button
                    key={value}
                    size="sm"
                    variant={reviewFilter === value ? "default" : "ghost"}
                    onClick={() => setReviewFilter(value)}
                  >
                    {value === "pending" ? "Pendientes" : "Todas"}
                  </Button>
                ))}
              </div>
              <div className="relative w-full md:w-80">
                <Search className="pointer-events-none absolute left-3 top-2.5 size-4 text-muted-foreground" />
                <Input
                  className="pl-9"
                  value={search}
                  onChange={(event) => setSearch(event.target.value)}
                  placeholder="Buscar pregunta"
                />
              </div>
              <Dialog
                open={resetOpen}
                onOpenChange={(open) => {
                  setResetOpen(open);
                  if (!open) setResetInput("");
                }}
              >
                <DialogTrigger asChild>
                  <Button variant="outline" type="button" disabled={reviewedCount === 0}>
                    <RotateCcw className="size-4" />
                    Reiniciar
                  </Button>
                </DialogTrigger>
                <DialogContent>
                  <DialogHeader>
                    <DialogTitle>Reiniciar revision del proveedor</DialogTitle>
                    <DialogDescription>
                      Esto elimina las marcas, observaciones y rutas de evidencia de todas las
                      preguntas revisadas. No elimina las preguntas ni los archivos adjuntos.
                    </DialogDescription>
                  </DialogHeader>
                  <div className="space-y-3">
                    <p className="text-sm text-muted-foreground">
                      Escriba <span className="font-medium text-foreground">{resetConfirmation}</span>{" "}
                      para confirmar.
                    </p>
                    <Input
                      value={resetInput}
                      onChange={(event) => setResetInput(event.target.value)}
                      placeholder={resetConfirmation}
                    />
                    <div className="flex justify-end gap-2">
                      <Button variant="ghost" type="button" onClick={() => setResetOpen(false)}>
                        Cancelar
                      </Button>
                      <Button
                        variant="destructive"
                        type="button"
                  disabled={
                          resetInput.trim() !== resetConfirmation || resetReviews.isPending
                        }
                        onClick={() =>
                          resetReviews.mutate({ confirmationText: resetInput })
                        }
                      >
                        Reiniciar revision
                      </Button>
                    </div>
                  </div>
                </DialogContent>
              </Dialog>
            </div>
          </CardHeader>
          <CardContent className="space-y-2">
            {paginatedItems.length ? (
              paginatedItems.map((item) => (
                <button
                  key={item.question.id}
                  type="button"
                  onClick={() => selectItem(item)}
                  className="w-full rounded-lg border bg-background/60 p-3 text-left hover:border-primary/35"
                >
                  <div className="flex items-start justify-between gap-3">
                    <div>
                      <p className="text-xs font-semibold text-muted-foreground">
                        {item.question.code}
                      </p>
                      <p className="mt-1 line-clamp-2 text-sm">{item.question.text}</p>
                    </div>
                    <ReviewBadge status={item.review?.status ?? "pending"} />
                  </div>
                </button>
              ))
            ) : (
              <div className="rounded-lg border border-dashed bg-background/50 p-8 text-center text-sm text-muted-foreground">
                {reviewFilter === "pending"
                  ? "No quedan preguntas pendientes para este instrumento."
                  : "No hay preguntas que coincidan con la busqueda."}
              </div>
            )}
            <Pagination
              page={page}
              pageCount={pageCount}
              totalItems={filtered.length}
              pageSize={providerPageSize}
              onPageChange={setPage}
            />
          </CardContent>
        </Card>

        <Card>
          <CardHeader>
            <CardTitle>{selected ? "Marcar pregunta" : "Documento de revision"}</CardTitle>
            <CardDescription>
              {selected
                ? `${selected.instrumentAudience} · ${selected.question.code}`
                : "Exporta el estado completo a Word."}
            </CardDescription>
          </CardHeader>
          <CardContent className="space-y-3">
            {selected ? (
              <>
                <p className="text-sm leading-6">{selected.question.text}</p>
                <div className="grid grid-cols-3 gap-2">
                  {(["correct", "needsModification", "missing"] as const).map((value) => (
                    <Button
                      key={value}
                      type="button"
                      variant={status === value ? "default" : "outline"}
                      onClick={() => setStatus(value)}
                    >
                      {label(value)}
                    </Button>
                  ))}
                </div>
                <Textarea
                  value={observation}
                  onChange={(event) => setObservation(event.target.value)}
                  onPaste={handleEvidencePaste}
                  placeholder="Observacion"
                />
                <div className="rounded-lg border bg-background/55 p-3 text-xs leading-5 text-muted-foreground">
                  <div className="flex items-start gap-2">
                    <Clipboard className="mt-0.5 size-4 shrink-0" />
                    <p>
                      Puede pegar una captura directamente en la observacion. La app la guarda
                      como evidencia y la asocia a esta pregunta.
                    </p>
                  </div>
                </div>
                <Button
                  variant="outline"
                  type="button"
                  onClick={chooseEvidence}
                  disabled={saveEvidence.isPending}
                >
                  Adjuntar evidencia
                </Button>
                {evidencePath ? (
                  <p className="break-words text-xs text-muted-foreground">{evidencePath}</p>
                ) : null}
                <Button
                  className="w-full"
                  disabled={saveReview.isPending}
                  onClick={() =>
                    saveReview.mutate({
                      questionId: selected.question.id,
                      instrumentAudience: selected.instrumentAudience,
                      status,
                      observation,
                      evidencePath: evidencePath || null,
                    })
                  }
                >
                  Guardar revision
                </Button>
              </>
            ) : (
              <>
                <Button
                  onClick={chooseDocxPath}
                  disabled={exportDocx.isPending || !selectedInstrument}
                >
                  <FileText className="size-4" />
                  Exportar Word del instrumento
                </Button>
                {exportDocx.isSuccess ? (
                  <p className="text-sm text-muted-foreground">Documento generado.</p>
                ) : null}
              </>
            )}
          </CardContent>
        </Card>
      </section>
    </div>
  );
}

function ReviewBadge({ status }: { status: ProviderQuestionReviewStatus }) {
  const variant =
    status === "correct" ? "secondary" : status === "pending" ? "outline" : "warning";
  return <Badge variant={variant}>{label(status)}</Badge>;
}

function label(status: ProviderQuestionReviewStatus) {
  return {
    pending: "Pendiente",
    correct: "OK",
    needsModification: "Modificar",
    missing: "No aparece",
  }[status];
}
