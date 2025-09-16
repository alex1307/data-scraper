## Обновяване — 2025-09-09
- Универсалният launcher (`listings_poc.js`) беше обновен → вече поддържа изпълнение на DSL flow-ове за избор на филтри преди започване на харвеста.
- Интегрирахме `flows_dir` от `config/sites.yml` → всеки сайт има свои YAML flow файлове (пример: `mobile.de/advanced-search.yml`).
- Добавихме hook `beforeHarvest` в `mobilede.adapter.js` → приема cookies, навигира правилно от landing page към SRP, заобикаляйки `detailsuche`.
- Raptor вече се връзва към **стартиран Chrome** през DevTools WebSocket (порт 9223) вместо да стартира собствен browser.
- Добавихме автоматично определяне на startUrl след изпълнение на flow-а → ако адаптерът потвърди SRP страница, използваме текущия URL.
- Legacy PoC (`mobilede_listings_poc.js`) остава за референция, но универсалният launcher вече е готов за multi-site.
- Приет е план за реорганизация на проекта → въвеждаме **нов универсален launcher** в `src/boot/launcher.js` като основна входна точка за всички сайтове и адаптери.  
  * PoC (`src/poc/listings_poc.js`) остава само за тестове и референция.  
  * Създаваме регистър за адаптери в `src/raptor/core/adapters.js`.  
  * DSL flow файловете остават по сайтове → напр. `src/raptor/flows/mobile.de/advanced-search.yml`.  
  * Новият launcher ще изпълнява flow-а, ще стартира harvester-а и ще праща резултатите към Kafka.  
  * Архитектурата е подготвена за multi-site и multi-flow поддръжка.

## Архитектура — RAPTOR v2 (структура на папките)

```
src/
├─ boot/
│  └─ launcher.js              ← универсален production launcher (единствен entrypoint)
├─ poc/
│  └─ listings_poc.js          ← PoC/референция (за експерименти и бърз дебъг)
├─ raptor/
│  ├─ core/                    ← универсална бизнес логика (споделена)
│  │  ├─ adapters.js           ← регистър: getAdapter(siteId)
│  │  ├─ harvester.js          ← SRP harvesting (HTML → Kafka RAW)
│  │  ├─ paginator.js          ← навигация „Weiter“, maxPages, dwell, табове
│  │  └─ publisher.js          ← gzip + headers + Kafka publish
│  ├─ adapters/                ← 1 адаптер на сайт
│  │  ├─ mobilede.adapter.js
│  │  ├─ autoscout.adapter.js
│  │  └─ ...
│  └─ flows/                   ← DSL YAML-ите по сайтове
│     ├─ mobile.de/
│     │  ├─ basic-search.yml
│     │  └─ advanced-search.yml
│     ├─ autoscout24.de/
│     │  └─ advanced-search.yml
│     └─ ...
└─ config/
   ├─ sites.yml                ← per-site настройки (flows_dir, стартови URL-и, dwell, paginate)
   ├─ bindings/
   │  └─ mobile-de.yml         ← биндинги за DSL (selectors, константи)
   └─ catalog/
      └─ controls.yml          ← речник с контролите (общи имена → CSS/XPath)
```

### 1) Mermaid диаграма на пайплайна (визуално “къде какво влиза”)
```mermaid
graph LR
  A[User config<br/>(sites.yml, flows)] --> B[boot/launcher.js]
  B --> C[DSL executor<br/>(bindings + flow)]
  C --> D[Harvester SRP<br/>(raptor/core/harvester.js)]
  D --> E[Publisher<br/>(gzip + headers)]
  E --> F[(Kafka topic: mobile_de)]
  F --> G[Rust Consumer<br/>(KafkaConsumer.rs)]
  G --> H[SRP JSON extract]
  H --> I[(Postgres)]
```

