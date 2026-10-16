# 부엌 — 식재료·레시피 관리

혼자 쓰는 요리 도우미 웹앱.

- **재고 관리**: 수량(두부 1모, 멸치 100g)으로도, 대략(된장 넉넉/보통/조금/없음)으로도 관리. 유통기한, 보관 위치, 사진 지원
- **레시피**: 사진, 단계별 조리법, 재료(정량 또는 "약간"), 선택 재료, 태그
- **공통 재료 연결**: 된장·두부·액젓처럼 여러 레시피에 쓰이는 재료는 식재료 하나로 관리
- **묶음**: 고춧가루 ← 고운/청양 고춧가루처럼 종류를 묶어서, 레시피에 "고춧가루"라고 쓰면 아무거나 있으면 됨
- **대체 재료**: 레시피 재료마다 바꿔 쓸 수 있는 재료 지정 (차슈의 삼겹살 → 목살)
- **뭐 해먹지?**: 재료를 여러 개 고르면 만들 수 있는 요리를 부족한 재료가 적은 순으로 추천. 아무것도 안 고르면 현재 재고 기준
- **요리했어요**: 인분에 맞춰 재고 차감안을 만들어 주고(단위 변환, 유통기한 빠른 것부터), 확인 후 반영. 다 떨어진 재료는 장보기로
- **장보기**: 산 것은 바로 재고로 등록
- **알림**: 매일 정해진 시각에 유통기한 임박·지남과 재고 부족을 [ntfy](https://ntfy.sh)로 푸시
- 예정: OpenRouter AI로 사진 속 재료 인식, 메뉴 추천

구조는 [docs/ARCHITECTURE.md](docs/ARCHITECTURE.md) 참고.

## 기술 스택

| | |
|---|---|
| 백엔드 | Rust · axum · sqlx · SQLite (WAL) · tokio. 단일 바이너리 |
| 프론트엔드 | React 19 · Vite · TypeScript · TanStack Query · Tailwind CSS 4. 모바일 우선 PWA |
| 알림 | ntfy.sh |
| 배포 | 프론트 빌드 결과물을 바이너리에 내장 (`rust-embed`) → 파일 하나 + `data/` |

## 빠른 시작

필요한 것: Rust (stable), Node.js 22+, `build-essential`

```bash
cp .env.example .env        # 필요하면 수정
./scripts/build.sh          # web 빌드 + cargo build --release
./target/release/kitchen seed   # 기본 레시피 6개 (아욱된장국, 황태미역국 등)
./target/release/kitchen        # http://<서버IP>:8080
```

휴대폰에서 열고 브라우저 메뉴의 **홈 화면에 추가**를 누르면 앱처럼 쓸 수 있습니다.

### 개발

```bash
cargo run                   # API :8080 (변경 시 재실행)
cd web && npm run dev       # :5173, /api 는 :8080 으로 프록시
```

```bash
cargo test && cargo clippy --all-targets
cd web && npm run typecheck
```

## 명령

| 명령 | 설명 |
|---|---|
| `kitchen` / `kitchen serve` | 서버 실행 (HTTP + 스케줄러 + 워커). 시작 시 마이그레이션 자동 적용 |
| `kitchen seed [파일.json]` | 레시피 넣기. 파일을 생략하면 `seed/recipes.json`. 같은 제목은 건너뜀 |

## 설정 (`.env`)

| 변수 | 기본값 | 설명 |
|---|---|---|
| `KITCHEN_BIND` | `0.0.0.0:8080` | 서버 주소 |
| `KITCHEN_DATA_DIR` | `./data` | `kitchen.db`, `photos/` 위치 |
| `KITCHEN_PASSWORD` | (없음) | 설정하면 로그인 필요 (세션 30일) |
| `KITCHEN_PUBLIC_URL` | (없음) | 알림을 누르면 열 주소 |
| `KITCHEN_TZ_OFFSET_HOURS` | `9` | 시간대 (KST) |
| `NTFY_URL` | `https://ntfy.sh` | 자체 호스팅 ntfy 를 쓰면 변경 |
| `NTFY_TOPIC` | (없음) | 비우면 알림 꺼짐. **추측하기 어려운 이름**으로 |
| `NTFY_TOKEN` | (없음) | 보호된 토픽용 액세스 토큰 |
| `NOTIFY_HOUR` | `9` | 매일 알림 시각 (0-23) |
| `EXPIRY_WARN_DAYS` | `3` | 유통기한 며칠 전부터 알릴지 |

ntfy 앱에서 같은 토픽을 구독한 뒤, 웹의 **더보기 → 테스트 알림 보내기**로 확인하세요.

## 상시 실행 (systemd)

```bash
sudo cp deploy/kitchen.service /etc/systemd/system/
sudo systemctl daemon-reload
sudo systemctl enable --now kitchen
journalctl -u kitchen -f
```

업데이트: `git pull && ./scripts/build.sh && sudo systemctl restart kitchen`

## 백업

```bash
sqlite3 data/kitchen.db ".backup data/backup-$(date +%F).db"
tar czf kitchen-photos.tgz data/photos
```

## API 요약

모든 경로는 `/api` 아래에 있고 JSON을 주고받습니다. 오류는 `{"error": "..."}` 형식으로 옵니다.

| 경로 | 설명 |
|---|---|
| `GET/POST /ingredients`, `GET/PUT/DELETE /ingredients/{id}` | 식재료 (재고 상태 포함) |
| `GET/POST /pantry`, `GET/PUT/DELETE /pantry/{id}` | 재고 |
| `POST /pantry/adjust` | 수량/단계만 변경 `[{pantry_item_id, quantity \| level}]` |
| `GET /pantry/expiring?days=3` | 유통기한 임박 |
| `GET/POST /recipes?q=&tag=`, `GET/PUT/DELETE /recipes/{id}` | 레시피 (재료·대체 재료는 `ingredient_id` 또는 `name`. 이름만 주면 식재료 자동 생성) |
| `PUT /recipes/{id}/photo` | 대표 사진만 변경 `{photo_id}` |
| `POST /recipes/recommend` | `{ingredient_ids: []}` → 추천 |
| `GET /recipes/{id}/cook-plan?servings=` | 재고 차감안 |
| `POST /recipes/{id}/cook` | `{adjustments, note, add_missing_to_shopping}` |
| `GET/POST /shopping`, `PUT/DELETE /shopping/{id}`, `POST /shopping/clear-done` | 장보기 |
| `POST /photos` (multipart `file`), `GET /photos/{id}` | 사진 (긴 변 1600px JPEG 로 저장) |
| `POST /auth/login`, `POST /auth/logout`, `GET /auth/me` | 인증 |
| `POST /notifications/test`, `GET /info`, `GET /health` | 기타 |
