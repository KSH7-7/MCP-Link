package kr.co.mcplink.domain.mcpserver.dto.common;

import lombok.AllArgsConstructor;
import lombok.Builder;
import lombok.Getter;
import lombok.NoArgsConstructor;

@Getter
@Builder
@NoArgsConstructor
@AllArgsConstructor
public class McpSummaryDataDto {
    private long id;
    private String type;
    private String url;
    private int stars;
    private int views;
    private boolean scanned;
    private McpSummaryDto mcpServers;
}