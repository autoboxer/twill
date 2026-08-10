import js from '@eslint/js';
import { defineConfig } from 'eslint/config';
import globals from 'globals';
import vue from 'eslint-plugin-vue';

export default defineConfig([
  {
    ignores: [ 'dist/**', 'src-tauri/gen/**', 'src-tauri/target/**' ]
  },
  js.configs.recommended,
  ...vue.configs[ 'flat/recommended' ],
  {
    files: [ 'src/**/*.{js,vue}' ],
    languageOptions: {
      globals: globals.browser
    }
  },
  {
    files: [ '*.config.js' ],
    languageOptions: {
      globals: globals.node
    }
  },
  {
    files: [ '**/*.{js,mjs,cjs,vue}' ],
    rules: {
      'array-bracket-spacing': [
        'error',
        'always',
        {
          arraysInArrays: false,
          objectsInArrays: false
        }
      ],
      'brace-style': [
        'error',
        '1tbs',
        { allowSingleLine: false }
      ],
      'comma-dangle': [ 'error', 'never' ],
      'computed-property-spacing': [ 'error', 'always' ],
      'curly': [ 'error', 'all' ],
      'eol-last': [ 'error', 'always' ],
      'indent': [ 'error', 2 ],
      'no-multi-spaces': [
        'error',
        {
          exceptions: {
            ImportDeclaration: true,
            Property: true,
            VariableDeclarator: true
          },
          ignoreEOLComments: true
        }
      ],
      'no-trailing-spaces': 'error',
      'no-unused-vars': [
        'error',
        {
          args: 'after-used',
          ignoreRestSiblings: true
        }
      ],
      'object-curly-spacing': [ 'error', 'always' ],
      'quotes': [
        'error',
        'single',
        {
          allowTemplateLiterals: true,
          avoidEscape: true
        }
      ],
      'semi': [ 'error', 'always' ],
      'space-in-parens': [
        'error',
        'always',
        { exceptions: [ '{}', '[]', 'empty' ] }
      ],
      'space-infix-ops': 'error',
      'template-curly-spacing': [ 'error', 'always' ],
      'vue/block-lang': [ 'error', { script: { lang: 'js' } }],
      'vue/block-order': [ 'error', { order: [ 'script', 'template', 'style' ] }],
      'vue/html-indent': [ 'error', 2 ],
      'vue/max-attributes-per-line': [
        'error',
        {
          multiline: { max: 1 },
          singleline: { max: 3 }
        }
      ],
      'vue/no-unused-properties': [
        'error',
        {
          groups: [ 'props', 'data', 'computed', 'methods', 'setup' ]
        }
      ],
      'vue/singleline-html-element-content-newline': 'off'
    }
  }
]);
