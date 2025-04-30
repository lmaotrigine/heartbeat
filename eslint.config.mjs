import globals from "globals";
import path from "node:path";
import { fileURLToPath } from "node:url";
import js from "@eslint/js";
import { FlatCompat } from "@eslint/eslintrc";

const __filename = fileURLToPath(import.meta.url);
const __dirname = path.dirname(__filename);
const compat = new FlatCompat({
    baseDirectory: __dirname,
    recommendedConfig: js.configs.recommended,
    allConfig: js.configs.all
});

export default [...compat.extends("eslint:recommended"), {
    languageOptions: {
        globals: {
            ...globals.browser,
        },

        ecmaVersion: 8,
        sourceType: "module",
    },

    rules: {
        // possible errors
        "no-loss-of-precision": "error",
        "no-template-curly-in-string": "error",
        "no-sparse-arrays": "error",
        "no-unreachable-loop": "error",
        "no-useless-backreference": "error",
        // best practices
        "default-case-last": "error",
        "default-param-last": "error",
        "dot-location": ["error", "property"],
        eqeqeq: "error",
        "grouped-accessor-pairs": ["error", "getBeforeSet"],
        "guard-for-in": "error",
        "no-alert": "error",
        "no-caller": "error",
        "no-constructor-return": "error",
        "no-else-return": "error",
        "no-eval": "error",
        "no-extend-native": "error",
        "no-extra-bind": "error",
        "no-floating-decimal": "error",
        "no-implied-eval": "error",
        "no-iterator": "error",
        "no-labels": "error",
        "no-lone-blocks": "error",
        "no-loop-func": "error",
        "no-multi-spaces": "error",
        "no-new-wrappers": "error",
        "no-octal": "error",
        "no-octal-escape": "error",
        "no-proto": "error",
        "no-return-assign": ["error", "except-parens"],
        "no-return-await": "error",
        "no-self-compare": "error",
        "no-sequences": "error",
        "no-throw-literal": "error",
        "no-unmodified-loop-condition": "error",
        "no-unused-expressions": "error",
        "no-warning-comments": "error",
        "prefer-promise-reject-errors": "error",
        radix: "error",
        "wrap-iife": "error",
        yoda: "error",
        // vars
        "no-undefined": "error",
        "array-bracket-newline": ["error", "consistent"],
        "array-bracket-spacing": ["error", "always"],
        "block-spacing": "error",

        "brace-style": ["error", "1tbs", {
            allowSingleLine: true,
        }],

        "comma-dangle": ["error", {
            arrays: "always-multiline",
            objects: "always-multiline",
            imports: "always-multiline",
            exports: "always-multiline",
            functions: "never",
        }],

        "comma-spacing": "error",
        "comma-style": "error",
        "computed-property-spacing": "error",
        "eol-last": "error",
        "func-call-spacing": "error",
        "func-name-matching": "error",
        "function-call-argument-newline": ["error", "consistent"],
        "function-paren-newline": ["error", "multiline-arguments"],
        indent: ["error", 2],
        "key-spacing": "error",
        "keyword-spacing": "error",
        "linebreak-style": ["error", "unix"],
        "lines-between-class-members": "error",

        "max-len": ["error", {
            code: 120,
            ignoreComments: false,
            tabWidth: 2,
        }],

        "multiline-ternary": ["error", "always-multiline"],

        "new-cap": ["error", {
            properties: false,
        }],

        "new-parens": "error",
        "no-array-constructor": "error",
        "no-lonely-if": "error",
        "no-mixed-operators": "error",
        "no-multi-assign": "error",
        "no-multiple-empty-lines": "error",
        "no-new-object": "error",
        "no-trailing-spaces": "error",
        "no-unneeded-ternary": "error",
        "no-whitespace-before-property": "error",
        "nonblock-statement-body-position": "error",

        "object-curly-newline": ["error", {
            multiline: true,
            consistent: true,
        }],

        "object-curly-spacing": ["error", "always"],

        "object-property-newline": ["error", {
            allowAllPropertiesOnSameLine: true,
        }],

        "one-var": ["error", "never"],
        "operator-assignment": "error",
        "operator-linebreak": ["error", "before"],
        "padded-blocks": ["error", "never"],
        "prefer-exponentiation-operator": "error",
        quotes: ["error", "single"],
        semi: ["error", "always"],
        "semi-style": ["error", "last"],
        "space-before-blocks": "error",

        "space-before-function-paren": ["error", {
            anonymous: "always",
            asyncArrow: "always",
            named: "never",
        }],

        "space-in-parens": "error",
        "space-infix-ops": "error",

        "space-unary-ops": ["error", {
            words: true,
            nonwords: false,
        }],

        "switch-colon-spacing": "error",
        "template-tag-spacing": "error",
        "unicode-bom": "error",
        // es6
        "arrow-body-style": "error",
        "arrow-parens": "error",
        "arrow-spacing": "error",

        "generator-star-spacing": ["error", {
            named: "after",
            anonymous: "after",
            method: "before",
        }],

        "no-useless-computed-key": "error",
        "no-useless-rename": "error",
        "no-var": "error",
        "object-shorthand": ["error", "never"],
        "prefer-arrow-callback": "error",
        "prefer-rest-params": "error",
        "prefer-template": "error",
        "rest-spread-spacing": "error",
        "symbol-description": "error",
        "template-curly-spacing": "error",
        "yield-star-spacing": "error",
    },
}];
