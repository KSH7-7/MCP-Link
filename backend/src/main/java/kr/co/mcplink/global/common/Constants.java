package kr.co.mcplink.global.common;

public final class Constants {

    // 컬렉션 이름 (MongoDB)
    public static final String COLLECTION_SEQUENCE_COUNTER      = "sequence_counter";
    public static final String COLLECTION_MCP_SERVERS           = "mcp_servers";
    public static final String COLLECTION_MCP_TAGS              = "mcp_tags";
    public static final String COLLECTION_GITHUB_PENDING_QUEUE  = "github_pending_queue";
    public static final String COLLECTION_GEMINI_PENDING_QUEUE  = "gemini_pending_queue";

    // 인덱스 이름 (MongoDB)
    // stars 내림차순, seq 오름차순 복합 정렬 인덱스
    public static final String IDX_MCP_SERVERS_SORT             = "idx_mcp_servers_stars_desc_seq_asc";
    // name+description 필드에 대한 검색 인덱스
    public static final String IDX_MCP_SERVERS_NAME_SEARCH      = "idx_mcp_servers_name_search";
    public static final String IDX_MCP_SERVERS_NAME_SEARCH_KR   = "idx_mcp_servers_name_search_kr";

    // Github
    public static final String GITHUB_SEARCH_PATH               = "/search/repositories";
    public static final String GITHUB_README_PATH               = "/repos/{owner}/{repo}/readme";
    public static final String GITHUB_REPO_PATH                 = "/repos/{owner}/{repo}";
    public static final String[] GITHUB_LANGUAGES               = { "typescript", "javascript", "python" };
    public static final String[] GITHUB_LICENSES                = { "mit", "apache-2.0", "gpl-3.0" };

    // Gemini
    public static final String GEMINI_GENERATE_CONTENT_PATH     = "/v1beta/models/{model}:generateContent";
    public static final String DESCRIPTION_NOT_YET_GENERATED    = "Generating description using AI. Please refer to the README available at %s. We apologize for the inconvenience.";
    public static final String GEMINI_FALLBACK_SUMMARY_TEXT     = "Failed to generate summary using AI. Please refer to the README available at %s. We apologize for the inconvenience.";
    public static final String GEMINI_DEFAULT_FALLBACK_TEXT     = "Failed to generate summary using AI. We apologize for the inconvenience.";

    // 성공 메시지
    public static final String SUCCESS                          = "SUCCESS";
    public static final String MSG_SUCCESS_LIST                 = "전체 목록 조회에 성공했습니다";
    public static final String MSG_SUCCESS_SEARCH               = "검색 목록 조회에 성공했습니다";
    public static final String MSG_SUCCESS_BATCH                = "선택된 목록 조회에 성공했습니다";
    public static final String MSG_SUCCESS_DETAIL               = "데이터 조회에 성공했습니다.";
    public static final String MSG_SUCCESS_TAG_LIST             = "태그 목록 조회에 성공했습니다";

    // 에러 메시지
    public static final String MSG_BAD_REQUEST                  = "잘못된 요청입니다. 요청 파라미터를 확인하세요.";
    public static final String MSG_NOT_FOUND                    = "해당 파일을 찾을 수 없습니다.";
    public static final String MSG_NOT_FOUNDS                   = "요청한 파일 목록 중 일부 파일을 찾을 수 없습니다.";
    public static final String MSG_INTERNAL_ERROR               = "서버 처리 중 오류가 발생했습니다.";

    public static final String ACCESS_TOKEN_NAME                = "accessToken";
}
