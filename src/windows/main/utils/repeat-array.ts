export function repeatArray<T>(array: T[], count: number) {
  let out: T[] = [];
  for (let i = 0; i < count; i++) {
    out = out.concat(array);
  }
  return out;
}
