package kr.co.mcplink.domain.mcpserver.dto;

import lombok.Builder;

@Builder
public record PageInfoDto (
    long startCursor,
    long endCursor,
    boolean hasNextPage,
    long totalItems
) {

}