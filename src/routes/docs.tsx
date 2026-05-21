import { createFileRoute } from "@tanstack/react-router";
import type React from "react";

import {
  Card,
  CardContent,
  CardDescription,
  CardHeader,
  CardTitle,
} from "@/components/ui/card";

export const Route = createFileRoute("/docs")({
  component: DocsPage,
});

const methodology = [
  {
    title: "1. Configurar responsable",
    text: "Al abrir la app por primera vez, registre el nombre completo del editor. Toda importacion, alta manual y ajuste queda asociado a ese nombre en la bitacora local.",
  },
  {
    title: "2. Elegir fuente de base",
    text: "En Configuración, decida si trabajara con una base local, una base existente, un paquete .acna, una carpeta OneDrive o Microsoft Graph como respaldo. La colaboración Turso viene configurada desde el build.",
  },
  {
    title: "3. Cargar o revisar lineamientos",
    text: "Primero revise Lineamientos. Los Excel consolidados traen Factor, Caracteristica y Aspecto en columnas A-G. Al importar, la app extrae esos aspectos automaticamente, omite filas marcadas en rojo y permite agregar factores o caracteristicas desde modales compactos.",
  },
  {
    title: "4. Importar banco actual",
    text: "Use el consolidado BASE/BASEvs como fuente. Cada fila representa una asignacion pregunta-publico; la app agrupa por numero de pregunta, evita duplicados, guarda los subpublicos asociados y conserva las preguntas eliminadas como estado Eliminar.",
  },
  {
    title: "5. Fijar original",
    text: "Cuando el consolidado importado sea la fuente oficial del ciclo, fijelo como original desde el Panel. La app pide confirmacion reforzada, crea respaldo y guarda una copia inmutable para comparar cambios posteriores.",
  },
  {
    title: "6. Actualizar preguntas",
    text: "En Banco, agregue preguntas nuevas para aspectos no cubiertos, marque cambios como agregar/modificar/eliminar y documente la justificacion cuando aplique.",
  },
  {
    title: "7. Validar antes de exportar",
    text: "Ejecute Validacion para detectar preguntas sin publico, preguntas cerradas sin convencion, eliminaciones sin justificacion y enlaces pendientes.",
  },
  {
    title: "8. Exportar instrumentos",
    text: "Genere instrumentos por publico o el consolidado completo. Las exportaciones deben conservar el formato de muestra y marcar eliminadas en rojo, modificadas en azul y agregadas en verde.",
  },
  {
    title: "9. Preparar proveedor",
    text: "Registre entregas y enlaces del proveedor. Valide que cada subpublico reciba las preguntas correctas antes de distribuir.",
  },
];

const guideSections = [
  {
    title: "Inicio y base vacia",
    goal: "Registrar responsable, elegir sincronizacion y cargar el primer consolidado.",
    steps: [
      "Abra la app y registre el nombre completo del editor.",
      "Si aparece el aviso de base vacia, seleccione Importar Excel.",
      "Si necesita preparar OneDrive primero, omita el aviso y vaya a Configuración.",
    ],
    shot: "dashboard",
  },
  {
    title: "Configuración e importacion",
    goal: "Conectar carpeta, abrir una base existente o importar el consolidado CNA.",
    steps: [
      "Seleccione carpeta local sincronizada, base existente o Microsoft Graph.",
      "Importe el consolidado. La app lee todas las hojas compatibles.",
      "Revise el resumen: preguntas importadas, aspectos detectados y filas omitidas.",
    ],
    shot: "workspace",
  },
  {
    title: "Lineamientos",
    goal: "Revisar factores, caracteristicas y aspectos extraidos del Excel.",
    steps: [
      "Busque por factor, caracteristica o aspecto.",
      "Seleccione un lineamiento para ver preguntas relacionadas.",
      "Use el boton + junto a cada selector solo si debe crear un factor o caracteristica nueva.",
    ],
    shot: "lineaments",
  },
  {
    title: "Banco y proveedor",
    goal: "Actualizar preguntas y preparar la revision del proveedor.",
    steps: [
      "Filtre por lineamiento para revisar cobertura.",
      "Marque preguntas como mantener, modificar, agregar o eliminar.",
      "En Proveedor, marque cada pregunta como correcta, requiere modificacion o faltante.",
    ],
    shot: "questions",
  },
  {
    title: "Validacion, historial y exportacion",
    goal: "Cerrar inconsistencias y generar entregables trazables.",
    steps: [
      "Ejecute Validacion antes de exportar.",
      "Use Historial si necesita recuperar un estado anterior.",
      "Exporte el consolidado: rojo eliminadas, azul modificadas, verde agregadas.",
    ],
    shot: "exports",
  },
];

