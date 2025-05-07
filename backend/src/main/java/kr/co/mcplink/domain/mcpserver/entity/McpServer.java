package kr.co.mcplink.domain.mcpserver.entity;

import kr.co.mcplink.global.annotation.AutoIndex;
import kr.co.mcplink.global.annotation.AutoSequence;
import kr.co.mcplink.global.common.Constants;
import lombok.*;
import org.springframework.data.annotation.Id;
import org.springframework.data.mongodb.core.mapping.Document;
import org.springframework.data.mongodb.core.mapping.Field;

import java.util.List;
import java.util.Map;

@Getter
@Builder
@AllArgsConstructor
@NoArgsConstructor(access = AccessLevel.PROTECTED)
@AutoSequence(collection = Constants.COLLECTION_MCP_SERVERS)
@AutoIndex(collection = Constants.COLLECTION_MCP_SERVERS)
@Document(Constants.COLLECTION_MCP_SERVERS)
public class McpServer {

    @Id
    private String id;
    private Long seq;

    @Builder.Default
    private String type = "STDIO";
    private String url;
    private int stars;
    @Builder.Default
    private int views = 0;
    @Builder.Default
    private boolean scanned = true;

    private List<String> tags;

    @Field("mcpServers")
    private McpServerDetail detail;

    @Getter
    @Builder
    @AllArgsConstructor
    @NoArgsConstructor(access = AccessLevel.PROTECTED)
    public static class McpServerDetail {
        private String name;
        private String description;
        private String command;
        private List<String> args;
        private Map<String, String> env;
    }
}