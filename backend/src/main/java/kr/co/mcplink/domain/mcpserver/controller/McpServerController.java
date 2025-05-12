package kr.co.mcplink.domain.mcpserver.controller;

import kr.co.mcplink.domain.mcpserver.dto.request.McpBatchRequest;
import kr.co.mcplink.domain.mcpserver.dto.response.*;
import kr.co.mcplink.domain.mcpserver.service.core.McpServerService;
import kr.co.mcplink.global.common.ApiResponse;
import lombok.RequiredArgsConstructor;
import org.springframework.web.bind.annotation.*;

import java.util.List;

@RestController
@RequestMapping("/v1/mcp/servers")
@RequiredArgsConstructor
public class McpServerController {

    private final McpServerService mcpServerService;

    @GetMapping
    public ApiResponse<McpListResponse> getAllServers(
            @RequestParam(required = false, defaultValue = "5") Integer size,
            @RequestParam(required = false, defaultValue = "0") Long cursorId
    ) {
        return mcpServerService.findAllServers(size, cursorId);
    }

    @GetMapping("/search")
    public ApiResponse<McpSearchResponse> getServersByName(
            @RequestParam("name") String name,
            @RequestParam(required = false, defaultValue = "5") Integer size,
            @RequestParam(required = false, defaultValue = "0") Long cursorId
    ) {
        return mcpServerService.searchServersByName(name, size, cursorId);
    }

    @PostMapping("/batch")
    public ApiResponse<McpBatchResponse> getServersByIds(
            @RequestParam(required = false, defaultValue = "5") Integer size,
            @RequestParam(required = false, defaultValue = "0") Long cursorId,
            @RequestBody McpBatchRequest batchRequest
    ) {
        List<Long> serverIds = batchRequest.serverIds();
        return mcpServerService.findServersByIds(serverIds, size, cursorId);
    }

    @GetMapping("/{serverId}")
    public ApiResponse<McpDetailResponse> getServerDetail(
            @PathVariable("serverId") Long seq
    ) {
        return mcpServerService.findServerById(seq);
    }

    @GetMapping("/tags")
    public ApiResponse<McpTagResponse> getAllTags() {

        return mcpServerService.findAllTags();
    }
}