function DocsPage() {
  return (
    <div className="space-y-6">
      <section className="apple-hero p-6 md:p-8">
        <h1 className="text-3xl font-semibold md:text-4xl">Metodologia de trabajo</h1>
        <p className="mt-3 max-w-3xl text-sm leading-6 text-muted-foreground">
          La app convierte los Excel operativos en salidas controladas: la base
          libSQL es la fuente de verdad, los lineamientos justifican el banco y
          las validaciones evitan enviar instrumentos incompletos.
        </p>
      </section>

      <section className="grid gap-4 lg:grid-cols-[0.9fr_1.1fr]">
        <Card>
          <CardHeader>
            <CardTitle>Estructura de los Excel</CardTitle>
            <CardDescription>Como se relacionan las hojas actuales.</CardDescription>
          </CardHeader>
          <CardContent className="space-y-3 text-sm leading-6">
            <Relation title="Consolidado BASE / BASEvs">
              Fuente principal. Columnas A-G guardan Factor, Caracteristica y
              Aspecto; columnas H-O guardan tipo, estado, convencion, pregunta,
              publico, subpublico y observaciones. Al importarlo se deduplican
              preguntas por codigo y lineamientos por jerarquia CNA. Las filas o
              celdas estructurales en rojo se interpretan como removidas.
            </Relation>
            <Relation title="Por lineamiento">
              Vista de instrumento por jerarquia CNA. Repite la estructura de
              lineamiento y despliega columnas por subpublico.
            </Relation>
            <Relation title="Por orden">
              Vista de entrega por secuencia de preguntas. Sirve para revisar
              orden de aparicion y textos por subpublico.
            </Relation>
            <Relation title="Convencion">
              Tabla de escalas de respuesta. En Excel se codifica como A-J para
              escribir rapido, pero en la app se elige por significado: acuerdo,
              cantidad, calidad, frecuencia, nivel, exigencia, favorecimiento,
              probabilidad, satisfaccion o medida. El codigo se conserva solo
              para importar y exportar el formato original.
            </Relation>
          </CardContent>
        </Card>

        <div className="grid gap-3">
          {methodology.map((item) => (
            <Card key={item.title}>
              <CardContent className="p-4">
                <h2 className="font-semibold">{item.title}</h2>
                <p className="mt-2 text-sm leading-6 text-muted-foreground">{item.text}</p>
              </CardContent>
            </Card>
          ))}
        </div>
      </section>

      <Card>
        <CardHeader>
          <CardTitle>Guia por pantallas</CardTitle>
          <CardDescription>
            Flujo de uso con capturas representativas de la plataforma.
          </CardDescription>
        </CardHeader>
        <CardContent className="space-y-4">
          {guideSections.map((section) => (
            <GuideSection key={section.title} {...section} />
          ))}
        </CardContent>
      </Card>

      <Card>
        <CardHeader>
          <CardTitle>Línea base y exportaciones</CardTitle>
          <CardDescription>
            Reglas adicionales para trazabilidad contra el Excel original.
          </CardDescription>
        </CardHeader>
        <CardContent className="grid gap-3 text-sm leading-6 lg:grid-cols-3">
          <Relation title="Original protegido">
            Un consolidado puede marcarse como original del ciclo desde el Panel,
            solo con confirmacion reforzada, conteos visibles, hash del archivo y
            respaldo previo para evitar cambios accidentales.
          </Relation>
          <Relation title="Comparacion de cambios">
            Cada pregunta se compara contra la copia original: sin cambio,
            modificada, agregada o eliminada. Esa diferencia alimenta validaciones
            y reportes.
          </Relation>
          <Relation title="Colores de Excel">
            Las exportaciones deben marcar eliminadas en rojo, modificadas en
            azul y agregadas en verde, tanto en instrumentos por publico como en
            consolidado completo.
          </Relation>
        </CardContent>
      </Card>
    </div>
  );
}

function GuideSection({
  title,
  goal,
  steps,
  shot,
}: {
  title: string;
  goal: string;
  steps: string[];
  shot: string;
}) {
  return (
    <section className="grid gap-4 rounded-lg border bg-background/55 p-4 lg:grid-cols-[0.9fr_1.1fr]">
      <div>
        <h2 className="font-semibold">{title}</h2>
        <p className="mt-2 text-sm leading-6 text-muted-foreground">{goal}</p>
        <div className="mt-4 space-y-2">
          {steps.map((step, index) => (
            <div key={step} className="flex gap-3 text-sm leading-6">
              <span className="workflow-pill flex size-7 shrink-0 items-center justify-center text-xs font-semibold text-primary">
                {index + 1}
              </span>
              <span>{step}</span>
            </div>
          ))}
        </div>
      </div>
      <PlatformShot type={shot} />
    </section>
  );
}

