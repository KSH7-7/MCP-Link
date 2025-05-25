package kr.co.mcplink.domain.mcpserver.repository;

import com.mongodb.client.MongoCollection;
import com.mongodb.client.result.UpdateResult;
import kr.co.mcplink.domain.mcpserver.entity.McpServer;
import kr.co.mcplink.global.common.Constants;
import lombok.RequiredArgsConstructor;
import org.bson.Document;
import org.bson.types.ObjectId;
import org.springframework.data.mongodb.core.MongoTemplate;
import org.springframework.data.mongodb.core.aggregation.Aggregation;
import org.springframework.data.mongodb.core.aggregation.AggregationOperation;
import org.springframework.data.mongodb.core.query.BasicQuery;
import org.springframework.data.mongodb.core.query.Query;
import org.springframework.stereotype.Repository;

import java.util.Arrays;
import java.util.Date;
import java.util.List;

@Repository
@RequiredArgsConstructor
public class McpServerCustomRepositoryImpl implements McpServerCustomRepository {

    private final MongoTemplate mongoTemplate;

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
    public List<McpServer> listAllWithOffset(int size, int offset) {
        Aggregation agg = Aggregation.newAggregation(
                context -> new Document("$sort",
                        new Document("stars", -1).append("seq", 1)),
                context -> new Document("$skip", offset),
                context -> new Document("$limit", size)
        );

        return mongoTemplate
                .aggregate(agg, mongoTemplate.getCollectionName(McpServer.class), McpServer.class)
                .getMappedResults();
    }

    @Override
    public long countByName(String name) {
        Aggregation agg = Aggregation.newAggregation(
                createNameAndDescriptionMatch(name),
                context -> new Document("$count", "count")
        );

        return aggregateCount(agg);
    }

    @Override
    public long countRemainingByName(String name, Long cursor) {
        Aggregation agg = Aggregation.newAggregation(
                createNameAndDescriptionMatch(name),
                createCursorMatch(cursor),
                context -> new Document("$count", "count")
        );

        return aggregateCount(agg);
    }

    @Override
    public List<McpServer> searchByName(String name, int size, Long cursor) {
        Aggregation agg = Aggregation.newAggregation(
                createNameAndDescriptionMatch(name),
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
    public List<McpServer> searchByNameWithOffset(String name, int size, int offset) {
        Aggregation agg = Aggregation.newAggregation(
                createNameAndDescriptionMatch(name),
                context -> new Document("$sort",
                        new Document("stars", -1).append("seq", 1)),
                context -> new Document("$skip", offset),
                context -> new Document("$limit", size)
        );

        return mongoTemplate
                .aggregate(agg, mongoTemplate.getCollectionName(McpServer.class), McpServer.class)
                .getMappedResults();
    }

    @Override
    public long updateSummary(String id, String description, List<String> tags) {
        Document filter = new Document("_id", new ObjectId(id));

        Date now = new Date();
        Document set = new Document()
                .append("mcpServers.description", description)
                .append("tags", tags)
                .append("updated_at", now);
        Document update = new Document("$set", set);

        MongoCollection<Document> coll = mongoTemplate.getCollection(
                mongoTemplate.getCollectionName(McpServer.class)
        );
        UpdateResult result = coll.updateOne(filter, update);

        return result.getModifiedCount();
    }

    @Override
    public long countByNameKr(String name) {
        Aggregation agg = Aggregation.newAggregation(
                createNameAndDescriptionMatchKr(name),
                context -> new Document("$count", "count")
        );

        return aggregateCount(agg);
    }

    @Override
    public long countRemainingByNameKr(String name, Long cursor) {
        Aggregation agg = Aggregation.newAggregation(
                createNameAndDescriptionMatchKr(name),
                createCursorMatch(cursor),
                context -> new Document("$count", "count")
        );

        return aggregateCount(agg);
    }

    @Override
    public List<McpServer> searchByNameKr(String name, int size, Long cursor) {
        Aggregation agg = Aggregation.newAggregation(
                createNameAndDescriptionMatchKr(name),
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
    public List<McpServer> searchByNameWithOffsetKr(String name, int size, int offset) {
        Aggregation agg = Aggregation.newAggregation(
                createNameAndDescriptionMatchKr(name),
                context -> new Document("$sort",
                        new Document("stars", -1).append("seq", 1)),
                context -> new Document("$skip", offset),
                context -> new Document("$limit", size)
        );

        return mongoTemplate
                .aggregate(agg, mongoTemplate.getCollectionName(McpServer.class), McpServer.class)
                .getMappedResults();
    }

    @Override
    public long updateKr(String id, List<String> name, String description, List<String> tags) {
        Document filter = new Document("_id", new ObjectId(id));

        Date now = new Date();
        Document set = new Document()
                .append("mcpServersKr.name", name)
                .append("mcpServersKr.description", description)
                .append("tags", tags)
                .append("updated_at", now);
        Document update = new Document("$set", set);

        MongoCollection<Document> coll = mongoTemplate.getCollection(
                mongoTemplate.getCollectionName(McpServer.class)
        );
        UpdateResult result = coll.updateOne(filter, update);

        return result.getModifiedCount();
    }

    private AggregationOperation createNameAndDescriptionMatch(String name) {
        return context -> new Document("$search",
                new Document("index", Constants.IDX_MCP_SERVERS_NAME_SEARCH)
                        .append("compound", new Document("should", Arrays.asList(
                                new Document("autocomplete",
                                        new Document("query", name)
                                                .append("path", "mcpServers.name")
                                ),
                                new Document("autocomplete",
                                        new Document("query", name)
                                                .append("path", "mcpServers.description")
                                )
                        )))
        );
    }

    private AggregationOperation createNameAndDescriptionMatchKr(String name) {
        return context -> new Document("$search",
                new Document("index", Constants.IDX_MCP_SERVERS_NAME_SEARCH_KR)
                        .append("compound", new Document("should", Arrays.asList(
                                new Document("autocomplete",
                                        new Document("query", name)
                                                .append("path", "mcpServersKr.name")
                                ),
                                new Document("autocomplete",
                                        new Document("query", name)
                                                .append("path", "mcpServersKr.description")
                                )
                        )))
        );
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