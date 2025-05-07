package kr.co.mcplink.domain.mcpserver.dto.common;

import lombok.AllArgsConstructor;
import lombok.Builder;
import lombok.Getter;
import lombok.NoArgsConstructor;

@Getter
@Builder
@NoArgsConstructor
@AllArgsConstructor
public class PageInfoDto {
    private long startCursor;
    private long endCursor;
    private boolean hasNextPage;
    private long totalItems;
}