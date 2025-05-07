package kr.co.mcplink.domain.mcpserver.controller;

import kr.co.mcplink.domain.mcpserver.dto.McpDetailDataDto;
import kr.co.mcplink.domain.mcpserver.dto.request.McpServerSearchRequest;
import kr.co.mcplink.domain.mcpserver.dto.response.McpListResponse;
import kr.co.mcplink.domain.mcpserver.dto.response.McpSearchByNameResponse;
import kr.co.mcplink.domain.mcpserver.service.core.McpServerService;
import kr.co.mcplink.global.common.ApiResponse;
import lombok.RequiredArgsConstructor;

import org.springframework.data.web.PageableDefault;
import org.springframework.web.bind.annotation.*;

@RestController
@RequestMapping("/v1/mcp/servers")
@RequiredArgsConstructor
public class McpServerRestController {

    private final McpServerService mcpServerService;

    @GetMapping
    public ApiResponse<McpListResponse> listAll(
            @RequestParam(required = false, defaultValue = "5") Integer size,
            @RequestParam(required = false, defaultValue = "0") Long cursorId
    ) {
        return mcpServerService.getLists(size, cursorId);
    }

    @GetMapping("/search")
    public ApiResponse<McpSearchByNameResponse> searchByName(
        @ModelAttribute(name = "mcpServerName") McpServerSearchRequest request
    ) {
        return mcpServerService.searchByName(request);
    }

    @GetMapping("/{id}")
    public ApiResponse<McpDetailDataDto> getDetail(@PathVariable("id") Long seq) {
        return mcpServerService.getDetail(seq);
    }

    @GetMapping("/tags")
    public ApiResponse<String> listTags() {
        return mcpServerService.listTags();
    }
}