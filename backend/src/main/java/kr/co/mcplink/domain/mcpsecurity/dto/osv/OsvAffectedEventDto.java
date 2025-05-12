package kr.co.mcplink.domain.mcpsecurity.dto.osv;

/**
 * 버전 범위의 시작(introduced)과 수정(fixed) 정보를 나타냅니다.
 */
public record OsvAffectedEventDto(
	String introduced,
	String fixed
) {}