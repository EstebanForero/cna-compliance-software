import { open } from "@tauri-apps/plugin-dialog";
import { useMutation, useQuery, useQueryClient } from "@tanstack/react-query";
import { Link, createFileRoute } from "@tanstack/react-router";
import type React from "react";
import {
  AlertTriangle,
  CalendarDays,
  CheckCircle2,
  Clock3,
  FileUp,
  Layers3,
} from "lucide-react";
import { useEffect, useState } from "react";

import { BaselineCard } from "@/features/dashboard/BaselineCard";
import { EmptyWorkspaceImportDialog } from "@/features/dashboard/EmptyWorkspaceImportDialog";
import { InstrumentConfiguration } from "@/features/instruments/InstrumentConfiguration";
import { Badge } from "@/components/ui/badge";
import { Button } from "@/components/ui/button";
import {
  Card,
  CardContent,
  CardDescription,
  CardHeader,
  CardTitle,
} from "@/components/ui/card";
import { Progress } from "@/components/ui/progress";
import { api } from "@/lib/api";

export const Route = createFileRoute("/")({
  component: DashboardPage,
});

function DashboardPage() {
  const queryClient = useQueryClient();
  const [confirmationText, setConfirmationText] = useState("");
  const [acknowledgeReplacement, setAcknowledgeReplacement] = useState(false);
  const [acknowledgeBackup, setAcknowledgeBackup] = useState(false);
  const [checkedTursoBeforeImport, setCheckedTursoBeforeImport] = useState(false);
  const [emptyImportPromptOpen, setEmptyImportPromptOpen] = useState(() => {
    if (typeof window === "undefined") return false;
    return localStorage.getItem("autoeval.skipEmptyImportPrompt") !== "true";
  });
  const dashboard = useQuery({
    queryKey: ["dashboard"],
    queryFn: api.dashboard,
  });
  const baseline = useQuery({
    queryKey: ["baseline-status"],
    queryFn: api.baselineStatus,
  });
  const refreshTurso = useMutation({
    mutationFn: api.refreshTursoWorkspace,
    onSuccess: async (workspace) => {
      await queryClient.invalidateQueries({ queryKey: ["workspace"] });
      await queryClient.invalidateQueries({ queryKey: ["dashboard"] });
      if (workspace.hasQuestions) {
        setEmptyImportPromptOpen(false);
        await queryClient.invalidateQueries({ queryKey: ["questions"] });
        await queryClient.invalidateQueries({ queryKey: ["guideline-aspects"] });
      }
    },
    onSettled: () => setCheckedTursoBeforeImport(true),
  });
  const markBaseline = useMutation({
    mutationFn: api.markOriginalBaseline,
    onSuccess: async () => {
      setConfirmationText("");
      setAcknowledgeReplacement(false);
      setAcknowledgeBackup(false);
      await queryClient.invalidateQueries({ queryKey: ["baseline-status"] });
      await queryClient.invalidateQueries({ queryKey: ["dashboard"] });
    },
  });
  const importWorkbook = useMutation({
    mutationFn: api.importWorkbook,
    onSuccess: async () => {
      setEmptyImportPromptOpen(false);
      await queryClient.invalidateQueries({ queryKey: ["workspace"] });
      await queryClient.invalidateQueries({ queryKey: ["dashboard"] });
      await queryClient.invalidateQueries({ queryKey: ["questions"] });
      await queryClient.invalidateQueries({ queryKey: ["guideline-aspects"] });
      await queryClient.invalidateQueries({ queryKey: ["baseline-status"] });
    },
  });

  async function chooseInitialWorkbook() {
    const selected = await open({
      multiple: false,
      filters: [{ name: "Excel workbook", extensions: ["xlsx", "xlsm", "xls"] }],
    });
    if (typeof selected !== "string") return;
    importWorkbook.mutate({ path: selected, cycleName: "Autoevaluacion CNA" });
  }

  function skipEmptyPrompt(persist: boolean) {
    if (persist) {
      localStorage.setItem("autoeval.skipEmptyImportPrompt", "true");
    }
    setEmptyImportPromptOpen(false);
  }

  useEffect(() => {
    if (
      dashboard.data &&
      !dashboard.data.workspace.hasQuestions &&
      !checkedTursoBeforeImport &&
      !refreshTurso.isPending
    ) {
      refreshTurso.mutate();
    }
  }, [checkedTursoBeforeImport, dashboard.data, refreshTurso]);

  if (dashboard.isLoading) {
    return <div className="text-sm text-muted-foreground">Cargando panel...</div>;
  }

  if (dashboard.isError || !dashboard.data) {
    return (
      <div className="rounded-lg border bg-card p-4 text-sm text-destructive">
        No se pudo cargar el panel.
      </div>
    );
  }

  const data = dashboard.data;
  if (!data.workspace.hasQuestions && !checkedTursoBeforeImport) {
    return (
      <div className="space-y-6">
        <Hero
          title="Verificando base colaborativa"
          description="Antes de importar un consolidado, la app esta conectando con Turso para revisar si el banco ya existe y traer esos datos."
        />
        <div className="rounded-xl border bg-card p-5 text-sm text-muted-foreground shadow-sm">
          <div className="mb-3 h-2 w-full overflow-hidden rounded-sm bg-muted">
            <div className="h-full w-1/3 animate-pulse rounded-sm bg-primary" />
          </div>
          Validando datos remotos...
        </div>
      </div>
    );
  }
  if (!data.workspace.hasQuestions) {
    return (
      <div className="space-y-6">
        <EmptyWorkspaceImportDialog
          open={emptyImportPromptOpen}
          importing={importWorkbook.isPending}
          error={importWorkbook.isError}
          onImport={chooseInitialWorkbook}
          onSkip={() => skipEmptyPrompt(false)}
          onNeverShow={() => skipEmptyPrompt(true)}
        />
        <Hero
          title="Prepare el banco de autoevaluacion"
          description="Siga un flujo guiado para conectar Microsoft, importar lineamientos CNA y preguntas desde el consolidado Excel, validar cobertura y preparar la entrega."
        />
        <section className="grid gap-4 lg:grid-cols-3">
          <SetupStep
            step="1"
            title="Configuración"
            description="Conecte Microsoft y elija carpeta OneDrive o Graph app-folder."
            to="/workspace"
            icon={<FileUp />}
          />
          <SetupStep
            step="2"
            title="Revisar lineamientos"
            description="Confirme factores, caracteristicas y aspectos CNA detectados o agregue los nuevos."
            to="/lineaments"
            icon={<Layers3 />}
          />
          <SetupStep
            step="3"
            title="Importar Excel"
            description="Extraiga preguntas, publicos y lineamientos desde las hojas del consolidado actual."
            to="/workspace"
            icon={<CheckCircle2 />}
          />
        </section>
      </div>
    );
  }

  const changed =
    data.questionsByStatus.find((item) => item.status === "modify")?.count ?? 0;
  const added = data.questionsByStatus.find((item) => item.status === "add")?.count ?? 0;
  const kept = data.questionsByStatus.find((item) => item.status === "keep")?.count ?? 0;
  const progress =
    data.totalQuestions === 0 ? 0 : Math.round((kept / data.totalQuestions) * 100);

  return (
    <div className="space-y-6">
      <Hero
        title={`Ciclo ${data.activeCycle?.name ?? "sin configurar"}`}
        description="Control operativo para comparar lineamientos CNA, mantener el banco unico de preguntas, validar cobertura y preparar entregas al proveedor."
        action={<Badge variant="secondary">{cycleLabel(data.activeCycle?.status ?? "planning")}</Badge>}
      />

      <section className="grid gap-3 sm:grid-cols-2 xl:grid-cols-4">
        <MetricCard
          title="Preguntas"
          value={data.totalQuestions}
          detail={`${data.pendingChanges} con cambios pendientes`}
          icon={<CheckCircle2 />}
        />
        <MetricCard
          title="Validaciones"
          value={data.blockingValidations}
          detail="bloqueantes antes de exportar"
          icon={<AlertTriangle />}
        />
        <MetricCard
          title="Enlaces"
          value={data.providerLinksPending}
          detail="pendientes por revisar"
          icon={<Clock3 />}
        />
        <MetricCard
          title="Aplicacion"
          value={
            data.activeCycle
              ? new Date(data.activeCycle.applicationStartsOn).toLocaleDateString()
              : "Pendiente"
          }
          detail="inicio planeado"
          icon={<CalendarDays />}
        />
      </section>

      <section className="grid gap-4 lg:grid-cols-[1.15fr_0.85fr]">
        <Card>
          <CardHeader>
            <CardTitle>Estado del banco</CardTitle>
            <CardDescription>
              Distribucion de preguntas por decision operativa.
            </CardDescription>
          </CardHeader>
          <CardContent className="space-y-4">
            <Progress value={progress} />
            <div className="grid gap-3 sm:grid-cols-4">
              {data.questionsByStatus.map((item) => (
                <div key={item.status} className="rounded-lg border bg-background/70 p-3">
                  <p className="text-xs text-muted-foreground">{statusLabel(item.status)}</p>
                  <p className="mt-2 text-2xl font-semibold">{item.count}</p>
                </div>
              ))}
            </div>
          </CardContent>
        </Card>

        <Card>
          <CardHeader>
            <CardTitle>Siguiente trabajo</CardTitle>
            <CardDescription>
              Prioridades para dejar listo el paquete de exportacion.
            </CardDescription>
          </CardHeader>
          <CardContent className="space-y-3 text-sm">
            <PriorityRow label="Resolver validaciones bloqueantes" value={data.blockingValidations} />
            <PriorityRow label="Documentar cambios modificados" value={changed} />
            <PriorityRow label="Revisar preguntas nuevas" value={added} />
            <PriorityRow label="Validar enlaces del proveedor" value={data.providerLinksPending} />
          </CardContent>
        </Card>
      </section>

      <BaselineCard
        baseline={baseline.data}
        pendingChanges={data.pendingChanges}
        confirmationText={confirmationText}
        acknowledgeReplacement={acknowledgeReplacement}
        acknowledgeBackup={acknowledgeBackup}
        pending={markBaseline.isPending}
        error={markBaseline.isError}
        onConfirmationTextChange={setConfirmationText}
        onReplacementChange={setAcknowledgeReplacement}
        onBackupChange={setAcknowledgeBackup}
        onMarkOriginal={() =>
          markBaseline.mutate({
            confirmationText,
            acknowledgeReplacement,
            acknowledgeBackup,
          })
        }
      />

      <InstrumentConfiguration />
    </div>
  );
}

