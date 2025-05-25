package kr.co.mcplink.domain.github.dto;

import com.fasterxml.jackson.annotation.JsonCreator;
import com.fasterxml.jackson.annotation.JsonIgnoreProperties;
import com.fasterxml.jackson.annotation.JsonProperty;
import kr.co.mcplink.domain.mcpserver.entity.SecurityRank;

@JsonIgnoreProperties(ignoreUnknown = true)
public record GithubMetaDataDto(
        String url,
        int stars,
        boolean official,
        boolean scanned,
        SecurityRank securityRank
) {
    @JsonCreator
    public GithubMetaDataDto(
            @JsonProperty("clone_url") String url,
            @JsonProperty("stargazers_count") int stars
    ) {
        this(
                url,
                stars,
                false,
                false,
                SecurityRank.UNRATED
        );
    }
}