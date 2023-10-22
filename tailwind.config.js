/** @type {import('tailwindcss').Config} */
export default {
  content: [
    "./index.html",
    "./src/**/*.{js,ts,jsx,tsx,vue,css}",
    "./node_modules/equal-vue/dist/theme/*.{js,ts,json}",
  ],
  theme: {
    extend: {},
  },
  darkMode: "class",
  plugins: [],
};
