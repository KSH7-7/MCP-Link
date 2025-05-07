package kr.co.mcplink.domain.mcpserver.dto;

import com.fasterxml.jackson.annotation.JsonInclude;
import lombok.AllArgsConstructor;
import lombok.Builder;
import lombok.Getter;
import lombok.NoArgsConstructor;

import java.util.List;
import java.util.Map;

@Getter
@Builder
@NoArgsConstructor
@AllArgsConstructor
public class McpDetailDto {
    private String name;
    private String description;
    private String command;
    private List<String> args;

    @JsonInclude(JsonInclude.Include.NON_NULL)
    private Map<String, String> env;
}