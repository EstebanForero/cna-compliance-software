import { ChevronLeft, ChevronRight } from "lucide-react";

import { Button } from "@/components/ui/button";

type PaginationProps = {
  page: number;
  pageCount: number;
  totalItems: number;
  pageSize: number;
  onPageChange: (page: number) => void;
};

export function Pagination({
  page,
  pageCount,
  totalItems,
  pageSize,
  onPageChange,
}: PaginationProps) {
  const safePageCount = Math.max(pageCount, 1);
  const firstItem = totalItems === 0 ? 0 : (page - 1) * pageSize + 1;
  const lastItem = Math.min(page * pageSize, totalItems);

  return (
    <div className="flex flex-col gap-3 border-t pt-4 text-sm text-muted-foreground sm:flex-row sm:items-center sm:justify-between">
      <p>
        {firstItem}-{lastItem} de {totalItems}
      </p>
      <div className="flex items-center gap-2">
        <Button
          type="button"
          variant="outline"
          size="sm"
          disabled={page <= 1}
          onClick={() => onPageChange(Math.max(page - 1, 1))}
        >
          <ChevronLeft className="size-4" />
          Anterior
        </Button>
        <span className="min-w-24 text-center text-xs">
          Pagina {page} de {safePageCount}
        </span>
        <Button
          type="button"
          variant="outline"
          size="sm"
          disabled={page >= safePageCount}
          onClick={() => onPageChange(Math.min(page + 1, safePageCount))}
        >
          Siguiente
          <ChevronRight className="size-4" />
        </Button>
      </div>
    </div>
  );
}
