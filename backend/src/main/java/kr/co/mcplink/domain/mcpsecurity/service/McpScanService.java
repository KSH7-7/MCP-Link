package kr.co.mcplink.domain.mcpsecurity.service;

import org.springframework.stereotype.Service;

import kr.co.mcplink.domain.mcpsecurity.dto.McpScanResultDto;
import lombok.RequiredArgsConstructor;
import lombok.extern.slf4j.Slf4j;

@Service
@RequiredArgsConstructor
@Slf4j
public class McpScanService {

	private final McpAnalysisService analysisService;
	private final McpJsonParsingService parsingService;

	// 전체 서버 스캔을 담당하는 메소드
	public void triggerScan() {
		McpScanResultDto result = analysisService.scanSpecificServer();

		if (!result.scanSuccess()) {
			log.error("실패 원인 (JSON): {}", result.osvOutputJson());
			throw new RuntimeException("서버 스캔 실패");
		}

		String output = result.osvOutputJson();

		int jsonStart = output.indexOf("{");
		if (jsonStart != -1) {
			output = output.substring(jsonStart);
		}

		parsingService.processOsvResult(output);
	}
}
