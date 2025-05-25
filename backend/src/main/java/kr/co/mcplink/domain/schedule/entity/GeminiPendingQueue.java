package kr.co.mcplink.domain.schedule.entity;

import kr.co.mcplink.global.annotation.AutoSequence;
import kr.co.mcplink.global.common.BaseTimeMongoEntity;
import kr.co.mcplink.global.common.Constants;
import lombok.*;
import org.springframework.data.annotation.Id;
import org.springframework.data.mongodb.core.index.Indexed;
import org.springframework.data.mongodb.core.mapping.Document;

@Getter
@Builder
@AllArgsConstructor
@NoArgsConstructor(access = AccessLevel.PROTECTED)
@AutoSequence(collection = Constants.COLLECTION_GEMINI_PENDING_QUEUE)
@Document(collection = Constants.COLLECTION_GEMINI_PENDING_QUEUE)
public class GeminiPendingQueue extends BaseTimeMongoEntity {

    @Id
    private String id;

    @Indexed
    private Long seq;

    @Indexed(unique = true)
    private String serverId;
    private String serverName;
    private String prepReadme;

    @Indexed
    private boolean processed;
}