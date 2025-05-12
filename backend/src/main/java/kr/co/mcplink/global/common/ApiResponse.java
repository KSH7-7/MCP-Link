package kr.co.mcplink.global.common;

import java.time.Instant; // 시간 정보를 위해 Instant 사용
import java.util.List;    // 데이터 목록을 위해 List 사용
import java.util.Objects; // 필드 null 체크 등에 활용 (기본 record 생성자 외 커스텀 시)

import com.fasterxml.jackson.annotation.JsonFormat;

/**
 * API 공통 응답을 위한 Record
 *
 * @param timestamp 응답 생성 시간
 * @param code    응답 상태 코드 (애플리케이션 레벨 코드, 예: "SUCCESS", "ERROR_INVALID_INPUT")
 * @param message 응답 메시지
 * @param data    실제 응답 데이터 목록
 * @param <T>     응답 데이터 목록의 요소 타입
 */
public record ApiResponse<T>(

	@JsonFormat(shape = JsonFormat.Shape.STRING, pattern = "yyyy-MM-dd HH:mm:ss", timezone = "Asia/Seoul")
	Instant timestamp,
	String code,
	String message,
	T data
) {
	public ApiResponse {
		Objects.requireNonNull(timestamp, "timestamp는 null이 될 수 없습니다.");
		Objects.requireNonNull(message, "message는 null이 될 수 없습니다.");
		Objects.requireNonNull(code, "code는 null이 될 수 없습니다.");
	}

	public static <T> ApiResponse<List<T>> success(String code, String message, List<T> data) {
		return new ApiResponse<>(Instant.now(), code, message, data);
	}

	public static <T> ApiResponse<T> success(String code, String message, T data) {
		return new ApiResponse<>(Instant.now(), code, message, data);
	}

	public static <T> ApiResponse<T> error(String code, String message) {
		return new ApiResponse<>(Instant.now(), code, message, null);
	}
}