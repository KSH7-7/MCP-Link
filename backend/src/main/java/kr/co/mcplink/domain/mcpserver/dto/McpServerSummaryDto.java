package kr.co.mcplink.domain.mcpserver.dto;

import lombok.Builder;

@Builder
public record McpServerSummaryDto (
    String name,
    String description
) {

}