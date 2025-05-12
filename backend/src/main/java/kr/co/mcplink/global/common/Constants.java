package kr.co.mcplink.global.common;

public final class Constants {

    // 컬렉션 이름 (MongoDB)
    public static final String COLLECTION_DATABASE_SEQUENCE     = "database_sequences";
    public static final String COLLECTION_MCP_SERVERS           = "mcp_servers";
    public static final String COLLECTION_MCP_TAGS              = "mcp_tags";

    // 인덱스 이름 (MongoDB)
    // stars 내림차순, seq 오름차순 복합 정렬 인덱스
    public static final String IDX_MCP_SERVERS_SORT             = "idx_mcp_servers_stars_desc_seq_asc";
    // name 필드에 대한 검색(text) 인덱스
    public static final String IDX_MCP_SERVERS_NAME_SEARCH      = "idx_mcp_servers_name_search_text";

    // 성공 메시지
    public static final String MSG_SUCCESS_LIST                 = "전체 목록 조회에 성공했습니다";
    public static final String MSG_SUCCESS_SEARCH               = "검색 목록 조회에 성공했습니다";
    public static final String MSG_SUCCESS_BATCH                = "선택된 목록 조회에 성공했습니다";
    public static final String MSG_SUCCESS_DETAIL               = "데이터 조회에 성공했습니다.";
    public static final String MSG_SUCCESS_TAG_LIST             = "태그 목록 조회에 성공했습니다";

    // 에러 메시지
    public static final String MSG_BAD_REQUEST                  = "잘못된 요청입니다. 요청 파라미터를 확인하세요.";
    public static final String MSG_NOT_FOUND                    = "해당 파일을 찾을 수 없습니다.";
    public static final String MSG_INTERNAL_ERROR               = "서버 처리 중 오류가 발생했습니다.";
}
