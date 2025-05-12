package kr.co.mcplink.domain.mcpsecurity.dto.osv;

import java.util.List;

/**
 * "results" 배열의 각 항목을 나타냅니다.
 */
public record OsvResultDto(
	OsvSourceDto source,
	List<OsvPackageEntryDto> packages
) {}