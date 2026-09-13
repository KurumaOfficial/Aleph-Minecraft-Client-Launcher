# Aleph Minecraft Client Launcher (AMC Launcher) — Архитектурная спецификация полного рекода

## 1. Основная информация
- **Проект:** Aleph-Minecraft-Client-Launcher (AMC Launcher)
- **Цель:** Создание масштабируемого, модульного, оптимизированного Open-Source лаунчера Minecraft с нуля на Rust.
- **Рабочая директория:** `C:\Users\gameg\Desktop\WeTTeA_Projects\AMC_Launcher`
- **Git Remote:** `git@github.com:KurumaOfficial/Aleph-Minecraft-Client-Launcher.git`
- **Платформы:** 
  1. Windows (первый приоритет)
  2. Linux (второй приоритет)
  3. macOS (базовая поддержка)

---

## 2. Технологический стек
- **Язык:** Rust (2021 edition)
- **GUI:** `egui` (0.29+) + `eframe`
- **Графический бэкенд:** `wgpu` (DirectX 12 / Vulkan / Metal) с fallback на `glow` (OpenGL)
- **Асинхронность и сеть:** `tokio` + `reqwest` (асинхронный пул, rustls-tls)
- **Сериализация:** `serde`, `serde_json`
- **Векторная графика:** `resvg`, `usvg`, `tiny-skia`
- **Аудио/Шрифты/Ассеты:** `image`, `ttf-parser`

---

## 3. Архитектура Cargo Workspace (по образцу Aleph-Minecraft-Client)
Корневой `Cargo.toml` организует проект в виде мульти-крейта:

```
AMC_Launcher/
├── Cargo.toml                 # Workspace root manifest
├── ARCH_SPEC.md               # Данная спецификация
├── assets/                    # Шрифты, SVG, иконки, темы
│   ├── fonts/
│   ├── svg/
│   └── png/
├── crates/
│   ├── amc-core/              # Общие типы, утилиты, кроссплатформенные пути, конфигурация
│   ├── amc-auth/              # 3 системы аккаунтов (Microsoft, WetID, Offline)
│   ├── amc-downloader/        # Многопоточный асинхронный движок загрузок (побайтовый прогресс, пауза, проверка SHA-1)
│   ├── amc-minecraft/         # Mojang API, версии, манифесты, лоадеры (Fabric, Quilt, Forge, NeoForge, OptiFine), генерация JVM аргументов и запуск
│   ├── amc-mods/              # Интеграция с Modrinth API, CurseForge API и парсинг локальной папки mods/
│   └── amc-ui/                # egui/wgpu рендерер, дизайн-система (по дизайн-мокапу), роутинг вкладок и компонентов
└── src/
    └── main.rs                # Точка входа, инициализация eframe/wgpu и запуск приложения
```

### Профили сборки (Cargo Profiles)
По образцу `Aleph-Minecraft-Client`:
- **`[profile.release]`**:
  - `opt-level = 3`
  - `lto = "thin"`
  - `codegen-units = 1`
  - `panic = "unwind"`
- **`[profile.dev]`**:
  - `opt-level = 2` (для отзывчивого UI даже в дебаге)
  - `debug = true`
- **`[profile.dev.package."*"]`**:
  - `opt-level = 2`

---

## 4. Системы аккаунтов (`amc-auth`)
Лаунчер поддерживает 3 независимых режима:
1. **Microsoft Account (Лицензия Mojang):**
   - OAuth2 Device Code Flow / Authorization Code Flow
   - Xbox Live Token -> XSTS -> Mojang Auth Token -> Minecraft Profile
   - Требуется для игры на официальных лицензионных серверах (Hypixel и т.д.).
2. **WetID (AlephTrust):**
   - Внутренняя система аккаунтов экосистемы Aleph.
   - Дает доступ к серверам с AlephTrust и эксклюзивным аккаунтным фичам лаунчера (облачные профили, синхронизация скинов, статистика).
3. **Offline / Free (Безаккаунтный / Никнейм):**
   - Прямой ввод никнейма (как в TLauncher / Prism в оффлайн-режиме).
   - Генерация Offline-UUID по стандарту Mojang (`MD5("OfflinePlayer:" + name)`).
   - Игра на любых пиратских/свободных серверах.

---

## 5. Менеджер модов (`amc-mods`)
- **Внешние источники:**
  - **Modrinth API v2:** Поиск, фильтрация по лоадерам (Fabric, Quilt, Forge, NeoForge), версиями игры, зависимостям и скачивание.
  - **CurseForge API:** Поиск через CurseTools прокси, парсинг версий и загрузка jar-файлов.
- **Локальный режим:**
  - Отслеживание и управление файлами в папке `mods/` инстанса.
  - Включение/выключение модов (переименование `.jar` <-> `.jar.disabled`).
  - Просмотр метаданных (`fabric.mod.json`, `mods.toml`, `mcmod.info`).

---

## 6. Движок загрузок и версий (`amc-downloader` + `amc-minecraft`)
- Асинхронный пул скачивания на `tokio` (контроль числа параллельных потоков, таймаутов и повторов).
- Честный побайтовый прогресс с вычислением текущей скорости (МБ/с) и оставшегося времени.
- Проверка целостности через SHA-1.
- Автоматическая установка и распаковка Java JRE (Adoptium API) для Java 8, 17, 21.

---

## 7. Дизайн и визуальный стиль (`amc-ui`)
- Источник стилей: HTML/CSS мокап из `message (33).txt` и темная палитра прототипа:
  - Background: `#080606`
  - Elevated/Card: `#0F0B0B`
  - Hover: `#110A0A`
  - Ruby / Accent: `#8B1A2A`
  - Ruby Light: `#B52239`
  - Ruby Dim: `#5C1019`
  - Text Primary: `#E8DADA`
  - Text Heading: `#D4C4BB`
  - Text Muted: `#5A4A44`
- Кастомные компоненты: Sidebar, профили, карточки новостей/серверов/модов, стилизованные поля ввода и кнопки с тонкими рамками и плавными переходами.
