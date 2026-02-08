// eslint.config.ts
import { includeIgnoreFile } from '@eslint/compat';
import js from '@eslint/js';
import prettier from 'eslint-config-prettier';
import svelte from 'eslint-plugin-svelte';
import { defineConfig } from 'eslint/config';
import globals from 'globals';
import path from 'node:path';
import { configs as tsConfigs, parser as tsParser } from 'typescript-eslint';
import svelteConfig from './svelte.config.js';

const gitignorePath = path.resolve(import.meta.dirname, '.gitignore');

export default defineConfig(
  includeIgnoreFile(gitignorePath),
  {
    ignores: ['src-tauri/target/**', 'src-tauri/gen/schemas/**']
  },
  js.configs.recommended,
  ...tsConfigs.recommended,
  ...svelte.configs['flat/recommended'], // Empfohlener Weg für Flat Config
  prettier,
  ...svelte.configs['flat/prettier'], // Spezielle Flat-Variante von Prettier-Svelte

  {
    languageOptions: {
      globals: {
        ...globals.browser,
        ...globals.node
      }
    },
    rules: {
      // Deaktiviert no-undef, da TypeScript das besser im Griff hat
      'no-undef': 'off'
    }
  },

  {
    files: ['**/*.svelte', '**/*.svelte.ts', '**/*.svelte.js'],
    languageOptions: {
      parserOptions: {
        projectService: true,
        extraFileExtensions: ['.svelte'],
        parser: tsParser,
        // Übergibt die Svelte-Konfiguration an den Parser
        svelteConfig
      }
    }
  }
);
