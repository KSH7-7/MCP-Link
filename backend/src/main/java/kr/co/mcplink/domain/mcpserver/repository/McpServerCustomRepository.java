package kr.co.mcplink.domain.mcpserver.repository;


import kr.co.mcplink.domain.mcpserver.entity.McpServer;

import java.util.List;

public interface McpServerCustomRepository {

    long countAll();
    long countRemaining(Long cursor);
    List<McpServer> listAll(int limit, Long cursor);
    long countByName(String name);
    long countRemainingByName(String name, Long cursor);
    List<McpServer> searchByName(String name, int limit, Long cursor);
    void incrementViews(Long seq);
}