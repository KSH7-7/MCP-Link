package kr.co.mcplink.domain.mcpserver.service.core;

import kr.co.mcplink.domain.mcpserver.dto.common.*;
import kr.co.mcplink.domain.mcpserver.dto.response.McpDetailResponse;
import kr.co.mcplink.domain.mcpserver.dto.response.McpListResponse;
import kr.co.mcplink.domain.mcpserver.dto.response.McpTagResponse;
import kr.co.mcplink.domain.mcpserver.entity.McpServer;
import kr.co.mcplink.domain.mcpserver.repository.McpServerRepository;
import kr.co.mcplink.domain.mcpserver.repository.McpTagRepository;
import kr.co.mcplink.domain.mcpserver.service.support.ViewCountUpdaterService;
import kr.co.mcplink.global.common.Constants;
import kr.co.mcplink.global.util.PaginationUtil;
import lombok.RequiredArgsConstructor;
import org.springframework.stereotype.Service;

import java.time.Instant;
import java.util.List;
import java.util.stream.Collectors;

@Service
@RequiredArgsConstructor
public class McpServerService {
    private final McpServerRepository serverRepository;
    private final McpTagRepository tagRepository;
    private final ViewCountUpdaterService viewCountUpdaterService;

    public McpListResponse listAll(Integer limit, Long cursor) {
        int lim = PaginationUtil.validate(limit, cursor);
        List<McpServer> items = serverRepository.listAll(lim, cursor);
        long total = serverRepository.countAll();
        long endCursor = items.isEmpty() ? 0L : items.get(items.size() - 1).getSeq();
        long remaining = serverRepository.countRemaining(endCursor);
        PageInfoDto pageInfo = PaginationUtil.buildPageInfo(items, total, remaining);
        List<McpSummaryDataDto> data = items.stream()
                .map(this::toSummaryDataDto)
                .collect(Collectors.toList());
        return McpListResponse.builder()
                .timestamp(Instant.now())
                .status(Constants.STATUS_OK)
                .message(Constants.MSG_SUCCESS_LIST)
                .pageInfo(pageInfo)
                .data(data)
                .build();
    }

    public McpListResponse searchByName(String name, Integer limit, Long cursor) {
        int lim = PaginationUtil.validate(limit, cursor);
        List<McpServer> items = serverRepository.searchByName(name, lim, cursor);
        long total = serverRepository.countByName(name);
        long endCursor = items.isEmpty() ? 0L : items.get(items.size() - 1).getSeq();
        long remaining = serverRepository.countRemainingByName(name, endCursor);
        PageInfoDto pageInfo = PaginationUtil.buildPageInfo(items, total, remaining);
        List<McpSummaryDataDto> data = items.stream()
                .map(this::toSummaryDataDto)
                .collect(Collectors.toList());
        return McpListResponse.builder()
                .timestamp(Instant.now())
                .status(Constants.STATUS_OK)
                .message(Constants.MSG_SUCCESS_SEARCH)
                .pageInfo(pageInfo)
                .data(data)
                .build();
    }

    public McpDetailResponse getDetail(Long seq) {
        McpServer server = serverRepository.findBySeq(seq);
//                .orElseThrow(() -> new ResourceNotFoundException(Constants.MSG_NOT_FOUND));
        viewCountUpdaterService.incrementViews(seq);
        McpDetailDataDto data = toDetailDataDto(server);
        return McpDetailResponse.builder()
                .timestamp(Instant.now())
                .status(Constants.STATUS_OK)
                .message(Constants.MSG_SUCCESS_DETAIL)
                .data(data)
                .build();
    }

    public McpTagResponse listTags() {
        List<String> tags = tagRepository.listAll();
        return McpTagResponse.builder()
                .timestamp(Instant.now())
                .status(Constants.STATUS_OK)
                .message(Constants.MSG_SUCCESS_TAG_LIST)
                .data(tags)
                .build();
    }

    private McpSummaryDataDto toSummaryDataDto(McpServer s) {
        return McpSummaryDataDto.builder()
                .id(s.getSeq())
                .type(s.getType())
                .url(s.getUrl())
                .stars(s.getStars())
                .views(s.getViews())
                .scanned(s.isScanned())
                .mcpServers(
                        McpSummaryDto.builder()
                                .name(s.getDetail().getName())
                                .description(s.getDetail().getDescription())
                                .build()
                )
                .build();
    }

    private McpDetailDataDto toDetailDataDto(McpServer s) {
        return McpDetailDataDto.builder()
                .id(s.getSeq())
                .type(s.getType())
                .url(s.getUrl())
                .stars(s.getStars())
                .views(s.getViews())
                .scanned(s.isScanned())
                .mcpServers(
                        McpDetailDto.builder()
                                .name(s.getDetail().getName())
                                .description(s.getDetail().getDescription())
                                .command(s.getDetail().getCommand())
                                .args(s.getDetail().getArgs())
                                .env(s.getDetail().getEnv())
                                .build()
                )
                .build();
    }
}