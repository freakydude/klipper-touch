/** @type {import('tailwindcss').Config} */
module.exports = {
  content: ['./src/**/*.{html,js,svelte,ts}'],
  theme: {
    extend: {},
    screens: {
      xs: '480px',
      sm: '800px',
      md: '1024px'
    }
  },
  plugins: []
};
