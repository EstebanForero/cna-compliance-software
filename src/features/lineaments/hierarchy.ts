import type { GuidelineAspect, NewGuidelineAspect, Question } from "@/lib/types";

export type CodeName = { code: string; name: string };

export type CharacteristicChoice = {
  factorCode: string;
  factorName: string;
  code: string;
  name: string;
};

export const initialAspect: NewGuidelineAspect = {
  guidelineTitle: "Lineamiento CNA importado",
  scope: "institutional",
  factorCode: "",
  factorName: "",
  characteristicCode: "",
  characteristicName: "",
  aspectCode: "",
  aspectDescription: "",
  requiresAppreciation: true,
};

export function aspectToDraft(aspect: GuidelineAspect): NewGuidelineAspect {
  return {
    guidelineTitle: aspect.guidelineTitle,
    scope: aspect.scope,
    factorCode: aspect.factorCode,
    factorName: aspect.factorName,
    characteristicCode: aspect.characteristicCode,
    characteristicName: aspect.characteristicName,
    aspectCode: aspect.aspectCode,
    aspectDescription: aspect.aspectDescription,
    requiresAppreciation: aspect.requiresAppreciation,
  };
}

export function questionMatchesLineament(question: Question, aspect: GuidelineAspect) {
  return (
    splitNumberName(question.factor).code === aspect.factorCode &&
    splitNumberName(question.characteristic).code === aspect.characteristicCode &&
    splitNumberName(question.aspect).code === aspect.aspectCode
  );
}

export function factorChoicesFromData(
  aspects: GuidelineAspect[] = [],
  questions: Question[] = [],
) {
  const seen = new Map<string, string>();
  for (const aspect of aspects) {
    seen.set(aspect.factorCode, aspect.factorName);
  }
  for (const question of questions) {
    const factor = splitNumberName(question.factor);
    if (factor.code && factor.name) seen.set(factor.code, factor.name);
  }
  return Array.from(seen, ([code, name]) => ({ code, name })).sort(compareCodeName);
}

export function collectCharacteristicChoices(
  aspects: GuidelineAspect[] = [],
  questions: Question[] = [],
): CharacteristicChoice[] {
  const seen = new Map<string, CharacteristicChoice>();

  for (const aspect of aspects) {
    if (aspect.factorCode && aspect.characteristicCode) {
      const key = characteristicValue(aspect.factorCode, aspect.characteristicCode);
      seen.set(key, {
        factorCode: aspect.factorCode,
        factorName: aspect.factorName,
        code: aspect.characteristicCode,
        name: aspect.characteristicName,
      });
    }
  }

  for (const question of questions) {
    const factor = splitNumberName(question.factor);
    const characteristic = splitNumberName(question.characteristic);
    if (factor.code && factor.name && characteristic.code && characteristic.name) {
      const key = characteristicValue(factor.code, characteristic.code);
      if (!seen.has(key)) {
        seen.set(key, {
          factorCode: factor.code,
          factorName: factor.name,
          code: characteristic.code,
          name: characteristic.name,
        });
      }
    }
  }

  return Array.from(seen.values());
}

export function groupCharacteristicsByFactor(options: CharacteristicChoice[]) {
  const groups = new Map<
    string,
    {
      factorCode: string;
      factorName: string;
      options: CharacteristicChoice[];
    }
  >();

  for (const option of options) {
    const key = option.factorCode;
    const group =
      groups.get(key) ??
      {
        factorCode: option.factorCode,
        factorName: option.factorName,
        options: [],
      };
    group.options.push(option);
    groups.set(key, group);
  }

  return Array.from(groups.values()).sort((left, right) =>
    compareNaturalCode(left.factorCode, right.factorCode),
  );
}

export function characteristicValue(factorCode: string, characteristicCode: string) {
  return `${factorCode}::${characteristicCode}`;
}

export function parseCharacteristicValue(value: string) {
  const [factorCode = "", characteristicCode = ""] = value.split("::");
  return { factorCode, characteristicCode };
}

export function compareCharacteristicChoice(
  left: CharacteristicChoice,
  right: CharacteristicChoice,
) {
  return (
    compareNaturalCode(left.factorCode, right.factorCode) ||
    compareNaturalCode(left.code, right.code) ||
    left.name.localeCompare(right.name, "es", { sensitivity: "base" })
  );
}

export function compareCodeName(left: CodeName, right: CodeName) {
  return (
    compareNaturalCode(left.code, right.code) ||
    left.name.localeCompare(right.name, "es", { sensitivity: "base" })
  );
}

export function nextNumericCode(options: Array<{ code: string }>) {
  const next =
    options
      .map((option) => Number(option.code))
      .filter((value) => Number.isFinite(value))
      .reduce((max, value) => Math.max(max, value), 0) + 1;
  return String(next);
}

function splitNumberName(value: string) {
  const [code, ...nameParts] = value.split(". ");
  const name = nameParts.join(". ").trim();
  return name ? { code: code.trim(), name } : { code: "", name: value.trim() };
}

function compareNaturalCode(left: string, right: string) {
  const leftParts = left.match(/\d+|\D+/g) ?? [left];
  const rightParts = right.match(/\d+|\D+/g) ?? [right];
  const length = Math.max(leftParts.length, rightParts.length);
  for (let index = 0; index < length; index += 1) {
    const leftPart = leftParts[index] ?? "";
    const rightPart = rightParts[index] ?? "";
    const leftNumber = Number(leftPart);
    const rightNumber = Number(rightPart);
    const bothNumeric = !Number.isNaN(leftNumber) && !Number.isNaN(rightNumber);
    const result = bothNumeric
      ? leftNumber - rightNumber
      : leftPart.localeCompare(rightPart, "es", { sensitivity: "base" });
    if (result !== 0) return result;
  }
  return 0;
}
