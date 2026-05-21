import type { QueryClient } from "@tanstack/react-query";
import { useMutation, useQuery, useQueryClient } from "@tanstack/react-query";
import { Link, Outlet, createRootRouteWithContext } from "@tanstack/react-router";
import {
  ClipboardCheck,
  Cloud,
  Database,
  FileSpreadsheet,
  BookOpen,
  Clock3,
  Layers3,
  LinkIcon,
  Moon,
  RouteIcon,
  Save,
  Settings,
  Sun,
} from "lucide-react";
import { useState } from "react";

import { Button } from "@/components/ui/button";
import { Badge } from "@/components/ui/badge";
import { OnboardingGate } from "@/components/onboarding-gate";
import { ToastProvider } from "@/components/ui/toast";
import { api } from "@/lib/api";
import { useTheme } from "@/lib/theme";
import { cn } from "@/lib/utils";

interface RouterContext {
  queryClient: QueryClient;
}

const navigation = [
  { to: "/", label: "Panel", icon: ClipboardCheck },
  { to: "/lineaments", label: "Lineamientos", icon: Layers3 },
  { to: "/questions", label: "Banco", icon: Database },
  { to: "/validation", label: "Validacion", icon: FileSpreadsheet },
  { to: "/exports", label: "Exportar", icon: FileSpreadsheet },
  { to: "/history", label: "Historial", icon: Clock3 },
  { to: "/provider", label: "Proveedor", icon: LinkIcon },
  { to: "/workspace", label: "Configuración", icon: Settings },
  { to: "/docs", label: "Docs", icon: BookOpen },
] as const;

export const Route = createRootRouteWithContext<RouterContext>()({
  component: RootLayout,
});

