/** @type {import('tailwindcss').Config} */
module.exports = {
  content: ['./src/**/*.{html,js,svelte,ts}'],
  theme: {
    extend: {},
    screens: {
      'tft35': '480px',
      'tft50': '800px',
      'tft70': '1024px',
    },
  },
  plugins: []
};
