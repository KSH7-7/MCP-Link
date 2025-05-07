package kr.co.mcplink.domain.mcpserver.repository;

import kr.co.mcplink.domain.mcpserver.entity.McpTag;
import org.springframework.data.mongodb.repository.MongoRepository;
import org.springframework.stereotype.Repository;

import java.util.List;
import java.util.stream.Collectors;

@Repository
public interface McpTagRepository extends MongoRepository<McpTag, String> {

    @Override
    McpTag save(McpTag tag);

    default List<String> listAll() {
        return findAll().stream()
                .map(McpTag::getTag)
                .collect(Collectors.toList());
    }
}