package kr.co.mcplink.domain.mcpserver.entity;

import kr.co.mcplink.global.annotation.AutoIndex;
import kr.co.mcplink.global.annotation.AutoSequence;
import kr.co.mcplink.global.common.BaseTimeMongoEntity;
import kr.co.mcplink.global.common.Constants;
import lombok.*;
import org.springframework.data.annotation.Id;
import org.springframework.data.mongodb.core.index.Indexed;
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
@Document(collection = Constants.COLLECTION_MCP_SERVERS)
public class McpServer extends BaseTimeMongoEntity {

    @Id
    private String id;
    private Long seq;

    @Builder.Default
    private String type = "STDIO";

    @Indexed(unique = true)
    private String url;
    private int stars;
    private int views;
    private boolean official;
    private boolean scanned;

    @Builder.Default
    private SecurityRank securityRank = SecurityRank.UNRATED;
    private List<String> tags;

    @Field("mcpServers")
    private McpServerDetail detail;

    @Getter
    @AllArgsConstructor
    @NoArgsConstructor(access = AccessLevel.PROTECTED)
    public static class McpServerDetail {
        private String name;
        private String description;
        private String command;
        private List<String> args;
        private Map<String, String> env;

        @Builder
        public static McpServerDetail build(
                String name,
                String description,
                String command,
                List<String> args,
                Map<String, String> env
        ) {

            if (env != null && env.isEmpty()) {
                env = null;
            }

            return new McpServerDetail(name, description, command, args, env);
        }
    }

    @Field("mcpServersKr")
    private McpServerSummary summary;

    @Getter
    @AllArgsConstructor
    @NoArgsConstructor(access = AccessLevel.PROTECTED)
    public static class McpServerSummary {
        private List<String> name;
        private String description;
    }
}