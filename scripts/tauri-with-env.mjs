import { existsSync, readFileSync } from "node:fs";
import { spawnSync } from "node:child_process";
import { resolve } from "node:path";

const envPath = resolve(process.cwd(), ".env.build");
const env = { ...process.env };

if (existsSync(envPath)) {
  const content = readFileSync(envPath, "utf8");
  for (const rawLine of content.split(/\r?\n/)) {
    const line = rawLine.trim();
    if (!line || line.startsWith("#")) continue;
    const separator = line.indexOf("=");
    if (separator === -1) continue;
    const key = line.slice(0, separator).trim();
    let value = line.slice(separator + 1).trim();
    if (
      (value.startsWith('"') && value.endsWith('"')) ||
      (value.startsWith("'") && value.endsWith("'"))
    ) {
      value = value.slice(1, -1);
    }
    if (key) env[key] = value;
  }
  console.log("Loaded Tauri environment from .env.build");
} else {
  console.warn(
    "No .env.build found. Copy .env.build.example to .env.build to provide Turso defaults.",
  );
}

const hasTursoUrl = Boolean(env.AUTOCNA_TURSO_DATABASE_URL || env.TURSO_DATABASE_URL);
const hasTursoToken = Boolean(env.AUTOCNA_TURSO_AUTH_TOKEN || env.TURSO_AUTH_TOKEN);
if (!hasTursoUrl || !hasTursoToken) {
  console.warn("Turso defaults are incomplete. URL and token are both required.");
}

const tauriBin = resolve(
  process.cwd(),
  "node_modules",
  ".bin",
  process.platform === "win32" ? "tauri.cmd" : "tauri",
);
const result = spawnSync(tauriBin, process.argv.slice(2), {
  env,
  stdio: "inherit",
  shell: process.platform === "win32",
});

process.exit(result.status ?? 1);
