import { Badge } from "@/components/ui/badge";
import type { QuestionStatus } from "@/lib/types";

export function StatusBadge({ status }: { status: QuestionStatus }) {
  const variant = {
    keep: "secondary",
    modify: "warning",
    add: "default",
    delete: "destructive",
  }[status] as "secondary" | "warning" | "default" | "destructive";

  const label = {
    keep: "Mantener",
    modify: "Modificar",
    add: "Agregar",
    delete: "Eliminar",
  }[status];

  return <Badge variant={variant}>{label}</Badge>;
}
