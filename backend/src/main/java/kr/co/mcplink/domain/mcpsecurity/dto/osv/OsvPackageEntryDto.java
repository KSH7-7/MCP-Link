package kr.co.mcplink.domain.mcpsecurity.dto.osv;

import com.fasterxml.jackson.annotation.JsonProperty;
import java.util.List;

/**
 * "packages" 배열의 각 항목으로, 패키지 정보와 관련 취약점을 포함합니다.
 */
public record OsvPackageEntryDto(
	@JsonProperty("package") // JSON 필드명이 "package"인 경우
	OsvPackageInfoDto packageInfo,
	@JsonProperty("dependency_groups")
	List<String> dependencyGroups,
	List<OsvVulnerabilityDto> vulnerabilities,
	List<OsvGroupDto> groups
) {}