package kr.co.mcplink.domain.gemini.service;

import com.fasterxml.jackson.databind.JsonNode;
import com.fasterxml.jackson.databind.ObjectMapper;
import kr.co.mcplink.domain.gemini.client.GeminiApiClient;
import kr.co.mcplink.domain.gemini.dto.GeminiRequestDto;
import kr.co.mcplink.domain.gemini.dto.GeminiResponseDto;
import kr.co.mcplink.domain.mcpserver.entity.McpServer;
import kr.co.mcplink.domain.mcpserver.repository.McpServerRepository;
import kr.co.mcplink.global.annotation.ExcludeParamLog;
import kr.co.mcplink.global.common.Constants;
import lombok.RequiredArgsConstructor;
import lombok.extern.slf4j.Slf4j;
import org.springframework.stereotype.Service;

import java.util.Optional;
import java.util.regex.Matcher;
import java.util.regex.Pattern;

@Service
@Slf4j
@RequiredArgsConstructor
public class FetchSummaryService {

    private static final ObjectMapper objectMapper = new ObjectMapper();

    private final McpServerRepository serverRepository;
    private final GeminiApiClient geminiClient;

    @ExcludeParamLog
    public String fetchSummary(String readmeContent, String serverId) {
        try {
            GeminiRequestDto request = createSummaryRequest(readmeContent);
            GeminiResponseDto response = geminiClient.generateContent(request).block();
            String result = extractSummary(response);

            if (serverId != null && (result == null || result.isEmpty() ||
                    result.startsWith("Failed to") || result.startsWith("Error"))) {
                return generateFallbackSummary(serverId);
            }

            return result;
        } catch (Exception e) {
            log.error("Error fetching summary: ", e);

            if (serverId != null) {
                return generateFallbackSummary(serverId);
            }

            return "Failed to generate summary";
        }
    }

    private String generateFallbackSummary(String serverId) {
        try {
            Optional<McpServer> serverOpt = serverRepository.findById(serverId);

            if (serverOpt.isPresent()) {
                McpServer server = serverOpt.get();
                String serverUrl = server.getUrl();

                return String.format(
                        Constants.GEMINI_FALLBACK_SUMMARY_TEXT,
                        serverUrl
                );
            } else {
                log.warn("Server not found for id: {}", serverId);
                return Constants.GEMINI_DEFAULT_FALLBACK_TEXT;
            }
        } catch (Exception e) {
            log.error("Error generating fallback summary for serverId {}: {}", serverId, e.getMessage());
            return Constants.GEMINI_DEFAULT_FALLBACK_TEXT;
        }
    }

    private GeminiRequestDto createSummaryRequest(String readmeContent) {
        String prompt = buildPrompt(readmeContent);
        return GeminiRequestDto.createRequest(prompt);
    }

    private String buildPrompt(String readmeContent) {
        return """
        Analyze the following README content and extract a summary.
        
        For the summary:
        - Write 1-2 concise, clear sentences that explain what this MCP server implementation does
        - Focus on the core capabilities and integrations of this server
        - Use present tense, active voice
        - Be specific about technologies and protocols used
        - Write in professional English
        
        Please respond ONLY with valid JSON in the following format:
        {
            "summary": "Your concise functional description here"
        }
        
        README content:
        %s
        """.formatted(readmeContent);
    }

    private String extractSummary(GeminiResponseDto response) {
        try {
            String generatedText = extractTextSafely(response);
            return parseJsonForSummary(generatedText);
        } catch (Exception e) {
            log.error("Error parsing response to extract summary: ", e);
            return "Error parsing response";
        }
    }

    private String extractTextSafely(GeminiResponseDto response) {
        return Optional.ofNullable(response)
                .map(GeminiResponseDto::candidates)
                .filter(candidates -> !candidates.isEmpty())
                .map(candidates -> candidates.get(0))
                .map(GeminiResponseDto.CandidateDto::content)
                .map(GeminiResponseDto.ContentDto::parts)
                .filter(parts -> !parts.isEmpty())
                .map(parts -> parts.get(0))
                .map(GeminiResponseDto.PartDto::text)
                .orElse("");
    }

    private String parseJsonForSummary(String jsonText) {
        try {
            Pattern pattern = Pattern.compile("\\{[\\s\\S]*\\}", Pattern.DOTALL);
            Matcher matcher = pattern.matcher(jsonText);

            if (matcher.find()) {
                String cleanedJson = matcher.group();
                JsonNode root = objectMapper.readTree(cleanedJson);

                return root.has("summary") ? root.get("summary").asText() : "";
            }

            log.warn("Failed to extract JSON from response: {}", jsonText);
            return jsonText.length() > 100 ? jsonText.substring(0, 100) + "..." : jsonText;
        } catch (Exception e) {
            log.error("Failed to parse JSON response: {}", jsonText, e);
            return "Failed to parse response";
        }
    }
}