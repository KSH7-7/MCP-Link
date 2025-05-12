package kr.co.mcplink.domain.mcpsecurity.dto.osv;

import java.util.List;

/**
 * "experimental_config" 내의 "licenses" 객체를 나타냅니다.
 */
public record OsvLicensesConfigDto(
	boolean summary,
	List<String> allowlist
) {}
