// 알림 시스템 테스트 스크립트
const http = require("http")

// 알림 테스트 함수
function sendNotificationTest(keywords) {
  // HTTP 요청 옵션
  const options = {
    hostname: "localhost",
    port: 8082,
    path: "/recommendations",
    method: "POST",
    headers: {
      "Content-Type": "application/json",
    },
  }

  // 요청 데이터 준비
  const data = JSON.stringify({
    keywords: Array.isArray(keywords) ? keywords : [keywords],
  })

  // HTTP 요청 생성
  const req = http.request(options, (res) => {
    res.on("data", (chunk) => {})

    res.on("end", () => {})
  })

  // 오류 처리
  req.on("error", (error) => {
    console.error(`요청 오류: ${error.message}`)
  })

  // 데이터 전송
  req.write(data)
  req.end()
}

// 명령행 인자 처리 (테스트 키워드)
const testKeywords = process.argv.slice(2)
if (testKeywords.length === 0) {
} else {
  sendNotificationTest(testKeywords)
}
