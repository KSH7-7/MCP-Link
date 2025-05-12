package kr.co.mcplink.domain.mcpserver.repository;

import kr.co.mcplink.domain.mcpserver.entity.McpServer;
import org.bson.Document;
import org.springframework.data.mongodb.core.MongoTemplate;
import org.springframework.data.mongodb.core.aggregation.Aggregation;
import org.springframework.data.mongodb.core.aggregation.AggregationOperation;
import org.springframework.data.mongodb.core.query.BasicQuery;
import org.springframework.data.mongodb.core.query.Query;
import org.springframework.stereotype.Repository;

import java.util.Arrays;
import java.util.List;
import java.util.regex.Pattern;

@Repository
public class McpServerCustomRepositoryImpl implements McpServerCustomRepository {

    private final MongoTemplate mongoTemplate;

    public McpServerCustomRepositoryImpl(MongoTemplate mongoTemplate) {
        this.mongoTemplate = mongoTemplate;
    }

    @Override
    public long countRemaining(Long cursor) {
        Aggregation agg = Aggregation.newAggregation(
                createCursorMatch(cursor),
                context -> new Document("$count", "count")
        );

        return aggregateCount(agg);
    }

    @Override
    public List<McpServer> listAll(int size, Long cursor) {
        Aggregation agg = Aggregation.newAggregation(
                createCursorMatch(cursor),
                context -> new Document("$sort",
                        new Document("stars", -1).append("seq", 1)),
                context -> new Document("$limit", size)
        );

        return mongoTemplate
                .aggregate(agg, mongoTemplate.getCollectionName(McpServer.class), McpServer.class)
                .getMappedResults();
    }

    @Override
    public long countRemainingByName(String name, Long cursor) {
        Aggregation agg = Aggregation.newAggregation(
                createNameMatch(name),
                createCursorMatch(cursor),
                context -> new Document("$count", "count")
        );

        return aggregateCount(agg);
    }

    @Override
    public List<McpServer> searchByName(String name, int size, Long cursor) {
        Aggregation agg = Aggregation.newAggregation(
                createNameMatch(name),
                createCursorMatch(cursor),
                context -> new Document("$sort",
                        new Document("stars", -1).append("seq", 1)),
                context -> new Document("$limit", size)
        );

        return mongoTemplate
                .aggregate(agg, mongoTemplate.getCollectionName(McpServer.class), McpServer.class)
                .getMappedResults();
    }

    private AggregationOperation createCursorMatch(Long cursor) {
        return context -> {
            if (cursor == null || cursor <= 0) {
                return new Document("$match", new Document());
            }
            String queryStr = String.format("{ \"seq\" : %d }", cursor);
            Query query = new BasicQuery(queryStr);
            McpServer last = mongoTemplate.findOne(query, McpServer.class);

            if (last == null) {
                return new Document("$match", new Document("seq", new Document("$exists", false)));
            }
            long lastStars = last.getStars();
            Document orCond = new Document("$or", Arrays.asList(
                    new Document("stars", new Document("$lt", lastStars)),
                    new Document("$and", Arrays.asList(
                            new Document("stars", lastStars),
                            new Document("seq", new Document("$gt", cursor))
                    ))
            ));

            return new Document("$match", orCond);
        };
    }

    private AggregationOperation createNameMatch(String name) {
        String regex = ".*" + Pattern.quote(name) + ".*";
        Document pattern = new Document("$regex", regex).append("$options", "i");

        return context -> new Document("$match", new Document("mcpServers.name", pattern));
    }

    private long aggregateCount(Aggregation agg) {
        Document result = mongoTemplate
                .aggregate(agg, mongoTemplate.getCollectionName(McpServer.class), Document.class)
                .getUniqueMappedResult();
        if (result == null) {
            return 0L;
        }
        Number countNum = result.get("count", Number.class);

        return countNum == null ? 0L : countNum.longValue();
    }
}