package kr.co.mcplink.global.util;

import kr.co.mcplink.domain.mcpserver.dto.common.PageInfoDto;
import kr.co.mcplink.domain.mcpserver.entity.McpServer;
import kr.co.mcplink.global.common.Constants;

import java.util.List;

public class PaginationUtil {

    public static int validate(Integer limit, Long cursor) {
        int lim = (limit == null) ? Constants.DEFAULT_PAGINATION_LIMIT : limit;
        if (lim <= 0 || lim > Constants.MAX_PAGINATION_LIMIT) {
//            throw new InvalidRequestException("limit 파라미터가 잘못되었습니다.");
        }
        if (cursor != null && cursor < 0) {
//            throw new InvalidRequestException("cursor 파라미터가 잘못되었습니다.");
        }
        return lim;
    }

    public static PageInfoDto buildPageInfo(List<McpServer> items, long totalItems, long remainings) {
        long startCursor = items.isEmpty() ? 0 : items.get(0).getSeq();
        long endCursor   = items.isEmpty() ? 0 : items.get(items.size() - 1).getSeq();
        boolean hasNext  = remainings > 0;

        return PageInfoDto.builder()
                .startCursor(startCursor)
                .endCursor(endCursor)
                .hasNextPage(hasNext)
                .totalItems(totalItems)
                .build();
    }
}