import { FileUp } from "lucide-react";

import { Button } from "@/components/ui/button";
import {
  Dialog,
  DialogContent,
  DialogDescription,
  DialogHeader,
  DialogTitle,
} from "@/components/ui/dialog";

type EmptyWorkspaceImportDialogProps = {
  open: boolean;
  importing: boolean;
  error: boolean;
  onImport: () => void;
  onSkip: () => void;
  onNeverShow: () => void;
};

export function EmptyWorkspaceImportDialog({
  open,
  importing,
  error,
  onImport,
  onSkip,
  onNeverShow,
}: EmptyWorkspaceImportDialogProps) {
  return (
    <Dialog open={open} onOpenChange={(next) => !next && onSkip()}>
      <DialogContent>
        <DialogHeader>
          <DialogTitle>Importar consolidado inicial</DialogTitle>
          <DialogDescription>
            La base esta vacia. Importe el consolidado Excel para extraer preguntas,
            factores, caracteristicas y lineamientos desde todas las hojas compatibles.
          </DialogDescription>
        </DialogHeader>
        <div className="space-y-4">
          <div className="apple-tile p-3 text-sm leading-6 text-muted-foreground">
            La app no crea factores por defecto. La lista se construye con el Excel
            importado o con los lineamientos que agregue manualmente despues.
          </div>
          <div className="grid gap-2 sm:grid-cols-3">
            <Button
              className="glass-action border-0 sm:col-span-2"
              disabled={importing}
              onClick={onImport}
            >
              <FileUp className="size-4" />
              Importar Excel
            </Button>
            <Button variant="outline" disabled={importing} onClick={onSkip}>
              Omitir
            </Button>
          </div>
          <Button
            variant="ghost"
            className="w-full text-muted-foreground"
            disabled={importing}
            onClick={onNeverShow}
          >
            No mostrar de nuevo
          </Button>
          {error ? (
            <p className="text-sm text-destructive">
              No se pudo importar el consolidado. Verifique que sea un archivo Excel
              con hojas BASE/BASEvs o columnas de consolidado.
            </p>
          ) : null}
        </div>
      </DialogContent>
    </Dialog>
  );
}
