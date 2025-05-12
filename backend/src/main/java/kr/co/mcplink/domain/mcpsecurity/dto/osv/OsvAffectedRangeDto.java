package kr.co.mcplink.domain.mcpsecurity.dto.osv;

import java.util.List;

/**
 * 영향을 받는 버전 범위를 나타냅니다.
 */
public record OsvAffectedRangeDto(
	String type, // 예: "SEMVER"
	List<OsvAffectedEventDto> events
) {}
