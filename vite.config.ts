import {defineConfig, loadEnv} from "vite";
import vue from "@vitejs/plugin-vue";
import {quasar, transformAssetUrls} from "@quasar/vite-plugin";

export default defineConfig(({mode}) => {
    const env = loadEnv(mode, process.cwd());
    const tauriDevHost = process.env.TAURI_DEV_HOST || env.VITE_TAURI_DEV_HOST;
    const hmr = tauriDevHost ? {
        protocol: "ws",
        host: tauriDevHost,
        port: 5174,
    } : undefined;
    return {
        plugins: [
            vue({
                template: {transformAssetUrls}
            }),
            quasar({
                sassVariables: '/src/quasar-variables.sass'
            })
        ],
        clearScreen: false,
        server: {
            port: 5173,
            strictPort: true,
            host: tauriDevHost ? "0.0.0.0" : false,
            hmr,
            watch: {
                ignored: ["**/src-tauri/**"],
            },
        },
    }
});
