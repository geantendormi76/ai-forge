/** @type {import('tailwindcss').Config} */
export default {
  content: [
    "./index.html",
    "./src/**/*.{vue,js,ts,jsx,tsx}",
  ],
  theme: {
    extend: {
      colors: {
        'zidian-green': '#02c3b4',
        'zidian-gold': '#ffb74d',
      },
    },
  },
  plugins: [],
}
