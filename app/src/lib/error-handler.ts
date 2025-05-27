import toastStore from "./stores/toast"

// error type
export enum ErrorType {
  API = "API Error",
  FILESYSTEM = "File System Error",
  CONFIG = "Config Error",
  JSON = "JSON Error",
  OS = "OS Error",
  NOTIFICATION = "Notification Error",
  PERMISSION = "Permission Error",
  NETWORK = "Network Error",
  GENERIC = "Error",
}

// error info interface
export interface ErrorInfo {
  type: ErrorType
  message: string
  details?: string
  action?: string
}

/**
 * parse backend error message into structured error info
 */
export function parseErrorMessage(errorMessage: string): ErrorInfo {
  // identify error type (assume consistent format from backend)
  let type = ErrorType.GENERIC
  let message = errorMessage
  let details = ""

  // parse error type
  if (errorMessage.includes("API Error")) {
    type = ErrorType.API
  } else if (errorMessage.includes("File System Error")) {
    type = ErrorType.FILESYSTEM
  } else if (errorMessage.includes("Config Error")) {
    type = ErrorType.CONFIG
  } else if (errorMessage.includes("JSON Error")) {
    type = ErrorType.JSON
  } else if (errorMessage.includes("OS Error")) {
    type = ErrorType.OS
  } else if (errorMessage.includes("Notification Error")) {
    type = ErrorType.NOTIFICATION
  } else if (errorMessage.includes("Permission Error")) {
    type = ErrorType.PERMISSION
  } else if (errorMessage.includes("Network Error")) {
    type = ErrorType.NETWORK
  }

  // separate basic message and details
  const colonIndex = message.indexOf(":")
  if (colonIndex > 0) {
    details = message.substring(colonIndex + 1).trim()
    message = message.substring(0, colonIndex).trim()
  }

  // convert to user friendly message
  const userFriendlyMessage = getUserFriendlyMessage(type, details)

  return {
    type,
    message: userFriendlyMessage,
    details,
    action: getSuggestedAction(type, details),
  }
}

/**
 * create user friendly message based on error type and details
 */
function getUserFriendlyMessage(type: ErrorType, details: string): string {
  switch (type) {
    case ErrorType.API:
      return "Server communication problem occurred"
    case ErrorType.FILESYSTEM:
      return "File read/write problem occurred"
    case ErrorType.CONFIG:
      if (details.includes("does not exist")) {
        return "Config file not found"
      }
      return "Config file problem occurred"
    case ErrorType.JSON:
      return "Data format error occurred"
    case ErrorType.OS:
      return "System operation not possible"
    case ErrorType.NOTIFICATION:
      return "Notification not possible"
    case ErrorType.PERMISSION:
      return "Required permission not found"
    case ErrorType.NETWORK:
      if (details.includes("connect")) {
        return "Server connection not possible"
      }
      return "Network error occurred"
    default:
      return "Unexpected error occurred"
  }
}

/**
 * create suggested action based on error type and details
 */
function getSuggestedAction(type: ErrorType, details: string): string {
  switch (type) {
    case ErrorType.API:
      return "Please try again later"
    case ErrorType.FILESYSTEM:
      return "Run with admin permission or try another path"
    case ErrorType.CONFIG:
      if (details.includes("does not exist")) {
        return "Try restoring default settings in the settings menu"
      }
      return "Try recreating the config file using the settings menu"
    case ErrorType.NETWORK:
      return "Check your internet connection and try again"
    case ErrorType.PERMISSION:
      return "Run the app with admin permission"
    default:
      return "Restart the app and try again"
  }
}

/**
 * show error toast
 */
export function showErrorToast(error: Error | string, position = "bottom-center") {
  const errorMessage = typeof error === "string" ? error : error.message
  const errorInfo = parseErrorMessage(errorMessage)

  let toastMessage = `${errorInfo.message}`
  if (errorInfo.action) {
    toastMessage += `. ${errorInfo.action}`
  }

  toastStore.update((state) => ({
    ...state,
    show: true,
    message: toastMessage,
    type: "error",
    duration: 5000,
    position,
  }))

  // auto hide
  setTimeout(() => {
    toastStore.update((state) => ({
      ...state,
      show: false,
    }))
  }, 5000)
}

/**
 * handle error from Tauri API call
 */
export async function handleTauriError<T>(apiCall: () => Promise<T>, fallbackValue?: T, options = { showToast: true, position: "bottom-center" }): Promise<T> {
  try {
    return await apiCall()
  } catch (error) {
    // show error toast (depending on options)
    if (options.showToast) {
      showErrorToast(error, options.position as string)
    }

    // return fallbackValue if provided
    if (fallbackValue !== undefined) {
      return fallbackValue
    }

    // throw error again
    throw error
  }
}
