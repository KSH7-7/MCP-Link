package kr.co.mcplink.domain.mcpserver.service.core;

import kr.co.mcplink.domain.mcpserver.dto.*;
import kr.co.mcplink.domain.mcpserver.dto.response.*;
import kr.co.mcplink.domain.mcpserver.entity.McpServer;
import kr.co.mcplink.domain.mcpserver.repository.McpServerRepository;
import kr.co.mcplink.domain.mcpserver.repository.McpTagRepository;
import kr.co.mcplink.global.common.ApiResponse;
import kr.co.mcplink.global.common.Constants;
import kr.co.mcplink.global.util.PaginationUtil;
import lombok.RequiredArgsConstructor;
import org.springframework.http.HttpStatus;
import org.springframework.stereotype.Service;

import java.util.List;
import java.util.stream.Collectors;

@Service
@RequiredArgsConstructor
public class McpServerService {
	private final McpServerRepository serverRepository;
	private final McpTagRepository tagRepository;

	public ApiResponse<McpListResponse> findAllServers(Integer size, Long cursorId) {
		List<McpServer> servers = serverRepository.listAll(size, cursorId);

		long total = serverRepository.countAll();
		long endCursor = servers.isEmpty() ? 0L : servers.get(servers.size() - 1).getSeq();
		long remaining = serverRepository.countRemaining(endCursor);

		PageInfoDto pageInfo = PaginationUtil.buildPageInfo(servers, total, remaining);

		List<McpSummaryDataDto> mcpServers = servers.stream()
			.map(this::toSummaryDataDto)
			.collect(Collectors.toList());

		McpListResponse response = new McpListResponse(pageInfo, mcpServers);

		return ApiResponse.success(HttpStatus.OK.toString(), Constants.MSG_SUCCESS_LIST, response);
	}

	public ApiResponse<McpSearchResponse> searchServersByName(String name, Integer size, Long cursorId) {
		List<McpServer> servers = serverRepository.searchByName(name, size, cursorId);

		long total = serverRepository.countByName(name);
		long endCursor = servers.isEmpty() ? 0L : servers.get(servers.size() - 1).getSeq();
		long remaining = serverRepository.countRemainingByName(name, endCursor);

		PageInfoDto pageInfo = PaginationUtil.buildPageInfo(servers, total, remaining);

		List<McpSummaryDataDto> mcpServers = servers.stream()
		        .map(this::toSummaryDataDto)
		        .collect(Collectors.toList());

		McpSearchResponse response = new McpSearchResponse(pageInfo, mcpServers);

		return ApiResponse.success(HttpStatus.OK.toString(), Constants.MSG_SUCCESS_SEARCH, response);
	}

	public ApiResponse<McpBatchResponse> findServersByIds(List<Long> serverIds, Integer size, Long cursorId) {
		List<Long> pageIds = PaginationUtil.slicePageIdsForBatch(serverIds, size, cursorId);
		List<McpServer> servers = serverRepository.findBySeqInOrder(pageIds);

		PageInfoDto pageInfo = PaginationUtil.buildPageInfoForBatch(serverIds, size, cursorId);

		List<McpSummaryDataDto> mcpServers = servers.stream()
				.map(this::toSummaryDataDto)
				.collect(Collectors.toList());

		McpBatchResponse response = new McpBatchResponse(pageInfo, mcpServers);

		return ApiResponse.success(HttpStatus.OK.toString(), Constants.MSG_SUCCESS_BATCH, response);
	}

	public ApiResponse<McpDetailResponse> findServerById(Long seq) {
		McpServer server = serverRepository.findBySeq(seq).orElse(null);

		if(server == null) {
			return ApiResponse.error(HttpStatus.NOT_FOUND.toString(), Constants.MSG_NOT_FOUND);
		}

		long updated = serverRepository.findAndIncrementViewsBySeq(seq);
		if (updated != 1) {
			// Exception 추가 예정
		}

		McpDetailDataDto mcpServer = toDetailDataDto(server);
		McpDetailResponse response = new McpDetailResponse(mcpServer);

		return ApiResponse.success(HttpStatus.OK.toString(), Constants.MSG_SUCCESS_DETAIL, response);
	}

	public ApiResponse<McpTagResponse> findAllTags() {
		List<String> tags = tagRepository.listAll();

		McpTagResponse response = new McpTagResponse(tags);

		return ApiResponse.success(HttpStatus.OK.toString(), Constants.MSG_SUCCESS_TAG_LIST, response);
	}

	private McpSummaryDataDto toSummaryDataDto(McpServer s) {
		return McpSummaryDataDto.builder()
			.id(s.getSeq())
			.type(s.getType())
			.url(s.getUrl())
			.stars(s.getStars())
			.views(s.getViews())
			.official(s.isOfficial())
			.scanned(s.isScanned())
			.securityRank(s.getSecurityRank())
			.mcpServer(
				McpServerSummaryDto.builder()
					.name(s.getDetail().getName())
					.description(s.getDetail().getDescription())
					.build()
			)
			.build();
	}

	private McpDetailDataDto toDetailDataDto(McpServer s) {
		return McpDetailDataDto.builder()
			.id(s.getSeq())
			.type(s.getType())
			.url(s.getUrl())
			.stars(s.getStars())
			.views(s.getViews())
			.official(s.isOfficial())
			.scanned(s.isScanned())
			.securityRank(s.getSecurityRank())
			.mcpServer(
				McpServerDetailDto.builder()
					.name(s.getDetail().getName())
					.description(s.getDetail().getDescription())
					.command(s.getDetail().getCommand())
					.args(s.getDetail().getArgs())
					.env(s.getDetail().getEnv())
					.build()
			)
			.build();
	}
}