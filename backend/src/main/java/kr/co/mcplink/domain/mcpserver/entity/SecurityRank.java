package kr.co.mcplink.domain.mcpserver.entity;

import lombok.extern.slf4j.Slf4j;

@Slf4j
public enum SecurityRank {
    UNRATED,
    LOW,
    MODERATE,
    HIGH,
    CRITICAL;

    public static SecurityRank fromString(String severityString) {
        if (severityString == null) {
            return UNRATED;
        }

        try {
            return SecurityRank.valueOf(severityString.trim().toUpperCase());
        } catch (IllegalArgumentException e) {
            log.error("Invalid security rank: {}", severityString);
            return SecurityRank.UNRATED;
        }
    }
}