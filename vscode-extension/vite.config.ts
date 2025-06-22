import { defineConfig } from 'vite';
import { resolve } from 'path';

export default defineConfig({
  // Entry point for the extension
  build: {
    // Output directory
    outDir: 'out',
    // Library mode for VSCode extension
    lib: {
      entry: resolve(__dirname, 'src/extension.ts'),
      name: 'extension',
      fileName: 'extension',
      formats: ['cjs'] // CommonJS format for Node.js
    },
    // Rollup options
    rollupOptions: {
      // External dependencies that shouldn't be bundled
      external: [
        'vscode',
        'fs',
        'path',
        'os',
        'crypto',
        'child_process',
        'util',
        'events',
        'stream',
        'buffer',
        'url',
        'querystring'
      ],
      output: {
        // Global variables for external dependencies
        globals: {
          vscode: 'vscode'
        }
      }
    },
    // Target Node.js environment
    target: 'node18',
    // Generate source maps
    sourcemap: true,
    // Minify in production
    minify: 'esbuild',
    // Emit CJS
    commonjsOptions: {
      transformMixedEsModules: true
    }
  },
  // Resolve TypeScript files
  resolve: {
    extensions: ['.ts', '.js']
  },
  // Define for Node.js environment
  define: {
    global: 'globalThis'
  },
  // SSR mode for Node.js
  ssr: {
    target: 'node',
    noExternal: ['vscode-languageclient']
  }
});
