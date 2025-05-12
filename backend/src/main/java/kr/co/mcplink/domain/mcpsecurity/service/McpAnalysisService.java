package kr.co.mcplink.domain.mcpsecurity.service;

import java.io.BufferedReader;
import java.io.IOException;
import java.io.InputStreamReader;
import java.nio.charset.StandardCharsets;
import java.nio.file.Files;
import java.nio.file.Path;
import java.nio.file.Paths;
import java.util.Comparator;
import java.util.List;
import java.util.concurrent.TimeUnit;

import org.springframework.beans.factory.annotation.Value;
import org.springframework.stereotype.Service;

import jakarta.annotation.PostConstruct;
import kr.co.mcplink.domain.mcpsecurity.dto.McpScanResultDto;
import lombok.extern.slf4j.Slf4j;

/**
 * 작업 흐름도:
 * 1. getGitCloneUrl()로 깃 클론할 URL 획득
 * 2. cloneRepository()로 리포지토리 클론
 * 3. runOsvScanner()로 OSV-Scanner 실행
 * 4. cleanup()로 임시 디렉토리 삭제
 */
@Service
@Slf4j
public class McpAnalysisService {

	private Path tempDir;

	@Value("${app.analysis.osv-scanner-cmd}")
	private String osvScannerCommand;

	@Value("${app.analysis.temp-dir}")
	private String tempDirStr;

	@PostConstruct
	public void init() {
		tempDir = Paths.get(tempDirStr);
		try {
			Files.createDirectories(tempDir);
		} catch (IOException e) {
			log.error("임시 분석 디렉토리 생성 실패: {}", tempDir, e);
			throw new RuntimeException("임시 분석 디렉토리 생성 실패", e);
		}
	}

	/**
	 * 모든 몽고DB에 있는 Git URL OSV 스캔, 현재는 노션 하드코딩
	 * @return McpServerScanResultDto 스캔 결과를 담은 DTO
	 */
	public McpScanResultDto scanSpecificServer() {
		// 1. Git URL 획득
		List<String> gitUrls = getGitCloneUrl();

		String serverId = "notion-mcp-server-id"; // 식별을 위한 임의의 ID
		String serverName = "Notion MCP Server";    // 식별을 위한 임의의 이름

		Path cloneDir = tempDir.resolve("notion-mcp-server"); // 클론된 리포지토리 이름
		Path reportOutputFile = tempDir.resolve(serverId + "_" + serverName + ".json"); // 결과 저장 이름


		log.info("지정된 URL 스캔 시작: {}, 클론 위치: {}, 리포트 파일: {}", gitUrls, cloneDir, reportOutputFile);
		McpScanResultDto result = performScanForUrl(gitUrls, serverId, serverName, cloneDir);

		if (result.scanSuccess() && result.osvOutputJson() != null) {
			saveReport(reportOutputFile, result.osvOutputJson());
		}

		return result;
	}

	public List<String> getGitCloneUrl() {
		// TODO: 나중에 DB나 설정에서 동적으로 가져오도록 변경

		return List.of("https://github.com/makenotion/notion-mcp-server.git");
	}

	/**
	 * 주어진 Git URL에 대해 클론, OSV 스캔, 정리를 수행하는 내부 메소드.
	 * @param cloneDir 스캔을 위해 리포지토리를 클론할 대상 디렉토리
	 */
	private McpScanResultDto performScanForUrl(List<String> gitUrl, String serverId, String serverName, Path cloneDir) {
		String osvJsonOutput = null; // 결과 JSON
		boolean success = true;

		try {
			// 1. 클론 대상 디렉토리 생성
			Files.createDirectories(cloneDir);

			// 2. 클론
			log.info("'{}' 클론 시작...", gitUrl);
			if (!cloneRepository(gitUrl, cloneDir)) {
				log.error("Git 리포지토리 클론 실패: {}", gitUrl);
				success = false;
			}

			// 3. OSV 스캔
			log.info("'{}' 디렉토리에 OSV-Scanner 실행...", cloneDir);
			osvJsonOutput = runOsvScanner(cloneDir);

			// osv-scanner 실행 중 오류 확인
			if (osvJsonOutput == null || osvJsonOutput.trim().startsWith("{\"error\":")) {
				success = false;
			}

			log.info("'{}' OSV-Scanner 실행 완료.", cloneDir);
		} catch (IOException | InterruptedException e) {
			log.error("'{}' (URL: '{}') 스캔 중 오류 발생", serverName, gitUrl, e);
			Thread.currentThread().interrupt();
			success = false;
		} finally {
			cleanup(cloneDir); // 디렉토리 삭제
		}

		// 실패
		if (!success) {
			log.error("'{}' OSV-Scanner 실행 실패, 결과: {}", cloneDir, osvJsonOutput);
			return new McpScanResultDto(serverId, serverName, gitUrl.get(0), false, null);
		}

		// 성공
		return new McpScanResultDto(serverId, serverName, gitUrl.get(0), success, osvJsonOutput);
	}

