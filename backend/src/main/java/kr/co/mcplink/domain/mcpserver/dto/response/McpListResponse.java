package kr.co.mcplink.domain.mcpserver.dto.response;

import com.fasterxml.jackson.annotation.JsonFormat;
import com.fasterxml.jackson.annotation.JsonInclude;
import kr.co.mcplink.domain.mcpserver.dto.common.McpSummaryDataDto;
import kr.co.mcplink.domain.mcpserver.dto.common.PageInfoDto;
import lombok.AllArgsConstructor;
import lombok.Builder;
import lombok.Getter;
import lombok.NoArgsConstructor;

import java.time.Instant;
import java.util.List;

@Getter
@Builder
@NoArgsConstructor
@AllArgsConstructor
@JsonInclude(JsonInclude.Include.NON_NULL)
public class McpListResponse {
    @JsonFormat(shape = JsonFormat.Shape.STRING, pattern = "yyyy-MM-dd'T'HH:mm:ss'Z'", timezone = "UTC")
    private Instant timestamp;
    private int status;
    private String message;
    private PageInfoDto pageInfo;
    private List<McpSummaryDataDto> data;
}