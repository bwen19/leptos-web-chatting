/** @type {import('tailwindcss').Config} */
module.exports = {
  content: ["./app/src/**/*.rs"],
  darkMode: "class",
  theme: {
    extend: {
      colors: {
        container: "hsl(var(--container))",
        surface: {
          DEFAULT: "hsl(var(--surface))",
          on: "hsl(var(--surface-on))",
        },
        primary: {
          DEFAULT: "hsl(var(--primary))",
          on: "hsl(var(--primary-on))",
        },
        accent: {
          DEFAULT: "hsl(var(--accent))",
          on: "hsl(var(--accent-on))",
        },
        success: {
          DEFAULT: "hsl(var(--success))",
          on: "hsl(var(--success-on))",
        },
        danger: {
          DEFAULT: "hsl(var(--danger))",
          on: "hsl(var(--danger-on))",
        },
        muted: "hsl(var(--muted))",
        border: "hsl(var(--border))",
        ring: "hsl(var(--ring))",
      },
      keyframes: {
        "slide-in-up": {
          "0%": {
            visibility: "visible",
            transform: "translateY(100%)",
          },
          "100%": {
            transform: "translateY(0)",
          },
        },
        "slide-out-left": {
          "0%": {
            transform: "translateX(0)",
          },
          "100%": {
            visibility: "hidden",
            transform: "translateX(-100%)",
          },
        },
        "slide-in-down": {
          "0%": {
            visibility: "visible",
            transform: "translateY(-100%)",
          },
          "100%": {
            transform: "translateY(0)",
          },
        },
        "slide-out-up": {
          "0%": {
            transform: "translateY(0)",
          },
          "100%": {
            visibility: "hidden",
            transform: "translateY(-100%)",
          },
        },
        "fade-in": {
          "0%": {
            opacity: "0",
          },
          "100%": {
            opacity: "1",
          },
        },
        "fade-out": {
          "0%": {
            opacity: "1",
          },
          "100%": {
            opacity: "0",
          },
        },
      },
      animation: {
        "slide-in-up": "slide-in-up 200ms ease-out forwards",
        "slide-out-left": "slide-out-left 150ms ease-in forwards",
        "slide-in-down": "slide-in-down 200ms ease-out forwards",
        "slide-out-up": "slide-out-up 150ms ease-in forwards",
        "fade-in": "fade-in 200ms ease-in-out forwards",
        "fade-out": "fade-out 150ms ease-in-out forwards",
      },
    },
  },
  plugins: [],
};
