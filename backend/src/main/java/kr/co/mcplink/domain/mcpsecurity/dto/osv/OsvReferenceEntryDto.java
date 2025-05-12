package kr.co.mcplink.domain.mcpsecurity.dto.osv;

/**
 * 취약점 관련 참고 자료(URL) 정보를 나타냅니다.
 */
public record OsvReferenceEntryDto(
	String type,
	String url
) {}
