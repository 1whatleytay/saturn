const hardString = ':;<>,./?\'\"[]{}-=_+()\`~'

const hardCharacters = new Set(hardString.split(''))

export function isHard(character: string, hard: Set<string>): boolean {
  return hard.has(character) || /\s/.test(character)
}

export function consumeBackwards(line: string, index: number, hard: Set<string> = hardCharacters): number {
  if (index <= 0) {
    return 1
  }

  let result = 0
  const first = isHard(line[index - 1], hard)

  while (index - result > 0 && first === isHard(line[index - result - 1], hard)) {
    result += 1
  }

  return Math.max(1, result)
}

export function consumeForwards(line: string, index: number, hard: Set<string> = hardCharacters): number {
  if (index >= line.length) {
    return 1
  }

  let result = 0
  const first = isHard(line[index], hard)

  while (index + result < line.length && first === isHard(line[index + result], hard)) {
    result += 1
  }

  return Math.max(1, result)
}
