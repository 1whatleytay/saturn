export interface Token {
  text: string,
  color: string
}

export const style = {
  comment: 'text-blue-200',
  comma: 'text-yellow-200',
  directive: 'text-red-200',
  register: 'text-orange-200',
  numeric: 'text-teal-200',
  symbol: 'text-purple-200',
  text: 'text-magenta-200',
  nothing: 'text-white'
}

export interface Highlighter {
  highlight(line: string): Token[]
}
