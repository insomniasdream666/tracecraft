/** @type {import('tailwindcss').Config} */
export default {
  content: ["./index.html", "./src/**/*.{js,ts,jsx,tsx}"],
  theme: {
    extend: {
      colors: {
        copper: { DEFAULT: "#D4783C", dark: "#A55A2A" },
        bg: { deep: "#1A1B2E", surface: "#232540", elevated: "#2D2F52", canvas: "#0F1020" },
        text: { primary: "#E8E9F3", secondary: "#9496B8", muted: "#5C5E7E" },
        border: { DEFAULT: "#343760", subtle: "#282A4A" },
        error: "#E6454A",
        warning: "#F0A030",
        success: "#42BE65",
        info: "#5096E0",
      },
    },
  },
  plugins: [],
};
