package kr.co.mcplink.global.common;

import java.time.Instant;
import java.time.LocalDateTime;

import org.springframework.http.HttpStatus;
import org.springframework.http.MediaType;
import org.springframework.http.ResponseEntity;

import com.fasterxml.jackson.annotation.JsonFormat;

import lombok.Builder;
import lombok.Getter;

@Getter
@Builder
public class ErrorResponse {
	@JsonFormat(shape = JsonFormat.Shape.STRING, pattern = "yyyy-MM-dd HH:mm:ss", timezone = "Asia/Seoul")
	private Instant timestamp;
	private int status;
	private String error;
	private String message;
	private String path;

	public static ResponseEntity<ErrorResponse> toResponseEntity(HttpStatus httpStatus, String message, String path) {
		return ResponseEntity
			.status(httpStatus)
			.contentType(MediaType.APPLICATION_JSON)
			.body(ErrorResponse.builder()
				.timestamp(Instant.now())
				.status(httpStatus.value())
				.error(httpStatus.name())
				.message(message)
				.path(path)
				.build());
	}
}
