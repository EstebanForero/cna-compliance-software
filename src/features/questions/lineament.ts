import type { GuidelineAspect, NewQuestion, Question } from "@/lib/types";

export type FactorOption = {
  code: string;
  name: string;
};

export type CharacteristicOption = FactorOption & {
  factorCode: string;
};

export function fillQuestionFromLineament(
  question: NewQuestion,
  aspect: GuidelineAspect,
): NewQuestion {
  return {
    ...question,
    scope: aspect.scope,
    factor: joinCodeName(aspect.factorCode, aspect.factorName),
    characteristic: joinCodeName(aspect.characteristicCode, aspect.characteristicName),
    aspect: joinCodeName(aspect.aspectCode, aspect.aspectDescription),
  };
}

export function questionMatchesLineament(question: Question, aspect: GuidelineAspect) {
  return (
    splitNumberName(question.factor).code === aspect.factorCode &&
    splitNumberName(question.characteristic).code === aspect.characteristicCode &&
    splitNumberName(question.aspect).code === aspect.aspectCode
  );
}

export function compareGuidelineAspect(left: GuidelineAspect, right: GuidelineAspect) {
  return (
    compareNaturalCode(left.factorCode, right.factorCode) ||
    compareNaturalCode(left.characteristicCode, right.characteristicCode) ||
    compareNaturalCode(left.aspectCode, right.aspectCode) ||
    left.aspectDescription.localeCompare(right.aspectDescription, "es", {
      sensitivity: "base",
    })
  );
}

export function factorOptionsFromLineaments(aspects: GuidelineAspect[]) {
  const seen = new Map<string, FactorOption>();
  for (const aspect of aspects) {
    if (!seen.has(aspect.factorCode)) {
      seen.set(aspect.factorCode, {
        code: aspect.factorCode,
        name: aspect.factorName,
      });
    }
  }
  return Array.from(seen.values()).sort(compareCodeName);
}

export function characteristicOptionsFromLineaments(
  aspects: GuidelineAspect[],
  factorCode = "",
) {
  const seen = new Map<string, CharacteristicOption>();
  for (const aspect of aspects) {
    if (factorCode && aspect.factorCode !== factorCode) continue;
    const key = `${aspect.factorCode}:${aspect.characteristicCode}`;
    if (!seen.has(key)) {
      seen.set(key, {
        factorCode: aspect.factorCode,
        code: aspect.characteristicCode,
        name: aspect.characteristicName,
      });
    }
  }
  return Array.from(seen.values()).sort(compareCodeName);
}

export function aspectsForCharacteristic(
  aspects: GuidelineAspect[],
  factorCode: string,
  characteristicCode: string,
) {
  return aspects
    .filter(
      (aspect) =>
        aspect.factorCode === factorCode &&
        aspect.characteristicCode === characteristicCode,
    )
    .sort(compareGuidelineAspect);
}

export function findLineamentIdForQuestion(
  question: Question,
  aspects: GuidelineAspect[],
) {
  return aspects.find((aspect) => questionMatchesLineament(question, aspect))?.id ?? "";
}

function joinCodeName(code: string, name: string) {
  return `${code}. ${name}`;
}

function compareCodeName(left: FactorOption, right: FactorOption) {
  return (
    compareNaturalCode(left.code, right.code) ||
    left.name.localeCompare(right.name, "es", { sensitivity: "base" })
  );
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
