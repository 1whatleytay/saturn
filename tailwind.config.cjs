/** @type {import('tailwindcss').Config} */
module.exports = {
  content: [
    "./index.html",
    "./src/**/*.{vue,js,ts,jsx,tsx}"
  ],
  theme: {
    extend: {
      colors: {
        dark: '#2C3350',
        light: '#FFC35D'
      }
    },
  },
  plugins: [],
}
