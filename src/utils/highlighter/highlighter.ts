export interface Token {
  text: string,
  color: string
}

export const style = {
  comment: 'text-neutral-400',
  comma: 'text-yellow-400',
  directive: 'text-rose-500 font-bold',
  register: 'text-purple-300',
  numeric: 'text-teal-300',
  symbol: 'text-white',
  text: 'text-lime-300',
  nothing: 'text-white'
}

export interface Highlighter {
  highlight(line: string): Token[]
}
