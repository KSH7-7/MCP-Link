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

WORKDIR /app

COPY --from=builder /app/build/libs/mcplink-0.0.1-SNAPSHOT.jar /app/mcplink.jar

ENTRYPOINT ["java", "-jar", "/app/mcplink.jar"]

EXPOSE 8080