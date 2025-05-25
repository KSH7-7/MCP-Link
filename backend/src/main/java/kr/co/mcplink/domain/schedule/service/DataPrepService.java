package kr.co.mcplink.domain.schedule.service;

import kr.co.mcplink.domain.gemini.service.FetchSummaryService;
import kr.co.mcplink.domain.gemini.service.FetchTagService;
import kr.co.mcplink.domain.github.dto.GithubMetaDataDto;
import kr.co.mcplink.domain.github.dto.ParsedReadmeInfoDto;
import kr.co.mcplink.domain.github.service.FetchMetaDataService;
import kr.co.mcplink.domain.github.service.FetchReadmeService;
import kr.co.mcplink.domain.github.service.PrepReadmeService;
import kr.co.mcplink.domain.schedule.entity.GeminiPendingQueue;
import kr.co.mcplink.domain.schedule.entity.GithubPendingQueue;
import kr.co.mcplink.domain.schedule.repository.GeminiPendingQueueRepository;
import kr.co.mcplink.domain.schedule.repository.GithubPendingQueueRepository;
import lombok.RequiredArgsConstructor;
import lombok.extern.slf4j.Slf4j;
import org.springframework.stereotype.Service;
import org.springframework.web.reactive.function.client.WebClientResponseException;

import java.util.ArrayList;
import java.util.List;
import java.util.Optional;

@Service
@Slf4j
@RequiredArgsConstructor
public class DataPrepService {

    private final DataStoreService dataStoreService;
    private final EnQueueService enqueueService;
    private final FetchMetaDataService fetchMetaDataService;
    private final FetchReadmeService fetchReadmeService;
    private final FetchSummaryService fetchSummaryService;
    private final FetchTagService fetchTagService;
    private final PrepReadmeService prepReadmeService;
    private final GithubPendingQueueRepository githubRepository;
    private final GeminiPendingQueueRepository geminiRepository;

    public void prepGithub() {
        List<GithubPendingQueue> items = githubRepository.findTop10ByProcessedFalseOrderBySeqAsc();
        if (items == null) {
            log.info("No pending items in Github queue");
            return;
        }

        List<String> errorNameList = new ArrayList<>();
        int forbiddenCnt = 3;

        for (GithubPendingQueue item : items) {
            String pendingItemId = item.getId();

            try {
                String name = item.getName();
                String owner = item.getOwner();
                String repo = item.getRepo();
                if (owner == null || repo == null) {
                    log.warn("☢️☢️☢️☢️☢️ Invalid Github pending item → {} ☢️☢️☢️☢️☢️", pendingItemId);
                    githubRepository.updateProcessedById(pendingItemId, true);
                    continue;
                }

                String rawReadme;
                try {
                    rawReadme = fetchReadmeService.fetchReadme(owner, repo);
                } catch (WebClientResponseException.Forbidden e) {
                    log.error("☢️☢️☢️☢️☢️ 403 Forbidden error for {}/{} ☢️☢️☢️☢️☢️", owner, repo);
                    forbiddenCnt--;
                    if (forbiddenCnt == 0) {
                        break;
                    }
                    continue;
                }

                if (rawReadme == null) {
                    log.warn("☢️☢️☢️☢️☢️ No README fetched for {}/{} ☢️☢️☢️☢️☢️", owner, repo);
                    githubRepository.updateProcessedById(pendingItemId, true);
                    errorNameList.add(name);
                    continue;
                }

                String prepReadme = prepReadmeService.decodeReadme(rawReadme);
                if (prepReadme == null) {
                    log.warn("☢️☢️☢️☢️☢️ Failed to decode README for {}/{} ☢️☢️☢️☢️☢️", owner, repo);
                    githubRepository.updateProcessedById(pendingItemId, true);
                    errorNameList.add(name);
                    continue;
                }

                ParsedReadmeInfoDto parsedReadmeInfo = prepReadmeService.parseReadme(prepReadme);
                if (parsedReadmeInfo == null) {
                    log.warn("☢️☢️☢️☢️☢️ README parsing failed for {}/{} ☢️☢️☢️☢️☢️", owner, repo);
                    githubRepository.updateProcessedById(pendingItemId, true);
                    continue;
                }

                log.info("Parsed README info for {}/{} → {}", owner, repo, parsedReadmeInfo);

                GithubMetaDataDto metaData;
                try {
                    metaData = fetchMetaDataService.fetchMetaData(owner, repo);
                } catch (WebClientResponseException.Forbidden e) {
                    log.error("☢️☢️☢️☢️☢️ 403 Forbidden error for {}/{} ☢️☢️☢️☢️☢️", owner, repo);
                    forbiddenCnt--;
                    if (forbiddenCnt == 0) {
                        break;
                    }
                    continue;
                }

                if (metaData == null) {
                    log.warn("☢️☢️☢️☢️☢️ Metadata not found for {}/{} ☢️☢️☢️☢️☢️", owner, repo);
                    githubRepository.updateProcessedById(pendingItemId, true);
                    errorNameList.add(name);
                    continue;
                }

                String savedServerId = dataStoreService.saveMcpServer(metaData, parsedReadmeInfo);
                if (savedServerId == null) {
                    log.warn("☢️☢️☢️☢️☢️ McpServer not found for {}/{} ☢️☢️☢️☢️☢️", owner, repo);
                    githubRepository.updateProcessedById(pendingItemId, true);
                    errorNameList.add(name);
                    continue;
                }

                String savedServerName = parsedReadmeInfo.name();
                githubRepository.updateProcessedById(pendingItemId, true);
                enqueueService.enqueueGemini(savedServerId, savedServerName, prepReadme);
                log.info("Successfully processed and enqueued Gemini task for {}/{}", owner, repo);
            } catch (Exception e) {
                log.error("☢️☢️☢️☢️☢️ Error processing Github item {} → {} ☢️☢️☢️☢️☢️", pendingItemId, e.getMessage(), e);
            }
        }

        if (!errorNameList.isEmpty()) {
            log.warn("⚠️⚠️⚠️⚠️⚠️ Processed with errors for following items: {} ⚠️⚠️⚠️⚠️⚠️", errorNameList);
        }
    }