function RootLayout() {
  const { theme, toggleTheme } = useTheme();
  const queryClient = useQueryClient();
  const [saveNotice, setSaveNotice] = useState("");
  const workspace = useQuery({
    queryKey: ["workspace"],
    queryFn: api.workspace,
    refetchInterval: 30000,
  });
  const connected = Boolean(workspace.data?.tursoConnected);
  const presence = useQuery({
    queryKey: ["collaboration-presence"],
    queryFn: api.heartbeatCollaborationPresence,
    enabled: connected && Boolean(workspace.data?.editorProfile),
    refetchInterval: 30000,
  });
  const currentEditor = workspace.data?.editorProfile?.fullName ?? "";
  const otherEditors = (presence.data ?? []).filter(
    (item) => item.editorName !== currentEditor,
  );
  const connectionLabel = connected
    ? summarizeTursoWorkspace(workspace.data?.tursoDatabaseUrl)
    : "Los cambios se guardan en esta base local.";
  const saveHistory = useMutation({
    mutationFn: api.saveManualHistorySnapshot,
    onSuccess: async () => {
      setSaveNotice("Guardado");
      window.setTimeout(() => setSaveNotice(""), 2400);
      await queryClient.invalidateQueries({ queryKey: ["history-snapshots"] });
      await queryClient.invalidateQueries({ queryKey: ["change-logs"] });
    },
  });

  return (
    <ToastProvider>
      <OnboardingGate>
        <div className="min-h-screen">
        <aside className="fixed inset-y-0 left-0 hidden w-64 border-r bg-card/70 p-4 shadow-xl shadow-black/5 backdrop-blur-2xl md:flex md:flex-col">
          <div className="mb-8 flex items-center gap-3">
            <div className="flex size-10 items-center justify-center rounded-md bg-primary text-primary-foreground">
              <RouteIcon className="size-5" />
            </div>
            <div>
              <p className="text-sm font-semibold">Autoevaluacion CNA</p>
              <p className="text-xs text-muted-foreground">
                {connected ? "Fuente unica colaborativa" : "Fuente unica local"}
              </p>
            </div>
          </div>
          <nav className="space-y-1">
            {navigation.map((item) => (
              <Link key={item.to} to={item.to}>
                {({ isActive }) => (
                  <Button
                    asChild
                    variant="ghost"
                    className={cn(
                      "w-full justify-start",
                      isActive && "bg-muted text-primary",
                    )}
                  >
                    <span>
                      <item.icon className="size-4" />
                      {item.label}
                    </span>
                  </Button>
                )}
              </Link>
            ))}
          </nav>
          <div className="mt-auto space-y-2 border-t pt-4">
            <div
              className={cn(
                "rounded-lg border p-3 text-xs shadow-sm",
                connected
                  ? "border-blue-300 bg-blue-50 text-blue-950 dark:border-blue-400/30 dark:bg-blue-950/70 dark:text-blue-100"
                  : "border-border bg-background/80 text-foreground",
              )}
            >
              <div className="flex items-center justify-between gap-2">
                <span className="flex items-center gap-2 font-semibold text-blue-950 dark:text-blue-100">
                  <Cloud className="size-4 text-blue-700 dark:text-blue-200" />
                  {connected ? "Conectado" : "Desconectado"}
                </span>
                <Badge
                  variant={connected ? "secondary" : "outline"}
                  className={cn(
                    connected &&
                      "border-blue-200 bg-white/80 text-blue-800 dark:border-blue-400/30 dark:bg-blue-900/80 dark:text-blue-100",
                  )}
                >
                  {connected ? "Turso" : "Local"}
                </Badge>
              </div>
              <p className="mt-2 text-[11px] leading-5 text-blue-800 dark:text-blue-200">
                {connectionLabel}
              </p>
              {connected ? (
                <p className="mt-2 font-medium text-blue-900 dark:text-blue-100">
                  {otherEditors.length > 0
                    ? `${otherEditors.length} editor(es) activo(s): ${otherEditors
                        .map((item) => item.editorName)
                        .join(", ")}`
                    : "No hay otros editores activos ahora."}
                </p>
              ) : null}
            </div>
            <Button
              className="w-full justify-start bg-blue-600 text-white hover:bg-blue-700"
              onClick={() => saveHistory.mutate()}
              disabled={saveHistory.isPending}
            >
              <Save className="size-4" />
              {saveHistory.isPending ? "Guardando..." : saveNotice || "Guardar historial"}
            </Button>
            <Button variant="outline" className="w-full justify-start" onClick={toggleTheme}>
              {theme === "dark" ? <Sun className="size-4" /> : <Moon className="size-4" />}
              {theme === "dark" ? "Modo claro" : "Modo oscuro"}
            </Button>
          </div>
        </aside>

      <header className="sticky top-0 z-10 border-b bg-card/70 px-4 py-3 backdrop-blur-2xl md:hidden">
        <div className="flex items-center justify-between gap-3">
          <div>
            <p className="text-sm font-semibold">Autoevaluacion CNA</p>
            <p className="text-xs text-muted-foreground">
              {connected ? "Conectado a Turso" : "Modo local"}
            </p>
          </div>
          <nav className="flex gap-1">
            <Button
              size="icon"
              variant="ghost"
              title="Guardar historial"
              onClick={() => saveHistory.mutate()}
              disabled={saveHistory.isPending}
            >
              <Save className="size-4 text-blue-600" />
            </Button>
            <Button size="icon" variant="ghost" title="Cambiar tema" onClick={toggleTheme}>
              {theme === "dark" ? <Sun className="size-4" /> : <Moon className="size-4" />}
            </Button>
            {navigation.map((item) => (
              <Link key={item.to} to={item.to}>
                <Button size="icon" variant="ghost" title={item.label}>
                  <item.icon className="size-4" />
                </Button>
              </Link>
            ))}
          </nav>
        </div>
      </header>

        <main className="md:pl-64">
          <div className="mx-auto max-w-none px-3 py-5 md:px-4 lg:px-5">
            <Outlet />
          </div>
        </main>
        </div>
      </OnboardingGate>
    </ToastProvider>
  );
}

function summarizeTursoWorkspace(databaseUrl?: string | null) {
  if (!databaseUrl) return "Base Turso remota activa.";

  try {
    const host = new URL(databaseUrl).host;
    const database = host.split(".")[0]?.split("-")[0]?.trim();
    return database ? `Base Turso activa: ${database}` : "Base Turso remota activa.";
  } catch {
    return "Base Turso remota activa.";
  }
}
