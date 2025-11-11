export function formatVlanRanges(tags: ReadonlyArray<number>): string {
  if (tags.length === 0) {
    return "-";
  }

  const segments: Array<string> = [];
  let rangeStart = tags[0];
  let prev = tags[0];

  for (let i = 1; i < tags.length; i++) {
    const current = tags[i];
    const isConsecutive = current === prev + 1;

    if (!isConsecutive) {
      segments.push(
        rangeStart === prev ? `${rangeStart}` : `${rangeStart} to ${prev}`
      );
      rangeStart = current;
    }

    prev = current;
  }

  segments.push(
    rangeStart === prev ? `${rangeStart}` : `${rangeStart} to ${prev}`
  );

  return segments.join(", ");
}
