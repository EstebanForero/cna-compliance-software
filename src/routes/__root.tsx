import type { QueryClient } from "@tanstack/react-query";
import { useMutation, useQueryClient } from "@tanstack/react-query";
import { Link, Outlet, createRootRouteWithContext } from "@tanstack/react-router";
import {
  ClipboardCheck,
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
import { OnboardingGate } from "@/components/onboarding-gate";
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
  { to: "/workspace", label: "Workspace", icon: Settings },
  { to: "/docs", label: "Docs", icon: BookOpen },
] as const;

export const Route = createRootRouteWithContext<RouterContext>()({
  component: RootLayout,
});

function RootLayout() {
  const { theme, toggleTheme } = useTheme();
  const queryClient = useQueryClient();
  const [saveNotice, setSaveNotice] = useState("");
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
    <OnboardingGate>
      <div className="min-h-screen">
        <aside className="fixed inset-y-0 left-0 hidden w-64 border-r bg-card/70 p-4 shadow-xl shadow-black/5 backdrop-blur-2xl md:flex md:flex-col">
          <div className="mb-8 flex items-center gap-3">
            <div className="flex size-10 items-center justify-center rounded-md bg-primary text-primary-foreground">
              <RouteIcon className="size-5" />
            </div>
            <div>
              <p className="text-sm font-semibold">Autoevaluacion CNA</p>
              <p className="text-xs text-muted-foreground">Fuente unica local</p>
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
            <p className="text-xs text-muted-foreground">Fuente unica local</p>
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
          <div className="mx-auto max-w-7xl px-4 py-6 md:px-8">
            <Outlet />
          </div>
        </main>
      </div>
    </OnboardingGate>
  );
}
