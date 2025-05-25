package kr.co.mcplink.global.util;

import kr.co.mcplink.domain.mcpserver.dto.PageInfoDto;
import kr.co.mcplink.domain.mcpserver.entity.McpServer;

import java.util.Collections;
import java.util.List;
import java.util.Optional;

public class PaginationUtil {

    public static PageInfoDto buildPageInfo(List<McpServer> items, long totalItems, long remaining) {
        long startCursor = items.isEmpty() ? 0 : items.get(0).getSeq();
        long endCursor   = items.isEmpty() ? 0 : items.get(items.size() - 1).getSeq();
        boolean hasNext  = remaining > 0;

        return PageInfoDto.builder()
                .startCursor(Optional.of(startCursor))
                .endCursor(Optional.of(endCursor))
                .hasNextPage(hasNext)
                .totalItems(totalItems)
                .build();
    }

    public static PageInfoDto buildPageInfoForBatch(List<Long> items, int size, Long cursorId) {
        long totalItems = items.size();
        int startIndex = 0;
        if (cursorId > 0) {
            int idx = items.indexOf(cursorId);
            if (idx >= 0) {
                startIndex = idx + 1;
            }
        }
        long startCursor = (startIndex < totalItems) ? items.get(startIndex) : 0L;
        int endIndex = (int) Math.min(totalItems, startIndex + size);
        long endCursor   = (endIndex > startIndex) ? items.get(endIndex - 1) : 0L;
        boolean hasNext = endIndex < totalItems;

        return PageInfoDto.builder()
                .startCursor(Optional.of(startCursor))
                .endCursor(Optional.of(endCursor))
                .hasNextPage(hasNext)
                .totalItems(totalItems)
                .build();
    }

    public static List<Long> slicePageIdsForBatch(List<Long> items, int size, Long cursorId) {
        long totalItems = items.size();
        int startIndex = 0;
        if (cursorId > 0) {
            int idx = items.indexOf(cursorId);
            if (idx >= 0) {
                startIndex = idx + 1;
            }
        }
        int endIndex = (int) Math.min(totalItems, startIndex + size);

        return items.subList(startIndex, endIndex);
    }

    public static int calculateOffset(int size, int page) {
        if (page <= 1) {
            return 0;
        }
        return (page - 1) * size;
    }

    public static PageInfoDto buildPageInfoForWeb(long totalItems, int size, int page) {
        long totalPages = (totalItems + size - 1) / size;
        boolean hasNext = page < totalPages;

        return PageInfoDto.builder()
                .hasNextPage(hasNext)
                .totalItems(totalItems)
                .build();
    }

    public static PageInfoDto buildPageInfoForBatchForWeb(List<Long> items, int size, int page) {
        long totalItems = items.size();

        return buildPageInfoForWeb(totalItems, size, page);
    }

    public static List<Long> slicePageIdsForBatchForWeb(List<Long> items, int size, int page) {
        int startIndex = (page <= 1) ? 0 : (page - 1) * size;
        if (startIndex >= items.size()) {
            return Collections.emptyList();
        }
        int endIndex = Math.min(items.size(), startIndex + size);

        return items.subList(startIndex, endIndex);
    }
}