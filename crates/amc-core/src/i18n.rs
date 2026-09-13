use serde::{Deserialize, Serialize};

/// Supported launcher UI languages
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, Default)]
#[serde(rename_all = "lowercase")]
pub enum Language {
    #[serde(rename = "ru")]
    #[default]
    Russian,
    #[serde(rename = "en")]
    English,
    #[serde(rename = "uk", alias = "ua")]
    Ukrainian,
}

impl Language {
    pub const ALL: [Language; 3] = [Language::English, Language::Russian, Language::Ukrainian];

    pub fn code(&self) -> &'static str {
        match self {
            Self::English => "en",
            Self::Russian => "ru",
            Self::Ukrainian => "uk",
        }
    }

    pub fn display_name(&self) -> &'static str {
        match self {
            Self::English => "English",
            Self::Russian => "Русский",
            Self::Ukrainian => "Українська",
        }
    }

    pub fn display_with_flag(&self) -> &'static str {
        match self {
            Self::English => "🇬🇧 English",
            Self::Russian => "🇷🇺 Русский",
            Self::Ukrainian => "🇺🇦 Українська",
        }
    }

    pub fn parse_code(s: &str) -> Self {
        match s.trim().to_lowercase().as_str() {
            "en" | "eng" | "english" => Self::English,
            "uk" | "ua" | "ukr" | "ukrainian" => Self::Ukrainian,
            "ru" | "rus" | "russian" => Self::Russian,
            _ => Self::Russian,
        }
    }

    // ==========================================
    // Sidebar Navigation
    // ==========================================
    pub fn nav_home(&self) -> &'static str {
        match self {
            Self::English => "Home",
            Self::Russian => "Главная",
            Self::Ukrainian => "Головна",
        }
    }

    pub fn nav_instances(&self) -> &'static str {
        match self {
            Self::English => "Instances",
            Self::Russian => "Сборки",
            Self::Ukrainian => "Збірки",
        }
    }

    pub fn nav_mods(&self) -> &'static str {
        match self {
            Self::English => "Mods",
            Self::Russian => "Моды",
            Self::Ukrainian => "Моди",
        }
    }

    pub fn nav_skins(&self) -> &'static str {
        match self {
            Self::English => "Skins",
            Self::Russian => "Скины",
            Self::Ukrainian => "Скіни",
        }
    }

    pub fn nav_settings(&self) -> &'static str {
        match self {
            Self::English => "Settings",
            Self::Russian => "Настройки",
            Self::Ukrainian => "Налаштування",
        }
    }

    pub fn nav_exit(&self) -> &'static str {
        match self {
            Self::English => "Exit",
            Self::Russian => "Выход",
            Self::Ukrainian => "Вихід",
        }
    }

    // ==========================================
    // Titlebar
    // ==========================================
    pub fn titlebar_console_tooltip(&self) -> &'static str {
        match self {
            Self::English => "Game Console / Process Logs",
            Self::Russian => "Консоль / Логи игры",
            Self::Ukrainian => "Консоль / Логи гри",
        }
    }

    // ==========================================
    // Bottom Bar
    // ==========================================
    pub fn bottom_not_authorized(&self) -> &'static str {
        match self {
            Self::English => "Not authorized",
            Self::Russian => "Не авторизован",
            Self::Ukrainian => "Не авторизовано",
        }
    }

    pub fn bottom_login_prompt(&self) -> &'static str {
        match self {
            Self::English => "Sign in to play",
            Self::Russian => "Войдите для игры",
            Self::Ukrainian => "Увійдіть для гри",
        }
    }

    pub fn bottom_btn_switch(&self) -> &'static str {
        match self {
            Self::English => "SWITCH",
            Self::Russian => "СМЕНИТЬ",
            Self::Ukrainian => "ЗМІНИТИ",
        }
    }

    pub fn bottom_btn_login(&self) -> &'static str {
        match self {
            Self::English => "LOG IN",
            Self::Russian => "ВОЙТИ",
            Self::Ukrainian => "УВІЙТИ",
        }
    }

    pub fn bottom_btn_play(&self) -> &'static str {
        match self {
            Self::English => "▶  PLAY",
            Self::Russian => "▶  ИГРАТЬ",
            Self::Ukrainian => "▶  ГРАТИ",
        }
    }

    pub fn bottom_btn_launching(&self) -> &'static str {
        match self {
            Self::English => "LAUNCHING...",
            Self::Russian => "ЗАПУСК...",
            Self::Ukrainian => "ЗАПУСК...",
        }
    }

    // ==========================================
    // Home Page
    // ==========================================
    pub fn home_title(&self) -> &'static str {
        match self {
            Self::English => "Minecraft Versions",
            Self::Russian => "Версии Minecraft",
            Self::Ukrainian => "Версії Minecraft",
        }
    }

    pub fn home_selected_badge(&self, version: &str) -> String {
        match self {
            Self::English => format!("Selected: {version}"),
            Self::Russian => format!("Выбрана: {version}"),
            Self::Ukrainian => format!("Обрана: {version}"),
        }
    }

    pub fn home_pinging(&self) -> &'static str {
        match self {
            Self::English => "⏳ Pinging server...",
            Self::Russian => "⏳ Пинг сервера...",
            Self::Ukrainian => "⏳ Пінг сервера...",
        }
    }

    pub fn home_search_hint(&self) -> &'static str {
        match self {
            Self::English => "🔍 Search versions...",
            Self::Russian => "🔍 Поиск версий...",
            Self::Ukrainian => "🔍 Пошук версій...",
        }
    }

    pub fn home_sort_newest(&self) -> &'static str {
        match self {
            Self::English => "Newest first",
            Self::Russian => "Сначала новые",
            Self::Ukrainian => "Спочатку нові",
        }
    }

    pub fn home_sort_oldest(&self) -> &'static str {
        match self {
            Self::English => "Oldest first",
            Self::Russian => "Сначала старые",
            Self::Ukrainian => "Спочатку старі",
        }
    }

    pub fn home_tab_all(&self) -> &'static str {
        match self {
            Self::English => "ALL",
            Self::Russian => "ВСЕ",
            Self::Ukrainian => "ВСІ",
        }
    }

    pub fn home_tab_releases(&self) -> &'static str {
        match self {
            Self::English => "RELEASES",
            Self::Russian => "РЕЛИЗЫ",
            Self::Ukrainian => "РЕЛІЗИ",
        }
    }

    pub fn home_tab_snapshots(&self) -> &'static str {
        match self {
            Self::English => "SNAPSHOTS",
            Self::Russian => "СНАПШОТЫ",
            Self::Ukrainian => "СНАПШОТИ",
        }
    }

    pub fn home_tab_betas(&self) -> &'static str {
        match self {
            Self::English => "BETAS",
            Self::Russian => "БЕТА",
            Self::Ukrainian => "БЕТА",
        }
    }

    pub fn home_tab_alphas(&self) -> &'static str {
        match self {
            Self::English => "ALPHAS",
            Self::Russian => "АЛЬФА",
            Self::Ukrainian => "АЛЬФА",
        }
    }

    pub fn home_tab_old(&self) -> &'static str {
        match self {
            Self::English => "OLD",
            Self::Russian => "СТАРЫЕ",
            Self::Ukrainian => "СТАРІ",
        }
    }

    pub fn home_btn_launch(&self) -> &'static str {
        match self {
            Self::English => "Launch",
            Self::Russian => "Запустить",
            Self::Ukrainian => "Запустити",
        }
    }

    pub fn home_btn_install_play(&self) -> &'static str {
        match self {
            Self::English => "Install & Play",
            Self::Russian => "Установить и играть",
            Self::Ukrainian => "Встановити та грати",
        }
    }

    pub fn home_btn_files(&self) -> &'static str {
        match self {
            Self::English => "📂 Files",
            Self::Russian => "📂 Файлы",
            Self::Ukrainian => "📂 Файли",
        }
    }

    pub fn home_empty(&self) -> &'static str {
        match self {
            Self::English => "No versions matching your search",
            Self::Russian => "Версии не найдены",
            Self::Ukrainian => "Версії не знайдено",
        }
    }

    // ==========================================
    // Instances Page
    // ==========================================
    pub fn inst_title(&self) -> &'static str {
        match self {
            Self::English => "Minecraft Instances",
            Self::Russian => "Сборки Minecraft",
            Self::Ukrainian => "Збірки Minecraft",
        }
    }

    pub fn inst_search_hint(&self) -> &'static str {
        match self {
            Self::English => "🔍 Search instances...",
            Self::Russian => "🔍 Поиск сборок...",
            Self::Ukrainian => "🔍 Пошук збірок...",
        }
    }

    pub fn inst_btn_create(&self) -> &'static str {
        match self {
            Self::English => "+ CREATE INSTANCE",
            Self::Russian => "+ СОЗДАТЬ СБОРКУ",
            Self::Ukrainian => "+ СТВОРИТИ ЗБІРКУ",
        }
    }

    pub fn inst_empty(&self) -> &'static str {
        match self {
            Self::English => "No instances created yet. Click '+ CREATE INSTANCE' to add one!",
            Self::Russian => "Сборки еще не созданы. Нажмите '+ СОЗДАТЬ СБОРКУ', чтобы добавить!",
            Self::Ukrainian => "Збірок ще не створено. Натисніть '+ СТВОРИТИ ЗБІРКУ', щоб додати!",
        }
    }

    pub fn inst_version_label(&self, ver: &str) -> String {
        match self {
            Self::English => format!("Version: {ver}"),
            Self::Russian => format!("Версия: {ver}"),
            Self::Ukrainian => format!("Версія: {ver}"),
        }
    }

    pub fn inst_loader_label(&self, loader: &str) -> String {
        match self {
            Self::English => format!("Loader: {loader}"),
            Self::Russian => format!("Загрузчик: {loader}"),
            Self::Ukrainian => format!("Завантажувач: {loader}"),
        }
    }

    pub fn inst_ram_label(&self, mb: u32) -> String {
        match self {
            Self::English => format!("Allocated: {mb} MB"),
            Self::Russian => format!("Выделено: {mb} МБ"),
            Self::Ukrainian => format!("Виділено: {mb} МБ"),
        }
    }

    pub fn inst_playtime_label(&self, time_str: &str) -> String {
        match self {
            Self::English => format!("Played: {time_str}"),
            Self::Russian => format!("В игре: {time_str}"),
            Self::Ukrainian => format!("У грі: {time_str}"),
        }
    }

    pub fn inst_mins_suffix(&self) -> &'static str {
        match self {
            Self::English => "min",
            Self::Russian => "мин",
            Self::Ukrainian => "хв",
        }
    }

    pub fn inst_hours_suffix(&self) -> &'static str {
        match self {
            Self::English => "h",
            Self::Russian => "ч",
            Self::Ukrainian => "год",
        }
    }

    pub fn inst_last_played_label(&self, dt_str: &str) -> String {
        match self {
            Self::English => format!("Last session: {dt_str}"),
            Self::Russian => format!("Был в игре: {dt_str}"),
            Self::Ukrainian => format!("Востаннє у грі: {dt_str}"),
        }
    }

    pub fn inst_never_played(&self) -> &'static str {
        match self {
            Self::English => "Never",
            Self::Russian => "Никогда",
            Self::Ukrainian => "Ніколи",
        }
    }

    pub fn inst_btn_play(&self) -> &'static str {
        match self {
            Self::English => "▶ Play",
            Self::Russian => "▶ Играть",
            Self::Ukrainian => "▶ Грати",
        }
    }

    pub fn inst_btn_folder(&self) -> &'static str {
        match self {
            Self::English => "📂 Folder",
            Self::Russian => "📂 Папка",
            Self::Ukrainian => "📂 Папка",
        }
    }

    pub fn inst_btn_edit(&self) -> &'static str {
        match self {
            Self::English => "⚙ Edit",
            Self::Russian => "⚙ Настроить",
            Self::Ukrainian => "⚙ Налаштувати",
        }
    }

    pub fn inst_btn_delete(&self) -> &'static str {
        match self {
            Self::English => "🗑 Delete",
            Self::Russian => "🗑 Удалить",
            Self::Ukrainian => "🗑 Видалити",
        }
    }

    pub fn inst_modal_create_title(&self) -> &'static str {
        match self {
            Self::English => "Create New Instance",
            Self::Russian => "Создать новую сборку",
            Self::Ukrainian => "Створити нову збірку",
        }
    }

    pub fn inst_modal_edit_title(&self) -> &'static str {
        match self {
            Self::English => "Edit Instance",
            Self::Russian => "Редактирование сборки",
            Self::Ukrainian => "Редагування збірки",
        }
    }

    pub fn inst_modal_name(&self) -> &'static str {
        match self {
            Self::English => "Instance Name:",
            Self::Russian => "Название сборки:",
            Self::Ukrainian => "Назва збірки:",
        }
    }

    pub fn inst_modal_version(&self) -> &'static str {
        match self {
            Self::English => "Game Version:",
            Self::Russian => "Версия игры:",
            Self::Ukrainian => "Версія гри:",
        }
    }

    pub fn inst_modal_loader(&self) -> &'static str {
        match self {
            Self::English => "Mod Loader:",
            Self::Russian => "Загрузчик модов:",
            Self::Ukrainian => "Завантажувач модів:",
        }
    }

    pub fn inst_modal_ram(&self) -> &'static str {
        match self {
            Self::English => "RAM Allocation:",
            Self::Russian => "Оперативная память:",
            Self::Ukrainian => "Оперативна пам'ять:",
        }
    }

    pub fn inst_modal_btn_save(&self) -> &'static str {
        match self {
            Self::English => "SAVE",
            Self::Russian => "СОХРАНИТЬ",
            Self::Ukrainian => "ЗБЕРЕГТИ",
        }
    }

    pub fn inst_modal_btn_create(&self) -> &'static str {
        match self {
            Self::English => "CREATE",
            Self::Russian => "СОЗДАТЬ",
            Self::Ukrainian => "СТВОРИТИ",
        }
    }

    pub fn inst_modal_btn_cancel(&self) -> &'static str {
        match self {
            Self::English => "CANCEL",
            Self::Russian => "ОТМЕНА",
            Self::Ukrainian => "СКАСУВАТИ",
        }
    }

    // ==========================================
    // Mods Page
    // ==========================================
    pub fn mods_title(&self) -> &'static str {
        match self {
            Self::English => "Mod Manager",
            Self::Russian => "Управление модами",
            Self::Ukrainian => "Керування модами",
        }
    }

    pub fn mods_tab_installed(&self) -> &'static str {
        match self {
            Self::English => "INSTALLED",
            Self::Russian => "УСТАНОВЛЕННЫЕ",
            Self::Ukrainian => "ВСТАНОВЛЕНІ",
        }
    }

    pub fn mods_tab_search_modrinth(&self) -> &'static str {
        match self {
            Self::English => "SEARCH (MODRINTH)",
            Self::Russian => "ПОИСК (MODRINTH)",
            Self::Ukrainian => "ПОШУК (MODRINTH)",
        }
    }

    pub fn mods_tab_search_curseforge(&self) -> &'static str {
        match self {
            Self::English => "SEARCH (CURSEFORGE)",
            Self::Russian => "ПОИСК (CURSEFORGE)",
            Self::Ukrainian => "ПОШУК (CURSEFORGE)",
        }
    }

    pub fn mods_tab_search_generic(&self) -> &'static str {
        match self {
            Self::English => "SEARCH MODS",
            Self::Russian => "ПОИСК МОДОВ",
            Self::Ukrainian => "ПОШУК МОДІВ",
        }
    }

    pub fn mods_provider_btn(&self, prov: &str) -> String {
        match self {
            Self::English => format!("Provider: {prov}"),
            Self::Russian => format!("Провайдер: {prov}"),
            Self::Ukrainian => format!("Провайдер: {prov}"),
        }
    }

    pub fn mods_search_hint(&self, prov: &str) -> String {
        match self {
            Self::English => format!("🔍 Search mods on {prov}..."),
            Self::Russian => format!("🔍 Поиск модов на {prov}..."),
            Self::Ukrainian => format!("🔍 Пошук модів на {prov}..."),
        }
    }

    pub fn mods_btn_search(&self) -> &'static str {
        match self {
            Self::English => "SEARCH",
            Self::Russian => "ИСКАТЬ",
            Self::Ukrainian => "ШУКАТИ",
        }
    }

    pub fn mods_searching(&self) -> &'static str {
        match self {
            Self::English => "Searching mods...",
            Self::Russian => "Поиск модов...",
            Self::Ukrainian => "Пошук модів...",
        }
    }

    pub fn mods_btn_open_folder(&self) -> &'static str {
        match self {
            Self::English => "📂 Open mods folder",
            Self::Russian => "📂 Открыть папку mods",
            Self::Ukrainian => "📂 Відкрити папку mods",
        }
    }

    pub fn mods_btn_rescan(&self) -> &'static str {
        match self {
            Self::English => "🔄 Refresh list",
            Self::Russian => "🔄 Обновить список",
            Self::Ukrainian => "🔄 Оновити список",
        }
    }

    pub fn mods_count(&self, count: usize) -> String {
        match self {
            Self::English => format!("Installed mods: {count}"),
            Self::Russian => format!("Установлено модов: {count}"),
            Self::Ukrainian => format!("Встановлено модів: {count}"),
        }
    }

    pub fn mods_status_enabled(&self) -> &'static str {
        match self {
            Self::English => "Enabled",
            Self::Russian => "Включен",
            Self::Ukrainian => "Увімкнено",
        }
    }

    pub fn mods_status_disabled(&self) -> &'static str {
        match self {
            Self::English => "Disabled",
            Self::Russian => "Отключен",
            Self::Ukrainian => "Вимкнено",
        }
    }

    pub fn mods_btn_install(&self) -> &'static str {
        match self {
            Self::English => "INSTALL",
            Self::Russian => "УСТАНОВИТЬ",
            Self::Ukrainian => "ВСТАНОВИТИ",
        }
    }

    pub fn mods_btn_downloading(&self) -> &'static str {
        match self {
            Self::English => "DOWNLOADING...",
            Self::Russian => "ЗАГРУЗКА...",
            Self::Ukrainian => "ЗАВАНТАЖЕННЯ...",
        }
    }

    pub fn mods_badge_installed(&self) -> &'static str {
        match self {
            Self::English => "✔ Installed",
            Self::Russian => "✔ Установлен",
            Self::Ukrainian => "✔ Встановлено",
        }
    }

    pub fn mods_btn_delete(&self) -> &'static str {
        match self {
            Self::English => "🗑 Delete",
            Self::Russian => "🗑 Удалить",
            Self::Ukrainian => "🗑 Видалити",
        }
    }

    pub fn mods_no_installed(&self) -> &'static str {
        match self {
            Self::English => "No mods in the 'mods/' folder. Use the search tab to find and install mods!",
            Self::Russian => "В папке 'mods/' нет установленных модов. Переключитесь на поиск, чтобы скачать!",
            Self::Ukrainian => "У папці 'mods/' немає встановлених модів. Перемкніться на пошук, щоб завантажити!",
        }
    }

    pub fn mods_no_results(&self) -> &'static str {
        match self {
            Self::English => "No mods found matching your query",
            Self::Russian => "По вашему запросу ничего не найдено",
            Self::Ukrainian => "За вашим запитом нічого не знайдено",
        }
    }

    // ==========================================
    // Skins Page
    // ==========================================
    pub fn skins_title(&self) -> &'static str {
        match self {
            Self::English => "Skin Manager",
            Self::Russian => "Управление скинами",
            Self::Ukrainian => "Керування скінами",
        }
    }

    pub fn skins_classic(&self) -> &'static str {
        match self {
            Self::English => "Classic (Steve 4px)",
            Self::Russian => "Classic (4px)",
            Self::Ukrainian => "Classic (4px)",
        }
    }

    pub fn skins_slim(&self) -> &'static str {
        match self {
            Self::English => "Slim (Alex 3px)",
            Self::Russian => "Slim (Alex 3px)",
            Self::Ukrainian => "Slim (Alex 3px)",
        }
    }

    pub fn skins_active_label(&self, name: &str) -> String {
        match self {
            Self::English => format!("Current Skin: {name}"),
            Self::Russian => format!("Текущий скин: {name}"),
            Self::Ukrainian => format!("Поточний скін: {name}"),
        }
    }

    pub fn skins_default_name(&self) -> &'static str {
        match self {
            Self::English => "Steve (Default)",
            Self::Russian => "Steve (По умолчанию)",
            Self::Ukrainian => "Steve (За замовчуванням)",
        }
    }

    pub fn skins_btn_load(&self) -> &'static str {
        match self {
            Self::English => "📁 LOAD SKIN (PNG)",
            Self::Russian => "📁 ЗАГРУЗИТЬ СКИН (PNG)",
            Self::Ukrainian => "📁 ЗАВАНТАЖИТИ СКІН (PNG)",
        }
    }

    pub fn skins_btn_reset_steve(&self) -> &'static str {
        match self {
            Self::English => "Reset to Steve",
            Self::Russian => "Сбросить на Steve",
            Self::Ukrainian => "Скинути до Steve",
        }
    }

    pub fn skins_btn_reset_alex(&self) -> &'static str {
        match self {
            Self::English => "Reset to Alex",
            Self::Russian => "Сбросить на Alex",
            Self::Ukrainian => "Скинути до Alex",
        }
    }

    pub fn skins_btn_export(&self) -> &'static str {
        match self {
            Self::English => "💾 Export PNG",
            Self::Russian => "💾 Экспортировать PNG",
            Self::Ukrainian => "💾 Експортувати PNG",
        }
    }

    pub fn skins_hint(&self) -> &'static str {
        match self {
            Self::English => "Supports standard 64x64 and legacy 64x32 PNG skins",
            Self::Russian => "Поддерживаются скины форматов 64x64 и классические 64x32 PNG",
            Self::Ukrainian => "Підтримуються скіни форматів 64x64 та класичні 64x32 PNG",
        }
    }

    // ==========================================
    // Settings Page
    // ==========================================
    pub fn settings_title(&self) -> &'static str {
        match self {
            Self::English => "Settings",
            Self::Russian => "Настройки",
            Self::Ukrainian => "Налаштування",
        }
    }

    pub fn settings_tab_general(&self) -> &'static str {
        match self {
            Self::English => "GENERAL",
            Self::Russian => "ОСНОВНЫЕ",
            Self::Ukrainian => "ОСНОВНІ",
        }
    }

    pub fn settings_tab_java(&self) -> &'static str {
        match self {
            Self::English => "JAVA & MEMORY",
            Self::Russian => "JAVA И ПАМЯТЬ",
            Self::Ukrainian => "JAVA ТА ПАМ'ЯТЬ",
        }
    }

    pub fn settings_tab_accounts(&self) -> &'static str {
        match self {
            Self::English => "ACCOUNTS",
            Self::Russian => "АККАУНТЫ",
            Self::Ukrainian => "АКАУНТИ",
        }
    }

    pub fn settings_language_title(&self) -> &'static str {
        match self {
            Self::English => "Interface Language",
            Self::Russian => "Язык интерфейса",
            Self::Ukrainian => "Мова інтерфейсу",
        }
    }

    pub fn settings_behavior_title(&self) -> &'static str {
        match self {
            Self::English => "Launcher Behavior",
            Self::Russian => "Поведение лаунчера",
            Self::Ukrainian => "Поведінка лаунчера",
        }
    }

    pub fn settings_close_after_launch(&self) -> &'static str {
        match self {
            Self::English => "Close launcher after Minecraft starts",
            Self::Russian => "Закрывать лаунчер после запуска Minecraft",
            Self::Ukrainian => "Закривати лаунчер після запуску Minecraft",
        }
    }

    pub fn settings_show_snapshots(&self) -> &'static str {
        match self {
            Self::English => "Show snapshots in version list",
            Self::Russian => "Отображать снапшоты в списке версий",
            Self::Ukrainian => "Відображати снапшоти у списку версій",
        }
    }

    pub fn settings_show_old(&self) -> &'static str {
        match self {
            Self::English => "Show legacy versions (Alpha / Beta)",
            Self::Russian => "Отображать старые версии (Alpha / Beta)",
            Self::Ukrainian => "Відображати старі версії (Alpha / Beta)",
        }
    }

    pub fn settings_resolution_title(&self) -> &'static str {
        match self {
            Self::English => "Game Window Resolution",
            Self::Russian => "Разрешение окна игры",
            Self::Ukrainian => "Роздільна здатність вікна гри",
        }
    }

    pub fn settings_width(&self) -> &'static str {
        match self {
            Self::English => "Width:",
            Self::Russian => "Ширина:",
            Self::Ukrainian => "Ширина:",
        }
    }

    pub fn settings_height(&self) -> &'static str {
        match self {
            Self::English => "Height:",
            Self::Russian => "Высота:",
            Self::Ukrainian => "Висота:",
        }
    }

    pub fn settings_fullscreen(&self) -> &'static str {
        match self {
            Self::English => "Launch in fullscreen mode",
            Self::Russian => "Запускать в полноэкранном режиме",
            Self::Ukrainian => "Запускати у повноекранному режимі",
        }
    }

    pub fn settings_folders_title(&self) -> &'static str {
        match self {
            Self::English => "Launcher Directories & Files",
            Self::Russian => "Папки и файлы лаунчера",
            Self::Ukrainian => "Папки та файли лаунчера",
        }
    }

    pub fn settings_folder_root(&self) -> &'static str {
        match self {
            Self::English => "📂 Root Folder",
            Self::Russian => "📂 Корневая папка",
            Self::Ukrainian => "📂 Коренева папка",
        }
    }

    pub fn settings_folder_logs(&self) -> &'static str {
        match self {
            Self::English => "📂 Logs",
            Self::Russian => "📂 Логи",
            Self::Ukrainian => "📂 Логи",
        }
    }

    pub fn settings_folder_instances(&self) -> &'static str {
        match self {
            Self::English => "📂 Instances",
            Self::Russian => "📂 Сборки",
            Self::Ukrainian => "📂 Збірки",
        }
    }

    pub fn settings_ram_title(&self) -> &'static str {
        match self {
            Self::English => "Memory Allocation (RAM)",
            Self::Russian => "Оперативная память (RAM)",
            Self::Ukrainian => "Оперативна пам'ять (RAM)",
        }
    }

    pub fn settings_min_ram(&self, mb: u32) -> String {
        let gb = mb as f64 / 1024.0;
        match self {
            Self::English => format!("Minimum RAM: {mb} MB ({gb:.1} GB)"),
            Self::Russian => format!("Минимальная память: {mb} МБ ({gb:.1} ГБ)"),
            Self::Ukrainian => format!("Мінімальна пам'ять: {mb} МБ ({gb:.1} ГБ)"),
        }
    }

    pub fn settings_max_ram(&self, mb: u32) -> String {
        let gb = mb as f64 / 1024.0;
        match self {
            Self::English => format!("Maximum RAM: {mb} MB ({gb:.1} GB)"),
            Self::Russian => format!("Максимальная память: {mb} МБ ({gb:.1} ГБ)"),
            Self::Ukrainian => format!("Максимальна пам'ять: {mb} МБ ({gb:.1} ГБ)"),
        }
    }

    pub fn settings_java_title(&self) -> &'static str {
        match self {
            Self::English => "Java Runtime Environment",
            Self::Russian => "Среда выполнения Java",
            Self::Ukrainian => "Середовище виконання Java",
        }
    }

    pub fn settings_java_path(&self, path_str: &str) -> String {
        match self {
            Self::English => format!("Current path: {path_str}"),
            Self::Russian => format!("Текущий путь: {path_str}"),
            Self::Ukrainian => format!("Поточний шлях: {path_str}"),
        }
    }

    pub fn settings_java_auto(&self) -> &'static str {
        match self {
            Self::English => "Auto-detect (Adoptium OpenJDK 8 / 17 / 21)",
            Self::Russian => "Автоопределение (Adoptium OpenJDK 8 / 17 / 21)",
            Self::Ukrainian => "Автовизначення (Adoptium OpenJDK 8 / 17 / 21)",
        }
    }

    pub fn settings_btn_select_java(&self) -> &'static str {
        match self {
            Self::English => "SELECT JAVA.EXE",
            Self::Russian => "ВЫБРАТЬ JAVA.EXE",
            Self::Ukrainian => "ОБРАТИ JAVA.EXE",
        }
    }

    pub fn settings_btn_reset_java(&self) -> &'static str {
        match self {
            Self::English => "Reset to Auto-detect",
            Self::Russian => "Сбросить на автовыбор",
            Self::Ukrainian => "Скинути до автовибору",
        }
    }

    pub fn settings_jvm_args_title(&self) -> &'static str {
        match self {
            Self::English => "Custom JVM Arguments",
            Self::Russian => "Пользовательские JVM аргументы",
            Self::Ukrainian => "Користувацькі JVM аргументи",
        }
    }

    pub fn settings_btn_add_arg(&self) -> &'static str {
        match self {
            Self::English => "+ Add",
            Self::Russian => "+ Добавить",
            Self::Ukrainian => "+ Додати",
        }
    }

    pub fn settings_gc_presets_title(&self) -> &'static str {
        match self {
            Self::English => "Optimization presets:",
            Self::Russian => "Готовые пресеты оптимизации:",
            Self::Ukrainian => "Готові пресети оптимізації:",
        }
    }

    pub fn settings_btn_preset_g1gc(&self) -> &'static str {
        match self {
            Self::English => "⚡ G1GC Preset (High FPS)",
            Self::Russian => "⚡ Пресет G1GC (Высокий FPS)",
            Self::Ukrainian => "⚡ Пресет G1GC (Високий FPS)",
        }
    }

    pub fn settings_tooltip_preset_g1gc(&self) -> &'static str {
        match self {
            Self::English => "Optimized garbage collector flags for maximum FPS and smooth gameplay without stutters",
            Self::Russian => "Оптимизированные флаги сборщика мусора для максимального FPS и плавной игры без фризов",
            Self::Ukrainian => "Оптимізовані прапорці збирача сміття для максимального FPS та плавної гри без фризів",
        }
    }

    pub fn settings_btn_preset_zgc(&self) -> &'static str {
        match self {
            Self::English => "🚀 ZGC Preset (Low Latency)",
            Self::Russian => "🚀 Пресет ZGC (Низкие задержки)",
            Self::Ukrainian => "🚀 Пресет ZGC (Низькі затримки)",
        }
    }

    pub fn settings_tooltip_preset_zgc(&self) -> &'static str {
        match self {
            Self::English => "Ultra-fast ZGC collector with sub-millisecond pauses (for Java 17 and 21)",
            Self::Russian => "Сверхбыстрый сборщик мусора ZGC с субмиллисекундными паузами (для Java 17 и 21)",
            Self::Ukrainian => "Надшвидкий збирач сміття ZGC з субмілісекундними паузами (для Java 17 та 21)",
        }
    }

    pub fn settings_btn_reset_args(&self) -> &'static str {
        match self {
            Self::English => "Reset",
            Self::Russian => "Сбросить",
            Self::Ukrainian => "Скинути",
        }
    }

    pub fn settings_tooltip_reset_args(&self) -> &'static str {
        match self {
            Self::English => "Clear all custom JVM flags",
            Self::Russian => "Очистить все пользовательские флаги",
            Self::Ukrainian => "Очистити всі користувацькі прапорці",
        }
    }

    pub fn settings_accounts_title(&self) -> &'static str {
        match self {
            Self::English => "Saved Accounts",
            Self::Russian => "Сохраненные аккаунты",
            Self::Ukrainian => "Збережені акаунти",
        }
    }

    pub fn settings_account_active(&self) -> &'static str {
        match self {
            Self::English => "✔ (Active)",
            Self::Russian => "✔ (Активен)",
            Self::Ukrainian => "✔ (Активний)",
        }
    }

    pub fn settings_btn_set_active(&self) -> &'static str {
        match self {
            Self::English => "Set Active",
            Self::Russian => "Сделать активным",
            Self::Ukrainian => "Зробити активним",
        }
    }

    pub fn settings_btn_delete_acc(&self) -> &'static str {
        match self {
            Self::English => "Delete",
            Self::Russian => "Удалить",
            Self::Ukrainian => "Видалити",
        }
    }

    // ==========================================
    // Login Modal
    // ==========================================
    pub fn login_modal_title(&self) -> &'static str {
        match self {
            Self::English => "Account Authorization",
            Self::Russian => "Авторизация",
            Self::Ukrainian => "Авторизація",
        }
    }

    pub fn login_tab_offline(&self) -> &'static str {
        match self {
            Self::English => "FREE / NICKNAME",
            Self::Russian => "БЕСПЛАТНО / НИК",
            Self::Ukrainian => "БЕЗКОШТОВНО / НІК",
        }
    }

    pub fn login_tab_wetid(&self) -> &'static str {
        match self {
            Self::English => "WETID (ALEPH)",
            Self::Russian => "WETID (ALEPH)",
            Self::Ukrainian => "WETID (ALEPH)",
        }
    }

    pub fn login_tab_microsoft(&self) -> &'static str {
        match self {
            Self::English => "MICROSOFT LICENSE",
            Self::Russian => "MICROSOFT ЛИЦЕНЗИЯ",
            Self::Ukrainian => "MICROSOFT ЛІЦЕНЗІЯ",
        }
    }

    pub fn login_offline_prompt(&self) -> &'static str {
        match self {
            Self::English => "Enter player nickname:",
            Self::Russian => "Введите никнейм для игры:",
            Self::Ukrainian => "Введіть нікнейм для гри:",
        }
    }

    pub fn login_offline_hint(&self) -> &'static str {
        match self {
            Self::English => "Works with any offline, cracked, and LAN servers",
            Self::Russian => "Подходит для любых пиратских и локальных серверов",
            Self::Ukrainian => "Підходить для будь-яких офлайн та локальних серверів",
        }
    }

    pub fn login_btn_play(&self) -> &'static str {
        match self {
            Self::English => "PLAY NOW",
            Self::Russian => "ВОЙТИ В ИГРУ",
            Self::Ukrainian => "УВІЙТИ В ГРУ",
        }
    }

    pub fn login_wetid_login(&self) -> &'static str {
        match self {
            Self::English => "Login or Email:",
            Self::Russian => "Логин или Email:",
            Self::Ukrainian => "Логін або Email:",
        }
    }

    pub fn login_wetid_pass(&self) -> &'static str {
        match self {
            Self::English => "Password:",
            Self::Russian => "Пароль:",
            Self::Ukrainian => "Пароль:",
        }
    }

    pub fn login_btn_wetid(&self) -> &'static str {
        match self {
            Self::English => "SIGN IN VIA WETID",
            Self::Russian => "ВОЙТИ ЧЕРЕЗ WETID",
            Self::Ukrainian => "УВІЙТИ ЧЕРЕЗ WETID",
        }
    }

    pub fn login_ms_code_label(&self) -> &'static str {
        match self {
            Self::English => "Device code:",
            Self::Russian => "Код устройства:",
            Self::Ukrainian => "Код пристрою:",
        }
    }

    pub fn login_ms_instructions(&self, url: &str) -> String {
        match self {
            Self::English => format!("Go to {url} and enter the code above:"),
            Self::Russian => format!("Перейдите по ссылке {url} и введите код выше:"),
            Self::Ukrainian => format!("Перейдіть за посиланням {url} та введіть код вище:"),
        }
    }

    pub fn login_btn_open_browser(&self) -> &'static str {
        match self {
            Self::English => "🔗 Open Browser",
            Self::Russian => "🔗 Открыть браузер",
            Self::Ukrainian => "🔗 Відкрити браузер",
        }
    }

    pub fn login_ms_waiting(&self) -> &'static str {
        match self {
            Self::English => "⏳ Waiting for login confirmation in browser...",
            Self::Russian => "⏳ Ожидание подтверждения входа в браузере...",
            Self::Ukrainian => "⏳ Очікування підтвердження входу в браузері...",
        }
    }

    pub fn login_btn_get_code(&self) -> &'static str {
        match self {
            Self::English => "GET DEVICE LOGIN CODE",
            Self::Russian => "ПОЛУЧИТЬ КОД ДЛЯ ВХОДА",
            Self::Ukrainian => "ОТРИМАТИ КОД ДЛЯ ВХОДУ",
        }
    }

    pub fn login_btn_cancel(&self) -> &'static str {
        match self {
            Self::English => "CANCEL",
            Self::Russian => "ОТМЕНА",
            Self::Ukrainian => "СКАСУВАТИ",
        }
    }

    // ==========================================
    // Console Modal
    // ==========================================
    pub fn console_title(&self) -> &'static str {
        match self {
            Self::English => "Game Console (Process Logs)",
            Self::Russian => "Консоль игры (Логи процесса)",
            Self::Ukrainian => "Консоль гри (Логи процесу)",
        }
    }

    pub fn console_filter_hint(&self) -> &'static str {
        match self {
            Self::English => "🔍 Filter logs...",
            Self::Russian => "🔍 Фильтр логов...",
            Self::Ukrainian => "🔍 Фільтр логів...",
        }
    }

    pub fn console_autoscroll(&self) -> &'static str {
        match self {
            Self::English => "Autoscroll down",
            Self::Russian => "Автоскролл вниз",
            Self::Ukrainian => "Автоскрол донизу",
        }
    }

    pub fn console_copy(&self) -> &'static str {
        match self {
            Self::English => "📋 Copy",
            Self::Russian => "📋 Копировать",
            Self::Ukrainian => "📋 Копіювати",
        }
    }

    pub fn console_clear(&self) -> &'static str {
        match self {
            Self::English => "🧹 Clear",
            Self::Russian => "🧹 Очистить",
            Self::Ukrainian => "🧹 Очистити",
        }
    }

    pub fn console_export(&self) -> &'static str {
        match self {
            Self::English => "💾 Export",
            Self::Russian => "💾 Экспорт",
            Self::Ukrainian => "💾 Експорт",
        }
    }

    // ==========================================
    // Launch Status Texts
    // ==========================================
    pub fn status_preparing(&self) -> &'static str {
        match self {
            Self::English => "Preparing launch...",
            Self::Russian => "Подготовка к запуску...",
            Self::Ukrainian => "Підготовка до запуску...",
        }
    }

    pub fn status_preparing_version(&self, ver: &str) -> String {
        match self {
            Self::English => format!("Preparing version {ver}..."),
            Self::Russian => format!("Подготовка версии {ver}..."),
            Self::Ukrainian => format!("Підготовка версії {ver}..."),
        }
    }

    pub fn status_checking_java(&self) -> &'static str {
        match self {
            Self::English => "Checking Java runtime...",
            Self::Russian => "Проверка Java...",
            Self::Ukrainian => "Перевірка середовища Java...",
        }
    }

    pub fn status_downloading_java(&self, ver: u32) -> String {
        match self {
            Self::English => format!("Downloading Adoptium OpenJDK {ver}..."),
            Self::Russian => format!("Загрузка Adoptium OpenJDK {ver}..."),
            Self::Ukrainian => format!("Завантаження Adoptium OpenJDK {ver}..."),
        }
    }

    pub fn status_fetching_manifest(&self) -> &'static str {
        match self {
            Self::English => "Fetching version manifest...",
            Self::Russian => "Загрузка манифеста версии...",
            Self::Ukrainian => "Завантаження маніфесту версії...",
        }
    }

    pub fn status_checking_assets(&self) -> &'static str {
        match self {
            Self::English => "Verifying game assets...",
            Self::Russian => "Проверка ассетов игры...",
            Self::Ukrainian => "Перевірка асетів гри...",
        }
    }

    pub fn status_downloading_files(&self, count: usize) -> String {
        match self {
            Self::English => format!("Downloading {count} files..."),
            Self::Russian => format!("Загрузка {count} файлов..."),
            Self::Ukrainian => format!("Завантаження {count} файлів..."),
        }
    }

    pub fn status_extracting_natives(&self) -> &'static str {
        match self {
            Self::English => "Extracting natives...",
            Self::Russian => "Распаковка natives...",
            Self::Ukrainian => "Розпакування natives...",
        }
    }

    pub fn status_launching(&self) -> &'static str {
        match self {
            Self::English => "Launching Minecraft process...",
            Self::Russian => "Запуск процесса Minecraft...",
            Self::Ukrainian => "Запуск процесу Minecraft...",
        }
    }

    pub fn status_process_exited(&self, code: Option<i32>) -> String {
        match self {
            Self::English => format!("Minecraft process exited with code {:?}", code),
            Self::Russian => format!("Процесс Minecraft завершен с кодом {:?}", code),
            Self::Ukrainian => format!("Процес Minecraft завершився з кодом {:?}", code),
        }
    }

    pub fn status_process_crashed(&self, msg: &str) -> String {
        match self {
            Self::English => format!("Minecraft process error: {msg}"),
            Self::Russian => format!("Ошибка процесса Minecraft: {msg}"),
            Self::Ukrainian => format!("Помилка процесу Minecraft: {msg}"),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_language_code_parsing() {
        assert_eq!(Language::parse_code("en"), Language::English);
        assert_eq!(Language::parse_code("english"), Language::English);
        assert_eq!(Language::parse_code("ru"), Language::Russian);
        assert_eq!(Language::parse_code("russian"), Language::Russian);
        assert_eq!(Language::parse_code("uk"), Language::Ukrainian);
        assert_eq!(Language::parse_code("ua"), Language::Ukrainian);
        assert_eq!(Language::parse_code("ukrainian"), Language::Ukrainian);
        assert_eq!(Language::parse_code("unknown"), Language::Russian);
    }

    #[test]
    fn test_language_codes_and_names() {
        assert_eq!(Language::English.code(), "en");
        assert_eq!(Language::Russian.code(), "ru");
        assert_eq!(Language::Ukrainian.code(), "uk");

        assert_eq!(Language::English.display_name(), "English");
        assert_eq!(Language::Russian.display_name(), "Русский");
        assert_eq!(Language::Ukrainian.display_name(), "Українська");
    }

    #[test]
    fn test_translations_coverage() {
        for lang in Language::ALL {
            assert!(!lang.nav_home().is_empty());
            assert!(!lang.nav_instances().is_empty());
            assert!(!lang.nav_mods().is_empty());
            assert!(!lang.nav_skins().is_empty());
            assert!(!lang.nav_settings().is_empty());
            assert!(!lang.nav_exit().is_empty());
            assert!(!lang.settings_title().is_empty());
            assert!(!lang.home_title().is_empty());
            assert!(!lang.inst_title().is_empty());
            assert!(!lang.mods_title().is_empty());
            assert!(!lang.skins_title().is_empty());
            assert!(!lang.login_modal_title().is_empty());
            assert!(!lang.console_title().is_empty());
        }
    }
}

