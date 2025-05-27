package kr.co.mcplink.domain.mcpsecurity.service;

import kr.co.mcplink.domain.mcpsecurity.dto.McpScanResultDto;
import lombok.RequiredArgsConstructor;
import lombok.extern.slf4j.Slf4j;
import org.springframework.context.annotation.Profile;
import org.springframework.stereotype.Service;

import java.util.List;

@Service
@RequiredArgsConstructor
@Slf4j
@Profile("!test") // 테스트 할 땐 비활성화
public class McpScanService {

	private final McpAnalysisService analysisService;
	private final McpJsonParsingService parsingService;

	// 현재는 public, 추후 private 호출 불가 설정 예정, 1시간 마다 수행
//	@Scheduled(cron = "0 0 * * * *")
	public void triggerScan() {
		List<McpScanResultDto> result = analysisService.scanSpecificServer();

		for (McpScanResultDto scanResult : result) {
			if (!scanResult.scanSuccess()) {
				log.error("파싱 패스, 실패 원인 (JSON): {}", scanResult.osvOutputJson());
				return;
			}
	
			String output = scanResult.osvOutputJson();
	
			int jsonStart = output.indexOf("{");
			if (jsonStart != -1) {
				output = output.substring(jsonStart);
			}

			parsingService.processOsvResult(output, scanResult.mcpServerId());
		}
		
	}
}
