package kr.co.mcplink.domain.mcpserver.dto.common;

import lombok.AllArgsConstructor;
import lombok.Builder;
import lombok.Getter;
import lombok.NoArgsConstructor;

@Getter
@Builder
@NoArgsConstructor
@AllArgsConstructor
public class McpSummaryDto {
    private String name;
    private String description;
}