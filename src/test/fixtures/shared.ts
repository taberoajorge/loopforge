export type DeepPartial<TValue> =
  TValue extends readonly unknown[]
    ? TValue
    : TValue extends object
      ? { [TKey in keyof TValue]?: DeepPartial<TValue[TKey]> }
      : TValue;

function isObjectRecord(value: unknown): value is Record<string, unknown> {
  return typeof value === "object" && value !== null && !Array.isArray(value);
}

function cloneValue<TValue>(value: TValue): TValue {
  if (Array.isArray(value)) {
    return value.map((entry) => cloneValue(entry)) as TValue;
  }
  if (!isObjectRecord(value)) {
    return value;
  }
  const clonedEntries = Object.entries(value).map(([entryKey, entryValue]) => [
    entryKey,
    cloneValue(entryValue),
  ]);
  return Object.fromEntries(clonedEntries) as TValue;
}

export function mergeFixture<TValue>(
  defaults: TValue,
  overrides?: DeepPartial<TValue>,
): TValue {
  if (overrides === undefined) {
    return cloneValue(defaults);
  }
  if (Array.isArray(defaults)) {
    return cloneValue(overrides as TValue);
  }
  if (!isObjectRecord(defaults) || !isObjectRecord(overrides)) {
    return cloneValue(overrides as TValue);
  }

  const merged = cloneValue(defaults) as Record<string, unknown>;
  for (const [entryKey, overrideValue] of Object.entries(overrides)) {
    if (overrideValue === undefined) {
      continue;
    }
    const defaultValue = merged[entryKey];
    merged[entryKey] =
      isObjectRecord(defaultValue) && isObjectRecord(overrideValue)
        ? mergeFixture(defaultValue, overrideValue)
        : cloneValue(overrideValue);
  }
  return merged as TValue;
}
