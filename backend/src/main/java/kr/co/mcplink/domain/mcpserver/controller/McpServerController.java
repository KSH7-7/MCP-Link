package kr.co.mcplink.domain.mcpserver.controller;

import io.swagger.v3.oas.annotations.tags.Tag;
import kr.co.mcplink.domain.mcpserver.dto.request.McpBatchRequest;
import kr.co.mcplink.domain.mcpserver.dto.response.*;
import kr.co.mcplink.domain.mcpserver.service.McpServerService;
import kr.co.mcplink.global.common.ApiResponse;
import lombok.RequiredArgsConstructor;
import org.springframework.web.bind.annotation.*;

import java.util.List;

@RestController
@RequestMapping("/v3/mcp/servers")
@RequiredArgsConstructor
@Tag(name = "McpServer API v3")
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

    @GetMapping("/web")
    public ApiResponse<McpListResponse> getAllServersForWeb(
            @RequestParam(required = false, defaultValue = "10") Integer size,
            @RequestParam(required = false, defaultValue = "1") Integer page
    ) {
        return mcpServerService.findAllServersForWeb(size, page);
    }

    @GetMapping("/web/search")
    public ApiResponse<McpSearchResponse> getServersByNameForWeb(
            @RequestParam("name") String name,
            @RequestParam(required = false, defaultValue = "10") Integer size,
            @RequestParam(required = false, defaultValue = "1") Integer page
    ) {
        return mcpServerService.searchServersByNameForWeb(name, size, page);
    }

    @PostMapping("/web/batch")
    public ApiResponse<McpBatchResponse> getServersByIdsForWeb(
            @RequestParam(required = false, defaultValue = "10") Integer size,
            @RequestParam(required = false, defaultValue = "1") Integer page,
            @RequestBody McpBatchRequest batchRequest
    ) {
        List<Long> serverIds = batchRequest.serverIds();
        return mcpServerService.findServersByIdsForWeb(serverIds, size, page);
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
