/** @type {import('tailwindcss').Config} */
export default {
  content: ['./index.html', './src/**/*.{js,ts,jsx,tsx}'],
  theme: {
    extend: {
      colors: {
        ice: { 50: '#f0f9ff', 500: '#06b6d4', 600: '#0891b2', 950: '#082f49' },
      },
    },
  },
  plugins: [],
}
