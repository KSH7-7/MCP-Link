// 알림 테스트 스크립트
// Node.js 환경에서 실행할 수 있는 스크립트

const http = require('http');

// 8082 포트로 HTTP 요청 보내기
async function sendKeywords(keywords) {
  return new Promise((resolve, reject) => {
    // 요청 데이터 구성
    const requestData = JSON.stringify({
      keywords: keywords
    });

    // 요청 옵션
    const options = {
      hostname: 'localhost',
      port: 8082,
      path: '/recommendations',
      method: 'POST',
      headers: {
        'Content-Type': 'application/json',
        'Content-Length': Buffer.byteLength(requestData)
      }
    };

    // 요청 생성
    const req = http.request(options, (res) => {
      let data = '';
      
      // 응답 데이터 수신
      res.on('data', (chunk) => {
        data += chunk;
      });
      
      // 응답 완료
      res.on('end', () => {
        console.log('Status Code:', res.statusCode);
        console.log('Response:', data);
        resolve(res.statusCode);
      });
    });
    
    // 에러 처리
    req.on('error', (error) => {
      console.error('Error sending request:', error);
      reject(error);
    });
    
    // 요청 데이터 전송
    req.write(requestData);
    req.end();
  });
}

// 테스트 실행
async function runTest() {
  console.log('테스트 알림 발송 중...');
  
  try {
    // 키워드 목록
    const keywords = ['React', 'Svelte', 'Vue'];
    
    // 요청 보내기
    const statusCode = await sendKeywords(keywords);
    
    if (statusCode === 200) {
      console.log('알림 발송 성공!');
      console.log(`보낸 키워드: ${keywords.join(', ')}`);
    } else {
      console.log('알림 발송 실패. 상태 코드:', statusCode);
    }
  } catch (error) {
    console.error('테스트 실행 중 오류 발생:', error);
  }
}

// 스크립트 직접 실행 시 테스트 실행
if (require.main === module) {
  runTest();
}

module.exports = { sendKeywords, runTest };