// 알림 시스템 테스트 스크립트
const http = require('http');

// 알림 테스트 함수
function sendNotificationTest(keywords) {
  // HTTP 요청 옵션
  const options = {
    hostname: 'localhost',
    port: 8082,
    path: '/recommendations',
    method: 'POST',
    headers: {
      'Content-Type': 'application/json'
    }
  };

  // 요청 데이터 준비
  const data = JSON.stringify({
    keywords: Array.isArray(keywords) ? keywords : [keywords]
  });

  // HTTP 요청 생성
  const req = http.request(options, (res) => {
    console.log(`상태 코드: ${res.statusCode}`);
    
    res.on('data', (chunk) => {
      console.log(`응답 데이터: ${chunk}`);
    });
    
    res.on('end', () => {
      console.log('응답 완료');
    });
  });

  // 오류 처리
  req.on('error', (error) => {
    console.error(`요청 오류: ${error.message}`);
  });

  // 데이터 전송
  req.write(data);
  req.end();
}

// 명령행 인자 처리 (테스트 키워드)
const testKeywords = process.argv.slice(2);
if (testKeywords.length === 0) {
  console.log('사용법: node test-notification.js 키워드1 [키워드2 ...]');
  console.log('예시: node test-notification.js "Docker" "Kubernetes"');
} else {
  console.log(`알림 테스트 실행: ${testKeywords.join(', ')}`);
  sendNotificationTest(testKeywords);
}