function Hero({
  title,
  description,
  action,
}: {
  title: string;
  description: string;
  action?: React.ReactNode;
}) {
  return (
    <section className="apple-hero p-6 md:p-8">
      <div className="flex flex-col justify-between gap-4 md:flex-row md:items-end">
        <div>
          <h1 className="max-w-3xl text-3xl font-semibold tracking-normal md:text-4xl">
            {title}
          </h1>
          <p className="mt-3 max-w-3xl text-sm leading-6 text-muted-foreground">
            {description}
          </p>
        </div>
        {action}
      </div>
    </section>
  );
}

function SetupStep({
  step,
  title,
  description,
  to,
  icon,
}: {
  step: string;
  title: string;
  description: string;
  to: string;
  icon: React.ReactNode;
}) {
  return (
    <Card>
      <CardContent className="space-y-4 p-5">
        <div className="flex items-center justify-between">
          <div className="workflow-pill flex size-10 items-center justify-center text-sm font-semibold text-primary">
            {step}
          </div>
          <div className="text-primary [&_svg]:size-5">{icon}</div>
        </div>
        <div>
          <h2 className="font-semibold">{title}</h2>
          <p className="mt-2 text-sm leading-6 text-muted-foreground">{description}</p>
        </div>
        <Button asChild variant="outline" className="w-full">
          <Link to={to}>Abrir</Link>
        </Button>
      </CardContent>
    </Card>
  );
}

