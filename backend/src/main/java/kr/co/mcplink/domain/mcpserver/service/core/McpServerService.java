package kr.co.mcplink.domain.mcpserver.service.core;

import java.time.Instant;
import java.util.List;
import java.util.stream.Collectors;

import org.springframework.http.HttpStatus;
import org.springframework.stereotype.Service;

import kr.co.mcplink.domain.mcpserver.dto.McpDetailDataDto;
import kr.co.mcplink.domain.mcpserver.dto.McpDetailDto;
import kr.co.mcplink.domain.mcpserver.dto.McpSummaryDataDto;
import kr.co.mcplink.domain.mcpserver.dto.McpSummaryDto;
import kr.co.mcplink.domain.mcpserver.dto.PageInfoDto;
import kr.co.mcplink.domain.mcpserver.dto.request.McpServerSearchRequest;
import kr.co.mcplink.domain.mcpserver.dto.response.McpListResponse;
import kr.co.mcplink.domain.mcpserver.dto.response.McpSearchByNameResponse;
import kr.co.mcplink.domain.mcpserver.entity.McpServer;
import kr.co.mcplink.domain.mcpserver.repository.McpServerRepository;
import kr.co.mcplink.domain.mcpserver.repository.McpTagRepository;
import kr.co.mcplink.domain.mcpserver.service.support.ViewCountUpdaterService;
import kr.co.mcplink.global.common.ApiResponse;
import kr.co.mcplink.global.common.Constants;
import kr.co.mcplink.global.util.PaginationUtil;
import lombok.RequiredArgsConstructor;

@Service
@RequiredArgsConstructor
public class McpServerService {
	private final McpServerRepository serverRepository;
	private final McpTagRepository tagRepository;
	private final ViewCountUpdaterService viewCountUpdaterService;
	private final McpServerRepository mcpServerRepository;

	public ApiResponse<McpListResponse> getLists(Integer size, Long cursorId) {
		List<McpServer> items = serverRepository.listAll(size, cursorId);
		long total = serverRepository.countAll();
		long endCursor = items.isEmpty() ? 0L : items.get(items.size() - 1).getSeq();
		long remaining = serverRepository.countRemaining(endCursor);

		PageInfoDto pageInfo = PaginationUtil.buildPageInfo(items, total, remaining);
		List<McpSummaryDataDto> data = items.stream()
			.map(this::toSummaryDataDto)
			.collect(Collectors.toList());

		McpListResponse response = new McpListResponse(pageInfo, data);

		return ApiResponse.success(HttpStatus.OK.toString(), "전체 목록", List.of(response));
	}

	public ApiResponse<McpSearchByNameResponse> searchByName(McpServerSearchRequest request) {
		// 디폴트 값 설정
		int size = request.size().orElse(5);
		long cursorId = request.cursorId().orElse(0L);

		List<McpServer> items = serverRepository.searchByName(request.mcpServerName(), size, cursorId);

		long total = serverRepository.countByName(request.mcpServerName());
		long endCursor = items.isEmpty() ? 0L : items.get(items.size() - 1).getSeq();
		long remaining = serverRepository.countRemainingByName(request.mcpServerName(), endCursor);

		PageInfoDto pageInfo = PaginationUtil.buildPageInfo(items, total, remaining);

		List<McpSummaryDataDto> data = items.stream()
		        .map(this::toSummaryDataDto)
		        .collect(Collectors.toList());

		McpSearchByNameResponse response = new McpSearchByNameResponse(pageInfo, data);

		return ApiResponse.success(HttpStatus.OK.toString(), "이름 검색결과", List.of(response));
	}

	public ApiResponse<McpDetailDataDto> getDetail(Long seq) {
		McpServer server = serverRepository.findBySeq(seq).orElse(null);

		if(server == null) {
			return ApiResponse.error(HttpStatus.NOT_FOUND.toString(), Constants.MSG_NOT_FOUND);
		}

		mcpServerRepository.incrementViews(seq);
		McpDetailDataDto data = toDetailDataDto(server);

		return ApiResponse.success(HttpStatus.OK.toString(), "상세 정보", List.of(data));
	}

	public ApiResponse<String> listTags() {
		return ApiResponse.success(HttpStatus.OK.toString(), "태그 목록", tagRepository.listAll());
	}

	private McpSummaryDataDto toSummaryDataDto(McpServer s) {
		return McpSummaryDataDto.builder()
			.id(s.getSeq())
			.type(s.getType())
			.url(s.getUrl())
			.stars(s.getStars())
			.views(s.getViews())
			.scanned(s.isScanned())
			.mcpServers(
				McpSummaryDto.builder()
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
			.scanned(s.isScanned())
			.mcpServers(
				McpDetailDto.builder()
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