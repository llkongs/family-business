import { defineConfig } from 'vite'

// https://vitejs.dev/config/
export default defineConfig({
    // Base path for GitHub Pages deployment
    base: '/family-business/',

    build: {
        outDir: 'dist',
        assetsDir: 'assets',
        sourcemap: false,
        minify: 'esbuild',
    },

    server: {
        port: 5173,
        open: true,
    },
})
