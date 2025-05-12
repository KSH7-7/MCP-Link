package kr.co.mcplink.domain.mcpsecurity.controller;

import org.springframework.http.HttpStatus;
import org.springframework.http.ResponseEntity;
import org.springframework.web.bind.annotation.*;


import kr.co.mcplink.domain.mcpsecurity.service.McpScanService;
import kr.co.mcplink.domain.mcpsecurity.service.McpAnalysisService;
import lombok.RequiredArgsConstructor;
import lombok.extern.slf4j.Slf4j;

@RestController
@RequestMapping("/secure")
@RequiredArgsConstructor
@Slf4j
public class McpSecureController {

	private final McpAnalysisService analysisService;
	private final McpScanService scanService;

	@PostMapping("/scan/all")
	public ResponseEntity<String> scanAllServers() {
		scanService.triggerScan();

		return ResponseEntity.status(HttpStatus.ACCEPTED).body("모든 서버에 대한 스캔 작업이 시작되었습니다. 완료까지 시간이 소요될 수 있습니다.");
	}
}