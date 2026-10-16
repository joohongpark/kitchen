# 아키텍처

혼자 쓰는 요리·식재료 관리 웹앱. **Rust 바이너리 1개 + SQLite 파일 1개 + 사진 디렉터리**로 동작한다.
백엔드 구조는 [Building scalable backend services with Rust and PostgreSQL](https://kerkour.com/rust-scalable-backend-services)
(한국어 요약: [GeekNews](https://news.hada.io/topic?id=32571))을 따르되, 단일 인스턴스 + SQLite 환경에 맞게 단순화했다.

## 전체 구성

```
 브라우저 (React SPA, 모바일 우선)
     │  same-origin  /api/*  ·  /*
     ▼
 ┌──────────────── kitchen (단일 바이너리) ────────────────┐
 │  HTTP (axum)      Scheduler (tokio)     Worker (tokio)  │  ← 1계층: 입구
 │       │                │                     │          │
 │       ▼                ▼                     ▼          │
 │  Services: pantry · recipes · media · queue             │  ← 2계층: 비즈니스 로직
 │       │                                                 │
 │       ▼                                                 │
 │  Repositories (sqlx, SQL만)                             │  ← 3계층: 데이터 접근
 └───────┬───────────────────────────┬─────────────────────┘
         ▼                           ▼
   data/kitchen.db (SQLite, WAL)   data/photos/*.jpg        ──▶ ntfy.sh (푸시)
                                                            ──▶ OpenRouter (AI, 예정)
```

## 계층 규칙

| 계층 | 하는 일 | 하지 않는 일 |
|---|---|---|
| **HTTP / Scheduler / Worker** (`crates/server`) | 요청·트리거를 서비스 입력으로 변환, 결과를 응답으로 변환, 인증 | SQL 실행, 비즈니스 규칙 |
| **Service** (`crates/*/src/service.rs`) | 검증, 불변조건, 트랜잭션 경계, 여러 저장소 조합 | HTTP 타입(axum) 의존 |
| **Repository** (`crates/*/src/repository.rs`) | SQL 캡슐화, DB 제약 오류 → 도메인 오류 변환 | 비즈니스 판단, 캐시 |

- 각 계층은 **인접 계층만** 호출한다. 핸들러가 저장소를 직접 부르지 않는다.
- 외부 라이브러리(axum, reqwest 등)는 경계 계층에 가둔다. 도메인 크레이트는 axum을 모른다.
- 저장소 메서드는 `Queryer` trait을 받아 **커넥션 풀과 트랜잭션 양쪽에서** 같은 코드로 동작한다.

## 크레이트 구성 (Cargo workspace)

```
crates/
  server/    실행 바이너리: config, http/(라우터·핸들러·인증), scheduler, worker, webapp(SPA 내장), seed
  common/    도메인 Error, Queryer, DB 연결·마이그레이션, ID(UUIDv7), 단위 변환
  pantry/    식재료 마스터(ingredients), 재고(pantry_items), 장보기(shopping_items)
  recipes/   레시피, 재료 구성, 조리 단계, 추천, 요리 기록·재고 차감
  media/     사진 업로드·리사이즈·저장
  queue/     SQLite 기반 작업 큐
  ntfy/      ntfy.sh 클라이언트
migrations/  SQL 마이그레이션 (바이너리에 내장, 시작 시 자동 적용)
web/         React (Vite + TS + TanStack Query + Tailwind)
seed/        기본 레시피 데이터
```

의존 방향: `server → {pantry, recipes, media, queue, ntfy} → common`, `recipes → pantry`.

## 재고 수량 모델 — "그때그때 다르게"

재고 항목(`pantry_items`)마다 추적 방식을 고른다.

| tracking | 저장 값 | 예 |
|---|---|---|
| `quantity` | `quantity` + `unit` | 두부 1모, 애호박 2개, 국간장 450ml, 다진마늘 200g |
| `level` | `level` = `plenty`/`medium`/`low`/`out` (넉넉/보통/조금/없음) | 된장, 고춧가루, 소금 |

- 같은 식재료라도 구매분(lot)마다 다른 항목이 될 수 있다 (유통기한이 다르면 따로 관리).
- 레시피 재료량은 `quantity`+`unit`(계산 가능) 또는 `amount_text`("약간", "1/2모")로 적는다.
- **단위 변환**(`common::units`): 무게(g, kg), 부피(ml, l, 큰술=15ml, 작은술=5ml, 컵=200ml)는 같은 계열끼리 변환한다. 개수 단위(개, 모, 단, 봉, 포기, 마리…)는 같은 단위끼리만 비교한다.
- **요리 완료 시 차감**: 서버가 차감안(cook-plan)을 만들고 → 사용자가 확인·수정하고 → 한 트랜잭션으로 적용한다.
  - quantity 항목: 변환 가능하면 유통기한 빠른 순으로 차감한다.
  - level 항목: 현재 단계를 그대로 제안하고 사용자가 낮출 수 있다.

## 데이터 모델

```
ingredients      식재료 마스터 (이름 UNIQUE, 카테고리, 기본 단위·보관, 부족 기준, 사진)
pantry_items     재고 lot (tracking, quantity/unit | level, 보관 위치, 구매일, 유통기한, 사진)
recipes          레시피 (제목 UNIQUE, 설명, 인분, 조리시간, 태그 JSON, 대표 사진)
recipe_ingredients  레시피 ↔ 재료 (양, 단위 | 텍스트, 선택재료 여부, 순서)
recipe_steps     조리 단계 (순서, 내용, 사진)
photos           사진 메타데이터 (파일은 data/photos/)
cook_logs        요리 기록
shopping_items   장보기 목록
jobs             작업 큐
kv               내부 상태 (마지막 알림 날짜 등)
sessions         로그인 세션
```

## 추천

입력은 선택한 재료 목록(비어 있으면 현재 재고 전체)이다.
1. 레시피의 **필수 재료**(선택재료 제외) 중 재고에 있는 것과 없는 것을 나눈다.
2. 재료를 선택했다면 그 재료를 하나 이상 쓰는 레시피만 남긴다.
3. 부족한 재료 수 ↑, 선택 재료 적중 수 ↓, 충족률 ↓ 순으로 정렬한다.
4. 결과에 부족한 재료를 함께 보여 주고, 장보기 목록에 바로 추가할 수 있다.

SQLite에 레시피 수백 개 수준이면 메모리에서 계산해도 충분하다.

## 백그라운드 작업

- **Scheduler**: 1분마다 깨어나서, 매일 `NOTIFY_HOUR`에 `expiry_digest` 작업을 큐에 넣는다. 같은 날 중복 실행은 `kv`로 막는다.
  단일 인스턴스라 원문의 advisory lock 리더 선출은 필요 없다.
- **Worker**: `jobs` 테이블을 폴링해 `UPDATE … RETURNING`으로 작업을 가져온다. 실패하면 지수 백오프로 재시도한다.
- **알림**: 유통기한 임박·지남, 재고 부족 요약을 ntfy JSON API로 보낸다. `NTFY_TOPIC`을 설정하지 않으면 비활성화된다.

## SQLite 운영

- WAL 모드, `busy_timeout=5s`, `foreign_keys=ON`, 쓰기 트랜잭션은 짧게 유지
- 백업: `sqlite3 data/kitchen.db ".backup backup.db"` (크론) 또는 Litestream
- 나중에 Postgres로 옮길 때 고칠 곳은 저장소 계층(SQL)과 `common::db`로 한정된다.

## 인증

`KITCHEN_PASSWORD`를 설정하면 로그인 후 세션 쿠키(HttpOnly, 30일)로 인증한다. 설정하지 않으면 인증 없이 동작한다 (LAN/VPN 전용으로 쓸 때).

## AI (예정, OpenRouter)

`crates/ai`에 OpenRouter 클라이언트를 두고 서비스 trait 뒤에 숨겨 모델을 쉽게 바꿀 수 있게 한다.
- **사진 → 재료 인식**: 업로드한 사진을 작업 큐에서 비전 모델로 분석해 JSON 스키마(이름, 수량, 단위, 추정 유통기한)로 받는다. **바로 반영하지 않고** 사용자 확인 후 저장한다.
- **메뉴 추천**: 재고와 임박 재료를 컨텍스트로 메뉴를 추천하고, 새 레시피 초안을 만든다.
