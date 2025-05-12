package kr.co.mcplink.domain.mcpsecurity.dto.osv;

/**
 * 스캔 대상 소스 정보를 나타냅니다. (예: lockfile 경로)
 */
public record OsvSourceDto(
	String path,
	String type
) {}