import { type Config } from "tailwindcss";
import tailwindForms from "tailwindcss/forms";
import colors from "tailwindcss/colors.js";

export default {
  content: [
    "{routes,islands,components}/**/*.{ts,tsx}",
  ],
  theme: {
    colors: {
      primary: colors.sky,
      red: colors.rose,
      gray: colors.stone,
      white: colors.white,
    },
  },
  plugins: [tailwindForms],
} satisfies Config;
