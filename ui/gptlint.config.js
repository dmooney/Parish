import { recommendedConfig } from 'gptlint'

/** @type {import('gptlint').GPTLintConfig[]} */
export default [
  // Include all built-in rules from gptlint.
  ...recommendedConfig,

  // Project-specific overrides.
  {
    files: ['src/**/*.ts', 'src/**/*.js', 'src/**/*.svelte'],
    ignores: ['src/**/*.test.ts', 'src/__mocks__/**', 'src/test-setup.ts'],

    rules: {
      // Requires .svelte-kit/tsconfig.json which only exists after `svelte-kit sync`.
      // TypeScript strictness is already enforced by svelte-check.
      'effective-tsconfig': 'off',

      // Not applicable — this is a Svelte project, not React.
      'react-avoid-class-components': 'off',
    },
  },
]
