const hardString = ':;<>,./?\'"[]{}-=+()`~*&^%$#@!'

const hardCharacters = new Set(hardString.split(''))

function isHard(character: string, hard: Set<string>): boolean {
  return hard.has(character) || /\s/.test(character)
}

export function consumeDirection(
  line: string,
  index: number,
  direction: number,
  spaced: boolean = true,
  hard: Set<string> = hardCharacters
): number {
  const directionOffset = direction < 0 ? 1 : 0
  const cursor = index - directionOffset

  if (cursor < 0 || cursor >= line.length) {
    return 1
  }

  // keep consuming space
  // then consume hard

  let result = 0
  const inbounds = () => cursor + result >= 0 && cursor + result < line.length

  if (spaced) {
    const space = /\s/
    while (inbounds() && space.test(line[cursor + result])) {
      result += direction
    }

    if (!inbounds()) {
      return Math.max(1, direction * result)
    }
  }

  const first = isHard(line[cursor + result], hard)

  while (inbounds() && first === isHard(line[cursor + result], hard)) {
    result += direction
  }

  return Math.max(1, direction * result)
}

export function consumeBackwards(
  line: string,
  index: number,
  hard: Set<string> = hardCharacters
): number {
  return consumeDirection(line, index, -1, true, hard)
}

export function consumeForwards(
  line: string,
  index: number,
  hard: Set<string> = hardCharacters
): number {
  return consumeDirection(line, index, +1, true, hard)
}

export function consumeSpace(
  line: string,
  index: number,
  direction: number,
  max: number
): number {
  const directionOffset = direction < 0 ? 1 : 0
  const cursor = index - directionOffset

  if (cursor < 0 || cursor >= line.length) {
    return 1
  }

  // keep consuming space
  // then consume hard

  let result = 0
  const inbounds = () => cursor + result >= 0 && cursor + result < line.length

  while (
    inbounds() &&
    Math.abs(result) < max &&
    line[cursor + result] === ' '
  ) {
    result += direction
  }

  return Math.max(1, direction * result)
}