function Relation({ title, children }: { title: string; children: React.ReactNode }) {
  return (
    <div className="rounded-lg border bg-background/70 p-3">
      <p className="font-medium">{title}</p>
      <p className="mt-1 text-muted-foreground">{children}</p>
    </div>
  );
}

function PlatformShot({ type }: { type: string }) {
  return (
    <div className="apple-tile overflow-hidden p-0">
      <div className="flex items-center gap-1 border-b bg-card/80 px-3 py-2">
        <span className="size-2 rounded-full bg-destructive/70" />
        <span className="size-2 rounded-full bg-secondary/70" />
        <span className="size-2 rounded-full bg-primary/70" />
        <span className="ml-3 text-xs text-muted-foreground">Autoevaluacion CNA</span>
      </div>
      <div className="grid min-h-52 grid-cols-[6.5rem_1fr] bg-card/70">
        <div className="space-y-2 border-r p-3">
          {["Panel", "Configuración", "Lineamientos", "Banco", "Exportar"].map((item) => (
            <div
              key={item}
              className={`h-6 rounded px-2 text-[10px] leading-6 ${
                activeNav(type, item) ? "bg-primary/15 text-primary" : "bg-muted/55"
              }`}
            >
              {item}
            </div>
          ))}
        </div>
        <div className="p-3">{shotContent(type)}</div>
      </div>
    </div>
  );
}

function activeNav(type: string, item: string) {
  return (
    (type === "dashboard" && item === "Panel") ||
    (type === "workspace" && item === "Configuración") ||
    (type === "lineaments" && item === "Lineamientos") ||
    (type === "questions" && item === "Banco") ||
    (type === "exports" && item === "Exportar")
  );
}

function shotContent(type: string) {
  if (type === "workspace") {
    return (
      <div className="space-y-3">
        <div className="h-8 rounded bg-primary/15" />
        <div className="grid grid-cols-3 gap-2">
          <div className="h-16 rounded bg-muted" />
          <div className="h-16 rounded bg-muted" />
          <div className="h-16 rounded bg-muted" />
        </div>
        <div className="h-20 rounded border bg-background/70" />
      </div>
    );
  }
  if (type === "lineaments") {
    return (
      <div className="grid grid-cols-[1fr_9rem] gap-3">
        <div className="space-y-2">
          <div className="h-7 rounded bg-muted" />
          <div className="h-12 rounded border bg-background/70" />
          <div className="h-12 rounded border bg-background/70" />
          <div className="h-12 rounded border bg-background/70" />
        </div>
        <div className="space-y-2">
          <div className="h-8 rounded bg-card" />
          <div className="h-8 rounded bg-primary/15" />
          <div className="h-8 rounded bg-muted" />
          <div className="h-16 rounded bg-muted" />
        </div>
      </div>
    );
  }
  if (type === "questions") {
    return (
      <div className="space-y-2">
        {[0, 1, 2, 3].map((row) => (
          <div key={row} className="grid grid-cols-[0.35fr_1fr_0.35fr] gap-2">
            <div className="h-7 rounded bg-primary/15" />
            <div className="h-7 rounded bg-muted" />
            <div className="h-7 rounded bg-accent/20" />
          </div>
        ))}
        <div className="h-14 rounded border bg-background/70" />
      </div>
    );
  }
  if (type === "exports") {
    return (
      <div className="space-y-3">
        <div className="grid grid-cols-4 gap-2">
          <div className="h-12 rounded bg-muted" />
          <div className="h-12 rounded bg-muted" />
          <div className="h-12 rounded bg-primary/15" />
          <div className="h-12 rounded bg-secondary/20" />
        </div>
        <div className="h-24 rounded border bg-background/70" />
        <div className="grid grid-cols-3 gap-2">
          <div className="h-5 rounded bg-destructive/25" />
          <div className="h-5 rounded bg-primary/20" />
          <div className="h-5 rounded bg-secondary/20" />
        </div>
      </div>
    );
  }
  return (
    <div className="space-y-3">
      <div className="h-10 rounded bg-primary/15" />
      <div className="grid grid-cols-4 gap-2">
        <div className="h-14 rounded bg-muted" />
        <div className="h-14 rounded bg-muted" />
        <div className="h-14 rounded bg-muted" />
        <div className="h-14 rounded bg-muted" />
      </div>
      <div className="h-20 rounded border bg-background/70" />
    </div>
  );
}
