package kr.co.mcplink.domain.mcpserver.dto;

import kr.co.mcplink.domain.mcpserver.entity.SecurityRank;
import lombok.Builder;

@Builder
public record McpDetailDataDto (
    long id,
    String type,
    String url,
    int stars,
    int views,
    boolean official,
    boolean scanned,
    SecurityRank securityRank,
    McpServerDetailDto mcpServer
) {

}