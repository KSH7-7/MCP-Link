import { invoke } from "@tauri-apps/api/core"

// display native notification
export async function showNotification(title: string, body: string, keyword?: string): Promise<boolean> {
  try {
    await invoke("show_notification", { title, body, keyword })
    return true
  } catch (error) {
    return false
  }
}

// called when the app is activated by URI scheme
export async function handleUriScheme(uri: string): Promise<boolean> {
  try {
    await invoke("handle_uri_scheme", { uri })
    return true
  } catch (error) {
    return false
  }
}
