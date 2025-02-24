import { foldService } from '@codemirror/language'

// Taken from https://github.com/home-assistant/frontend/blob/20250109.0/src/resources/codemirror.ts#L279
export const foldOnIndent = foldService.of((state, from, to) => {
  const line = state.doc.lineAt(from)

  // empty lines continue their indentation from surrounding lines
  if (!line.length || !line.text.trim().length) {
    return null
  }

  let onlyEmptyNext = true

  const lineCount = state.doc.lines
  const indent = line.text.search(/\S|$/) // Indent level of the first line

  let foldStart = from // Start of the fold
  let foldEnd = to // End of the fold

  // Check if the next line is on a deeper indent level
  // If so, continue subsequent lines
  // If not, go on with the foldEnd
  let nextLine = line
  while (nextLine.number < lineCount) {
    nextLine = state.doc.line(nextLine.number + 1) // Next line
    const nextIndent = nextLine.text.search(/\S|$/) // Indent level of the next line

    // If the next line is on a deeper indent level, add it to the fold
    // empty lines continue their indentation from surrounding lines
    if (
      !nextLine.length ||
      !nextLine.text.trim().length ||
      nextIndent > indent
    ) {
      if (onlyEmptyNext) {
        onlyEmptyNext = nextLine.text.trim().length === 0
      }
      // include this line in the fold and continue
      foldEnd = nextLine.to
    } else {
      // If the next line is not on a deeper indent level, we found the end of the region
      break
    }
  }

  // Adjust foldEnd to exclude trailing empty lines
  while (foldEnd > foldStart && !state.doc.lineAt(foldEnd).text.trim().length) {
    foldEnd = state.doc.lineAt(foldEnd).from - 1
  }

  // Don't create fold if it's a single line
  if (
    onlyEmptyNext ||
    state.doc.lineAt(foldStart).number >= state.doc.lineAt(foldEnd).number
  ) {
    return null
  }

  // Set the fold start to the end of the first line
  // With this, the fold will not include the first line
  foldStart = line.to

  // Return a fold that covers the entire indent level
  return { from: foldStart, to: foldEnd }
})
