import type { NewQuestion, QuestionFormat } from "@/lib/types";

export type ResponseConventionCode =
  | "A"
  | "B"
  | "C"
  | "D"
  | "E"
  | "F"
  | "G"
  | "H"
  | "I"
  | "J"
  | "K";

export type ChoiceOption = {
  id: string;
  label: string;
};

export const questionFormats: Array<{
  value: QuestionFormat;
  label: string;
  description: string;
}> = [
  {
    value: "likert",
    label: "Escala / Likert",
    description: "Usa una convencion institucional A-J.",
  },
  {
    value: "singleChoice",
    label: "Seleccion unica",
    description: "Una sola respuesta, por ejemplo Si/No.",
  },
  {
    value: "multipleChoice",
    label: "Seleccion multiple",
    description: "Varias opciones o convenciones combinadas.",
  },
  {
    value: "open",
    label: "Abierta",
    description: "Respuesta textual, sin convencion de escala.",
  },
];

export const conventionOptions = ["A", "B", "C", "D", "E", "F", "G", "H", "I", "J", "K"];

export const responseConventions: Array<{
  code: ResponseConventionCode;
  name: string;
  description: string;
  values: string[];
}> = [
  {
    code: "A",
    name: "Acuerdo",
    description: "Total desacuerdo a total acuerdo",
    values: [
      "Total desacuerdo",
      "Desacuerdo",
      "Medianamente de acuerdo",
      "Acuerdo",
      "Total acuerdo",
    ],
  },
  {
    code: "B",
    name: "Cantidad",
    description: "Nada a muy bien",
    values: ["Nada", "Poco", "Regular", "Bien", "Muy bien"],
  },
  {
    code: "C",
    name: "Calidad",
    description: "Muy malo a excelente",
    values: ["Muy malo", "Malo", "Regular", "Bueno", "Excelente"],
  },
  {
    code: "D",
    name: "Frecuencia",
    description: "Nunca a siempre",
    values: ["Nunca", "Casi nunca", "Algunas veces", "Casi siempre", "Siempre"],
  },
  {
    code: "E",
    name: "Nivel",
    description: "Nulo a muy alto",
    values: ["Nulo", "Bajo", "Medio", "Alto", "Muy alto"],
  },
  {
    code: "F",
    name: "Exigencia",
    description: "Nada exigentes a muy exigentes",
    values: [
      "Nada exigentes",
      "Poco exigentes",
      "Medianamente exigentes",
      "Exigentes",
      "Muy exigentes",
    ],
  },
  {
    code: "G",
    name: "Favorecimiento",
    description: "No favorece para nada a favorece totalmente",
    values: [
      "No favorece para nada",
      "No favorece",
      "Es indiferente",
      "Favorece",
      "Favorece totalmente",
    ],
  },
  {
    code: "H",
    name: "Probabilidad",
    description: "Nada probable a totalmente probable",
    values: [
      "Nada probable",
      "Poco probable",
      "Medianamente probable",
      "Muy probable",
      "Totalmente probable",
    ],
  },
  {
    code: "I",
    name: "Satisfaccion",
    description: "Nada satisfecho a muy satisfecho",
    values: [
      "Nada satisfecho",
      "Poco satisfecho",
      "Medianamente satisfecho",
      "Satisfecho",
      "Muy satisfecho",
    ],
  },
  {
    code: "J",
    name: "Medida",
    description: "En ninguna medida a en muy alta medida",
    values: [
      "En ninguna medida",
      "En muy baja medida",
      "En baja medida",
      "En alta medida",
      "En muy alta medida",
    ],
  },
  {
    code: "K",
    name: "Si / No",
    description: "Respuesta binaria usada por preguntas de seleccion unica",
    values: ["Si", "No"],
  },
];

export function questionFormatLabel(format: QuestionFormat | string) {
  return (
    questionFormats.find((option) => option.value === format)?.label ??
    String(format || "Sin tipo")
  );
}

export function responseConventionLabel(value?: string | null) {
  if (!value) return "Sin convencion";

  const parsedMultipleChoice = parseMultipleChoiceConvention(value);
  if (parsedMultipleChoice) {
    return `Seleccion multiple: ${parsedMultipleChoice.join(", ")}`;
  }

  const convention = responseConventions.find((option) => option.code === value);
  return convention ? `${convention.name} (${convention.code})` : value;
}

export function responseConventionDescription(value?: string | null) {
  const convention = responseConventions.find((option) => option.code === value);
  return convention
    ? `${convention.description}. Opciones: ${convention.values.join(" · ")}`
    : "";
}

export function defaultConventionForFormat(format: QuestionFormat) {
  if (format === "singleChoice") return "K";
  if (format === "multipleChoice") return "";
  return "A";
}

export function normalizedChoiceOptions(options: ChoiceOption[]) {
  return options
    .map((option) => option.label.trim())
    .filter(Boolean)
    .filter((option, index, list) => list.indexOf(option) === index);
}

export function serializeChoiceOptions(options: ChoiceOption[]) {
  const normalized = normalizedChoiceOptions(options);
  if (normalized.length === 0) return "";
  return JSON.stringify({ type: "multipleChoice", options: normalized });
}

export function choiceOptionsFromConvention(value?: string | null): ChoiceOption[] {
  const parsed = value ? parseMultipleChoiceConvention(value) : null;
  const source = parsed && parsed.length > 0 ? parsed : ["", ""];
  return source.map((label, index) => ({
    id: `option-${index + 1}`,
    label,
  }));
}

function parseMultipleChoiceConvention(value: string) {
  try {
    const parsed = JSON.parse(value) as { type?: string; options?: unknown };
    if (
      parsed.type === "multipleChoice" &&
      Array.isArray(parsed.options) &&
      parsed.options.every((option) => typeof option === "string")
    ) {
      return parsed.options;
    }
  } catch {
    return null;
  }
  return null;
}

export function prepareQuestionForSave(
  question: NewQuestion,
  options: ChoiceOption[],
): NewQuestion {
  if (question.format === "multipleChoice") {
    return {
      ...question,
      conventionCode: serializeChoiceOptions(options),
    };
  }

  if (question.format === "open") {
    return {
      ...question,
      conventionCode: undefined,
    };
  }

  return question;
}
