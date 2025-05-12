// 이 파일에 다른 서버 전용 로직이 없다면 내용을 비우거나 파일을 삭제하세요.
// 만약 다른 로직이 있다면 그것만 남겨둡니다.

/** @type {import('@sveltejs/kit').Handle} */
export async function handle({ event, resolve }) {
  // Tauri API 관련 코드는 모두 제거합니다.
  const response = await resolve(event)
  return response
}
