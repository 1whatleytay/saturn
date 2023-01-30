export class SizeCalculator {
  element: HTMLElement

  calculate(text: string): { width: number, height: number } {
    this.element.textContent = text

    return {
      width: this.element.clientWidth,
      height: this.element.clientHeight
    }
  }

  position(text: string, position: number, push: number = 2): number {
    // Let's do some BST
    let startOffset = 0
    let leadingSize = 0 // here for centering the character at the end
    let start = 0
    let end = text.length

    let width = this.calculate(text).width

    if (position < start) {
      return 0
    }

    if (position > width) {
      return text.length
    }

    do {
      const mid = Math.floor((end + start) / 2)

      const leading = text.substring(start, mid)
      leadingSize = this.calculate(leading).width

      if (startOffset + leadingSize < position) {
        // region misses
        startOffset += leadingSize
        start = mid
      } else {
        end = mid
      }
    } while (start + 1 < end)

    const offset = Math.round((position + push - startOffset) / leadingSize)

    return start + Math.min(offset, 1)
  }

  constructor(classes: string) {
    this.element = document.createElement('div')

    this.element.className = classes

    document.body.appendChild(this.element)
  }
}

export const regular = new SizeCalculator('h-6 text-sm font-mono absolute top-0 -z-10 whitespace-pre')
