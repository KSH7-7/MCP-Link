package kr.co.mcplink.domain.mcpsecurity.dto.osv;

/**
 * "vulnerabilities" 객체 내의 "severity" 배열 항목을 나타냅니다. (예: CVSS 점수)
 */
public record OsvSeverityScoreEntryDto(
	String type,
	String score
) {}