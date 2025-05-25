package kr.co.mcplink.domain.gemini.service;

import com.fasterxml.jackson.databind.JsonNode;
import com.fasterxml.jackson.databind.ObjectMapper;
import kr.co.mcplink.domain.gemini.client.GeminiApiClient;
import kr.co.mcplink.domain.gemini.dto.GeminiRequestDto;
import kr.co.mcplink.domain.gemini.dto.GeminiResponseDto;
import lombok.RequiredArgsConstructor;
import lombok.extern.slf4j.Slf4j;
import org.springframework.stereotype.Service;

import java.util.*;
import java.util.regex.Matcher;
import java.util.regex.Pattern;
import java.util.stream.Collectors;
import java.util.stream.StreamSupport;

@Service
@Slf4j
@RequiredArgsConstructor
public class FetchTagService {

    private static final ObjectMapper objectMapper = new ObjectMapper();
    private final GeminiApiClient geminiClient;

    public List<String> fetchTags(String serverName) {
        try {
            if (serverName == null || serverName.trim().isEmpty()) {
                log.warn("Server name is empty, cannot generate tags");
                return List.of(serverName);
            }

            GeminiRequestDto request = createTagRequest(serverName);
            GeminiResponseDto response = geminiClient.generateContent(request).block();
            List<String> tags = extractTags(response);

            if (tags.isEmpty()) {
                log.info("Failed to generate tags, using server name as tag: {}", serverName);
                return List.of(serverName);
            }
            return tags;

        } catch (Exception e) {
            log.error("Error fetching tags for server name '{}': {}", serverName, e.getMessage());
            return List.of(serverName);
        }
    }

    public List<String> fetchTagsKr(String serverName) {
        try {
            if (serverName == null || serverName.trim().isEmpty()) {
                log.warn("Server name is empty, cannot generate tags");
                return List.of(serverName);
            }

            GeminiRequestDto request = createTagKrRequest(serverName);
            GeminiResponseDto response = geminiClient.generateContent(request).block();
            List<String> rawTags = extractTags(response);

            if (rawTags.isEmpty()) {
                log.info("Failed to generate tags, using server name as tag: {}", serverName);
                return List.of(serverName);
            }

            LinkedHashSet<String> filtered = rawTags.stream()
                    .filter(t -> !t.matches("^[a-zA-Z0-9\\- ]+$"))
                    .collect(Collectors.toCollection(LinkedHashSet::new));

            log.debug("KR tags raw={} filtered={}", rawTags, filtered);

            return new ArrayList<>(filtered);

        } catch (Exception e) {
            log.error("Error fetching tags for server name '{}': {}", serverName, e.getMessage());
            return List.of(serverName);
        }
    }

    private GeminiRequestDto createTagRequest(String serverName) {
        String prompt = buildTagPrompt(serverName);
        return GeminiRequestDto.createRequest(prompt);
    }

    private GeminiRequestDto createTagKrRequest(String serverName) {
        String prompt = buildTagKrPrompt(serverName);
        return GeminiRequestDto.createRequest(prompt);
    }

    private String buildTagPrompt(String serverName) {
        return """
        Generate relevant tags from the following MCP server name: "%s"
        
        Tag Generation Rules:
        1. Exclude the words "mcp" and "server" from tags
        2. Convert all tags to lowercase
        3. For hyphenated words (e.g., "google-maps"):
            - Include the combined version with space instead of hyphen ("google maps")
            - Include each component as a separate tag ("google", "maps")
        4. Handle camelCase by converting to lowercase with spaces and breaking into parts:
            - Example: "googleMaps" → include "google maps", "google", "maps"
        5. For meaningful compound words, include both the original word and the expanded version
            - Example: "gdrive" → include both "gdrive" and "google drive"
            - Example: "mcpGdrive" → include "gdrive" and "google drive" (exclude "mcp")
        6. Keep acronyms as is (e.g., "aws", "gcp")
        7. Do not generate any single-character tags (like "a", "b", "c", "1", "2", "3")
        8. For compound phrases, keep them intact including any single characters:
            - Example: "what-a-wonderful" → include "what a wonderful" as a tag
            - Example: "gpt-mcp-1" → include "gpt mcp 1" as a tag
        9. Exclude generic stop words such as "what", "my", "how", "your", "the", "a", "an", "and", "or", "in", "on"
        10. Only include distinctive, domain-specific keywords or proper nouns (e.g., "map", "notion", "google")
        
        Please respond ONLY with valid JSON in the following format:
        {
            "tags": ["tag1", "tag2", "tag3", ...]
        }
        """.formatted(serverName);
    }

