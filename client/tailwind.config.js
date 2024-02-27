/** @type {import('tailwindcss').Config} */
module.exports = {
  content: ["./src/**/*.{js,jsx,ts,tsx}"],
  darkMode: 'class',
  theme: {
    extend: {
      maxHeight: {
        350: "350px",
      },
    },
  },
  plugins: [],
};
