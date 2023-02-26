const hardString = ':;<>,./?\'\"[]{}-=+()\`~'

const hardCharacters = new Set(hardString.split(''))

function isHard(character: string, hard: Set<string>): boolean {
  return hard.has(character) || /\s/.test(character)
}

export function consumeDirection(line: string, index: number, direction: number, hard: Set<string> = hardCharacters): number {
  const directionOffset = direction < 0 ? 1 : 0
  const cursor = index - directionOffset

  if (cursor <= 0 || cursor >= line.length) {
    return 1
  }

  let result = 0
  const first = isHard(line[cursor], hard)

  while (cursor + result > 0 && cursor + result < line.length
    && first === isHard(line[cursor + result], hard)) {
    result += direction
  }

  console.log(direction * result)
  return Math.max(1, direction * result)
}

export function consumeBackwards(line: string, index: number, hard: Set<string> = hardCharacters): number {
  return consumeDirection(line, index, -1, hard)
}

export function consumeForwards(line: string, index: number, hard: Set<string> = hardCharacters): number {
  return consumeDirection(line, index, +1, hard)
}
