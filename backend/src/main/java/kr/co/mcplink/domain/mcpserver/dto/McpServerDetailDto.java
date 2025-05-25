package kr.co.mcplink.domain.mcpserver.dto;

import com.fasterxml.jackson.annotation.JsonInclude;
import lombok.Builder;

import java.util.List;
import java.util.Map;

@Builder
public record McpServerDetailDto (
    String name,
    String description,
    String command,
    List<String> args,

    @JsonInclude(JsonInclude.Include.NON_NULL)
    Map<String, String> env
) {

}