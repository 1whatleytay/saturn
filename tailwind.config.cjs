/** @type {import('tailwindcss').Config} */
module.exports = {
  darkMode: ['selector'],

  content: ['./index.html', './src/**/*.{vue,js,ts,jsx,tsx}'],

  theme: {
    extend: {
      maxWidth: {
        '3/4': '75%',
      },
      colors: {
        'breakpoint-neutral': '#2b1515',
        'breakpoint-neutral-light': '#e0cccc',
        'breakpoint-stopped': '#3f3b18',
        'breakpoint-stopped-light': '#e0decc',
        'readonly-neutral': '#2b2b15',
      },

      keyframes: {
        blink: {
          '0%, 100%': { opacity: 0 },
          '50%': { opacity: 1 },
        },
        bump: {
          '0%, 100%': { transform: 'scale(1.0)' },
          '50%': { transform: 'scale(1.2)' },
        },
        hide: {
          from: { opacity: 1 },
          to: { opacity: 0 },
        },
        slideIn: {
          from: {
            transform: 'translateX(calc(100% + var(--viewport-padding)))',
          },
          to: { transform: 'translateX(0)' },
        },
        swipeOut: {
          from: { transform: 'translateX(var(--reka-toast-swipe-end-x))' },
          to: { transform: 'translateX(calc(100% + var(--viewport-padding)))' },
        },
      },

      animation: {
        blink: 'blink 1s steps(1, end) infinite',
        bump: 'bump 0.15s',
        hide: 'hide 100ms ease-in',
        slideIn: 'slideIn 150ms cubic-bezier(0.16, 1, 0.3, 1)',
        swipeOut: 'swipeOut 100ms ease-out',
      },
    },
  },

  plugins: [],
}
