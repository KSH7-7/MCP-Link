package kr.co.mcplink.domain.schedule.service;

import kr.co.mcplink.domain.github.dto.GithubSearchResultDto;
import kr.co.mcplink.domain.github.service.FetchSearchResultService;
import kr.co.mcplink.domain.schedule.entity.GeminiPendingQueue;
import kr.co.mcplink.domain.schedule.entity.GithubPendingQueue;
import kr.co.mcplink.domain.schedule.repository.GeminiPendingQueueRepository;
import kr.co.mcplink.domain.schedule.repository.GithubPendingQueueRepository;
import kr.co.mcplink.global.annotation.ExcludeParamLog;
import lombok.RequiredArgsConstructor;
import org.springframework.stereotype.Service;

import java.util.List;

@Service
@RequiredArgsConstructor
public class EnQueueService {

    private final FetchSearchResultService fetchSearchResultService;
    private final GithubPendingQueueRepository githubRepository;
    private final GeminiPendingQueueRepository geminiRepository;

    public void enqueueGithub(int queryNum1, int queryNum2) {
        List<GithubSearchResultDto> results = fetchSearchResultService.fetchSearchResult(queryNum1, queryNum2);
        for (GithubSearchResultDto dto : results) {
            String owner = dto.owner();
            String repo  = dto.repo();
            String name  = owner + "|" + repo;

            if (!githubRepository.existsByName(name)) {
                GithubPendingQueue entity = GithubPendingQueue.builder()
                        .name(name)
                        .owner(owner)
                        .repo(repo)
                        .build();

                githubRepository.save(entity);
            }
        }
    }

    @ExcludeParamLog
    public void enqueueGemini(String serverId, String serverName, String prepReadme) {
        if (!geminiRepository.existsByServerId(serverId)) {
            GeminiPendingQueue entity = GeminiPendingQueue.builder()
                    .serverId(serverId)
                    .serverName(serverName)
                    .prepReadme(prepReadme)
                    .build();

            geminiRepository.save(entity);
        }
    }
}