# Локальная разработка: окружение и сборки (RU)

> English version: [DEV-ENV.md](DEV-ENV.md).
> Дата: 2026-09-14. Машина: Windows, пользователь без прав администратора.

## Главное правило

**В разработке — только debug-сборки.** Никаких `cargo build --release` до решения о релизе.
CI (`../.github/workflows/ci.yml`) также переведён на `cargo build --workspace` (debug),
артефакт — `amc-launcher-windows-x64-debug`.

## Smart App Control блокирует свежий тулчейн

На этой машине включён Smart App Control в режиме enforcement
(`HKLM\SYSTEM\...\CI\Policy\VerifiedAndReputablePolicyState = 1`,
события CodeIntegrity 3033/3077/3118 в журнале `Microsoft-Windows-CodeIntegrity/Operational`).

Что это даёт на практике (проверено 2026-09-14):

- `stable` (rustc 1.98) **не запускается вообще**: `rustc.exe -vV` падает с `0xc0e90002`,
  в журнале — блокировка загрузки `rustc_driver-*.dll` («did not meet the Enterprise
  signing level requirements», Policy ID `{0283ac0f-...}`).
- Вердикт SAC — **лотерея по хешу файла**: один и тот же исходник, собранный разными
  версиями rustc, то запускается, то блокируется (`os error 4551`);
  повторный запуск того же файла вердикт не меняет. Пересборка с тем же хешем не помогает.
- Следствие: `cargo` под `stable` мёртв полностью; под старыми тулчейнами работает,
  но каждый свежесобранный exe/build-script — отдельный билет в лотерею.
  Полный workspace локально **не собирается**: в дереве ~40+ build-скриптов
  (`ring`, `serde_json`, `khronos_api`, `icu_*_data`, ...), всем нужно выиграть одновременно.

## Что работает локально

Тулчейн **`1.85.0`** (оставлен в `rustup toolchain list` рядом со `stable`):

```powershell
cargo +1.85.0 check -p amc-core   # ✅ проверка типов
cargo +1.85.0 build -p amc-core   # ✅ debug-артефакт target/debug/libamc_core.rlib
cargo +1.85.0 test -p amc-core    # ✅ 5 тестов, зелёные (проверено 2026-09-14)
```

Почему только `amc-core`: его build-скрипты уже выиграли лотерею и закэшированы в `target/`.
**`target/` не удалять и не чистить без нужды** — перевыигрывать лотерею нечем.
`rust-toolchain.toml` НЕ менять (там `stable` для CI и остальных) — локально всегда `+1.85.0`.

Остальные крейты (`amc-auth`, `amc-downloader`, `amc-minecraft`, `amc-mods`, `amc-ui`)
и весь workspace проверяются/собираются **только в CI** (GitHub Actions, Windows).
Локальный `cargo +1.85.0 check --workspace` упирается в MSRV зависимостей
(`icu_* 2.3.0`, `image 0.25.10` требуют rustc ≥1.88) — даунгрейдить зависимости
под локальную машину **запрещено**, это общая кодовая база.

## Полные debug-сборки и тесты — в CI

Пуш в `main` запускает workflow «CI»: `cargo check --workspace`,
`cargo test --workspace`, `cargo build --workspace` (всё debug).
Готовый `amc-launcher.exe` (debug) — в Artifacts рана (`amc-launcher-windows-x64-debug`).

## Расширенное логирование (уже встроено)

`crates/amc-core/src/logging.rs`: дефолтный фильтр `info,amc_*=debug`,
вывод в stdout **и** в файл `logs/launcher.log` в каталоге данных лаунчера.
Запуск с максимумом деталей:

```powershell
$env:RUST_LOG = "debug"   # или "trace" для совсем подробного
.\amc-launcher.exe
```

`launcher.log` прикладывать к разборам — по нему видно, что и где менять.

## Как починить навсегда (действие пользователя, нужна админка)

Smart App Control выключается пользователем и обратно без переустановки не включается:

1. «Безопасность Windows» → «Управление приложениями и браузером» →
   «Параметры Smart App Control» → **Выкл.** (UAC запросит пароль администратора).
2. После выключения `stable`-тулчейн должен заработать — проверить `rustc --version`
   и `cargo check --workspace`.

Без прав администратора (WSL, allowlist WDAC, смена политики) варианты отсутствуют —
проверено: прав нет (`BUILTIN\Administrators` — deny-only), WSL не установлен.
