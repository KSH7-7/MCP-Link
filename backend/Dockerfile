FROM gradle:8.11.1-jdk17 AS builder

WORKDIR /app

COPY gradlew .
COPY gradle ./gradle
COPY build.gradle .
COPY settings.gradle .

COPY src ./src

RUN chmod +x gradlew
RUN ./gradlew clean build -x test

FROM bellsoft/liberica-openjdk-alpine:17
# 새로 추가 git 설치
RUN apk add --no-cache git ca-certificates wget

# https://github.com/google/osv-scanner/releases
ENV OSV_SCANNER_VERSION=v2.0.2
RUN wget https://github.com/google/osv-scanner/releases/download/${OSV_SCANNER_VERSION}/osv-scanner_linux_amd64 -O /usr/local/bin/osv-scanner && \
    chmod +x /usr/local/bin/osv-scanner && \
    osv-scanner --version \
# 추가 끝

WORKDIR /app

COPY --from=builder /app/build/libs/mcplink-0.0.1-SNAPSHOT.jar /app/mcplink.jar

ENTRYPOINT ["java", "-jar", "/app/mcplink.jar"]

EXPOSE 8080