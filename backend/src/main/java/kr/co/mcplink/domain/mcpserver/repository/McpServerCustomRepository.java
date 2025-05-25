package kr.co.mcplink.domain.mcpserver.repository;

import kr.co.mcplink.domain.mcpserver.entity.McpServer;

import java.util.List;

public interface McpServerCustomRepository {

    long countRemaining(Long cursor);
    List<McpServer> listAll(int size, Long cursor);
    List<McpServer> listAllWithOffset(int size, int offset);
    long countByName(String name);
    long countRemainingByName(String name, Long cursor);
    List<McpServer> searchByName(String name, int size, Long cursor);
    List<McpServer> searchByNameWithOffset(String name, int size, int offset);
    long updateSummary(String id, String description, List<String> tags);
    long countByNameKr(String name);
    long countRemainingByNameKr(String name, Long cursor);
    List<McpServer> searchByNameKr(String name, int size, Long cursor);
    List<McpServer> searchByNameWithOffsetKr(String name, int size, int offset);
    long updateKr(String id, List<String> name, String description, List<String> tags);
}