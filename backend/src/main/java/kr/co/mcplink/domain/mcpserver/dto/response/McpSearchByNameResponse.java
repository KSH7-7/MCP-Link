package kr.co.mcplink.domain.mcpserver.dto.response;

import java.util.List;

import kr.co.mcplink.domain.mcpserver.dto.McpSummaryDataDto;
import kr.co.mcplink.domain.mcpserver.dto.PageInfoDto;

public record McpSearchByNameResponse(
	PageInfoDto pageInfo,
	List<McpSummaryDataDto> mcpSummaryData
) {
}
