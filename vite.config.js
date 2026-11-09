import { cpSync, existsSync } from 'node:fs';
import { networkInterfaces } from 'node:os';
import { fileURLToPath, URL } from 'node:url';
import { defineConfig } from 'vite';
import tailwindcss from '@tailwindcss/vite';
import laravel from 'laravel-vite-plugin';

// laravel-vite-plugin 1.2 does not copy images and fonts into the build, so
// `asset('resources/images/x.svg')` would have nothing to point at. Copy them
// into public/build/resources, next to the hashed files. The site has none yet,
// so this does nothing until one is added.
function copyStaticAssets() {
    return {
        name: 'rocketboost:copy-static-assets',
        apply: 'build',
        closeBundle() {
            for (const dir of ['images', 'fonts']) {
                const source = `resources/${dir}`;
                if (existsSync(source)) {
                    cpSync(source, `public/build/resources/${dir}`, { recursive: true });
                }
            }
        },
    };
}

const alias = (path) => fileURLToPath(new URL(path, import.meta.url));

// The dev server URL that Propel reads from public/hot. Vite's default would be
// localhost, which a phone on the same network cannot reach, so use this
// machine's LAN address. It is read when `npm run dev` starts, so restart the
// dev server after the address changes.
function lanOrigin(port = 5173) {
    for (const addresses of Object.values(networkInterfaces())) {
        for (const address of addresses ?? []) {
            if (address.family === 'IPv4' && !address.internal) {
                return `http://${address.address}:${port}`;
            }
        }
    }
    return undefined;
}

export default defineConfig({
    plugins: [
        tailwindcss(),
        laravel({
            input: ['resources/css/app.css'],
            // Views are read at runtime in a debug build and are not part of the
            // Vite graph, so a view edit reloads the page instead of rebuilding.
            refresh: ['resources/views/**/*.propel'],
        }),
        copyStaticAssets(),
    ],
    server: {
        host: true,
        origin: lanOrigin(),
    },
    resolve: {
        alias: {
            '@styles': alias('./resources/css'),
            '@fonts': alias('./resources/fonts'),
            '@images': alias('./resources/images'),
        },
    },
});
