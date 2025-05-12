#!/bin/bash
set -e

# env 읽기
if [ -f .env ]; then
  set -a
    source .env
fi

IMAGE_TO_DEPLOY="${BACKEND_IMAGE_TAG}"

if [ -z "$IMAGE_TO_DEPLOY" ]; then
  echo "오류: 배포할 Docker 이미지 태그(BACKEND_IMAGE_TAG 환경 변수)가 지정되지 않았습니다."
  exit 1
fi

# 이미지 전체 이름
FULL_IMAGE_NAME="resd/backend:${IMAGE_TO_DEPLOY}"
echo "==== 배포 시작: ${IMAGE_TO_DEPLOY} ===="

# 몽고디비 실행
docker compose up -d mongodb --wait || { echo "오류: MongoDB 컨테이너 시작 실패"; exit 1; }

echo "Docker Image pull... ${IMAGE_TO_DEPLOY}"
docker pull "${FULL_IMAGE_NAME}" || { echo "오류: 이미지 풀(${FULL_IMAGE_NAME}) 실패"; exit 1; }

# 기존 백엔드 컨테이너 중지 / 삭제 / 생성 (--no-deps: no dependencies, 백엔드 자체만 적용)
echo "New Container Running... ${IMAGE_TO_DEPLOY}"
docker compose up -d --no-deps --force-recreate backend --wait

echo "Docker Image prune"
docker images resd/backend --format "{{.Repository}}:{{.Tag}}" | sort -r | tail -n +6 | xargs -r docker rmi
docker image prune -f || true