### 2) Мини “Runbook” (1-минутно връщане в контекста)
```md
## Runbook (quick start)
1) Стартиран Chrome с DevTools WS на 9223.
2) Проверка на sites.yml → flows_dir сочи към реалните YAML-и.
3) Пускане:
   RAPTOR_HTML_TOPIC=mobile_de node src/boot/launcher.js --site mobile.de --flow advanced-search
4) Очаквани лога:
   - [launcher] executing flow file: .../advanced-search.yml
   - [exec] OK → … (стъпките от YAML-а)
   - [harvester] SRP page 1
   - [kafka] SRP published …
5) Rust консьомер:
   ./target/release/mobile_de
   - RAW HTML received … bytes=…
   - Processing N items from SRP JSON
   - Vehicle written to database: rows_affected: 1
```

### 3) Снимка на “известно работеща” конфигурация
```yaml
# config/sites.yml (минимален snapshot)
sites:
  mobile.de:
    flows_dir: "config/flows"        # там са advanced-search.yml / basic-search.yml
    entry:
      landing_url: "https://www.mobile.de/"
    kafka:
      raw_html_topic: "mobile_de"
    human:
      dwell_ms: { min: 8000, max: 20000 }
    pagination:
      enabled: true
      maxPages: 5
    lazy_scroll:
      enabled: true
      steps: 3
      pause_ms: { min: 500, max: 1200 }
```

### 4) “Сигнатури” за бързо диагностициране от логове
```md
## Диагностика по лога
- flow executor not available → flow файл не е намерен (flows_dir грешен или липсващ *.yml).
- [launcher][debug] flowFile=… exists=false → коригирай `flows_dir` в sites.yml.
- [harvester] [guard] Not on results url → adapter.isResultsUrl връща false (не сме на SRP).
- [kafka] skip publish (non-SRP url) → пак не сме на SRP; провери flow или beforeHarvest.
- Rust: SRP JSON candidate length: … → екстракторът е намерил JSON в HTML.
- Vehicle written to database: rows_affected: 1 → DB insert/UPSERT е ок.
```

### Следващи стъпки
- Финално тестване на SRP paging логиката (бутон „Weiter“, симулация на нови табове, оптимално време между заявки).
- Поддръжка на `basic-search.yml` и multi-flow изпълнение.
- Интеграция на SRP JSON extractor-а в Rust consumer-а → унифицираме парсването за всички пазари.
- Подготовка за multi-market поддръжка в единна конфигурация.
- Подготвителна работа за AI агент, който ще прави query → JSON трансформация.

## Обновяване — 2025-09-06
- Финализирахме Rust consumer-а → извличаме SRP JSON от HTML точно както legacy parseHtml.
- Raptor вече публикува RAW HTML правилно → Kafka → Rust → Postgres.
- Потвърдено е, че pipeline-ът работи end-to-end.
- Следващи стъпки:
  * Подобряване на навигацията през SRP страниците (следваща страница → симулирани табове).
  * Подготовка за multi-market поддръжка.
  * Обмисляме AI слой за query-to-JSON.

# My Friend Resume

## Проектен контекст
Текущата архитектура на проекта е базирана на Rust и се фокусира върху създаването на ефективен и надежден scraper за данни. Използваме модулен подход, който позволява лесно разширяване и поддръжка. Основната цел е да събираме и обработваме данни от различни източници с висока производителност и минимални грешки.

## Какво постигнахме днес
- Добавихме структурирано резюме, което служи като журнал за проекта.
- Определихме ясни секции за проследяване на контекста, постиженията, следващите стъпки и важните бележки.
- Създадохме основа за по-добра организация и проследимост на работата по проекта.

## Следващи стъпки
- Продължаваме с имплементацията на нови функционалности в scraper-а.
- Оптимизиране на обработката на данните за по-бърза и надеждна работа.
- Добавяне на автоматизирани тестове за гарантиране на качеството.
- Разширяване на документацията и журнала с нови бележки и наблюдения.

## Важни бележки
- Модулният подход улеснява поддръжката и разширяването на проекта.
- Високата производителност и надеждност са ключови за успеха на scraper-а.
- Ясната документация и журналът помагат за по-добро разбиране и проследяване на напредъка.
- Постоянното тестване и оптимизиране са необходими за поддържане на качеството.
