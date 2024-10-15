/** @type {import('tailwindcss').Config} */
module.exports = {
  content: [ "./crates/**/*.rs" ],
  theme: {
    fontFamily: {
			'sans': [
        'inter', 'ui-sans-serif', 'system-ui', 'sans-serif', "Apple Color Emoji",
        "Segoe UI Emoji", "Segoe UI Symbol", "Noto Color Emoji"
      ],
    },
  },
  plugins: [require("rippleui")],
}
