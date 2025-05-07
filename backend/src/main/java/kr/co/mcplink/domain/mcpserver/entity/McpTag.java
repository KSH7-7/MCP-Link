package kr.co.mcplink.domain.mcpserver.entity;

import kr.co.mcplink.global.annotation.AutoSequence;
import kr.co.mcplink.global.common.Constants;
import lombok.*;
import org.springframework.data.annotation.Id;
import org.springframework.data.mongodb.core.index.Indexed;
import org.springframework.data.mongodb.core.mapping.Document;

@Getter
@Builder
@AllArgsConstructor
@NoArgsConstructor(access = AccessLevel.PROTECTED)
@AutoSequence(collection = Constants.COLLECTION_MCP_TAGS)
@Document(Constants.COLLECTION_MCP_TAGS)
public class McpTag {

    @Id
    private String id;
    private Long seq;

    @Indexed(unique = true)
    private String tag;
}