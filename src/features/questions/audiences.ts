import type { Question } from "@/lib/types";

export function audienceOptionsFromQuestions(questions: Question[] = []) {
  return Array.from(
    new Set(
      questions
        .flatMap((question) => question.audiences)
        .map(normalizeAudienceLabel)
        .filter(Boolean),
    ),
  ).sort((left, right) => left.localeCompare(right, "es", { sensitivity: "base" }));
}

export function normalizeAudienceList(values: string[]) {
  return Array.from(
    new Set(values.map(normalizeAudienceLabel).filter(Boolean)),
  ).sort((left, right) => left.localeCompare(right, "es", { sensitivity: "base" }));
}

export function normalizeAudienceLabel(value: string) {
  return value
    .trim()
    .replace(/_/g, " ")
    .replace(/^\d+\s*(?=[A-ZÁÉÍÓÚÑa-záéíóúñ])/, "")
    .replace(/\s+/g, " ")
    .trim();
}
