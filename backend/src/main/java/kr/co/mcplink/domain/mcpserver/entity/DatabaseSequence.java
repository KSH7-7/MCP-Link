package kr.co.mcplink.domain.mcpserver.entity;

import kr.co.mcplink.global.common.Constants;
import lombok.Getter;
import org.springframework.data.annotation.Id;
import org.springframework.data.mongodb.core.mapping.Document;

@Document(collection = Constants.COLLECTION_DATABASE_SEQUENCE)
public class DatabaseSequence {
    @Id
    private String id;
    @Getter
    private long seq;
}