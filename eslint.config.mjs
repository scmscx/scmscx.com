import globals from "globals";
import pluginJs from "@eslint/js";
import tseslint from "typescript-eslint";
import pluginSolidConfig from "eslint-plugin-solid/configs/recommended.js";
// import pluginViteConfig from "eslint-plugin-vite/configs/recommended.js";

export default [
  {
    languageOptions: { globals: globals.browser },
  },
  pluginJs.configs.recommended,
  ...tseslint.configs.recommended,
  pluginSolidConfig,
  // pluginViteConfig,
  {
    rules: {
      "@typescript-eslint/no-explicit-any": "off",
      "solid/reactivity": "error",
      "@typescript-eslint/no-unused-vars": "off",
    },
  },
];
