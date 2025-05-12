package kr.co.mcplink.domain.mcpserver.dto.request;

import java.util.List;

public record McpBatchRequest (
    List<Long> serverIds
) {

}