	// 2. 클론 실행
	public boolean cloneRepository(List<String> gitUrls, Path targetDir) throws IOException, InterruptedException {
		// 터미널 명령어 실행, 커밋기록 안가져오는 clone (얕은 클론)
		// git clone --depth 1 https://github.com/makenotion/notion-mcp-server.git /app/analysis_temp/notion-mcp-server
		ProcessBuilder processBuilder = new ProcessBuilder(
			"git", "clone", "--depth", "1", gitUrls.get(0), targetDir.toString());
		processBuilder.redirectErrorStream(true);

		log.info("Git 클론 실행: {}", String.join(" ", processBuilder.command()));
		Process process = processBuilder.start();

		StringBuilder gitOutput = new StringBuilder();
		try (BufferedReader reader = new BufferedReader(
			new InputStreamReader(process.getInputStream(), StandardCharsets.UTF_8))) {
			String line;
			while ((line = reader.readLine()) != null) {
				gitOutput.append(line).append(System.lineSeparator());
				log.trace("GIT CLONE: {}", line);
			}
		}
		log.info("Git 클론 결과 ({}):\n{}", gitUrls, gitOutput);

		boolean finished = process.waitFor(5, TimeUnit.MINUTES); // 타임아웃 5분
		if (!finished) {
			process.destroyForcibly();
			log.error("Git 클론 타임아웃: {}", gitUrls.get(0));
			return false;
		}

		return process.exitValue() == 0;
	}

	// 3. OSV 스캔 실행
	public String runOsvScanner(Path cloneDir) throws IOException, InterruptedException {
		// 명령어 순서: osv-scanner scan source --format json -r .
		ProcessBuilder processBuilder = new ProcessBuilder(
			osvScannerCommand, "scan", "source", "--format", "json", "-r", "."
		);
		processBuilder.directory(cloneDir.toFile()); // 작업 디렉토리 설정
		processBuilder.redirectErrorStream(true);

		log.info("run OSV-Scanner 실행: {} (작업폴더: {})", String.join(" ", processBuilder.command()), cloneDir);
		Process process = processBuilder.start();

		StringBuilder output = new StringBuilder();
		// 외부 프로세스 출력은 시스템 기본 인코딩 또는 UTF-8 사용 시도
		try (BufferedReader reader = new BufferedReader(
			new InputStreamReader(process.getInputStream(), StandardCharsets.UTF_8))) {
			String line;
			while ((line = reader.readLine()) != null) {
				output.append(line).append(System.lineSeparator());
			}
		}

		boolean finished = process.waitFor(10, TimeUnit.MINUTES); // 타임아웃 10분
		int exitCode = -1;
		if (!finished) {
			process.destroyForcibly();
			log.error("OSV-Scanner 타임아웃: {}", cloneDir);
			// 타임아웃 시 에러 JSON 반환
			return String.format("{\"error\": \"OSV-Scanner 실행 시간 초과 (경로: %s)\"}",
				cloneDir.toString().replace("\\", "\\\\"));
		}
		exitCode = process.exitValue();

		log.info("OSV-Scanner ({}): 종료 코드 {}. 출력 길이: {}", cloneDir, exitCode, output.length());
		// 종료 코드가 0(성공) 또는 1(취약점 발견)이 아니면서 출력이 비어있다면 문제로 간주
		if (output.toString().trim().isEmpty() && exitCode != 0 && exitCode != 1) {
			log.warn("OSV-Scanner ({}) 종료 코드 {}와 함께 비어있는 출력을 반환했습니다.", cloneDir, exitCode);
			// 비어있는 출력 대신 에러 JSON 반환
			return String.format("{\"error\": \"OSV-Scanner가 종료 코드 %d와 함께 비어있는 출력을 반환했습니다. (경로: %s)\"}", exitCode,
				cloneDir.toString().replace("\\", "\\\\"));
		}

		return output.toString();
	}

	public void cleanup(Path dir) {
		if (dir != null && Files.exists(dir)) {
			log.info("임시 디렉토리 삭제 시도: {}", dir);
			try {
				Files.walk(dir)
					.sorted(Comparator.reverseOrder())
					.map(Path::toFile)
					.forEach(file -> {
						if (!file.delete()) {
							log.warn("파일 삭제 실패: {}", file.getAbsolutePath());
						}
					});
				if (Files.exists(dir) && !dir.toFile().delete()) {
					log.warn("최종 디렉토리 삭제 실패: {}. 수동 삭제가 필요할 수 있습니다.", dir);
				} else if (!Files.exists(dir)) {
					log.info("임시 디렉토리 삭제 완료: {}", dir);
				}
			} catch (IOException e) {
				log.error("임시 디렉토리 삭제 중 오류 발생 {}: {}", dir, e.getMessage(), e);
			}
		}
	}

	/**
	 * OSV-Scanner 실행 결과 JSON 파일에 저장합니다.
	 * @param reportOutputFile 출력 파일 경로
	 * @param json JSON 콘텐츠
	 */
	private void saveReport(Path reportOutputFile, String json) {
		try {
			Files.writeString(reportOutputFile, json, StandardCharsets.UTF_8);
			log.info("OSV-Scanner JSON 출력을 파일에 저장했습니다: {}", reportOutputFile);
		} catch (IOException e) {
			log.error("OSV-Scanner JSON 출력을 파일에 저장 중 오류 발생 {}: {}", reportOutputFile, e.getMessage(), e);
		}
	}

}