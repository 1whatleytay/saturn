/** @type {import('tailwindcss').Config} */
module.exports = {
  content: ['./index.html', './src/**/*.{vue,js,ts,jsx,tsx}'],

  theme: {
    extend: {
      maxWidth: {
        '3/4': '75%'
      },
      colors: {
        'breakpoint-neutral': '#2b1515',
        'breakpoint-stopped': '#3f3b18',
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
      },

      animation: {
        blink: 'blink 1s steps(1, end) infinite',
        bump: 'bump 0.15s',
      },
    },
  },

  plugins: [],
}
