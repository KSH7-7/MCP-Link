package kr.co.mcplink.domain.mcpsecurity.dto.osv;

/**
 * 영향을 받는 패키지의 상세 정보를 나타냅니다.
 */
public record OsvAffectedPackageInfoDto(
	String ecosystem,
	String name,
	String purl
) {}