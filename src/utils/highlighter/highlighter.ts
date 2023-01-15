export interface Token {
  text: string,
  color: string
}

export const style = {
  known: 'font-bold',
  comment: 'text-neutral-400',
  comma: 'text-yellow-400',
  label: 'text-yellow-400',
  directive: 'text-red-400',
  instruction: 'text-sky-400',
  register: 'text-orange-300',
  numeric: 'text-teal-300',
  symbol: 'text-white',
  text: 'text-lime-300',
  nothing: 'text-white'
}

export interface Highlighter {
  highlight(line: string): Token[]
}
