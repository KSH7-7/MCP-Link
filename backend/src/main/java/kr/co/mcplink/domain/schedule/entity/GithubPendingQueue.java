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
@AutoSequence(collection = Constants.COLLECTION_GITHUB_PENDING_QUEUE)
@Document(collection = Constants.COLLECTION_GITHUB_PENDING_QUEUE)
public class GithubPendingQueue extends BaseTimeMongoEntity {

    @Id
    private String id;

    @Indexed
    private Long seq;

    @Indexed(unique = true)
    private String name;
    private String owner;
    private String repo;

    @Indexed
    private boolean processed;
}