    private String buildTagKrPrompt(String serverName) {
        return """
        Generate relevant Korean tags from the following MCP server name: "%s"
    
        Tag Generation Process:
        1.  Create a list of intermediate English tags using the 'English Tag Generation Rules' below.
        2.  For each English tag generated, apply the 'Korean Translation Rules' to produce one or more Korean tags.
        3.  Collect all generated Korean tags, remove duplicates, and form the final list.
        4.  The intermediate English tags are for internal processing only and must NOT appear in the final output.
        5.  The final JSON response MUST contain only Korean tags; do NOT include any English terms.
    
        English Tag Generation Rules:
        1.  Exclude the words "mcp" and "server" from tags.
        2.  Convert all tags to lowercase.
        3.  For hyphenated words (e.g., "google-maps"):
            -   Include the combined version with a space instead of a hyphen ("google maps").
            -   Include each component as a separate tag ("google", "maps").
        4.  Handle camelCase (e.g., "googleMaps"):
            -   Convert to lowercase with spaces ("google maps") and break into parts ("google", "maps").
        5.  For meaningful compound words (e.g., "gdrive"):
            -   Include both the original word ("gdrive") and the expanded version ("google drive").
            -   Example: "mcpGdrive" → include "gdrive" and "google drive" (exclude "mcp").
        6.  Keep acronyms as is (e.g., "aws", "gcp").
        7.  Do not generate any single-character English tags (unless part of Rule 8).
        8.  Keep compound phrases intact, including any single characters or numbers:
            -   Example: "what-a-wonderful" → "what a wonderful".
            -   Example: "gpt-mcp-1" → "gpt mcp 1".
        9.  Exclude generic stop words (e.g., "what", "my", "how", "your", "the", "a", "an", "and", "or", "in", "on").
        10. Only include distinctive, domain-specific keywords or proper nouns.
    
        Korean Translation Rules:
        1.  For brand names, product names, or proper nouns (e.g., Google, AWS, GitHub):
            -   ONLY provide the Korean transliteration (how Koreans pronounce it).
            -   Example: "google" → ["구글"]
            -   Example: "aws" → ["에이더블유에스"]
            -   NEVER include generic meanings or concepts.
        2.  For common nouns or verbs (e.g., chat, map, search):
            -   Include BOTH the Korean transliteration AND Korean translations.
            -   Example: "chat" → ["챗", "채팅"]
            -   Example: "maps" → ["맵스", "지도"]
        3.  For compound terms:
            A.  With brand names (e.g., GitLab):
                -   Treat as a single transliteration unit. Example: "gitlab" → ["깃랩"]
            B.  With brand name + common word (e.g., Google Maps):
                -   Keep the structure intact; transliterate the brand name, translate/transliterate the common word.
                -   Example: "google maps" → ["구글 맵스", "구글 지도"]
                -   Example: "gpt 1" → ["지피티 1"]
        4.  Do NOT include ANY English words in your final response. The output must be entirely in Korean (except for numbers).
        5.  Exclude single *Korean* character tags (like "맵", "챗") from the final list, unless they are part of a larger term (like "지피티 1").
    
        Please respond ONLY with valid JSON in the following format:
        {
            "tags": ["korean_tag1", "korean_tag2", "korean_tag3", ...]
        }
        """.formatted(serverName);
    }

    private List<String> extractTags(GeminiResponseDto response) {
        try {
            String generatedText = extractTextSafely(response);
            return parseJsonForTags(generatedText);
        } catch (Exception e) {
            log.error("Error parsing response to extract tags: ", e);
            return Collections.emptyList();
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

    private List<String> parseJsonForTags(String jsonText) {
        try {
            Pattern pattern = Pattern.compile("\\{[\\s\\S]*\\}", Pattern.DOTALL);
            Matcher matcher = pattern.matcher(jsonText);

            if (matcher.find()) {
                String cleanedJson = matcher.group();
                JsonNode root = objectMapper.readTree(cleanedJson);

                if (root.has("tags") && root.get("tags").isArray()) {
                    return StreamSupport.stream(root.get("tags").spliterator(), false)
                            .map(JsonNode::asText)
                            .toList();
                }
            }

            log.warn("Failed to extract tags JSON from response: {}", jsonText);
            return Collections.emptyList();
        } catch (Exception e) {
            log.error("Failed to parse JSON response for tags: {}", jsonText, e);
            return Collections.emptyList();
        }
    }
}