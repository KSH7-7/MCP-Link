package kr.co.mcplink.domain.mcpsecurity.dto.osv;

import com.fasterxml.jackson.annotation.JsonProperty;
import java.util.List;

/**
 * 취약점의 영향을 받는 패키지 정보를 나타냅니다.
 */
public record OsvAffectedEntryDto(
	@JsonProperty("package")
	OsvAffectedPackageInfoDto packageInfo,
	List<OsvAffectedRangeDto> ranges,
	@JsonProperty("database_specific")
	OsvAffectedDatabaseSpecificDto databaseSpecific // "affected" 내부의 database_specific (필요시 정의)
) {}
