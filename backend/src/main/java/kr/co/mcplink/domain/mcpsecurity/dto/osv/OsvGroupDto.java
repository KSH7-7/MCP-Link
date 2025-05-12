package kr.co.mcplink.domain.mcpsecurity.dto.osv;

import com.fasterxml.jackson.annotation.JsonProperty;
import java.util.List;

/**
 * "packages" 객체 내의 "groups" 배열 항목을 나타냅니다. (관련된 취약점 ID 그룹)
 */
public record OsvGroupDto(
	List<String> ids,
	List<String> aliases,
	@JsonProperty("max_severity")
	String maxSeverity // 그룹 내 최대 위험도 (JSON에 따라 숫자 또는 문자열일 수 있음)
) {}