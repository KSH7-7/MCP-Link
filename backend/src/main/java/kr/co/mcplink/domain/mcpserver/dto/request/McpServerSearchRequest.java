package kr.co.mcplink.domain.mcpserver.dto.request;

import java.util.Optional;

import lombok.Builder;

@Builder
public record McpServerSearchRequest(
	String mcpServerName,
	Optional<Integer> size,
	Optional<Long> cursorId
) {
}