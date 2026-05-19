import { Layers3 } from "lucide-react";
import { memo } from "react";

export const LineamentsHero = memo(function LineamentsHero() {
  return (
    <section className="rounded-lg border bg-card/78 p-6 shadow-sm shadow-black/5 backdrop-blur-xl md:p-8">
      <div className="flex items-start gap-4">
        <div className="flex size-11 shrink-0 items-center justify-center rounded-lg bg-primary/10 text-primary">
          <Layers3 className="size-5" />
        </div>
        <div>
          <h1 className="text-3xl font-semibold">Lineamientos CNA</h1>
          <p className="mt-2 max-w-3xl text-sm leading-6 text-muted-foreground">
            Registre la estructura Factor, Caracteristica y Aspecto que justifica
            el banco de preguntas y permite validar cobertura.
          </p>
        </div>
      </div>
    </section>
  );
});
