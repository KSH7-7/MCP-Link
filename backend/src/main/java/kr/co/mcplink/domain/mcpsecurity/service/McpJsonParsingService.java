package kr.co.mcplink.domain.mcpsecurity.service;

import com.fasterxml.jackson.databind.ObjectMapper;
import kr.co.mcplink.domain.mcpsecurity.dto.osv.OsvPackageEntryDto;
import kr.co.mcplink.domain.mcpsecurity.dto.osv.OsvResultDto;
import kr.co.mcplink.domain.mcpsecurity.dto.osv.OsvScanOutputWrapperDto;
import kr.co.mcplink.domain.mcpsecurity.dto.osv.OsvVulnerabilityDto;
import kr.co.mcplink.domain.mcpserver.repository.McpServerRepository;
import kr.co.mcplink.domain.mcpserver.entity.SecurityRank;
import lombok.RequiredArgsConstructor;
import lombok.extern.slf4j.Slf4j;
import org.springframework.stereotype.Service;

@Service
@Slf4j
@RequiredArgsConstructor
public class McpJsonParsingService {

	private final ObjectMapper objectMapper;
	private final McpServerRepository serverRepository;

	// @Transactional
	public void processOsvResult(String osvOutputJson, String mcpServerId) {
		// 결과가 비어있으면 LOW 단계
		if (osvOutputJson.contains("\"results\": [],")) {
			log.warn("result clean, security level is LOW");
			updateRepository(mcpServerId, SecurityRank.LOW);
			return;
		}

		String trimmedJson = osvOutputJson.trim();

		// JSON 파싱 전 순수 JSON 부분 추출 (로그 메시지 제거)
		int start = trimmedJson.indexOf('{');
		int end = trimmedJson.lastIndexOf('}');
		if (start >= 0 && end >= start) {
			trimmedJson = trimmedJson.substring(start, end + 1);
		}

		try {
			OsvScanOutputWrapperDto osvScanOutput = objectMapper.readValue(trimmedJson, OsvScanOutputWrapperDto.class);

			if (osvScanOutput.results() != null && !osvScanOutput.results().isEmpty()) {
				for (OsvResultDto resultItem : osvScanOutput.results()) {
					if (resultItem.packages() != null) {
						for (OsvPackageEntryDto pkgEntry : resultItem.packages()) {
							if (pkgEntry.vulnerabilities() != null) {
								for (OsvVulnerabilityDto vulnerability : pkgEntry.vulnerabilities()) {
									String riskSeverity = "UNKNOWN";
									if (vulnerability.databaseSpecific() != null
										&& vulnerability.databaseSpecific().severity() != null) {
										riskSeverity = vulnerability.databaseSpecific().severity();
									}
									log.info("패키지: {}, 버전: {}, 취약점 ID: {}, security_rank: {}",
										pkgEntry.packageInfo().name(),
										pkgEntry.packageInfo().version(),
										vulnerability.id(),
										riskSeverity);

									// DB 반영
									updateRepository(mcpServerId, SecurityRank.fromString(riskSeverity));
								}
							}
						}
					}
				}
			}
		} catch (Exception e) {
			log.error("OSV JSON 파싱 중 오류 발생 (타입 불일치 등): {}", e.getMessage(), e);
			log.debug("오류 발생 JSON 문자열 (앞 200자): {}", trimmedJson.substring(0, Math.min(trimmedJson.length(), 200)));
		}
	}

	private void updateRepository(String mcpServerId, SecurityRank securityRank) {
		serverRepository.updateScannedStatusById(mcpServerId);
		serverRepository.updateSecurityRankById(mcpServerId, securityRank);
	}

}