package kr.co.mcplink.domain.mcpsecurity.dto.osv;

/**
 * 패키지의 상세 정보를 나타냅니다.
 */
public record OsvPackageInfoDto(
	String name,
	String version,
	String ecosystem
) {}