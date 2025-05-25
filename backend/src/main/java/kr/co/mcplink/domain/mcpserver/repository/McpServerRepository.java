package kr.co.mcplink.domain.mcpserver.repository;

import kr.co.mcplink.domain.mcpserver.entity.SecurityRank;
import kr.co.mcplink.domain.mcpserver.entity.McpServer;
import org.springframework.data.mongodb.repository.CountQuery;
import org.springframework.data.mongodb.repository.MongoRepository;
import org.springframework.data.mongodb.repository.Query;
import org.springframework.data.mongodb.repository.Update;
import org.springframework.stereotype.Repository;

import java.util.List;
import java.util.Optional;

@Repository
public interface McpServerRepository extends MongoRepository<McpServer, String>, McpServerCustomRepository {

    @CountQuery("{}")
    long countAll();

    Optional<McpServer> findBySeq(Long seq);

    boolean existsByUrl(String url);

    @Query(value = "{'mcpServersKr.description': {$regex: ?0, $options: 'i'}}", fields = "{'_id': { $toString: '$_id' }}")
    List<String> findIdsByDetailDescriptionContaining(String text);

    @Update("{ '$inc': { 'views': 1 }, '$set': { 'updated_at': new Date() } }")
    long findAndIncrementViewsBySeq(Long seq);

    List<McpServer> findByOfficialFalse();

    List<McpServer> findByScannedFalse();

    @Query("{ '_id': ?0 }")
    @Update("{ '$set': { 'scanned': true, 'updated_at': new Date() } }")
    long updateScannedStatusById(String _id);

    @Query("{ '_id': ?0 }")
    @Update("{ '$set': { 'securityRank': ?1, 'updated_at': new Date() } }")
    long updateSecurityRankById(String _id, SecurityRank securityRank);
}