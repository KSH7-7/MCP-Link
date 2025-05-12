package kr.co.mcplink.domain.mcpserver.repository;


import kr.co.mcplink.domain.mcpserver.entity.McpServer;

import java.util.List;

public interface McpServerCustomRepository {

    long countRemaining(Long cursor);
    List<McpServer> listAll(int size, Long cursor);
    long countRemainingByName(String name, Long cursor);
    List<McpServer> searchByName(String name, int size, Long cursor);
}