package kr.co.mcplink.domain.gemini.service;

import com.fasterxml.jackson.databind.JsonNode;
import com.fasterxml.jackson.databind.ObjectMapper;
import kr.co.mcplink.domain.gemini.client.GeminiApiClient;
import kr.co.mcplink.domain.gemini.dto.GeminiRequestDto;
import kr.co.mcplink.domain.gemini.dto.GeminiResponseDto;
import lombok.RequiredArgsConstructor;
import lombok.extern.slf4j.Slf4j;
import org.springframework.stereotype.Service;

import java.util.Optional;
import java.util.regex.Matcher;
import java.util.regex.Pattern;

@Service
@Slf4j
@RequiredArgsConstructor
public class FetchTranslateService {

    private static final ObjectMapper objectMapper = new ObjectMapper();
    private final GeminiApiClient geminiClient;

    public String fetchDescriptionKr(String descriptionEn) {
        try {
            if (descriptionEn == null || descriptionEn.trim().isEmpty()) {
                log.warn("Description is empty, cannot translate to KR");
                return descriptionEn;
            }

            GeminiRequestDto request = createDescriptionKrRequest(descriptionEn);
            GeminiResponseDto response = geminiClient.generateContent(request).block();
            String descriptionKr = extractDescriptionKr(response);

            if (descriptionKr == null || descriptionKr.isEmpty()) {
                log.info("No KR description generated, using original: {}", descriptionEn);
                return descriptionEn;
            }
            return descriptionKr;

        } catch (Exception e) {
            log.error("Error fetching KR description for '{}': {}", descriptionEn, e.getMessage());
            return descriptionEn;
        }
    }

    private GeminiRequestDto createDescriptionKrRequest(String descriptionEn) {
        String prompt = buildDescriptionKrPrompt(descriptionEn);
        return GeminiRequestDto.createRequest(prompt);
    }

    private String buildDescriptionKrPrompt(String descriptionEn) {
        return """
        Translate the following English description into Korean.
    
        Translation Guidelines:
        - Produce a clear and accurate translation in natural, professional Korean.
        - Preserve all technical terms and details from the original.
        - Keep it concise (1–2 sentences) and faithful to the source.
        
        Respond ONLY with valid JSON in this format:
        {
            "description": "여기에 번역된 한국어 설명"
        }
        
        English description:
        %s
        """.formatted(descriptionEn);
    }

    private String extractDescriptionKr(GeminiResponseDto response) {
        try {
            String text = extractTextSafely(response);
            return parseJsonForDescription(text);
        } catch (Exception e) {
            log.error("Failed to parse KR description response: ", e);
            return "";
        }
    }

    private String extractTextSafely(GeminiResponseDto response) {
        return Optional.ofNullable(response)
                .map(GeminiResponseDto::candidates)
                .filter(cands -> !cands.isEmpty())
                .map(cands -> cands.get(0))
                .map(GeminiResponseDto.CandidateDto::content)
                .map(GeminiResponseDto.ContentDto::parts)
                .filter(parts -> !parts.isEmpty())
                .map(parts -> parts.get(0))
                .map(GeminiResponseDto.PartDto::text)
                .orElse("");
    }

    private String parseJsonForDescription(String jsonText) throws Exception {
        Pattern p = Pattern.compile("\\{[\\s\\S]*\\}", Pattern.DOTALL);
        Matcher m = p.matcher(jsonText);
        if (m.find()) {
            String j = m.group();
            JsonNode root = objectMapper.readTree(j);
            if (root.has("description")) {
                return root.get("description").asText();
            }
        }
        return "";
    }
}