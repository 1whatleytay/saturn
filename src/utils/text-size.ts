export class SizeCalculator {
  element: HTMLElement

  calculate(text: string): { width: number, height: number } {
    this.element.textContent = text

    return {
      width: this.element.clientWidth,
      height: this.element.clientHeight
    }
  }

  constructor(classes: string) {
    this.element = document.createElement('div')

    this.element.className = classes

    document.body.appendChild(this.element)
  }
}

export const regular = new SizeCalculator('h-6 text-sm font-mono font-bold absolute whitespace-pre')

export function findPosition(text: string, classes: string, position: number): number {
  return 0
}