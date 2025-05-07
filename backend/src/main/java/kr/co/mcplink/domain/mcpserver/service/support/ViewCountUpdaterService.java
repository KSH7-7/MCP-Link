package kr.co.mcplink.domain.mcpserver.service.support;

import kr.co.mcplink.domain.mcpserver.repository.McpServerRepository;
import lombok.RequiredArgsConstructor;
import org.springframework.stereotype.Service;

@Service
@RequiredArgsConstructor
public class ViewCountUpdaterService {

    private final McpServerRepository mcpServerRepository;

    public void incrementViews(Long seq) {
        mcpServerRepository.incrementViews(seq);
    }
}