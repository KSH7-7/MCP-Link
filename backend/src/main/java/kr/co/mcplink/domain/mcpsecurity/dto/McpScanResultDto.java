package kr.co.mcplink.domain.mcpsecurity.dto;

public record McpScanResultDto(
	String mcpServerId,
	String mcpServerName,
	String gitUrl,
	boolean scanSuccess,
	String osvOutputJson // OSV-Scanner의 원본 JSON 출력
) {}