    public void prepGemini(String updateId) {
        Optional<GeminiPendingQueue> itemOpt;

        if (updateId != null && updateId.startsWith("{\"_id\":")) {
            updateId = updateId.replaceAll("\\{\"_id\":\\s*\"([^\"]+)\"\\}", "$1");
        }

        if (updateId == null) {
            itemOpt = geminiRepository.findTop1ByProcessedFalseOrderBySeqAsc();
        } else {
            itemOpt = geminiRepository.findByServerId(updateId);
        }

        if (itemOpt.isEmpty()) {
            log.info("No pending items in Gemini queue");
            return;
        }

        GeminiPendingQueue item = itemOpt.get();
        String pendingItemId = item.getId();
        String serverId = item.getServerId();
        String serverName = item.getServerName();
        String prepReadme = item.getPrepReadme();

        if (serverId == null || prepReadme == null || prepReadme.isEmpty()) {
            log.warn("☢️☢️☢️☢️☢️ Invalid Gemini pending item → {} ☢️☢️☢️☢️☢️", pendingItemId);
            geminiRepository.updateProcessedById(pendingItemId, true);
            return;
        }

        try {
            String readmeSummary;
            try {
                readmeSummary = fetchSummaryService.fetchSummary(prepReadme, serverId);
            } catch (WebClientResponseException.Forbidden e) {
                log.error("☢️☢️☢️☢️☢️ 403 Forbidden error for serverId → {} ☢️☢️☢️☢️☢️", serverId);
                return;
            }

            if (readmeSummary == null) {
                log.warn("☢️☢️☢️☢️☢️ Failed to generate summary for serverId → {} ☢️☢️☢️☢️☢️", serverId);
                return;
            }

            log.info("Generated summary for serverId: {} → Summary: '{}'", serverId, readmeSummary);

            List<String> generatedTags = fetchTagService.fetchTags(serverName);
            if (generatedTags == null || generatedTags.isEmpty()) {
                log.warn("☢️☢️☢️☢️☢️ Failed to generate tags for serverId → {} ☢️☢️☢️☢️☢️", serverId);
                return;
            }

            dataStoreService.updateSummary(serverId, readmeSummary, generatedTags);
            for (String generatedTag : generatedTags) {
                dataStoreService.saveMcpTag(generatedTag);
            }
            geminiRepository.updateProcessedById(pendingItemId, true);
            log.info("Successfully processed for {}", serverId);
        } catch (Exception e) {
            log.error("☢️☢️☢️☢️☢️ Error processing Gemini item {} → {} ☢️☢️☢️☢️☢️", pendingItemId, e.getMessage(), e);
        }
    }
}