function MetricCard({
  title,
  value,
  detail,
  icon,
}: {
  title: string;
  value: number | string;
  detail: string;
  icon: React.ReactNode;
}) {
  return (
    <Card>
      <CardContent className="flex items-center justify-between gap-3 p-4">
        <div>
          <p className="text-sm text-muted-foreground">{title}</p>
          <p className="mt-2 text-2xl font-semibold">{value}</p>
          <p className="mt-1 text-xs text-muted-foreground">{detail}</p>
        </div>
        <div className="workflow-pill flex size-10 items-center justify-center text-primary [&_svg]:size-5">
          {icon}
        </div>
      </CardContent>
    </Card>
  );
}

function PriorityRow({ label, value }: { label: string; value: number }) {
  return (
    <div className="flex items-center justify-between gap-3 rounded-lg border bg-background/70 px-3 py-2">
      <span>{label}</span>
      <Badge variant={value > 0 ? "warning" : "secondary"}>{value}</Badge>
    </div>
  );
}

function statusLabel(status: string) {
  return {
    keep: "Mantener",
    modify: "Modificar",
    add: "Agregar",
    delete: "Eliminar",
  }[status];
}

function cycleLabel(status: string) {
  return {
    planning: "Planeacion",
    inReview: "En revision",
    inApplication: "En aplicacion",
    closed: "Cerrado",
  }[status];
}
