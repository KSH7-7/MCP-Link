package kr.co.mcplink.domain.mcpserver.dto;

import lombok.AllArgsConstructor;
import lombok.Builder;
import lombok.Getter;
import lombok.NoArgsConstructor;

@Getter
@Builder
@NoArgsConstructor
@AllArgsConstructor
public class McpDetailDataDto {
    private long id;
    private String type;
    private String url;
    private int stars;
    private int views;
    private boolean scanned;
    private McpDetailDto mcpServers;
}