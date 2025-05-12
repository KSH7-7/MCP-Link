package kr.co.mcplink.domain.mcpsecurity.dto.osv;

import com.fasterxml.jackson.annotation.JsonProperty;
import java.util.List;

/**
 * OSV-Scanner JSON 출력의 최상위 배열 요소에 해당하는 DTO입니다.
 * OSV-Scanner의 결과는 이 객체들의 리스트 형태일 수 있습니다.
 */
public record OsvScanOutputWrapperDto(
	List<OsvResultDto> results,
	@JsonProperty("experimental_config") // JSON 필드명 "experimental_config"에 매핑
	OsvExperimentalConfigDto experimentalConfig
) {}