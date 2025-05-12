package kr.co.mcplink.domain.mcpsecurity.dto.osv;

import com.fasterxml.jackson.annotation.JsonProperty;

/**
 * "affected" 객체 내의 "database_specific" 객체를 나타냅니다.
 * (제공된 JSON 샘플에서는 source 만 있지만, 확장성을 위해 별도 DTO로 정의)
 */
public record OsvAffectedDatabaseSpecificDto(
	String source,
	@JsonProperty("last_known_affected_version_range") // 예시 필드
	String lastKnownAffectedVersionRange
) {}
