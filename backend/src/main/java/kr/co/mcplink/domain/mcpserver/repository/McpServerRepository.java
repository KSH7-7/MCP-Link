package kr.co.mcplink.domain.mcpserver.repository;

import java.util.Optional;

import kr.co.mcplink.domain.mcpserver.entity.McpServer;
import org.springframework.data.mongodb.repository.MongoRepository;
import org.springframework.stereotype.Repository;

@Repository
public interface McpServerRepository extends MongoRepository<McpServer, String>, McpServerCustomRepository {
    Optional<McpServer> findBySeq(Long seq);
}