/** @type {import('tailwindcss').Config} */
module.exports = {
  content: [ "./crates/**/*.rs" ],
  theme: {
    extend: {
      colors: {
        grass: {
          1: "color(display-p3 0.062 0.083 0.067)",
          2: "color(display-p3 0.083 0.103 0.085)",
          3: "color(display-p3 0.118 0.163 0.122)",
          4: "color(display-p3 0.142 0.225 0.15)",
          5: "color(display-p3 0.178 0.279 0.186)",
          6: "color(display-p3 0.217 0.337 0.224)",
          7: "color(display-p3 0.258 0.4 0.264)",
          8: "color(display-p3 0.302 0.47 0.305)",
          9: "color(display-p3 0.38 0.647 0.378)",
          10: "color(display-p3 0.426 0.694 0.426)",
          11: "color(display-p3 0.535 0.807 0.542)",
          12: "color(display-p3 0.797 0.936 0.776)",
        },
      },
      fontFamily: {
  			'sans': [
          'inter', 'ui-sans-serif', 'system-ui', 'sans-serif', "Apple Color Emoji",
          "Segoe UI Emoji", "Segoe UI Symbol", "Noto Color Emoji"
        ],
      },
      keyframes: {
        'fade-in': {
          '0%': { opacity: '0' },
          '100%': { opacity: '1' },
        },
        'fade-out': {
          '0%': { opacity: '1' },
          '100%': { opacity: '0' },
        },
      },
      animation: {
        'fade-in': 'fade-in 0.3s ease-in-out',
        'fade-out': 'fade-out 0.3s ease-in-out',
      },
    },
  },

  plugins: [require("rippleui")],
}
