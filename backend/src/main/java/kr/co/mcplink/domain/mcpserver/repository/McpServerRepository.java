package kr.co.mcplink.domain.mcpserver.repository;

import kr.co.mcplink.domain.mcpserver.entity.McpServer;
import kr.co.mcplink.domain.mcpserver.entity.SecurityRank;
import org.springframework.data.mongodb.repository.CountQuery;
import org.springframework.data.mongodb.repository.MongoRepository;
import org.springframework.data.mongodb.repository.Query;
import org.springframework.data.mongodb.repository.Update;
import org.springframework.stereotype.Repository;

import java.util.*;
import java.util.function.Function;
import java.util.stream.Collectors;

@Repository
public interface McpServerRepository extends MongoRepository<McpServer, String>, McpServerCustomRepository {

    @CountQuery("{}")
    long countAll();

    @CountQuery("{ 'name': { $regex: ?0, $options: 'i' } }")
    long countByName(String nameRegex);

    Optional<McpServer> findBySeq(Long seq);

    @Query("{ 'seq': { $in: ?0 } }")
    List<McpServer> findBySeqIn(Collection<Long> seqs);

    default List<McpServer> findBySeqInOrder(List<Long> seqs) {
        Map<Long, McpServer> map = findBySeqIn(seqs)
                .stream()
                .collect(Collectors.toMap(McpServer::getSeq, Function.identity()));

        return seqs.stream()
                .map(map::get)
                .filter(Objects::nonNull)
                .collect(Collectors.toList());
    }

    @Update("{ '$inc': { 'views': 1 } }")
    long findAndIncrementViewsBySeq(Long seq);

    @Update("{ '$set': { 'scanned': true } }")
    long findAndUpdateScannedBySeq(Long seq);

    @Update("{ '$set': { 'scanned': false } }")
    long findAndUpdateNotScannedBySeq(Long seq);

    @Update(update = "{ '$set': { 'securityRank': ?1 } }")
    long findAndUpdateSecurityRankBySeq(Long seq, SecurityRank securityRank);
}