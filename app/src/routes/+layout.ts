// Tauri doesn't have a Node.js server to do proper SSR
// so we will use adapter-static to prerender the app (SSG)
// See: https://v2.tauri.app/start/frontend/sveltekit/ for more info
import { browser } from "$app/environment" // 브라우저 환경인지 확인
// import { goto } from "$app/navigation" // 페이지 이동 함수
import { redirect } from "@sveltejs/kit" // redirect 함수 임포트
// import { exists, BaseDirectory } from "@tauri-apps/plugin-fs" // 경로를 plugin-fs로 변경
import type { LayoutLoad } from "./$types" // LayoutLoad 타입을 임포트합니다.
// import { appConfigDir } from '@tauri-apps/api/path'; // appConfigDir는 BaseDirectory 사용 시 불필요할 수 있습니다.

// 기존 설정 유지
export const prerender = true
export const ssr = false
export const csr = true

const CONFIG_FILE_NAME = "claude_desktop_config.json" // 확인할 설정 파일 이름
const FIRST_INSTALL_PATH = "/first-install"
const DEFAULT_PATH_AFTER_INSTALL = "/dashboard" // 설정 후 이동할 기본 경로

/** @type {import('./$types').LayoutLoad} */
export const load: LayoutLoad = async ({ url }) => {
  // 이 load 함수는 이제 특별한 역할을 하지 않거나,
  // 페이지 전환 시 필요한 최소한의 데이터 로딩만 수행합니다.
  // 현재는 반환할 데이터가 없으므로 빈 객체 반환.
  console.log(`[Layout Load] Navigating to: ${url.pathname}`) // 네비게이션 로깅 (선택 사항)
  return {}
}
