import { SuggestionType } from '../languages/suggestions'

export function suggestionLetter(type?: SuggestionType): string {
  switch (type) {
    case SuggestionType.Instruction:
      return 'I'

    case SuggestionType.Register:
      return 'R'

    case SuggestionType.Directive:
      return 'D'

    case SuggestionType.Label:
      return 'L'

    case SuggestionType.Variable:
      return 'V'

    case SuggestionType.Function:
      return 'F'

    default:
      return 'O'
  }
}

export function suggestionStyle(type?: SuggestionType): string {
  switch (type) {
    case SuggestionType.Instruction:
      return 'bg-cyan-500 text-cyan-900'

    case SuggestionType.Register:
      return 'bg-orange-300 text-orange-800'

    case SuggestionType.Directive:
      return 'bg-red-300 text-red-800'

    case SuggestionType.Label:
      return 'bg-yellow-300 text-yellow-800'

    case SuggestionType.Variable:
      return 'bg-green-300 text-green-800'

    case SuggestionType.Function:
      return 'bg-purple-300 text-purple-800'

    default:
      return 'bg-gray-700'
  }
}
