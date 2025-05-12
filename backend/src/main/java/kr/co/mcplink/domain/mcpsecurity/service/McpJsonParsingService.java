package kr.co.mcplink.domain.mcpsecurity.service;

import org.springframework.stereotype.Service;

import com.fasterxml.jackson.databind.ObjectMapper;

import kr.co.mcplink.domain.mcpsecurity.dto.osv.OsvPackageEntryDto;
import kr.co.mcplink.domain.mcpsecurity.dto.osv.OsvResultDto;
import kr.co.mcplink.domain.mcpsecurity.dto.osv.OsvScanOutputWrapperDto;
import kr.co.mcplink.domain.mcpsecurity.dto.osv.OsvVulnerabilityDto;
import lombok.RequiredArgsConstructor;
import lombok.extern.slf4j.Slf4j;

@Service
@Slf4j
@RequiredArgsConstructor
public class McpJsonParsingService {

	private final ObjectMapper objectMapper;

	public void processOsvResult(String osvOutputJson) {
		if (osvOutputJson == null || osvOutputJson.trim().isEmpty()) {
			log.warn("OSV JSON 입력이 비어있거나 null입니다. 파싱을 건너뜁니다.");
			return;
		}

		String trimmedJson = osvOutputJson.trim();

		try {
			OsvScanOutputWrapperDto osvScanOutput = objectMapper.readValue(trimmedJson, OsvScanOutputWrapperDto.class);

			if (osvScanOutput != null && osvScanOutput.results() != null && !osvScanOutput.results().isEmpty()) {
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
									log.info("패키지: {}, 버전: {}, 취약점 ID: {}, 위험도: {}",
										pkgEntry.packageInfo().name(),
										pkgEntry.packageInfo().version(),
										vulnerability.id(),
										riskSeverity);

									// 여기에 추가적인 처리 로직 (예: DB 저장)
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
}