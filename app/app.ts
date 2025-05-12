// app.ts -> 임시 테스트용 파일
import { Client } from "https://deno.land/x/postgres@v0.17.0/mod.ts"

const DB_CONFIG = {
  user: "postgres", // 도커 PostgreSQL 기본 유저
  password: "1111", // 도커 설정에서의 비밀번호
  database: "test_db", // 도커 설정에서의 DB 이름
  hostname: "localhost",
  port: 5432,
}

const client = new Client(DB_CONFIG)

async function handler(request: Request): Promise<Response> {
  const url = new URL(request.url)
  const searchTerm = url.searchParams.get("search")?.toLowerCase() // 검색어 파라미터 지원

  // Tauri 앱에서 사용 중인 경로로 변경 (/api/mcp-cards)
  if (url.pathname === "/api/mcp-cards" && request.method === "GET") {
    console.log("요청 URL:", request.url)
    console.log("검색어:", searchTerm)
    try {
      await client.connect()

      let query = "SELECT id, title, description, url, stars FROM test_table"
      const params = []

      // 검색어가 있으면 WHERE 절 추가
      if (searchTerm) {
        query += " WHERE LOWER(title) LIKE $1 OR LOWER(description) LIKE $1"
        params.push(`%${searchTerm}%`)
      }

      query += ";"

      // 쿼리 실행
      const result = searchTerm ? await client.queryArray(query, params) : await client.queryArray(query)

      // Tauri 앱에서 기대하는 형식으로 변환
      const cards = result.rows.map((row) => ({
        id: row[0],
        title: row[1],
        description: row[2],
        url: row[3],
        stars: row[4],
      }))

      // CORS 헤더 추가하여 응답
      return new Response(JSON.stringify(cards), {
        status: 200,
        headers: {
          "Content-Type": "application/json",
          "Access-Control-Allow-Origin": "*",
        },
      })
    } catch (error) {
      console.error("Database error:", error)
      return new Response(JSON.stringify({ error: error.message }), {
        status: 500,
        headers: {
          "Content-Type": "application/json",
          "Access-Control-Allow-Origin": "*",
        },
      })
    } finally {
      try {
        await client.end()
      } catch (e) {
        console.error("Error closing DB connection:", e)
      }
    }
  }
  console.log("요청 URL:", request.url)
  console.log("검색어:", searchTerm)
  // 옵션 요청에 대한 CORS 헤더 처리
  if (request.method === "OPTIONS") {
    return new Response(null, {
      status: 204,
      headers: {
        "Access-Control-Allow-Origin": "*",
        "Access-Control-Allow-Methods": "GET, POST, OPTIONS",
        "Access-Control-Allow-Headers": "Content-Type",
      },
    })
  }

  return new Response("Not Found", { status: 404 })
}

// 8080 포트로 변경 (Tauri 앱에서 호출하는 포트)
console.log("API 서버가 http://localhost:8080 에서 실행 중입니다")
Deno.serve({ port: 8080 }, handler)
