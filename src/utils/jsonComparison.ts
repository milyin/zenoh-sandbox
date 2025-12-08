/**
 * Utilities for comparing and highlighting differences between JSON objects
 */

export interface JsonLine {
  lineNumber: number;
  content: string;
  isChanged: boolean;
}

/**
 * Compare two JSON objects and identify which lines are different
 * @param oldJson - Previous JSON object
 * @param newJson - New JSON object
 * @returns Array of lines with change indicators
 */
export function compareJsonObjects(
  oldJson: Record<string, any>,
  newJson: Record<string, any>
): JsonLine[] {
  const oldString = JSON.stringify(oldJson, null, 2);
  const newString = JSON.stringify(newJson, null, 2);

  const oldLines = oldString.split('\n');
  const newLines = newString.split('\n');

  const result: JsonLine[] = [];
  const maxLength = Math.max(oldLines.length, newLines.length);

  for (let i = 0; i < maxLength; i++) {
    const oldLine = i < oldLines.length ? oldLines[i] : '';
    const newLine = i < newLines.length ? newLines[i] : '';

    result.push({
      lineNumber: i + 1,
      content: newLine,
      isChanged: oldLine !== newLine,
    });
  }

  return result;
}

/**
 * Compare two JSON strings and identify which lines are different
 * @param oldJsonString - Previous JSON string
 * @param newJsonString - New JSON string
 * @returns Array of lines with change indicators
 */
export function compareJsonStrings(
  oldJsonString: string,
  newJsonString: string
): JsonLine[] {
  const oldLines = oldJsonString.split('\n');
  const newLines = newJsonString.split('\n');

  const result: JsonLine[] = [];
  const maxLength = Math.max(oldLines.length, newLines.length);

  for (let i = 0; i < maxLength; i++) {
    const oldLine = i < oldLines.length ? oldLines[i] : '';
    const newLine = i < newLines.length ? newLines[i] : '';

    result.push({
      lineNumber: i + 1,
      content: newLine,
      isChanged: oldLine !== newLine,
    });
  }

  return result;
}

/**
 * Deep compare two JSON objects to check if they are equal
 * @param obj1 - First object
 * @param obj2 - Second object
 * @returns true if objects are equal, false otherwise
 */
export function deepEquals(obj1: any, obj2: any): boolean {
  if (obj1 === obj2) return true;

  if (obj1 == null || obj2 == null) return false;
  if (typeof obj1 !== 'object' || typeof obj2 !== 'object') return false;

  const keys1 = Object.keys(obj1);
  const keys2 = Object.keys(obj2);

  if (keys1.length !== keys2.length) return false;

  for (const key of keys1) {
    if (!keys2.includes(key)) return false;
    if (!deepEquals(obj1[key], obj2[key])) return false;
  }

  return true;
}

/**
 * Format JSON with highlighting information
 * @param json - JSON object to format
 * @param changedLines - Set of line numbers that have changed
 * @returns Formatted string with line numbers and change markers
 */
export function formatJsonWithHighlights(
  json: Record<string, any>,
  changedLines: Set<number>
): string {
  const jsonString = JSON.stringify(json, null, 2);
  const lines = jsonString.split('\n');

  return lines
    .map((line, index) => {
      const lineNumber = index + 1;
      const marker = changedLines.has(lineNumber) ? '●' : ' ';
      return `${marker} ${line}`;
    })
    .join('\n');
}
