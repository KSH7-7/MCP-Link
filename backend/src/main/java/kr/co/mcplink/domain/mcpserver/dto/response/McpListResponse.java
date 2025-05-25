package kr.co.mcplink.domain.mcpserver.dto.response;

import kr.co.mcplink.domain.mcpserver.dto.McpSummaryDataDto;
import kr.co.mcplink.domain.mcpserver.dto.PageInfoDto;

import java.util.List;

public record McpListResponse(
    PageInfoDto pageInfo,
    List<McpSummaryDataDto> mcpServers
) {

}