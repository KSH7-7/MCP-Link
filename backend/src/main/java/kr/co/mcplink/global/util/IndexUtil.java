package kr.co.mcplink.global.util;

import org.springframework.beans.factory.annotation.Autowired;
import org.springframework.data.domain.Sort;
import org.springframework.data.mongodb.core.MongoOperations;
import org.springframework.data.mongodb.core.index.Index;
import org.springframework.data.mongodb.core.index.TextIndexDefinition;
import org.springframework.stereotype.Component;

import java.util.Map;

@Component
public class IndexUtil {

    private final MongoOperations mongoOperations;

    @Autowired
    public IndexUtil(MongoOperations mongoOperations) {
        this.mongoOperations = mongoOperations;
    }

    public void createCompoundIndex(String collection, String indexName, Map<String, Integer> fields) {
        Index idx = new Index().named(indexName);
        fields.forEach((field, dir) -> {
            if (dir > 0) {
                idx.on(field, Sort.Direction.ASC);
            } else {
                idx.on(field, Sort.Direction.DESC);
            }
        });
        mongoOperations.indexOps(collection).ensureIndex(idx);
    }

    public void createTextIndex(String collection, String indexName, String field) {
        TextIndexDefinition txtIdx = new TextIndexDefinition.TextIndexDefinitionBuilder()
                .onField(field)
                .named(indexName)
                .build();
        mongoOperations.indexOps(collection).ensureIndex(txtIdx);
    }
}