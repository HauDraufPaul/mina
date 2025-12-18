/** @type {import('tailwindcss').Config} */
export default {
  content: [
    "./index.html",
    "./src/**/*.{js,ts,jsx,tsx}",
  ],
  theme: {
    extend: {
      colors: {
        neon: {
          cyan: "#00d9ff",
          green: "#00ff88",
          amber: "#ffb000",
          red: "#ff2d55",
        },
      },
      fontFamily: {
        mono: ["'JetBrains Mono'", "monospace"],
      },
      backdropBlur: {
        glass: "28px",
      },
    },
  },
  plugins: [],
}

