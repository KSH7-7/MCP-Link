/**
 * 오류 처리 유틸리티 파일
 * 백엔드 오류 메시지를 해석하고 사용자 친화적인 메시지로 변환합니다.
 */

import toastStore from './stores/toast';

// 오류 유형
export enum ErrorType {
  API = 'API 오류',
  FILESYSTEM = '파일 시스템 오류',
  CONFIG = '설정 오류',
  JSON = 'JSON 오류',
  OS = 'OS 오류',
  NOTIFICATION = '알림 오류',
  PERMISSION = '권한 오류',
  NETWORK = '네트워크 오류',
  GENERIC = '오류'
}

// 오류 정보 인터페이스
export interface ErrorInfo {
  type: ErrorType;
  message: string;
  details?: string;
  action?: string;
}

/**
 * 백엔드 오류 메시지를 파싱하여 구조화된 오류 정보로 변환
 */
export function parseErrorMessage(errorMessage: string): ErrorInfo {
  // 오류 유형 식별 (백엔드에서 일관된 형식으로 온다고 가정)
  let type = ErrorType.GENERIC;
  let message = errorMessage;
  let details = '';
  
  // 오류 유형 파싱
  if (errorMessage.includes('API 오류')) {
    type = ErrorType.API;
  } else if (errorMessage.includes('파일 시스템 오류')) {
    type = ErrorType.FILESYSTEM;
  } else if (errorMessage.includes('설정 오류')) {
    type = ErrorType.CONFIG;
  } else if (errorMessage.includes('JSON 오류')) {
    type = ErrorType.JSON;
  } else if (errorMessage.includes('OS 오류')) {
    type = ErrorType.OS;
  } else if (errorMessage.includes('알림 오류')) {
    type = ErrorType.NOTIFICATION;
  } else if (errorMessage.includes('권한 오류')) {
    type = ErrorType.PERMISSION;
  } else if (errorMessage.includes('네트워크 오류')) {
    type = ErrorType.NETWORK;
  }
  
  // 기본 메시지와 상세 정보 분리
  const colonIndex = message.indexOf(':');
  if (colonIndex > 0) {
    details = message.substring(colonIndex + 1).trim();
    message = message.substring(0, colonIndex).trim();
  }
  
  // 사용자 친화적인 메시지로 변환
  const userFriendlyMessage = getUserFriendlyMessage(type, details);
  
  return {
    type,
    message: userFriendlyMessage,
    details,
    action: getSuggestedAction(type, details)
  };
}

/**
 * 오류 유형과 세부 정보에 따라 사용자 친화적인 메시지 생성
 */
function getUserFriendlyMessage(type: ErrorType, details: string): string {
  switch (type) {
    case ErrorType.API:
      return '서버와 통신 중 문제가 발생했습니다';
    case ErrorType.FILESYSTEM:
      return '파일 읽기/쓰기 문제가 발생했습니다';
    case ErrorType.CONFIG:
      if (details.includes('존재하지 않습니다')) {
        return '설정 파일을 찾을 수 없습니다';
      }
      return '설정 파일 문제가 발생했습니다';
    case ErrorType.JSON:
      return '데이터 형식 오류가 발생했습니다';
    case ErrorType.OS:
      return '시스템 작업을 수행할 수 없습니다';
    case ErrorType.NOTIFICATION:
      return '알림을 표시할 수 없습니다';
    case ErrorType.PERMISSION:
      return '필요한 권한이 없습니다';
    case ErrorType.NETWORK:
      if (details.includes('connect')) {
        return '서버에 연결할 수 없습니다';
      }
      return '네트워크 오류가 발생했습니다';
    default:
      return '예상치 못한 오류가 발생했습니다';
  }
}

/**
 * 오류 유형과 세부 정보에 따라 사용자에게 제안할 조치 생성
 */
function getSuggestedAction(type: ErrorType, details: string): string {
  switch (type) {
    case ErrorType.API:
      return '잠시 후 다시 시도해 주세요';
    case ErrorType.FILESYSTEM:
      return '관리자 권한으로 실행하거나 다른 경로를 시도해 보세요';
    case ErrorType.CONFIG:
      if (details.includes('존재하지 않습니다')) {
        return '설정 메뉴에서 기본 설정을 복원해 보세요';
      }
      return '설정 파일을 재생성하려면 설정 메뉴를 이용하세요';
    case ErrorType.NETWORK:
      return '인터넷 연결을 확인하고 다시 시도해 주세요';
    case ErrorType.PERMISSION:
      return '관리자 권한으로 앱을 실행해 보세요';
    default:
      return '앱을 재시작하여 다시 시도해 보세요';
  }
}

/**
 * 오류를 토스트 메시지로 표시
 */
export function showErrorToast(error: Error | string, position = 'bottom-center') {
  const errorMessage = typeof error === 'string' ? error : error.message;
  const errorInfo = parseErrorMessage(errorMessage);
  
  let toastMessage = `${errorInfo.message}`;
  if (errorInfo.action) {
    toastMessage += `. ${errorInfo.action}`;
  }
  
  toastStore.update(state => ({
    ...state,
    show: true,
    message: toastMessage,
    type: 'error',
    duration: 5000,
    position
  }));
  
  // 콘솔에 디버그 로그 표시
  console.error('Error:', {
    type: errorInfo.type,
    message: errorInfo.message,
    details: errorInfo.details,
    originalError: error
  });
  
  // 자동 숨김
  setTimeout(() => {
    toastStore.update(state => ({
      ...state,
      show: false
    }));
  }, 5000);
}

/**
 * Tauri API 호출에서 발생한 오류를 처리
 */
export async function handleTauriError<T>(
  apiCall: () => Promise<T>,
  fallbackValue?: T,
  options = { showToast: true, position: 'bottom-center' }
): Promise<T> {
  try {
    return await apiCall();
  } catch (error) {
    // 오류 토스트 표시 (옵션에 따라)
    if (options.showToast) {
      showErrorToast(error, options.position as string);
    }
    
    // fallbackValue가 제공되었으면 반환
    if (fallbackValue !== undefined) {
      return fallbackValue;
    }
    
    // 오류 다시 throw
    throw error;
  }
}