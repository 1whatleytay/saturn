/** @type {import('tailwindcss').Config} */
module.exports = {
  content: [
    "./index.html",
    "./src/**/*.{vue,js,ts,jsx,tsx}"
  ],

  theme: {
    extend: {
      colors: {
        'saturn-background': '#2C3350',
        'saturn-highlight': '#FFC35D'
      },

      keyframes: {
        blink: {
          '0%, 100%': { opacity: 0 },
          '50%': { opacity: 1 },
        }
      },

      animation: {
        blink: 'blink 1s steps(1, end) infinite'
      }
    },
  },

  plugins: [],
}
