package kr.co.mcplink.domain.schedule.service;

import kr.co.mcplink.domain.github.dto.GithubMetaDataDto;
import kr.co.mcplink.domain.github.dto.ParsedReadmeInfoDto;
import kr.co.mcplink.domain.mcpserver.entity.McpServer;
import kr.co.mcplink.domain.mcpserver.entity.McpTag;
import kr.co.mcplink.domain.mcpserver.repository.McpServerRepository;
import kr.co.mcplink.domain.mcpserver.repository.McpTagRepository;
import kr.co.mcplink.global.common.Constants;
import lombok.RequiredArgsConstructor;
import lombok.extern.slf4j.Slf4j;
import org.springframework.dao.DuplicateKeyException;
import org.springframework.stereotype.Service;

import java.util.List;

@Service
@Slf4j
@RequiredArgsConstructor
public class DataStoreService {

    private final McpServerRepository serverRepository;
    private final McpTagRepository tagRepository;

    public String saveMcpServer(GithubMetaDataDto metaData, ParsedReadmeInfoDto parsedReadmeInfo) {
        try {
            McpServer mcpServer = toMcpServerV3(metaData, parsedReadmeInfo);

            if (mcpServer == null) {
                log.warn("Failed to create McpServer from metadata → invalid data");
                return null;
            }

            if (serverRepository.existsByUrl(mcpServer.getUrl())) {
                log.warn("McpServer already exists with URL: {} → skip", mcpServer.getUrl());
                return null;
            }

            McpServer savedMcpServer = serverRepository.save(mcpServer);
            log.info("Successfully saved new McpServer with URL: {} → ID: {}", mcpServer.getUrl(), savedMcpServer.getId());
            return savedMcpServer.getId();
        } catch (Exception e) {
            log.error("Error saving McpServer for URL {}: {} → {}",
                    metaData.url() != null ? metaData.url() : "unknown",
                    e.getClass().getSimpleName(),
                    e.getMessage());
            return null;
        }
    }

    public void updateSummary(String serverId, String summary, List<String> tags) {
        try {
            long updatedCount = serverRepository.updateSummary(serverId, summary, tags);

            if (updatedCount > 0) {
                log.info("Updated summary and tags for server: {} → success", serverId);
            } else {
                log.warn("No server found with ID: {} for summary update → skip", serverId);
            }
        } catch (Exception e) {
            log.error("Error updating summary for server {} → {}: {}",
                    serverId,
                    e.getClass().getSimpleName(),
                    e.getMessage());
        }
    }

    public void updateKr(String serverId, List<String> tagsKr, String summary, List<String> tags) {
        try {
            long updatedCount = serverRepository.updateKr(serverId, tagsKr, summary, tags);

            if (updatedCount > 0) {
                log.info("Updated summary and tags for server: {} success", serverId);
            } else {
                log.warn("No server found with ID: {} for summary update <UNK> skip", serverId);
            }
        } catch (Exception e) {
            log.error("Error updating summary for server {} → {}: {}",
                    serverId,
                    e.getClass().getSimpleName(),
                    e.getMessage());
        }
    }

    public void saveMcpTag(String tag) {
        try {
            if (tagRepository.existsByTag(tag)) {
                log.debug("Tag already exists: {} → skip", tag);
                return;
            }

            McpTag mcpTag = McpTag.builder()
                    .tag(tag)
                    .build();

            tagRepository.save(mcpTag);
            log.info("Saved new tag: {} → success", tag);
        } catch (DuplicateKeyException e) {
            log.debug("Tag already exists (concurrent insert): {} → skip", tag);
        } catch (Exception e) {
            log.error("Error saving tag {} → {}: {}",
                    tag,
                    e.getClass().getSimpleName(),
                    e.getMessage());
        }
    }

    private McpServer toMcpServerV3(GithubMetaDataDto m, ParsedReadmeInfoDto p) {
        if (m.url() == null || m.stars() == 0 || p.name() == null || p.command() == null || p.args() == null) {
            return null;
        }

        String rawUrl = m.url();
        String prepUrl = rawUrl;

        if (rawUrl.endsWith(".git")) {
            prepUrl = rawUrl.substring(0, rawUrl.length() - 4);
        }

        String pendingSummary = generatePendingSummary(prepUrl);

        return McpServer.builder()
                .url(prepUrl)
                .stars(m.stars())
                .official(m.official())
                .scanned(m.scanned())
                .securityRank(m.securityRank())
                .detail(
                        McpServer.McpServerDetail.builder()
                                .name(p.name())
                                .description(pendingSummary)
                                .command(p.command())
                                .args(p.args())
                                .env(p.env())
                                .build()
                )
                .build();
    }

    private String generatePendingSummary(String serverUrl) {

        return String.format(
                Constants.DESCRIPTION_NOT_YET_GENERATED,
                serverUrl
        );
    }
}