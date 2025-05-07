package kr.co.mcplink.domain.mcpserver.controller;

import kr.co.mcplink.domain.mcpserver.dto.response.McpDetailResponse;
import kr.co.mcplink.domain.mcpserver.dto.response.McpListResponse;
import kr.co.mcplink.domain.mcpserver.dto.response.McpTagResponse;
import kr.co.mcplink.domain.mcpserver.service.core.McpServerService;
import kr.co.mcplink.global.common.Constants;
import lombok.RequiredArgsConstructor;
import org.springframework.web.bind.annotation.*;

@RestController
@RequestMapping(Constants.MCP_BASE_PATH)
@RequiredArgsConstructor
public class McpServerRestController {

    private final McpServerService mcpServerService;

    @GetMapping
    public McpListResponse listAll(
            @RequestParam(required = false) Integer limit,
            @RequestParam(required = false) Long cursor
    ) {
        return mcpServerService.listAll(limit, cursor);
    }

    @GetMapping("/search")
    public McpListResponse searchByName(
            @RequestParam("name") String name,
            @RequestParam(required = false) Integer limit,
            @RequestParam(required = false) Long cursor
    ) {
        return mcpServerService.searchByName(name, limit, cursor);
    }

    @GetMapping("/{id}")
    public McpDetailResponse getDetail(
            @PathVariable("id") Long seq
    ) {
        return mcpServerService.getDetail(seq);
    }

    @GetMapping("/tags")
    public McpTagResponse listTags() {
        return mcpServerService.listTags();
    }
}