import {defineConfig} from 'vite'
import vue from '@vitejs/plugin-vue'

// https://vitejs.dev/config/
export default defineConfig({
    base: './',
    server: {
        port: 30000,
        proxy: {
            '/api': {
                target: 'http://127.0.0.1:7845',
                changeOrigin: true,
            }
        }
    },
    plugins: [vue()],
})
