// Tauri doesn't have a Node.js server to do proper SSR
// 정적 파일을 생성하기 위한 설정
// See: https://v2.tauri.app/start/frontend/sveltekit/ for more info
import { vitePreprocess } from "@sveltejs/vite-plugin-svelte"
import adapter from "@sveltejs/adapter-static"
import { dirname, join } from "path"
import { fileURLToPath } from "url"

const __dirname = dirname(fileURLToPath(import.meta.url))

/** @type {import('@sveltejs/kit').Config} */
const config = {
  // preprocess 옵션
  preprocess: vitePreprocess(),

  kit: {
    // Tauri는 정적 사이트로 생성
    adapter: adapter(),
    alias: {
      $app: join(__dirname, ".svelte-kit/runtime/app"),
    },
  },
}

export default config
