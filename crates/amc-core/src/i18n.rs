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

    /// Map a BCP-47 / POSIX locale tag (e.g. `"ru-RU"`, `"uk_UA.UTF-8"`)
    /// to a supported UI language.
    ///
    /// Pure function (no OS calls) so it is fully unit-tested.
    /// Unlike [`parse_code`](Self::parse_code), unknown tags fall back to
    /// English — the lingua-franca default for fresh installs.
    pub fn from_locale_tag(tag: &str) -> Self {
        let primary = tag
            .split(['.', '@'])
            .next()
            .unwrap_or("")
            .split(['_', '-'])
            .next()
            .unwrap_or("")
            .trim()
            .to_lowercase();
        match primary.as_str() {
            "ru" => Self::Russian,
            // Belarusian UI is not shipped; Russian is the documented fallback.
            "be" => Self::Russian,
            "uk" | "ua" => Self::Ukrainian,
            "en" => Self::English,
            _ => Self::English,
        }
    }

    /// Detect the OS UI language (CONCEPT "Первый запуск", ROADMAP P1).
    ///
    /// Never panics: when the locale cannot be determined,
    /// [`from_locale_tag`](Self::from_locale_tag) yields English.
    pub fn detect_system() -> Self {
        Self::from_locale_tag(&system_locale_tag())
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

    pub fn home_loading_versions(&self) -> &'static str {
        match self {
            Self::English => "Fetching Minecraft versions from Mojang...",
            Self::Russian => "Загрузка списка версий от Mojang...",
            Self::Ukrainian => "Завантаження списку версій від Mojang...",
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

    /// Modrinth App shell: library page heading (reference: "Library").
    pub fn inst_library_title(&self) -> &'static str {
        match self {
            Self::English => "Library",
            Self::Russian => "Библиотека",
            Self::Ukrainian => "Бібліотека",
        }
    }

    /// Modrinth App shell: compact search placeholder (reference: "Search").
    pub fn inst_library_search(&self) -> &'static str {
        match self {
            Self::English => "Search",
            Self::Russian => "Поиск",
            Self::Ukrainian => "Пошук",
        }
    }

    /// Modrinth App shell: secondary "New group" toolbar button.
    pub fn inst_btn_new_group(&self) -> &'static str {
        match self {
            Self::English => "New group",
            Self::Russian => "Новая группа",
            Self::Ukrainian => "Нова група",
        }
    }

    /// Modrinth App shell: brand "New instance" toolbar button.
    pub fn inst_btn_new_instance(&self) -> &'static str {
        match self {
            Self::English => "New instance",
            Self::Russian => "Новая сборка",
            Self::Ukrainian => "Нова збірка",
        }
    }

    /// Modrinth App shell: group-by combobox value when grouping is off.
    pub fn inst_group_none(&self) -> &'static str {
        match self {
            Self::English => "No grouping",
            Self::Russian => "Без группировки",
            Self::Ukrainian => "Без групування",
        }
    }

    /// Modrinth App shell: filter combobox label (reference: "Filter by").
    pub fn inst_filter_by(&self) -> &'static str {
        match self {
            Self::English => "Filter by",
            Self::Russian => "Фильтр по",
            Self::Ukrainian => "Фільтр за",
        }
    }

    /// Modrinth library: sort option (reference: "Last played").
    pub fn inst_sort_last_played(&self) -> &'static str {
        match self {
            Self::English => "Last played",
            Self::Russian => "Последняя игра",
            Self::Ukrainian => "Остання гра",
        }
    }

    /// Modrinth library: sort option (reference: "Hours played").
    pub fn inst_sort_hours_played(&self) -> &'static str {
        match self {
            Self::English => "Hours played",
            Self::Russian => "Наиграно часов",
            Self::Ukrainian => "Награно годин",
        }
    }

    /// Modrinth library: sort option (reference: "Date created").
    pub fn inst_sort_date_created(&self) -> &'static str {
        match self {
            Self::English => "Date created",
            Self::Russian => "Дата создания",
            Self::Ukrainian => "Дата створення",
        }
    }

    /// Modrinth library: loader word used by sorting/grouping/filtering.
    pub fn inst_word_loader(&self) -> &'static str {
        match self {
            Self::English => "Loader",
            Self::Russian => "Загрузчик",
            Self::Ukrainian => "Завантажувач",
        }
    }

    /// Modrinth library: game version word used by grouping/filtering.
    pub fn inst_word_game_version(&self) -> &'static str {
        match self {
            Self::English => "Game version",
            Self::Russian => "Версия игры",
            Self::Ukrainian => "Версія гри",
        }
    }

    /// Modrinth library: group-by value for user-created folders.
    pub fn inst_group_custom(&self) -> &'static str {
        match self {
            Self::English => "Custom group",
            Self::Russian => "Своя группа",
            Self::Ukrainian => "Власна група",
        }
    }

    /// Modrinth library: group-by combobox heading (reference: "Group by").
    pub fn inst_group_by(&self) -> &'static str {
        match self {
            Self::English => "Group by",
            Self::Russian => "Группировать по",
            Self::Ukrainian => "Групувати за",
        }
    }

    /// Modrinth library: header for instances without a custom group.
    pub fn inst_group_ungrouped(&self) -> &'static str {
        match self {
            Self::English => "Ungrouped",
            Self::Russian => "Без группы",
            Self::Ukrainian => "Без групи",
        }
    }

    /// Modrinth library: New group modal title and create button.
    pub fn inst_group_create(&self) -> &'static str {
        match self {
            Self::English => "Create group",
            Self::Russian => "Создать группу",
            Self::Ukrainian => "Створити групу",
        }
    }

    /// Modrinth library: New group modal name field label.
    pub fn inst_group_name(&self) -> &'static str {
        match self {
            Self::English => "Group name",
            Self::Russian => "Название группы",
            Self::Ukrainian => "Назва групи",
        }
    }

    /// Modrinth library: New group modal name placeholder.
    pub fn inst_group_name_hint(&self) -> &'static str {
        match self {
            Self::English => "Enter group name",
            Self::Russian => "Введите название группы",
            Self::Ukrainian => "Введіть назву групи",
        }
    }

    /// Modrinth library: New group modal instance search placeholder.
    pub fn inst_group_search(&self) -> &'static str {
        match self {
            Self::English => "Search instance",
            Self::Russian => "Поиск сборки",
            Self::Ukrainian => "Пошук збірки",
        }
    }

    /// Modrinth library: New group modal empty search result.
    pub fn inst_group_empty(&self) -> &'static str {
        match self {
            Self::English => "No instances found",
            Self::Russian => "Сборки не найдены",
            Self::Ukrainian => "Збірки не знайдено",
        }
    }

    /// Modrinth library: New group modal add-to-selection action.
    pub fn inst_group_add(&self) -> &'static str {
        match self {
            Self::English => "Add",
            Self::Russian => "Добавить",
            Self::Ukrainian => "Додати",
        }
    }

    /// Modrinth library: New group modal already-selected action.
    pub fn inst_group_added(&self) -> &'static str {
        match self {
            Self::English => "Added",
            Self::Russian => "Добавлено",
            Self::Ukrainian => "Додано",
        }
    }

    /// Modrinth library: New group modal cancel button.
    pub fn inst_group_cancel(&self) -> &'static str {
        match self {
            Self::English => "Cancel",
            Self::Russian => "Отмена",
            Self::Ukrainian => "Скасувати",
        }
    }

    /// Modrinth library: filter popup clear action.
    pub fn inst_filter_clear(&self) -> &'static str {
        match self {
            Self::English => "Clear filters",
            Self::Russian => "Сбросить фильтры",
            Self::Ukrainian => "Скинути фільтри",
        }
    }

    /// Modrinth library: card context-menu play action.
    pub fn inst_ctx_play(&self) -> &'static str {
        match self {
            Self::English => "Play",
            Self::Russian => "Играть",
            Self::Ukrainian => "Грати",
        }
    }

    /// Modrinth library: card context-menu edit action.
    pub fn inst_ctx_edit(&self) -> &'static str {
        match self {
            Self::English => "Edit",
            Self::Russian => "Настроить",
            Self::Ukrainian => "Налаштувати",
        }
    }

    /// Modrinth library: card context-menu duplicate action.
    pub fn inst_ctx_duplicate(&self) -> &'static str {
        match self {
            Self::English => "Duplicate instance",
            Self::Russian => "Дублировать сборку",
            Self::Ukrainian => "Дублювати збірку",
        }
    }

    /// Modrinth library: card context-menu manage action.
    pub fn inst_ctx_manage(&self) -> &'static str {
        match self {
            Self::English => "Manage",
            Self::Russian => "Управление",
            Self::Ukrainian => "Керування",
        }
    }

    /// Modrinth library: card context-menu desktop shortcut action.
    pub fn inst_ctx_shortcut(&self) -> &'static str {
        match self {
            Self::English => "Create shortcut",
            Self::Russian => "Создать ярлык",
            Self::Ukrainian => "Створити ярлик",
        }
    }

    /// Modrinth library: card context-menu open-folder action.
    pub fn inst_ctx_folder(&self) -> &'static str {
        match self {
            Self::English => "Open folder",
            Self::Russian => "Открыть папку",
            Self::Ukrainian => "Відкрити папку",
        }
    }

    /// Modrinth library: card context-menu delete action.
    pub fn inst_ctx_delete(&self) -> &'static str {
        match self {
            Self::English => "Delete",
            Self::Russian => "Удалить",
            Self::Ukrainian => "Видалити",
        }
    }

    /// Modrinth shell: titlebar back button tooltip.
    pub fn modrinth_title_back(&self) -> &'static str {
        match self {
            Self::English => "Go back",
            Self::Russian => "Назад",
            Self::Ukrainian => "Назад",
        }
    }

    /// Modrinth shell: titlebar forward button tooltip.
    pub fn modrinth_title_forward(&self) -> &'static str {
        match self {
            Self::English => "Go forward",
            Self::Russian => "Вперёд",
            Self::Ukrainian => "Уперед",
        }
    }

    /// Modrinth shell: right sidebar toggle tooltip.
    pub fn modrinth_toggle_sidebar(&self) -> &'static str {
        match self {
            Self::English => "Toggle sidebar",
            Self::Russian => "Боковая панель",
            Self::Ukrainian => "Бічна панель",
        }
    }

    /// Modrinth shell: titlebar idle status.
    pub fn modrinth_no_running(&self) -> &'static str {
        match self {
            Self::English => "No instances running",
            Self::Russian => "Нет запущенных сборок",
            Self::Ukrainian => "Немає запущених збірок",
        }
    }

    /// Modrinth shell: right sidebar account heading.
    pub fn modrinth_playing_as(&self) -> &'static str {
        match self {
            Self::English => "Playing as",
            Self::Russian => "Вы играете как",
            Self::Ukrainian => "Ви граєте як",
        }
    }

    /// Modrinth shell: right sidebar empty account slot.
    pub fn modrinth_no_account(&self) -> &'static str {
        match self {
            Self::English => "No account selected",
            Self::Russian => "Аккаунт не выбран",
            Self::Ukrainian => "Акаунт не вибрано",
        }
    }

    /// Modrinth shell: sign-in action.
    pub fn modrinth_sign_in(&self) -> &'static str {
        match self {
            Self::English => "Sign in",
            Self::Russian => "Войти",
            Self::Ukrainian => "Увійти",
        }
    }

    /// Modrinth shell: right sidebar friends heading.
    pub fn modrinth_friends(&self) -> &'static str {
        match self {
            Self::English => "Friends",
            Self::Russian => "Друзья",
            Self::Ukrainian => "Друзі",
        }
    }

    /// Modrinth shell: friends section empty state.
    pub fn modrinth_no_friends(&self) -> &'static str {
        match self {
            Self::English => "Sign in to Modrinth to see friends",
            Self::Russian => "Войдите в Modrinth, чтобы видеть друзей",
            Self::Ukrainian => "Увійдіть у Modrinth, щоб бачити друзів",
        }
    }

    /// Modrinth shell: right sidebar news heading.
    pub fn modrinth_news(&self) -> &'static str {
        match self {
            Self::English => "News",
            Self::Russian => "Новости",
            Self::Ukrainian => "Новини",
        }
    }

    /// Modrinth shell: static news card (launcher updates).
    pub fn modrinth_news_updates(&self) -> &'static str {
        match self {
            Self::English => "Launcher updates",
            Self::Russian => "Обновления лаунчера",
            Self::Ukrainian => "Оновлення лаунчера",
        }
    }

    /// Modrinth shell: static news card (community).
    pub fn modrinth_news_community(&self) -> &'static str {
        match self {
            Self::English => "Community spotlight",
            Self::Russian => "Новости сообщества",
            Self::Ukrainian => "Новини спільноти",
        }
    }

    /// Modrinth shell: news card meta line.
    pub fn modrinth_news_today(&self) -> &'static str {
        match self {
            Self::English => "Today",
            Self::Russian => "Сегодня",
            Self::Ukrainian => "Сьогодні",
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

    pub fn inst_sessions_label(&self, count: u64) -> String {
        match self {
            Self::English => format!("{count} sessions"),
            Self::Russian => format!("{count} сессий"),
            Self::Ukrainian => format!("{count} сесій"),
        }
    }

    pub fn inst_ported(&self, old: &str, new: &str) -> String {
        match self {
            Self::English => format!("Ported {old} → {new}: re-checking mods"),
            Self::Russian => format!("Сборка перенесена {old} → {new}: проверяю моды"),
            Self::Ukrainian => format!("Збірку перенесено {old} → {new}: перевіряю моди"),
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

    /// Modrinth shell: sentence-case local tab label.
    pub fn mods_tab_installed_plain(&self) -> &'static str {
        match self {
            Self::English => "Installed",
            Self::Russian => "Установленные",
            Self::Ukrainian => "Встановлені",
        }
    }

    /// Modrinth shell: sentence-case search tab label.
    pub fn mods_tab_search_plain(&self) -> &'static str {
        match self {
            Self::English => "Search mods",
            Self::Russian => "Поиск модов",
            Self::Ukrainian => "Пошук модів",
        }
    }

    /// Modrinth shell: folder action without the emoji prefix.
    pub fn mods_open_folder_plain(&self) -> &'static str {
        match self {
            Self::English => "Open mods folder",
            Self::Russian => "Открыть папку mods",
            Self::Ukrainian => "Відкрити папку mods",
        }
    }

    /// Modrinth shell: local list empty state.
    pub fn mods_empty_title(&self) -> &'static str {
        match self {
            Self::English => "No mods installed",
            Self::Russian => "Моды не установлены",
            Self::Ukrainian => "Моди не встановлено",
        }
    }

    /// Modrinth shell: local list empty state hint.
    pub fn mods_empty_hint(&self) -> &'static str {
        match self {
            Self::English => "Find mods for this instance in the Modrinth catalog.",
            Self::Russian => "Найдите моды для этой сборки в каталоге Modrinth.",
            Self::Ukrainian => "Знайдіть моди для цієї збірки в каталозі Modrinth.",
        }
    }

    /// Modrinth shell: empty-state action.
    pub fn mods_browse_action(&self) -> &'static str {
        match self {
            Self::English => "Browse mods",
            Self::Russian => "Найти моды",
            Self::Ukrainian => "Знайти моди",
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
            Self::English => {
                "No mods in the 'mods/' folder. Use the search tab to find and install mods!"
            }
            Self::Russian => {
                "В папке 'mods/' нет установленных модов. Переключитесь на поиск, чтобы скачать!"
            }
            Self::Ukrainian => {
                "У папці 'mods/' немає встановлених модів. Перемкніться на пошук, щоб завантажити!"
            }
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

    /// Modrinth shell: skin selector page title.
    pub fn skins_selector_title(&self) -> &'static str {
        match self {
            Self::English => "Skin selector",
            Self::Russian => "Выбор скина",
            Self::Ukrainian => "Вибір скіна",
        }
    }

    /// Modrinth shell: saved skins list heading.
    pub fn skins_saved_title(&self) -> &'static str {
        match self {
            Self::English => "Saved skins",
            Self::Russian => "Сохранённые скины",
            Self::Ukrainian => "Збережені скіни",
        }
    }

    /// Modrinth shell: add-skin action.
    pub fn skins_add(&self) -> &'static str {
        match self {
            Self::English => "Add skin",
            Self::Russian => "Добавить скин",
            Self::Ukrainian => "Додати скін",
        }
    }

    /// Modrinth shell: active-skin badge.
    pub fn skins_active_badge(&self) -> &'static str {
        match self {
            Self::English => "Active",
            Self::Russian => "Активный",
            Self::Ukrainian => "Активний",
        }
    }

    /// Modrinth shell: default skin section heading.
    pub fn skins_defaults(&self) -> &'static str {
        match self {
            Self::English => "Default skins",
            Self::Russian => "Стандартные скины",
            Self::Ukrainian => "Стандартні скіни",
        }
    }

    /// Modrinth shell: empty saved-skins state.
    pub fn skins_no_saved(&self) -> &'static str {
        match self {
            Self::English => "No saved skins yet",
            Self::Russian => "Сохранённых скинов пока нет",
            Self::Ukrainian => "Збережених скінів ще немає",
        }
    }

    /// Modrinth shell: demo-account notice.
    pub fn skins_demo_notice(&self) -> &'static str {
        match self {
            Self::English => "Sign in to save and apply skins",
            Self::Russian => "Войдите, чтобы сохранять и применять скины",
            Self::Ukrainian => "Увійдіть, щоб зберігати та застосовувати скіни",
        }
    }

    /// Modrinth shell: load-skin action without the emoji prefix.
    pub fn skins_load_skin(&self) -> &'static str {
        match self {
            Self::English => "Load skin",
            Self::Russian => "Загрузить скин",
            Self::Ukrainian => "Завантажити скін",
        }
    }

    /// Modrinth shell: classic model label.
    pub fn skins_model_classic(&self) -> &'static str {
        match self {
            Self::English => "Classic",
            Self::Russian => "Классический",
            Self::Ukrainian => "Класичний",
        }
    }

    /// Modrinth shell: slim model label.
    pub fn skins_model_slim(&self) -> &'static str {
        match self {
            Self::English => "Slim",
            Self::Russian => "Тонкий",
            Self::Ukrainian => "Тонкий",
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

    pub fn settings_after_launch_title(&self) -> &'static str {
        match self {
            Self::English => "After game start",
            Self::Russian => "После запуска игры",
            Self::Ukrainian => "Після запуску гри",
        }
    }

    pub fn settings_after_close(&self) -> &'static str {
        match self {
            Self::English => "Close",
            Self::Russian => "Закрывать",
            Self::Ukrainian => "Закривати",
        }
    }

    pub fn settings_after_minimize(&self) -> &'static str {
        match self {
            Self::English => "Minimize",
            Self::Russian => "Сворачивать",
            Self::Ukrainian => "Згортати",
        }
    }

    pub fn settings_after_keep(&self) -> &'static str {
        match self {
            Self::English => "Keep open",
            Self::Russian => "Оставлять открытым",
            Self::Ukrainian => "Залишати відкритим",
        }
    }

    pub fn settings_autostart(&self) -> &'static str {
        match self {
            Self::English => "Start with OS",
            Self::Russian => "Запускать вместе с ОС",
            Self::Ukrainian => "Запускати разом з ОС",
        }
    }

    // ---- Tutorial tour (CONCEPT, P1) ----
    pub fn tour_home_title(&self) -> &'static str {
        match self {
            Self::English => "Welcome to Aleph!",
            Self::Russian => "Добро пожаловать в Aleph!",
            Self::Ukrainian => "Ласкаво просимо до Aleph!",
        }
    }

    pub fn tour_home_body(&self) -> &'static str {
        match self {
            Self::English => "Pick a version below and hit Play. Your first launch downloads everything automatically.",
            Self::Russian => "Выбери версию ниже и нажми «Играть». При первом запуске всё скачается само.",
            Self::Ukrainian => "Обери версію нижче й натисни «Грати». За першого запуску все завантажиться само.",
        }
    }

    pub fn tour_instances_title(&self) -> &'static str {
        match self {
            Self::English => "Your instances live here",
            Self::Russian => "Сборки живут здесь",
            Self::Ukrainian => "Збірки живуть тут",
        }
    }

    pub fn tour_instances_body(&self) -> &'static str {
        match self {
            Self::English => "Create isolated packs with their own mods, worlds and settings.",
            Self::Russian => {
                "Создавай изолированные сборки со своими модами, мирами и настройками."
            }
            Self::Ukrainian => {
                "Створюй ізольовані збірки зі своїми модами, світами й налаштуваннями."
            }
        }
    }

    pub fn tour_dismiss(&self) -> &'static str {
        match self {
            Self::English => "Got it",
            Self::Russian => "Понятно",
            Self::Ukrainian => "Зрозуміло",
        }
    }

    // ---- .minecraft reuse (CONCEPT, P1) ----
    pub fn mcreuse_title(&self) -> &'static str {
        match self {
            Self::English => "Found another Minecraft",
            Self::Russian => "Найден другой Minecraft",
            Self::Ukrainian => "Знайдено інший Minecraft",
        }
    }

    pub fn mcreuse_body(&self, path: &str) -> String {
        match self {
            Self::English => {
                format!("Copy already downloaded files from {path} instead of re-downloading?")
            }
            Self::Russian => {
                format!("Скопировать уже скачанные файлы из {path} вместо повторной загрузки?")
            }
            Self::Ukrainian => format!(
                "Скопіювати вже завантажені файли з {path} замість повторного завантаження?"
            ),
        }
    }

    pub fn mcreuse_use(&self) -> &'static str {
        match self {
            Self::English => "Copy files",
            Self::Russian => "Скопировать файлы",
            Self::Ukrainian => "Скопіювати файли",
        }
    }

    pub fn mcreuse_skip(&self) -> &'static str {
        match self {
            Self::English => "Skip",
            Self::Russian => "Пропустить",
            Self::Ukrainian => "Пропустити",
        }
    }

    pub fn mcreuse_copied(&self, files: u64, mb: u64) -> String {
        match self {
            Self::English => format!("Copied {files} files ({mb} MB)"),
            Self::Russian => format!("Скопировано файлов: {files} ({mb} МБ)"),
            Self::Ukrainian => format!("Скопійовано файлів: {files} ({mb} МБ)"),
        }
    }

    // ---- Instance notes & scripts (CONCEPT, P2) ----
    pub fn inst_notes_label(&self) -> &'static str {
        match self {
            Self::English => "Notes",
            Self::Russian => "Заметки",
            Self::Ukrainian => "Нотатки",
        }
    }

    pub fn inst_notes_hint(&self) -> &'static str {
        match self {
            Self::English => "seed, who we play with…",
            Self::Russian => "сид, с кем играем…",
            Self::Ukrainian => "сід, з ким граємо…",
        }
    }

    pub fn inst_pre_label(&self) -> &'static str {
        match self {
            Self::English => "Command before launch",
            Self::Russian => "Команда до запуска",
            Self::Ukrainian => "Команда до запуску",
        }
    }

    pub fn inst_post_label(&self) -> &'static str {
        match self {
            Self::English => "Command after exit",
            Self::Russian => "Команда после выхода",
            Self::Ukrainian => "Команда після виходу",
        }
    }

    // ---- Storage root (CONCEPT "Хранение на диске", P2) ----
    pub fn settings_root_title(&self) -> &'static str {
        match self {
            Self::English => "Storage folder",
            Self::Russian => "Папка хранения",
            Self::Ukrainian => "Папка зберігання",
        }
    }

    pub fn settings_root_pick(&self) -> &'static str {
        match self {
            Self::English => "Move…",
            Self::Russian => "Переместить…",
            Self::Ukrainian => "Перемістити…",
        }
    }

    pub fn settings_root_restart(&self) -> &'static str {
        match self {
            Self::English => "Applies on next launcher start",
            Self::Russian => "Применится при следующем запуске лаунчера",
            Self::Ukrainian => "Застосується за наступного запуску лаунчера",
        }
    }

    // ---- Config editor (CONCEPT, P2) ----
    pub fn editor_title(&self) -> &'static str {
        match self {
            Self::English => "Config files",
            Self::Russian => "Конфиги модов",
            Self::Ukrainian => "Конфіги модів",
        }
    }

    pub fn editor_empty(&self) -> &'static str {
        match self {
            Self::English => "No editable configs in this instance yet",
            Self::Russian => "В этой сборке пока нет редактируемых конфигов",
            Self::Ukrainian => "У цій збірці поки немає редагованих конфігів",
        }
    }

    pub fn editor_saved(&self) -> &'static str {
        match self {
            Self::English => "Saved",
            Self::Russian => "Сохранено",
            Self::Ukrainian => "Збережено",
        }
    }

    pub fn editor_filter_hint(&self) -> &'static str {
        match self {
            Self::English => "Filter files…",
            Self::Russian => "Фильтр файлов…",
            Self::Ukrainian => "Фільтр файлів…",
        }
    }

    pub fn editor_find_hint(&self) -> &'static str {
        match self {
            Self::English => "Find in file…",
            Self::Russian => "Найти в файле…",
            Self::Ukrainian => "Знайти у файлі…",
        }
    }

    pub fn editor_matches(&self, count: usize) -> String {
        match self {
            Self::English => format!("{count} matches"),
            Self::Russian => format!("Совпадений: {count}"),
            Self::Ukrainian => format!("Збігів: {count}"),
        }
    }

    // ---- Health verify/repair + trash (CONCEPT, P2) ----
    pub fn health_title(&self) -> &'static str {
        match self {
            Self::English => "Health check",
            Self::Russian => "Проверка целостности",
            Self::Ukrainian => "Перевірка цілісності",
        }
    }

    pub fn health_ok(&self) -> &'static str {
        match self {
            Self::English => "All files in place",
            Self::Russian => "Все файлы на месте",
            Self::Ukrainian => "Усі файли на місці",
        }
    }

    pub fn health_missing(&self, n: usize) -> String {
        match self {
            Self::English => format!("{n} files missing or broken"),
            Self::Russian => format!("Отсутствует или повреждено файлов: {n}"),
            Self::Ukrainian => format!("Відсутні або пошкоджені файли: {n}"),
        }
    }

    pub fn health_repair(&self) -> &'static str {
        match self {
            Self::English => "Repair",
            Self::Russian => "Восстановить",
            Self::Ukrainian => "Відновити",
        }
    }

    pub fn health_verify(&self) -> &'static str {
        match self {
            Self::English => "Verify again",
            Self::Russian => "Проверить снова",
            Self::Ukrainian => "Перевірити знову",
        }
    }

    pub fn trash_title(&self) -> &'static str {
        match self {
            Self::English => "Trash",
            Self::Russian => "Корзина",
            Self::Ukrainian => "Кошик",
        }
    }

    pub fn trash_restore(&self) -> &'static str {
        match self {
            Self::English => "Restore",
            Self::Russian => "Восстановить",
            Self::Ukrainian => "Відновити",
        }
    }

    pub fn trash_delete_forever(&self) -> &'static str {
        match self {
            Self::English => "Delete forever",
            Self::Russian => "Удалить навсегда",
            Self::Ukrainian => "Видалити назавжди",
        }
    }

    pub fn trash_empty(&self) -> &'static str {
        match self {
            Self::English => "Trash is empty",
            Self::Russian => "Корзина пуста",
            Self::Ukrainian => "Кошик порожній",
        }
    }

    pub fn trash_empty_all(&self) -> &'static str {
        match self {
            Self::English => "Empty trash",
            Self::Russian => "Очистить корзину",
            Self::Ukrainian => "Очистити кошик",
        }
    }

    pub fn badge_local_mods(&self) -> &'static str {
        match self {
            Self::English => "LOCAL MODS",
            Self::Russian => "ЛОКАЛЬНЫЕ МОДЫ",
            Self::Ukrainian => "ЛОКАЛЬНІ МОДИ",
        }
    }

    // ---- Server favorites without a browser (CONCEPT, P8) ----
    pub fn fav_title(&self) -> &'static str {
        match self {
            Self::English => "Favorite servers",
            Self::Russian => "Любимые серверы",
            Self::Ukrainian => "Улюблені сервери",
        }
    }

    pub fn fav_address_hint(&self) -> &'static str {
        match self {
            Self::English => "play.example.com:25565",
            Self::Russian => "play.example.com:25565",
            Self::Ukrainian => "play.example.com:25565",
        }
    }

    pub fn fav_add(&self) -> &'static str {
        match self {
            Self::English => "Add",
            Self::Russian => "Добавить",
            Self::Ukrainian => "Додати",
        }
    }

    pub fn fav_remove(&self) -> &'static str {
        match self {
            Self::English => "Remove",
            Self::Russian => "Убрать",
            Self::Ukrainian => "Прибрати",
        }
    }

    pub fn fav_empty(&self) -> &'static str {
        match self {
            Self::English => "No favorites yet — they appear in every instance automatically",
            Self::Russian => "Пока пусто — избранное само появится в каждой сборке",
            Self::Ukrainian => "Поки порожньо — обране само з'явиться в кожній збірці",
        }
    }

    pub fn fav_name_hint(&self) -> &'static str {
        match self {
            Self::English => "Name (optional)",
            Self::Russian => "Название (необязательно)",
            Self::Ukrainian => "Назва (необов'язково)",
        }
    }

    pub fn fav_refresh(&self) -> &'static str {
        match self {
            Self::English => "Refresh status",
            Self::Russian => "Обновить статус",
            Self::Ukrainian => "Оновити статус",
        }
    }

    pub fn fav_invalid(&self) -> &'static str {
        match self {
            Self::English => "Enter a valid address like play.example.com:25565",
            Self::Russian => "Введи корректный адрес вида play.example.com:25565",
            Self::Ukrainian => "Введи коректну адресу виду play.example.com:25565",
        }
    }

    pub fn server_offline(&self) -> &'static str {
        match self {
            Self::English => "Server unreachable",
            Self::Russian => "Сервер недоступен",
            Self::Ukrainian => "Сервер недоступний",
        }
    }

    // ---- Worlds backup/restore (CONCEPT, P9) ----
    pub fn worlds_title(&self) -> &'static str {
        match self {
            Self::English => "Worlds",
            Self::Russian => "Миры",
            Self::Ukrainian => "Світи",
        }
    }

    pub fn world_backup_now(&self) -> &'static str {
        match self {
            Self::English => "Back up",
            Self::Russian => "Сделать бэкап",
            Self::Ukrainian => "Зробити бекап",
        }
    }

    pub fn world_backups(&self) -> &'static str {
        match self {
            Self::English => "Backups",
            Self::Russian => "Бэкапы",
            Self::Ukrainian => "Бекапи",
        }
    }

    pub fn world_restore(&self) -> &'static str {
        match self {
            Self::English => "Restore",
            Self::Russian => "Восстановить",
            Self::Ukrainian => "Відновити",
        }
    }

    pub fn world_no_saves(&self) -> &'static str {
        match self {
            Self::English => "No saves in this instance yet",
            Self::Russian => "В этой сборке пока нет сохранений",
            Self::Ukrainian => "У цій збірці поки немає збережень",
        }
    }

    pub fn world_no_backups(&self) -> &'static str {
        match self {
            Self::English => "No backups yet",
            Self::Russian => "Бэкапов пока нет",
            Self::Ukrainian => "Бекапів поки немає",
        }
    }

    pub fn world_auto_label(&self) -> &'static str {
        match self {
            Self::English => "Auto-backup every N days (0 = off)",
            Self::Russian => "Автобэкап каждые N дней (0 = выкл)",
            Self::Ukrainian => "Автобекап кожні N днів (0 = вимк)",
        }
    }

    pub fn world_delete(&self) -> &'static str {
        match self {
            Self::English => "Delete backup",
            Self::Russian => "Удалить бэкап",
            Self::Ukrainian => "Видалити бекап",
        }
    }

    pub fn world_pick_folder(&self) -> &'static str {
        match self {
            Self::English => "Choose backup folder",
            Self::Russian => "Выбрать папку бэкапов",
            Self::Ukrainian => "Вибрати папку бекапів",
        }
    }

    // ---- Screenshots gallery (CONCEPT, P10) ----
    pub fn screens_title(&self) -> &'static str {
        match self {
            Self::English => "Screenshots",
            Self::Russian => "Скриншоты",
            Self::Ukrainian => "Скріншоти",
        }
    }

    pub fn screens_empty(&self) -> &'static str {
        match self {
            Self::English => "No screenshots yet — press F2 in game",
            Self::Russian => "Скриншотов пока нет — нажми F2 в игре",
            Self::Ukrainian => "Скріншотів поки немає — натисни F2 у грі",
        }
    }

    pub fn screens_open(&self) -> &'static str {
        match self {
            Self::English => "Open",
            Self::Russian => "Открыть",
            Self::Ukrainian => "Відкрити",
        }
    }

    // ---- Notifications (CONCEPT, P10) ----
    pub fn notif_title(&self) -> &'static str {
        match self {
            Self::English => "Notifications",
            Self::Russian => "Уведомления",
            Self::Ukrainian => "Сповіщення",
        }
    }

    pub fn notif_updates(&self) -> &'static str {
        match self {
            Self::English => "Mod and pack updates",
            Self::Russian => "Обновления модов и сборок",
            Self::Ukrainian => "Оновлення модів і збірок",
        }
    }

    pub fn notif_downloads(&self) -> &'static str {
        match self {
            Self::English => "Finished downloads",
            Self::Russian => "Завершённые загрузки",
            Self::Ukrainian => "Завершені завантаження",
        }
    }

    pub fn notif_launcher(&self) -> &'static str {
        match self {
            Self::English => "Launcher news",
            Self::Russian => "Новости лаунчера",
            Self::Ukrainian => "Новини лаунчера",
        }
    }

    // ---- Self-update (CONCEPT, P11) ----
    pub fn update_title(&self) -> &'static str {
        match self {
            Self::English => "Launcher updates",
            Self::Russian => "Обновления лаунчера",
            Self::Ukrainian => "Оновлення лаунчера",
        }
    }

    pub fn update_check(&self) -> &'static str {
        match self {
            Self::English => "Check now",
            Self::Russian => "Проверить сейчас",
            Self::Ukrainian => "Перевірити зараз",
        }
    }

    pub fn update_available(&self, ver: &str) -> String {
        match self {
            Self::English => format!("Version {ver} is available on GitHub"),
            Self::Russian => format!("На GitHub доступна версия {ver}"),
            Self::Ukrainian => format!("На GitHub доступна версія {ver}"),
        }
    }

    pub fn update_latest(&self) -> &'static str {
        match self {
            Self::English => "You have the latest version",
            Self::Russian => "Установлена последняя версия",
            Self::Ukrainian => "Встановлено останню версію",
        }
    }

    // ---- Crash reports opt-out + diagnostics (CONCEPT, P12) ----
    pub fn crash_title(&self) -> &'static str {
        match self {
            Self::English => "Diagnostics",
            Self::Russian => "Диагностика",
            Self::Ukrainian => "Діагностика",
        }
    }

    pub fn crash_enable(&self) -> &'static str {
        match self {
            Self::English => "Send anonymous crash reports",
            Self::Russian => "Отправлять анонимные краш-репорты",
            Self::Ukrainian => "Надсилати анонімні краш-репорти",
        }
    }

    pub fn crash_export(&self) -> &'static str {
        match self {
            Self::English => "Export diagnostics bundle",
            Self::Russian => "Экспорт пакета диагностики",
            Self::Ukrainian => "Експорт пакета діагностики",
        }
    }

    // ---- Settings transfer for moving to another PC (CONCEPT, P10) ----
    pub fn transfer_title(&self) -> &'static str {
        match self {
            Self::English => "Move to another PC",
            Self::Russian => "Переезд на другой ПК",
            Self::Ukrainian => "Переїзд на інший ПК",
        }
    }

    pub fn transfer_desc(&self) -> &'static str {
        match self {
            Self::English => "Settings, accounts, instances and favorites in one file.",
            Self::Russian => "Настройки, аккаунты, сборки и избранное в одном файле.",
            Self::Ukrainian => "Налаштування, акаунти, збірки та обране в одному файлі.",
        }
    }

    pub fn transfer_export(&self) -> &'static str {
        match self {
            Self::English => "Export settings file",
            Self::Russian => "Экспорт файла настроек",
            Self::Ukrainian => "Експорт файла налаштувань",
        }
    }

    pub fn transfer_import(&self) -> &'static str {
        match self {
            Self::English => "Import settings file",
            Self::Russian => "Импорт файла настроек",
            Self::Ukrainian => "Імпорт файла налаштувань",
        }
    }

    // ---- Shared Java/RAM groups (CONCEPT P2) ----
    pub fn group_title(&self) -> &'static str {
        match self {
            Self::English => "Instance groups",
            Self::Russian => "Группы сборок",
            Self::Ukrainian => "Групи збірок",
        }
    }

    pub fn group_desc(&self) -> &'static str {
        match self {
            Self::English => "One edit applies to every member instance at launch.",
            Self::Russian => "Одно изменение применяется ко всем сборкам группы при запуске.",
            Self::Ukrainian => "Одна зміна застосовується до всіх збірок групи під час запуску.",
        }
    }

    pub fn group_empty(&self) -> &'static str {
        match self {
            Self::English => "No groups yet — instances use their own settings.",
            Self::Russian => "Групп пока нет — сборки используют свои настройки.",
            Self::Ukrainian => "Груп поки немає — збірки використовують свої налаштування.",
        }
    }

    pub fn group_ram(&self) -> &'static str {
        match self {
            Self::English => "Group RAM limit",
            Self::Russian => "Лимит RAM группы",
            Self::Ukrainian => "Ліміт RAM групи",
        }
    }

    pub fn group_java_auto(&self) -> &'static str {
        match self {
            Self::English => "Java: auto",
            Self::Russian => "Java: авто",
            Self::Ukrainian => "Java: авто",
        }
    }

    pub fn group_java_select(&self) -> &'static str {
        match self {
            Self::English => "Select Java…",
            Self::Russian => "Выбрать Java…",
            Self::Ukrainian => "Вибрати Java…",
        }
    }

    pub fn group_name_hint(&self) -> &'static str {
        match self {
            Self::English => "New group name…",
            Self::Russian => "Название новой группы…",
            Self::Ukrainian => "Назва нової групи…",
        }
    }

    pub fn group_add(&self) -> &'static str {
        match self {
            Self::English => "Add group",
            Self::Russian => "Добавить группу",
            Self::Ukrainian => "Додати групу",
        }
    }

    // ---- Rebindable hotkeys (CONCEPT, P10) ----
    pub fn hotkey_title(&self) -> &'static str {
        match self {
            Self::English => "Hotkeys",
            Self::Russian => "Горячие клавиши",
            Self::Ukrainian => "Гарячі клавіші",
        }
    }

    pub fn hotkey_launch(&self) -> &'static str {
        match self {
            Self::English => "Quick launch",
            Self::Russian => "Быстрый запуск",
            Self::Ukrainian => "Швидкий запуск",
        }
    }

    pub fn hotkey_tabs(&self) -> &'static str {
        match self {
            Self::English => "Switch tabs",
            Self::Russian => "Переключение вкладок",
            Self::Ukrainian => "Перемикання вкладок",
        }
    }

    // ---- Loader version pin (reproducible launches) ----
    pub fn loader_pinned(&self, version: &str) -> String {
        match self {
            Self::English => format!("Loader {version} (pinned)"),
            Self::Russian => format!("Загрузчик {version} (зафиксирован)"),
            Self::Ukrainian => format!("Завантажувач {version} (зафіксовано)"),
        }
    }

    // ---- Download manager history (CONCEPT P3) ----
    pub fn dlman_title(&self) -> &'static str {
        match self {
            Self::English => "Downloads",
            Self::Russian => "Загрузки",
            Self::Ukrainian => "Завантаження",
        }
    }

    pub fn dlman_empty(&self) -> &'static str {
        match self {
            Self::English => "Nothing downloaded yet this session",
            Self::Russian => "В этой сессии пока ничего не скачано",
            Self::Ukrainian => "У цій сесії поки нічого не завантажено",
        }
    }

    pub fn dlman_clear(&self) -> &'static str {
        match self {
            Self::English => "Clear history",
            Self::Russian => "Очистить историю",
            Self::Ukrainian => "Очистити історію",
        }
    }

    pub fn loader_pin_reset(&self) -> &'static str {
        match self {
            Self::English => "Use latest",
            Self::Russian => "На последнюю версию",
            Self::Ukrainian => "На останню версію",
        }
    }

    // ---- Clone options (CONCEPT P2 worlds choice) ----
    pub fn clone_title(&self) -> &'static str {
        match self {
            Self::English => "Duplicate instance",
            Self::Russian => "Дублировать сборку",
            Self::Ukrainian => "Дублювати збірку",
        }
    }

    pub fn clone_saves(&self) -> &'static str {
        match self {
            Self::English => "Copy worlds (saves)",
            Self::Russian => "Копировать миры (saves)",
            Self::Ukrainian => "Копіювати світи (saves)",
        }
    }

    // ---- Full instance archive (CONCEPT P9) ----
    pub fn archive_title(&self) -> &'static str {
        match self {
            Self::English => "Full backup",
            Self::Russian => "Полный бэкап",
            Self::Ukrainian => "Повний бекап",
        }
    }

    pub fn archive_desc(&self) -> &'static str {
        match self {
            Self::English => "Zip the whole instance folder for a personal archive.",
            Self::Russian => "Запаковать всю папку сборки в личный архив.",
            Self::Ukrainian => "Запакувати всю папку збірки в особистий архів.",
        }
    }

    pub fn archive_create(&self) -> &'static str {
        match self {
            Self::English => "Create archive",
            Self::Russian => "Создать архив",
            Self::Ukrainian => "Створити архів",
        }
    }

    // ---- Local dedicated server (CONCEPT Steam section, P13) ----
    pub fn server_title(&self) -> &'static str {
        match self {
            Self::English => "Local server",
            Self::Russian => "Локальный сервер",
            Self::Ukrainian => "Локальний сервер",
        }
    }

    pub fn server_create(&self) -> &'static str {
        match self {
            Self::English => "Create local server",
            Self::Russian => "Создать локальный сервер",
            Self::Ukrainian => "Створити локальний сервер",
        }
    }

    pub fn server_stop(&self) -> &'static str {
        match self {
            Self::English => "Stop server",
            Self::Russian => "Остановить сервер",
            Self::Ukrainian => "Зупинити сервер",
        }
    }

    pub fn server_running(&self, port: u16) -> String {
        match self {
            Self::English => format!("Server is running on port {port}"),
            Self::Russian => format!("Сервер запущен на порту {port}"),
            Self::Ukrainian => format!("Сервер запущено на порту {port}"),
        }
    }

    pub fn server_eula(&self) -> &'static str {
        match self {
            Self::English => "I accept the Mojang EULA",
            Self::Russian => "Я принимаю EULA Mojang",
            Self::Ukrainian => "Я приймаю EULA Mojang",
        }
    }

    // ---- Launch identity picker (CONCEPT "Аккаунты", P6) ----
    pub fn picker_title(&self) -> &'static str {
        match self {
            Self::English => "Who is playing?",
            Self::Russian => "Кто играет?",
            Self::Ukrainian => "Хто грає?",
        }
    }

    pub fn picker_remember(&self) -> &'static str {
        match self {
            Self::English => "Remember choice",
            Self::Russian => "Запомнить выбор",
            Self::Ukrainian => "Запам'ятати вибір",
        }
    }

    pub fn settings_ask_identity(&self) -> &'static str {
        match self {
            Self::English => "Ask who is playing on every launch",
            Self::Russian => "Спрашивать, кто играет, при каждом запуске",
            Self::Ukrainian => "Питати, хто грає, за кожного запуску",
        }
    }

    pub fn settings_forget_identity(&self) -> &'static str {
        match self {
            Self::English => "Forget remembered choice",
            Self::Russian => "Забыть запомненный выбор",
            Self::Ukrainian => "Забути запам'ятований вибір",
        }
    }

    // ---- Player profile (CONCEPT, P7) ----
    pub fn profile_title(&self) -> &'static str {
        match self {
            Self::English => "Player profile",
            Self::Russian => "Профиль игрока",
            Self::Ukrainian => "Профіль гравця",
        }
    }

    pub fn profile_nickname(&self) -> &'static str {
        match self {
            Self::English => "Nickname",
            Self::Russian => "Никнейм",
            Self::Ukrainian => "Нікнейм",
        }
    }

    pub fn profile_cape_none(&self) -> &'static str {
        match self {
            Self::English => "No cape",
            Self::Russian => "Нет плаща",
            Self::Ukrainian => "Немає плаща",
        }
    }

    /// Modrinth profile: signed-in account state.
    pub fn modrinth_profile_status_signed_in(&self) -> &'static str {
        match self {
            Self::English => "Signed in and ready to play",
            Self::Russian => "Вы вошли и готовы играть",
            Self::Ukrainian => "Ви увійшли й готові грати",
        }
    }

    /// Modrinth profile: offline identity state.
    pub fn modrinth_profile_status_offline(&self) -> &'static str {
        match self {
            Self::English => "Offline profile · stored on this device",
            Self::Russian => "Офлайн-профиль · хранится на этом устройстве",
            Self::Ukrainian => "Офлайн-профіль · зберігається на цьому пристрої",
        }
    }

    /// Modrinth profile: no account state.
    pub fn modrinth_profile_status_signed_out(&self) -> &'static str {
        match self {
            Self::English => "Sign in to sync your Minecraft profile.",
            Self::Russian => "Войдите, чтобы синхронизировать профиль Minecraft.",
            Self::Ukrainian => "Увійдіть, щоб синхронізувати профіль Minecraft.",
        }
    }

    pub fn modrinth_profile_details(&self) -> &'static str {
        match self {
            Self::English => "Account details",
            Self::Russian => "Данные аккаунта",
            Self::Ukrainian => "Дані акаунта",
        }
    }

    pub fn modrinth_profile_player_uuid(&self) -> &'static str {
        match self {
            Self::English => "Player UUID",
            Self::Russian => "UUID игрока",
            Self::Ukrainian => "UUID гравця",
        }
    }

    pub fn modrinth_profile_skin_model(&self) -> &'static str {
        match self {
            Self::English => "Skin model",
            Self::Russian => "Модель скина",
            Self::Ukrainian => "Модель скіна",
        }
    }

    pub fn modrinth_profile_change_skin(&self) -> &'static str {
        match self {
            Self::English => "Change skin",
            Self::Russian => "Сменить скин",
            Self::Ukrainian => "Змінити скін",
        }
    }

    pub fn modrinth_profile_manage_account(&self) -> &'static str {
        match self {
            Self::English => "Manage account",
            Self::Russian => "Управление аккаунтом",
            Self::Ukrainian => "Керування акаунтом",
        }
    }

    pub fn modrinth_profile_copy_uuid(&self) -> &'static str {
        match self {
            Self::English => "Copy UUID",
            Self::Russian => "Скопировать UUID",
            Self::Ukrainian => "Копіювати UUID",
        }
    }

    pub fn modrinth_profile_copied(&self) -> &'static str {
        match self {
            Self::English => "Copied",
            Self::Russian => "Скопировано",
            Self::Ukrainian => "Скопійовано",
        }
    }

    pub fn modrinth_profile_no_uuid(&self) -> &'static str {
        match self {
            Self::English => "No UUID available",
            Self::Russian => "UUID недоступен",
            Self::Ukrainian => "UUID недоступний",
        }
    }

    // ---- Mod compatibility & friends (CONCEPT, P4) ----
    pub fn compat_ok(&self, mc: &str, loader: &str) -> String {
        match self {
            Self::English => format!("Compatible with {mc} + {loader}"),
            Self::Russian => format!("Совместимо с {mc} + {loader}"),
            Self::Ukrainian => format!("Сумісно з {mc} + {loader}"),
        }
    }

    pub fn compat_unknown(&self) -> &'static str {
        match self {
            Self::English => "Compatibility unknown — installs latest file",
            Self::Russian => "Совместимость неизвестна — ставится последний файл",
            Self::Ukrainian => "Сумісність невідома — ставиться останній файл",
        }
    }

    pub fn compat_none(&self, mc: &str) -> String {
        match self {
            Self::English => format!("No build for Minecraft {mc}"),
            Self::Russian => format!("Нет сборки под Minecraft {mc}"),
            Self::Ukrainian => format!("Немає збірки під Minecraft {mc}"),
        }
    }

    pub fn wishlist_add(&self) -> &'static str {
        match self {
            Self::English => "Wishlist",
            Self::Russian => "В вишлист",
            Self::Ukrainian => "У вішліст",
        }
    }

    pub fn wishlist_remove(&self) -> &'static str {
        match self {
            Self::English => "Wishlisted ✓",
            Self::Russian => "В вишлисте ✓",
            Self::Ukrainian => "У вішлісті ✓",
        }
    }

    pub fn batch_select_all(&self) -> &'static str {
        match self {
            Self::English => "Select all",
            Self::Russian => "Выбрать все",
            Self::Ukrainian => "Обрати всі",
        }
    }

    pub fn batch_enable(&self) -> &'static str {
        match self {
            Self::English => "Enable selected",
            Self::Russian => "Включить выбранные",
            Self::Ukrainian => "Увімкнути обрані",
        }
    }

    pub fn batch_disable(&self) -> &'static str {
        match self {
            Self::English => "Disable selected",
            Self::Russian => "Выключить выбранные",
            Self::Ukrainian => "Вимкнути обрані",
        }
    }

    pub fn batch_delete(&self) -> &'static str {
        match self {
            Self::English => "Delete selected",
            Self::Russian => "Удалить выбранные",
            Self::Ukrainian => "Видалити обрані",
        }
    }

    pub fn mods_update_available(&self) -> &'static str {
        match self {
            Self::English => "Update available",
            Self::Russian => "Есть обновление",
            Self::Ukrainian => "Є оновлення",
        }
    }

    pub fn mods_check_updates(&self) -> &'static str {
        match self {
            Self::English => "Check updates",
            Self::Russian => "Проверить обновления",
            Self::Ukrainian => "Перевірити оновлення",
        }
    }

    pub fn mods_updates_found(&self, count: usize) -> String {
        match self {
            Self::English => format!("Mods with updates: {count}"),
            Self::Russian => format!("Моды с обновлениями: {count}"),
            Self::Ukrainian => format!("Моди з оновленнями: {count}"),
        }
    }

    pub fn mods_update_all(&self) -> &'static str {
        match self {
            Self::English => "Update all",
            Self::Russian => "Обновить всё",
            Self::Ukrainian => "Оновити все",
        }
    }

    pub fn mods_rollback(&self) -> &'static str {
        match self {
            Self::English => "Roll back updates",
            Self::Russian => "Откатить обновления",
            Self::Ukrainian => "Відкотити оновлення",
        }
    }

    pub fn mods_pin(&self) -> &'static str {
        match self {
            Self::English => "Pin this version (skip in updates)",
            Self::Russian => "Зафиксировать версию (не обновлять)",
            Self::Ukrainian => "Зафіксувати версію (не оновлювати)",
        }
    }

    pub fn mods_unpin(&self) -> &'static str {
        match self {
            Self::English => "Unpin version",
            Self::Russian => "Открепить версию",
            Self::Ukrainian => "Відкріпити версію",
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

    pub fn settings_mode_title(&self) -> &'static str {
        match self {
            Self::English => "Interface Mode",
            Self::Russian => "Режим интерфейса",
            Self::Ukrainian => "Режим інтерфейсу",
        }
    }

    pub fn settings_mode_simple(&self) -> &'static str {
        match self {
            Self::English => "Simple",
            Self::Russian => "Упрощённый",
            Self::Ukrainian => "Спрощений",
        }
    }

    pub fn settings_mode_simple_desc(&self) -> &'static str {
        match self {
            Self::English => "One setup at a time, quick-start button",
            Self::Russian => "Одна сборка за раз и кнопка быстрого старта",
            Self::Ukrainian => "Одна збірка за раз і кнопка швидкого старту",
        }
    }

    pub fn settings_mode_pro(&self) -> &'static str {
        match self {
            Self::English => "Professional",
            Self::Russian => "Профессиональный",
            Self::Ukrainian => "Професійний",
        }
    }

    pub fn settings_mode_pro_desc(&self) -> &'static str {
        match self {
            Self::English => "Multiple isolated instances, like MultiMC",
            Self::Russian => "Несколько изолированных сборок, как в MultiMC",
            Self::Ukrainian => "Декілька ізольованих збірок, як у MultiMC",
        }
    }

    pub fn settings_mode_multimc(&self) -> &'static str {
        match self {
            Self::English => "MultiMC / Prism",
            Self::Russian => "MultiMC / Prism",
            Self::Ukrainian => "MultiMC / Prism",
        }
    }

    pub fn settings_mode_multimc_desc(&self) -> &'static str {
        match self {
            Self::English => "Toolbar, build list and status bar",
            Self::Russian => "Тулбар, список сборок и статус-бар",
            Self::Ukrainian => "Тулбар, список збірок і статус-бар",
        }
    }

    pub fn settings_mode_ultra(&self) -> &'static str {
        match self {
            Self::English => "Aleph Ultra",
            Self::Russian => "Aleph Ультра",
            Self::Ukrainian => "Aleph Ультра",
        }
    }

    pub fn settings_mode_ultra_desc(&self) -> &'static str {
        match self {
            Self::English => "Nickname, version and Play — nothing else",
            Self::Russian => "Ник, версия и Играть — больше ничего",
            Self::Ukrainian => "Нік, версія та Грати — більше нічого",
        }
    }

    pub fn settings_mode_legacy(&self) -> &'static str {
        match self {
            Self::English => "Legacy",
            Self::Russian => "Legacy",
            Self::Ukrainian => "Legacy",
        }
    }

    pub fn settings_mode_legacy_desc(&self) -> &'static str {
        match self {
            Self::English => "Text tabs and a bottom Play panel",
            Self::Russian => "Текстовые вкладки и нижняя панель Play",
            Self::Ukrainian => "Текстові вкладки та нижня панель грати",
        }
    }

    pub fn settings_mode_millida(&self) -> &'static str {
        match self {
            Self::English => "Millida",
            Self::Russian => "Millida",
            Self::Ukrainian => "Millida",
        }
    }

    pub fn settings_mode_millida_desc(&self) -> &'static str {
        match self {
            Self::English => "Lobby tiles plus content pages",
            Self::Russian => "Лобби-плитки плюс контентные страницы",
            Self::Ukrainian => "Лобі-плитки та контентні сторінки",
        }
    }

    pub fn settings_mode_modrinth(&self) -> &'static str {
        match self {
            Self::English => "Modrinth",
            Self::Russian => "Modrinth",
            Self::Ukrainian => "Modrinth",
        }
    }

    pub fn settings_mode_modrinth_desc(&self) -> &'static str {
        match self {
            Self::English => "Sidebar navigation, green accents on gray",
            Self::Russian => "Навигация-сайдбар, зелёные акценты на сером",
            Self::Ukrainian => "Навігація-сайдбар, зелені акценти на сірому",
        }
    }

    // ==========================================
    // Alternative shells (visual styles)
    // ==========================================
    pub fn status_ready(&self) -> &'static str {
        match self {
            Self::English => "Ready",
            Self::Russian => "Готово",
            Self::Ukrainian => "Готово",
        }
    }

    pub fn shell_nickname(&self) -> &'static str {
        match self {
            Self::English => "Nickname",
            Self::Russian => "Никнейм",
            Self::Ukrainian => "Нікнейм",
        }
    }

    pub fn shell_version(&self) -> &'static str {
        match self {
            Self::English => "Version",
            Self::Russian => "Версия",
            Self::Ukrainian => "Версія",
        }
    }

    pub fn shell_quick_play(&self) -> &'static str {
        match self {
            Self::English => "Continue",
            Self::Russian => "Продолжить",
            Self::Ukrainian => "Продовжити",
        }
    }

    pub fn shell_lobby_title(&self) -> &'static str {
        match self {
            Self::English => "Game lobby",
            Self::Russian => "Игровое лобби",
            Self::Ukrainian => "Ігрове лобі",
        }
    }

    pub fn shell_play_tile(&self) -> &'static str {
        match self {
            Self::English => "Play now",
            Self::Russian => "Играть сейчас",
            Self::Ukrainian => "Грати зараз",
        }
    }

    pub fn shell_versions_tile(&self) -> &'static str {
        match self {
            Self::English => "Latest versions",
            Self::Russian => "Последние версии",
            Self::Ukrainian => "Останні версії",
        }
    }

    /// Millida classic rail: the lobby (Home) entry.
    pub fn shell_rail_play(&self) -> &'static str {
        match self {
            Self::English => "Play",
            Self::Russian => "Играть",
            Self::Ukrainian => "Грати",
        }
    }

    /// Millida classic rail: the instance list entry.
    pub fn shell_rail_builds(&self) -> &'static str {
        match self {
            Self::English => "My builds",
            Self::Russian => "Мои сборки",
            Self::Ukrainian => "Мої збірки",
        }
    }

    /// Millida classic rail: the content catalog entry.
    pub fn shell_rail_catalog(&self) -> &'static str {
        match self {
            Self::English => "Catalog",
            Self::Russian => "Каталог",
            Self::Ukrainian => "Каталог",
        }
    }

    pub fn shell_tile_builds(&self) -> &'static str {
        match self {
            Self::English => "Builds",
            Self::Russian => "Сборки",
            Self::Ukrainian => "Збірки",
        }
    }

    pub fn shell_tile_mods(&self) -> &'static str {
        match self {
            Self::English => "Mods",
            Self::Russian => "Моды",
            Self::Ukrainian => "Моди",
        }
    }

    pub fn shell_tile_skins(&self) -> &'static str {
        match self {
            Self::English => "Skins",
            Self::Russian => "Скины",
            Self::Ukrainian => "Скіни",
        }
    }

    pub fn shell_tile_settings(&self) -> &'static str {
        match self {
            Self::English => "Settings",
            Self::Russian => "Настройки",
            Self::Ukrainian => "Налаштування",
        }
    }

    /// Millida lobby dock: caption above the selected build.
    pub fn shell_mode_today(&self) -> &'static str {
        match self {
            Self::English => "Playing today",
            Self::Russian => "Сегодня играем",
            Self::Ukrainian => "Граємо сьогодні",
        }
    }

    /// Millida lobby dock: no instance has been picked yet.
    pub fn shell_mode_empty(&self) -> &'static str {
        match self {
            Self::English => "Nothing selected",
            Self::Russian => "Ещё не выбрано",
            Self::Ukrainian => "Ще не обрано",
        }
    }

    pub fn shell_pick_build(&self) -> &'static str {
        match self {
            Self::English => "Pick a build",
            Self::Russian => "Выберите сборку",
            Self::Ukrainian => "Виберіть збірку",
        }
    }

    pub fn shell_ready_to_launch(&self) -> &'static str {
        match self {
            Self::English => "Ready to launch",
            Self::Russian => "Готов к запуску",
            Self::Ukrainian => "Готовий до запуску",
        }
    }

    /// Millida lobby promo card: small caption and its button.
    pub fn shell_promo_label(&self) -> &'static str {
        match self {
            Self::English => "Community",
            Self::Russian => "Сообщество",
            Self::Ukrainian => "Спільнота",
        }
    }

    pub fn shell_promo_action(&self) -> &'static str {
        match self {
            Self::English => "Open",
            Self::Russian => "Открыть",
            Self::Ukrainian => "Відкрити",
        }
    }

    pub fn shell_add_instance(&self) -> &'static str {
        match self {
            Self::English => "Add instance",
            Self::Russian => "Добавить сборку",
            Self::Ukrainian => "Додати збірку",
        }
    }

    pub fn shell_launch_instance(&self) -> &'static str {
        match self {
            Self::English => "Launch",
            Self::Russian => "Запустить",
            Self::Ukrainian => "Запустити",
        }
    }

    pub fn shell_folders(&self) -> &'static str {
        match self {
            Self::English => "Folders",
            Self::Russian => "Папки",
            Self::Ukrainian => "Папки",
        }
    }

    pub fn shell_view_instance_folder(&self) -> &'static str {
        match self {
            Self::English => "View Instance Folder",
            Self::Russian => "Открыть папку сборки",
            Self::Ukrainian => "Відкрити теку збірки",
        }
    }

    pub fn shell_view_central_mods(&self) -> &'static str {
        match self {
            Self::English => "View Central Mods Folder",
            Self::Russian => "Открыть папку общих модов",
            Self::Ukrainian => "Відкрити теку спільних модів",
        }
    }

    pub fn shell_help(&self) -> &'static str {
        match self {
            Self::English => "Help",
            Self::Russian => "Справка",
            Self::Ukrainian => "Довідка",
        }
    }

    pub fn shell_help_report_bug(&self) -> &'static str {
        match self {
            Self::English => "Report a Bug",
            Self::Russian => "Сообщить об ошибке",
            Self::Ukrainian => "Повідомити про помилку",
        }
    }

    pub fn shell_help_github(&self) -> &'static str {
        match self {
            Self::English => "GitHub",
            Self::Russian => "GitHub",
            Self::Ukrainian => "GitHub",
        }
    }

    pub fn shell_update(&self) -> &'static str {
        match self {
            Self::English => "Update",
            Self::Russian => "Обновление",
            Self::Ukrainian => "Оновлення",
        }
    }

    pub fn shell_support(&self) -> &'static str {
        match self {
            Self::English => "Support",
            Self::Russian => "Поддержать",
            Self::Ukrainian => "Підтримати",
        }
    }

    pub fn shell_meow(&self) -> &'static str {
        match self {
            Self::English => "Meow",
            Self::Russian => "Мяу",
            Self::Ukrainian => "Няв",
        }
    }

    pub fn shell_accounts(&self) -> &'static str {
        match self {
            Self::English => "Accounts",
            Self::Russian => "Аккаунты",
            Self::Ukrainian => "Акаунти",
        }
    }

    pub fn shell_manage_accounts(&self) -> &'static str {
        match self {
            Self::English => "Manage Accounts",
            Self::Russian => "Управление аккаунтами",
            Self::Ukrainian => "Керування акаунтами",
        }
    }

    pub fn shell_status_no_instance(&self) -> &'static str {
        match self {
            Self::English => "No instance selected",
            Self::Russian => "Сборка не выбрана",
            Self::Ukrainian => "Збірку не вибрано",
        }
    }

    pub fn shell_total_playtime(&self) -> &'static str {
        match self {
            Self::English => "Total playtime",
            Self::Russian => "Общее время в игре",
            Self::Ukrainian => "Загальний час у грі",
        }
    }

    pub fn shell_news_none(&self) -> &'static str {
        match self {
            Self::English => "No news available.",
            Self::Russian => "Новостей нет.",
            Self::Ukrainian => "Новин немає.",
        }
    }

    pub fn shell_more_news(&self) -> &'static str {
        match self {
            Self::English => "More news…",
            Self::Russian => "Ещё новости…",
            Self::Ukrainian => "Більше новин…",
        }
    }

    pub fn shell_change_group(&self) -> &'static str {
        match self {
            Self::English => "Change Group",
            Self::Russian => "Сменить группу",
            Self::Ukrainian => "Змінити групу",
        }
    }

    pub fn shell_launch_offline(&self) -> &'static str {
        match self {
            Self::English => "Launch Offline",
            Self::Russian => "Запуск без сети",
            Self::Ukrainian => "Запуск без мережі",
        }
    }

    pub fn shell_edit_instance(&self) -> &'static str {
        match self {
            Self::English => "Edit Instance",
            Self::Russian => "Изменить сборку",
            Self::Ukrainian => "Змінити збірку",
        }
    }

    pub fn shell_edit_notes(&self) -> &'static str {
        match self {
            Self::English => "Edit Notes",
            Self::Russian => "Изменить заметки",
            Self::Ukrainian => "Змінити нотатки",
        }
    }

    pub fn shell_view_worlds(&self) -> &'static str {
        match self {
            Self::English => "View Worlds",
            Self::Russian => "Просмотр миров",
            Self::Ukrainian => "Перегляд світів",
        }
    }

    pub fn shell_manage_screenshots(&self) -> &'static str {
        match self {
            Self::English => "Manage Screenshots",
            Self::Russian => "Управление скриншотами",
            Self::Ukrainian => "Керування скриншотами",
        }
    }

    pub fn shell_folder_minecraft(&self) -> &'static str {
        match self {
            Self::English => "Minecraft Folder",
            Self::Russian => "Папка Minecraft",
            Self::Ukrainian => "Тека Minecraft",
        }
    }

    pub fn shell_folder_config(&self) -> &'static str {
        match self {
            Self::English => "Config Folder",
            Self::Russian => "Папка конфигурации",
            Self::Ukrainian => "Тека конфігурації",
        }
    }

    pub fn shell_folder_instance(&self) -> &'static str {
        match self {
            Self::English => "Instance Folder",
            Self::Russian => "Папка сборки",
            Self::Ukrainian => "Тека збірки",
        }
    }

    pub fn shell_create_shortcut(&self) -> &'static str {
        match self {
            Self::English => "Create Shortcut",
            Self::Russian => "Создать ярлык",
            Self::Ukrainian => "Створити ярлик",
        }
    }

    pub fn shell_export_instance(&self) -> &'static str {
        match self {
            Self::English => "Export Instance",
            Self::Russian => "Экспорт сборки",
            Self::Ukrainian => "Експорт збірки",
        }
    }

    pub fn shell_delete(&self) -> &'static str {
        match self {
            Self::English => "Delete",
            Self::Russian => "Удалить",
            Self::Ukrainian => "Видалити",
        }
    }

    pub fn shell_copy_instance(&self) -> &'static str {
        match self {
            Self::English => "Copy Instance",
            Self::Russian => "Копировать сборку",
            Self::Ukrainian => "Копіювати збірку",
        }
    }

    pub fn shell_profile_label(&self) -> &'static str {
        match self {
            Self::English => "Profile",
            Self::Russian => "Профиль",
            Self::Ukrainian => "Профіль",
        }
    }

    pub fn shell_version_label(&self) -> &'static str {
        match self {
            Self::English => "Version",
            Self::Russian => "Версия",
            Self::Ukrainian => "Версія",
        }
    }

    pub fn shell_new_profile(&self) -> &'static str {
        match self {
            Self::English => "New profile…",
            Self::Russian => "Новый профиль…",
            Self::Ukrainian => "Новий профіль…",
        }
    }

    pub fn shell_no_profile(&self) -> &'static str {
        match self {
            Self::English => "No profile",
            Self::Russian => "Нет профиля",
            Self::Ukrainian => "Немає профілю",
        }
    }

    pub fn shell_use_latest(&self) -> &'static str {
        match self {
            Self::English => "Use latest version",
            Self::Russian => "Последняя версия",
            Self::Ukrainian => "Остання версія",
        }
    }

    // ==========================================
    // First-run wizard (ROADMAP P1)
    // ==========================================
    pub fn wizard_title(&self) -> &'static str {
        match self {
            Self::English => "Setup Wizard",
            Self::Russian => "Мастер настройки",
            Self::Ukrainian => "Майстер налаштування",
        }
    }

    pub fn wizard_step_language(&self) -> &'static str {
        match self {
            Self::English => "Step 1 — Language",
            Self::Russian => "Шаг 1 — Язык",
            Self::Ukrainian => "Крок 1 — Мова",
        }
    }

    pub fn wizard_step_mode(&self) -> &'static str {
        match self {
            Self::English => "Step 2 — Interface",
            Self::Russian => "Шаг 2 — Интерфейс",
            Self::Ukrainian => "Крок 2 — Інтерфейс",
        }
    }

    pub fn wizard_step_account(&self) -> &'static str {
        match self {
            Self::English => "Step 3 — Account",
            Self::Russian => "Шаг 3 — Аккаунт",
            Self::Ukrainian => "Крок 3 — Акаунт",
        }
    }

    pub fn wizard_next(&self) -> &'static str {
        match self {
            Self::English => "Next →",
            Self::Russian => "Далее →",
            Self::Ukrainian => "Далі →",
        }
    }

    pub fn wizard_back(&self) -> &'static str {
        match self {
            Self::English => "← Back",
            Self::Russian => "← Назад",
            Self::Ukrainian => "← Назад",
        }
    }

    pub fn wizard_ms_title(&self) -> &'static str {
        match self {
            Self::English => "Microsoft account",
            Self::Russian => "Аккаунт Microsoft",
            Self::Ukrainian => "Акаунт Microsoft",
        }
    }

    pub fn wizard_ms_desc(&self) -> &'static str {
        match self {
            Self::English => "Licensed servers. Opens the login window.",
            Self::Russian => "Лицензионные серверы. Откроет окно входа.",
            Self::Ukrainian => "Ліцензійні сервери. Відкриє вікно входу.",
        }
    }

    pub fn wizard_offline_title(&self) -> &'static str {
        match self {
            Self::English => "Play without an account",
            Self::Russian => "Играть без аккаунта",
            Self::Ukrainian => "Грати без акаунта",
        }
    }

    pub fn wizard_offline_hint(&self) -> &'static str {
        match self {
            Self::English => "Nickname, 3–16 chars: A–Z, 0–9, _",
            Self::Russian => "Ник, 3–16 символов: A–Z, 0–9, _",
            Self::Ukrainian => "Нік, 3–16 символів: A–Z, 0–9, _",
        }
    }

    pub fn wizard_offline_invalid(&self) -> &'static str {
        match self {
            Self::English => "Check the nickname format above",
            Self::Russian => "Проверь формат ника выше",
            Self::Ukrainian => "Перевір формат ніку вище",
        }
    }

    pub fn wizard_wetid_title(&self) -> &'static str {
        match self {
            Self::English => "WetID",
            Self::Russian => "WetID",
            Self::Ukrainian => "WetID",
        }
    }

    pub fn wizard_wetid_soon(&self) -> &'static str {
        match self {
            Self::English => "Browser sign-in arrives in P6",
            Self::Russian => "Вход через браузер появится в P6",
            Self::Ukrainian => "Вхід через браузер з'явиться в P6",
        }
    }

    pub fn wizard_hw_title(&self) -> &'static str {
        match self {
            Self::English => "Weak hardware detected",
            Self::Russian => "Обнаружено слабое железо",
            Self::Ukrainian => "Виявлено слабке залізо",
        }
    }

    pub fn wizard_hw_body(&self) -> &'static str {
        match self {
            Self::English => "The launcher will run, but expect lower FPS. Lightweight mod suggestions (Sodium/Lithium) arrive with the mod manager.",
            Self::Russian => "Лаунчер будет работать, но FPS может быть ниже. Подсказки лёгких модов (Sodium/Lithium) появятся вместе с менеджером модов.",
            Self::Ukrainian => "Лаунчер працюватиме, але FPS може бути нижчим. Підказки легких модів (Sodium/Lithium) з'являться разом із менеджером модів.",
        }
    }

    pub fn wizard_hw_specs(&self, ram: &str, threads: u32, disk: &str) -> String {
        match self {
            Self::English => {
                format!("RAM: {ram} (min 4 GB) • CPU threads: {threads} (min 4) • Disk: {disk} (min 2 GB)")
            }
            Self::Russian => {
                format!("RAM: {ram} (мин. 4 ГБ) • Потоков CPU: {threads} (мин. 4) • Диск: {disk} (мин. 2 ГБ)")
            }
            Self::Ukrainian => {
                format!("RAM: {ram} (мін. 4 ГБ) • Потоків CPU: {threads} (мін. 4) • Диск: {disk} (мін. 2 ГБ)")
            }
        }
    }

    pub fn wizard_hw_ok(&self) -> &'static str {
        match self {
            Self::English => "Understood",
            Self::Russian => "Понятно",
            Self::Ukrainian => "Зрозуміло",
        }
    }

    pub fn wizard_hw_sodium(&self) -> &'static str {
        match self {
            Self::English => "Install Sodium + Lithium",
            Self::Russian => "Установить Sodium + Lithium",
            Self::Ukrainian => "Встановити Sodium + Lithium",
        }
    }

    pub fn wizard_hw_no_instance(&self) -> &'static str {
        match self {
            Self::English => "Create or select an instance first",
            Self::Russian => "Сначала создай или выбери сборку",
            Self::Ukrainian => "Спершу створи або вибери збірку",
        }
    }

    pub fn wizard_hw_bad_loader(&self) -> &'static str {
        match self {
            Self::English => "Sodium needs a Fabric or Quilt instance",
            Self::Russian => "Для Sodium нужна сборка на Fabric или Quilt",
            Self::Ukrainian => "Для Sodium потрібна збірка на Fabric або Quilt",
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
            Self::English => {
                "Ultra-fast ZGC collector with sub-millisecond pauses (for Java 17 and 21)"
            }
            Self::Russian => {
                "Сверхбыстрый сборщик мусора ZGC с субмиллисекундными паузами (для Java 17 и 21)"
            }
            Self::Ukrainian => {
                "Надшвидкий збирач сміття ZGC з субмілісекундними паузами (для Java 17 та 21)"
            }
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

    pub fn status_backup_worlds(&self) -> &'static str {
        match self {
            Self::English => "Backing up worlds...",
            Self::Russian => "Бэкап миров...",
            Self::Ukrainian => "Бекап світів...",
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

    pub fn status_resolving_loader(&self, loader: &str, mc: &str) -> String {
        match self {
            Self::English => format!("Resolving {loader} for {mc}..."),
            Self::Russian => format!("Подбираем {loader} для {mc}..."),
            Self::Ukrainian => format!("Підбираємо {loader} для {mc}..."),
        }
    }

    pub fn err_loader_failed(&self, loader: &str, err: &str) -> String {
        match self {
            Self::English => format!("Failed to prepare {loader}: {err}"),
            Self::Russian => format!("Не удалось подготовить {loader}: {err}"),
            Self::Ukrainian => format!("Не вдалося підготувати {loader}: {err}"),
        }
    }

    pub fn status_loader_profile(&self, loader: &str, ver: &str) -> String {
        match self {
            Self::English => format!("Loading {loader} profile {ver}..."),
            Self::Russian => format!("Загрузка профиля {loader} {ver}..."),
            Self::Ukrainian => format!("Завантаження профілю {loader} {ver}..."),
        }
    }

    pub fn err_version_failed(&self, ver: &str, err: &str) -> String {
        match self {
            Self::English => format!("Failed to load version {ver}: {err}"),
            Self::Russian => format!("Ошибка загрузки версии {ver}: {err}"),
            Self::Ukrainian => format!("Помилка завантаження версії {ver}: {err}"),
        }
    }

    pub fn err_java_failed(&self, req: &str, err: &str) -> String {
        match self {
            Self::English => format!("Failed to install Java {req}: {err}"),
            Self::Russian => format!("Ошибка установки Java {req}: {err}"),
            Self::Ukrainian => format!("Помилка встановлення Java {req}: {err}"),
        }
    }

    pub fn err_components_failed(&self, err: &str) -> String {
        match self {
            Self::English => format!("Failed to download components: {err}"),
            Self::Russian => format!("Ошибка скачивания компонентов: {err}"),
            Self::Ukrainian => format!("Помилка завантаження компонентів: {err}"),
        }
    }

    pub fn err_launch_failed(&self, err: &str) -> String {
        match self {
            Self::English => format!("Failed to launch: {err}"),
            Self::Russian => format!("Ошибка запуска: {err}"),
            Self::Ukrainian => format!("Помилка запуску: {err}"),
        }
    }

    pub fn err_mod_file_failed(&self, filename: &str, err: &str) -> String {
        match self {
            Self::English => format!("Failed to download {filename}: {err}"),
            Self::Russian => format!("Ошибка загрузки {filename}: {err}"),
            Self::Ukrainian => format!("Помилка завантаження {filename}: {err}"),
        }
    }

    pub fn err_version_missing(&self, ver: &str) -> String {
        match self {
            Self::English => format!("Version {ver} not found in manifest"),
            Self::Russian => format!("Версия {ver} не найдена в манифесте"),
            Self::Ukrainian => format!("Версія {ver} не знайдена в маніфесті"),
        }
    }

    /// Human-readable byte count (CONCEPT "Место на диске").
    pub fn disk_size(&self, bytes: u64) -> String {
        const MB: f64 = 1024.0 * 1024.0;
        const GB: f64 = 1024.0 * MB;
        let (value, unit) = if bytes < 1024 * 1024 {
            (
                format!("{}", bytes / 1024),
                match self {
                    Self::English => "KB",
                    Self::Russian | Self::Ukrainian => "КБ",
                },
            )
        } else if bytes < 1024 * 1024 * 1024 {
            (
                Self::one_decimal(bytes as f64 / MB, self),
                match self {
                    Self::English => "MB",
                    Self::Russian | Self::Ukrainian => "МБ",
                },
            )
        } else {
            (
                Self::one_decimal(bytes as f64 / GB, self),
                match self {
                    Self::English => "GB",
                    Self::Russian | Self::Ukrainian => "ГБ",
                },
            )
        };
        format!("{value} {unit}")
    }

    fn one_decimal(value: f64, lang: &Self) -> String {
        let text = format!("{value:.1}");
        match lang {
            Self::English => text,
            Self::Russian | Self::Ukrainian => text.replace('.', ","),
        }
    }

    pub fn err_low_disk(&self, need: &str, free: &str) -> String {
        match self {
            Self::English => format!("Not enough disk space: need {need}, free {free}"),
            Self::Russian => format!("Не хватает места на диске: нужно {need}, свободно {free}"),
            Self::Ukrainian => {
                format!("Не вистачає місця на диску: потрібно {need}, вільно {free}")
            }
        }
    }

    /// Per-second transfer rate (CONCEPT P3 progress display).
    pub fn speed_value(&self, bytes_per_sec: f64) -> String {
        const KB: f64 = 1024.0;
        const MB: f64 = 1024.0 * 1024.0;
        if bytes_per_sec >= MB {
            let unit = match self {
                Self::English => "MB/s",
                Self::Russian | Self::Ukrainian => "МБ/с",
            };
            format!("{} {unit}", Self::one_decimal(bytes_per_sec / MB, self))
        } else if bytes_per_sec >= KB {
            let unit = match self {
                Self::English => "KB/s",
                Self::Russian | Self::Ukrainian => "КБ/с",
            };
            format!("{} {unit}", Self::one_decimal(bytes_per_sec / KB, self))
        } else {
            let unit = match self {
                Self::English => "B/s",
                Self::Russian | Self::Ukrainian => "Б/с",
            };
            format!("{:.0} {unit}", bytes_per_sec)
        }
    }

    /// Countdown text for an ETA in seconds.
    pub fn eta_text(&self, secs: u64) -> String {
        match self {
            Self::English => {
                if secs >= 60 {
                    format!("{} min {} sec", secs / 60, secs % 60)
                } else {
                    format!("{secs} sec")
                }
            }
            Self::Russian => {
                if secs >= 60 {
                    format!("{} мин {} сек", secs / 60, secs % 60)
                } else {
                    format!("{secs} сек")
                }
            }
            Self::Ukrainian => {
                if secs >= 60 {
                    format!("{} хв {} сек", secs / 60, secs % 60)
                } else {
                    format!("{secs} сек")
                }
            }
        }
    }

    pub fn status_download_cancelled(&self) -> &'static str {
        match self {
            Self::English => "Download cancelled",
            Self::Russian => "Загрузка отменена",
            Self::Ukrainian => "Завантаження скасовано",
        }
    }

    pub fn status_download_paused(&self) -> &'static str {
        match self {
            Self::English => "Download paused — press Resume to continue",
            Self::Russian => "Загрузка приостановлена — нажмите «Продолжить»",
            Self::Ukrainian => "Завантаження призупинено — натисніть «Продовжити»",
        }
    }

    pub fn overlay_title(&self) -> &'static str {
        match self {
            Self::English => "Downloading components",
            Self::Russian => "Загрузка компонентов",
            Self::Ukrainian => "Завантаження компонентів",
        }
    }

    pub fn overlay_file(&self, name: &str) -> String {
        match self {
            Self::English => format!("File: {name}"),
            Self::Russian => format!("Файл: {name}"),
            Self::Ukrainian => format!("Файл: {name}"),
        }
    }

    pub fn overlay_btn_cancel(&self) -> &'static str {
        match self {
            Self::English => "Cancel",
            Self::Russian => "Отмена",
            Self::Ukrainian => "Скасувати",
        }
    }

    pub fn overlay_btn_pause(&self) -> &'static str {
        match self {
            Self::English => "Pause",
            Self::Russian => "Пауза",
            Self::Ukrainian => "Пауза",
        }
    }

    pub fn overlay_btn_resume(&self) -> &'static str {
        match self {
            Self::English => "Resume",
            Self::Russian => "Продолжить",
            Self::Ukrainian => "Продовжити",
        }
    }

    pub fn settings_speed_title(&self) -> &'static str {
        match self {
            Self::English => "Download speed limit",
            Self::Russian => "Лимит скорости загрузки",
            Self::Ukrainian => "Ліміт швидкості завантаження",
        }
    }

    pub fn settings_speed_unlimited(&self) -> &'static str {
        match self {
            Self::English => "0 = unlimited, applies to game and installer downloads",
            Self::Russian => "0 — без лимита, действует на загрузки игры и установщиков",
            Self::Ukrainian => "0 — без ліміту, діє на завантаження гри та інсталяторів",
        }
    }

    // ==========================================
    // Instance templates (ROADMAP P2)
    // ==========================================
    pub fn tmpl_title(&self) -> &'static str {
        match self {
            Self::English => "Template",
            Self::Russian => "Шаблон",
            Self::Ukrainian => "Шаблон",
        }
    }

    pub fn tmpl_name(&self, id: &str) -> &'static str {
        match (self, id) {
            (Self::English, "vanilla_plus") => "Vanilla+",
            (Self::Russian, "vanilla_plus") => "Vanilla+",
            (Self::Ukrainian, "vanilla_plus") => "Vanilla+",
            (Self::English, "optimized") => "Optimized",
            (Self::Russian, "optimized") => "Оптимизированная",
            (Self::Ukrainian, "optimized") => "Оптимізована",
            (Self::English, _) => "Empty",
            (Self::Russian, _) => "Пустая",
            (Self::Ukrainian, _) => "Порожня",
        }
    }

    pub fn tmpl_desc(&self, id: &str) -> &'static str {
        match (self, id) {
            (Self::English, "vanilla_plus") => "Latest vanilla with comfortable memory",
            (Self::Russian, "vanilla_plus") => "Свежая ванилла с запасом памяти",
            (Self::Ukrainian, "vanilla_plus") => "Свіжа ванілла із запасом пам'яті",
            (Self::English, "optimized") => {
                "Fabric for performance mods — add Sodium/Lithium after creation"
            }
            (Self::Russian, "optimized") => {
                "Fabric под моды производительности — Sodium/Lithium поставьте после создания"
            }
            (Self::Ukrainian, "optimized") => {
                "Fabric під моди продуктивності — Sodium/Lithium поставте після створення"
            }
            (Self::English, _) => "Clean vanilla, default settings",
            (Self::Russian, _) => "Чистая ванилла, настройки по умолчанию",
            (Self::Ukrainian, _) => "Чиста ванілла, налаштування за замовчуванням",
        }
    }

    pub fn tmpl_custom_note(&self) -> &'static str {
        match self {
            Self::English => "Custom setup — tune freely",
            Self::Russian => "Своя конфигурация — настраивайте свободно",
            Self::Ukrainian => "Власна конфігурація — налаштовуйте вільно",
        }
    }

    pub fn settings_game_args_title(&self) -> &'static str {
        match self {
            Self::English => "Game arguments",
            Self::Russian => "Аргументы игры",
            Self::Ukrainian => "Аргументи гри",
        }
    }

    pub fn discover_title(&self) -> &'static str {
        match self {
            Self::English => "Trending",
            Self::Russian => "В тренде",
            Self::Ukrainian => "У тренді",
        }
    }

    pub fn discover_refresh(&self) -> &'static str {
        match self {
            Self::English => "Refresh",
            Self::Russian => "Обновить",
            Self::Ukrainian => "Оновити",
        }
    }

    pub fn discover_install(&self) -> &'static str {
        match self {
            Self::English => "Install pack",
            Self::Russian => "Установить сборку",
            Self::Ukrainian => "Встановити збірку",
        }
    }

    pub fn discover_no_instance(&self) -> &'static str {
        match self {
            Self::English => "Select an instance to install the pack into",
            Self::Russian => "Выбери сборку для установки пака",
            Self::Ukrainian => "Вибери збірку для встановлення паку",
        }
    }

    pub fn nav_profile(&self) -> &'static str {
        match self {
            Self::English => "Profile",
            Self::Russian => "Профиль",
            Self::Ukrainian => "Профіль",
        }
    }

    // ==========================================
    // Java manager: system discovery (ROADMAP P5)
    // ==========================================
    pub fn settings_java_found_title(&self) -> &'static str {
        match self {
            Self::English => "Found on this PC",
            Self::Russian => "Найдено на этом ПК",
            Self::Ukrainian => "Знайдено на цьому ПК",
        }
    }

    pub fn settings_java_scanning(&self) -> &'static str {
        match self {
            Self::English => "Scanning for Java…",
            Self::Russian => "Ищем Java…",
            Self::Ukrainian => "Шукаємо Java…",
        }
    }

    pub fn settings_java_none_found(&self) -> &'static str {
        match self {
            Self::English => "Nothing found — the launcher will download Java itself",
            Self::Russian => "Ничего не найдено — лаунчер скачает Java сам",
            Self::Ukrainian => "Нічого не знайдено — лаунчер завантажить Java сам",
        }
    }

    pub fn settings_btn_use_java(&self) -> &'static str {
        match self {
            Self::English => "Use",
            Self::Russian => "Использовать",
            Self::Ukrainian => "Використати",
        }
    }

    pub fn settings_runtimes_title(&self) -> &'static str {
        match self {
            Self::English => "Launcher runtimes",
            Self::Russian => "Среды лаунчера",
            Self::Ukrainian => "Середовища лаунчера",
        }
    }

    pub fn sess_title(&self) -> &'static str {
        match self {
            Self::English => "Session summary",
            Self::Russian => "Сводка сессии",
            Self::Ukrainian => "Підсумок сесії",
        }
    }

    // ==========================================
    // Instance organization: icons, tags, pin, sort (ROADMAP P2)
    // ==========================================
    pub fn inst_sort_title(&self) -> &'static str {
        match self {
            Self::English => "Sort",
            Self::Russian => "Сортировка",
            Self::Ukrainian => "Сортування",
        }
    }

    pub fn inst_sort_name(&self) -> &'static str {
        match self {
            Self::English => "Name",
            Self::Russian => "Имя",
            Self::Ukrainian => "Ім'я",
        }
    }

    pub fn inst_sort_recent(&self) -> &'static str {
        match self {
            Self::English => "Recent",
            Self::Russian => "Недавние",
            Self::Ukrainian => "Недавні",
        }
    }

    pub fn inst_sort_created(&self) -> &'static str {
        match self {
            Self::English => "Newest",
            Self::Russian => "Новые",
            Self::Ukrainian => "Нові",
        }
    }

    pub fn inst_sort_played(&self) -> &'static str {
        match self {
            Self::English => "Most played",
            Self::Russian => "Наигранное",
            Self::Ukrainian => "Награне",
        }
    }

    pub fn inst_tag_all(&self) -> &'static str {
        match self {
            Self::English => "All",
            Self::Russian => "Все",
            Self::Ukrainian => "Всі",
        }
    }

    pub fn inst_icon_title(&self) -> &'static str {
        match self {
            Self::English => "Icon",
            Self::Russian => "Иконка",
            Self::Ukrainian => "Іконка",
        }
    }

    pub fn inst_icon_custom(&self) -> &'static str {
        match self {
            Self::English => "Custom PNG…",
            Self::Russian => "Свой PNG…",
            Self::Ukrainian => "Свій PNG…",
        }
    }

    pub fn inst_icon_none(&self) -> &'static str {
        match self {
            Self::English => "No icon",
            Self::Russian => "Без иконки",
            Self::Ukrainian => "Без іконки",
        }
    }

    pub fn inst_icon_remove(&self) -> &'static str {
        match self {
            Self::English => "Remove custom",
            Self::Russian => "Убрать свою",
            Self::Ukrainian => "Прибрати свою",
        }
    }

    pub fn inst_tags_label(&self) -> &'static str {
        match self {
            Self::English => "Tags",
            Self::Russian => "Теги",
            Self::Ukrainian => "Теги",
        }
    }

    pub fn inst_tags_hint(&self) -> &'static str {
        match self {
            Self::English => "tag1, tag2",
            Self::Russian => "тег1, тег2",
            Self::Ukrainian => "тег1, тег2",
        }
    }

    pub fn inst_group_label(&self) -> &'static str {
        match self {
            Self::English => "Java/RAM group",
            Self::Russian => "Группа Java/RAM",
            Self::Ukrainian => "Група Java/RAM",
        }
    }

    pub fn inst_group_hint(&self) -> &'static str {
        match self {
            Self::English => "Empty = own settings",
            Self::Russian => "Пусто = свои настройки",
            Self::Ukrainian => "Порожньо = свої налаштування",
        }
    }

    pub fn inst_res_custom(&self) -> &'static str {
        match self {
            Self::English => "Custom resolution for this instance",
            Self::Russian => "Своё разрешение для этой сборки",
            Self::Ukrainian => "Своя роздільна здатність для цієї збірки",
        }
    }

    pub fn inst_res_mode(&self) -> &'static str {
        match self {
            Self::English => "Display mode",
            Self::Russian => "Режим экрана",
            Self::Ukrainian => "Режим екрана",
        }
    }

    pub fn inst_res_default(&self) -> &'static str {
        match self {
            Self::English => "Launcher default",
            Self::Russian => "Как в лаунчере",
            Self::Ukrainian => "Як у лаунчері",
        }
    }

    pub fn inst_res_windowed(&self) -> &'static str {
        match self {
            Self::English => "Windowed",
            Self::Russian => "Оконный",
            Self::Ukrainian => "Віконний",
        }
    }

    pub fn inst_res_fullscreen(&self) -> &'static str {
        match self {
            Self::English => "Fullscreen",
            Self::Russian => "Полный экран",
            Self::Ukrainian => "Повний екран",
        }
    }

    pub fn inst_pin_label(&self) -> &'static str {
        match self {
            Self::English => "Pin to top",
            Self::Russian => "Закрепить сверху",
            Self::Ukrainian => "Закріпити зверху",
        }
    }

    pub fn inst_account_label(&self) -> &'static str {
        match self {
            Self::English => "Instance account",
            Self::Russian => "Аккаунт сборки",
            Self::Ukrainian => "Акаунт збірки",
        }
    }

    pub fn inst_account_default(&self) -> &'static str {
        match self {
            Self::English => "Active account",
            Self::Russian => "Активный аккаунт",
            Self::Ukrainian => "Активний акаунт",
        }
    }

    pub fn inst_warn_legacy_forge(&self) -> &'static str {
        match self {
            Self::English => {
                "Old Forge line: known vulnerabilities possible, update at your own risk"
            }
            Self::Russian => {
                "Старая линейка Forge: возможны известные уязвимости, используйте на свой риск"
            }
            Self::Ukrainian => {
                "Стара лінійка Forge: можливі відомі вразливості, використовуйте на свій ризик"
            }
        }
    }

    pub fn home_hero_hint(&self) -> &'static str {
        match self {
            Self::English => "Pick a version and hit Play. Files download themselves.",
            Self::Russian => "Выбери версию и нажми «Играть». Файлы скачаются сами.",
            Self::Ukrainian => "Обери версію і натисни «Грати». Файли завантажаться самі.",
        }
    }

    pub fn home_hero_pick(&self) -> &'static str {
        match self {
            Self::English => "Choose version",
            Self::Russian => "Выбрать версию",
            Self::Ukrainian => "Обрати версію",
        }
    }

    pub fn home_stat_versions(&self) -> &'static str {
        match self {
            Self::English => "VERSIONS",
            Self::Russian => "ВЕРСИЙ",
            Self::Ukrainian => "ВЕРСІЙ",
        }
    }

    pub fn home_stat_instances(&self) -> &'static str {
        match self {
            Self::English => "INSTANCES",
            Self::Russian => "СБОРОК",
            Self::Ukrainian => "ЗБІРОК",
        }
    }

    pub fn home_stat_ram(&self) -> &'static str {
        match self {
            Self::English => "MEMORY",
            Self::Russian => "ПАМЯТЬ",
            Self::Ukrainian => "ПАМ'ЯТЬ",
        }
    }

    pub fn home_drawer_close(&self) -> &'static str {
        match self {
            Self::English => "Close",
            Self::Russian => "Закрыть",
            Self::Ukrainian => "Закрити",
        }
    }

    pub fn home_drawer_installed(&self) -> &'static str {
        match self {
            Self::English => "INSTALLED",
            Self::Russian => "УСТАНОВЛЕНА",
            Self::Ukrainian => "ВСТАНОВЛЕНО",
        }
    }

    pub fn home_drawer_not_installed(&self) -> &'static str {
        match self {
            Self::English => "NOT DOWNLOADED",
            Self::Russian => "НЕ СКАЧАНА",
            Self::Ukrainian => "НЕ ЗАВАНТАЖЕНА",
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

    // ==========================================
    // Crash Diagnostic Assistant
    // ==========================================
    pub fn diag_header_label(&self) -> &'static str {
        match self {
            Self::English => "Crash Diagnostic Assistant",
            Self::Russian => "Диагностика сбоя игры",
            Self::Ukrainian => "Діагностика збою гри",
        }
    }

    pub fn diag_oom_title(&self) -> &'static str {
        match self {
            Self::English => "Out of Memory (Java Heap)",
            Self::Russian => "Нехватка оперативной памяти (OOM)",
            Self::Ukrainian => "Нестача оперативної пам'яті (OOM)",
        }
    }

    pub fn diag_oom_desc(&self) -> &'static str {
        match self {
            Self::English => "Minecraft ran out of allocated Java heap memory.",
            Self::Russian => "Процессу Minecraft не хватило выделенной памяти Java Heap.",
            Self::Ukrainian => "Процесу Minecraft не вистачило виділеної пам'яті Java Heap.",
        }
    }

    pub fn diag_oom_solution(&self) -> &'static str {
        match self {
            Self::English => {
                "Increase maximum RAM allocation in Instance Settings (4096–6144 MB recommended)."
            }
            Self::Russian => {
                "Увеличьте выделение ОЗУ в параметрах сборки (рекомендуется 4096–6144 МБ)."
            }
            Self::Ukrainian => {
                "Збільшіть виділення ОЗП у параметрах збірки (рекомендовано 4096–6144 МБ)."
            }
        }
    }

    pub fn diag_java_title(&self) -> &'static str {
        match self {
            Self::English => "Java Version Incompatibility",
            Self::Russian => "Несовместимая версия Java",
            Self::Ukrainian => "Несумісна версія Java",
        }
    }

    pub fn diag_java_desc(&self) -> &'static str {
        match self {
            Self::English => {
                "The game or one of its mods requires a different Java runtime version."
            }
            Self::Russian => {
                "Игра или один из модов скомпилированы под другую версию Java Runtime."
            }
            Self::Ukrainian => "Гра або один із модів скомпільовані під іншу версію Java Runtime.",
        }
    }

    pub fn diag_java_solution(&self) -> &'static str {
        match self {
            Self::English => "Ensure appropriate Java is selected (Java 8 for ≤1.12, Java 17 for 1.18+, Java 21 for 1.20.5+).",
            Self::Russian => "Проверьте версию Java (Java 8 для ≤1.12, Java 17 для 1.18+, Java 21 для 1.20.5+).",
            Self::Ukrainian => "Перевірте версію Java (Java 8 для ≤1.12, Java 17 для 1.18+, Java 21 для 1.20.5+).",
        }
    }

    pub fn diag_fabric_dep_title(&self) -> &'static str {
        match self {
            Self::English => "Missing Fabric Mod Dependencies",
            Self::Russian => "Отсутствуют зависимости Fabric",
            Self::Ukrainian => "Відсутні залежності Fabric",
        }
    }

    pub fn diag_fabric_dep_desc(&self) -> &'static str {
        match self {
            Self::English => "One or more mods require additional libraries or Fabric API.",
            Self::Russian => {
                "Один или несколько модов требуют дополнительные библиотеки или Fabric API."
            }
            Self::Ukrainian => {
                "Один або кілька модів вимагають додаткові бібліотеки або Fabric API."
            }
        }
    }

    pub fn diag_fabric_dep_solution(&self) -> &'static str {
        match self {
            Self::English => "Install missing dependencies (such as Fabric API) from the Mods tab.",
            Self::Russian => {
                "Установите недостающие моды (например, Fabric API) во вкладке «Моды»."
            }
            Self::Ukrainian => "Встановіть відсутні моди (наприклад, Fabric API) у вкладці «Моди».",
        }
    }

    pub fn diag_mixin_title(&self) -> &'static str {
        match self {
            Self::English => "Mixin Conflict Between Mods",
            Self::Russian => "Конфликт Mixin между модами",
            Self::Ukrainian => "Конфлікт Mixin між модами",
        }
    }

    pub fn diag_mixin_desc(&self) -> &'static str {
        match self {
            Self::English => "Two or more mods are trying to inject conflicting bytecode patches into the same class.",
            Self::Russian => "Два или более мода пытаются применить конфликтующие патчи к одному и тому же классу игры.",
            Self::Ukrainian => "Два або більше модів намагаються застосувати конфліктні патчі до одного й того самого класу гри.",
        }
    }

    pub fn diag_mixin_solution(&self) -> &'static str {
        match self {
            Self::English => "Disable recently installed mods one by one in the Mods tab to find the conflict.",
            Self::Russian => "Отключайте недавно добавленные моды по одному во вкладке «Моды», чтобы изолировать конфликт.",
            Self::Ukrainian => "Вимикайте нещодавно додані моди по одному у вкладці «Моди», щоб ізолювати конфлікт.",
        }
    }

    pub fn diag_gpu_title(&self) -> &'static str {
        match self {
            Self::English => "OpenGL / GPU Driver Error",
            Self::Russian => "Сбой графического драйвера OpenGL",
            Self::Ukrainian => "Збій графічного драйвера OpenGL",
        }
    }

    pub fn diag_gpu_desc(&self) -> &'static str {
        match self {
            Self::English => {
                "The graphics driver failed to create a valid OpenGL context (GLFW error 65542)."
            }
            Self::Russian => "Видеодрайвер не смог создать контекст OpenGL (ошибка GLFW 65542).",
            Self::Ukrainian => {
                "Відеодрайвер не зміг створити контекст OpenGL (помилка GLFW 65542)."
            }
        }
    }

    pub fn diag_gpu_solution(&self) -> &'static str {
        match self {
            Self::English => "Update your GPU driver and ensure Minecraft runs on your dedicated GPU.",
            Self::Russian => "Обновите драйвер видеокарты и убедитесь, что игра запускается на дискретном GPU.",
            Self::Ukrainian => "Оновіть драйвер відеокарти та переконайтеся, що гра запускається на дискретному GPU.",
        }
    }

    // ==========================================
    // Instance Clone & Content Categories
    // ==========================================
    pub fn inst_btn_clone(&self) -> &'static str {
        match self {
            Self::English => "Duplicate",
            Self::Russian => "Клонировать",
            Self::Ukrainian => "Дублювати",
        }
    }

    pub fn inst_btn_shortcut(&self) -> &'static str {
        match self {
            Self::English => "Desktop shortcut (direct launch)",
            Self::Russian => "Ярлык на рабочий стол (прямой запуск)",
            Self::Ukrainian => "Ярлик на робочий стіл (прямий запуск)",
        }
    }

    pub fn mods_cat_mods(&self) -> &'static str {
        match self {
            Self::English => "Mods",
            Self::Russian => "Моды",
            Self::Ukrainian => "Моди",
        }
    }

    pub fn mods_cat_resourcepacks(&self) -> &'static str {
        match self {
            Self::English => "Resource Packs",
            Self::Russian => "Ресурспаки",
            Self::Ukrainian => "Ресурспаки",
        }
    }

    pub fn mods_cat_shaders(&self) -> &'static str {
        match self {
            Self::English => "Shaders",
            Self::Russian => "Шейдеры",
            Self::Ukrainian => "Шейдери",
        }
    }

    pub fn mods_sort_relevance(&self) -> &'static str {
        match self {
            Self::English => "Relevance",
            Self::Russian => "По релевантности",
            Self::Ukrainian => "За релевантністю",
        }
    }

    pub fn mods_sort_downloads(&self) -> &'static str {
        match self {
            Self::English => "Popular",
            Self::Russian => "Популярные",
            Self::Ukrainian => "Популярні",
        }
    }

    pub fn mods_install_optifine(&self) -> &'static str {
        match self {
            Self::English => "Install OptiFine",
            Self::Russian => "Установить OptiFine",
            Self::Ukrainian => "Встановити OptiFine",
        }
    }

    pub fn mods_optifine_need_instance(&self) -> &'static str {
        match self {
            Self::English => "Select an instance first",
            Self::Russian => "Сначала выбери сборку",
            Self::Ukrainian => "Спочатку вибери збірку",
        }
    }

    pub fn toast_mod_installed(&self, name: &str) -> String {
        match self {
            Self::English => format!("Mod \"{name}\" installed!"),
            Self::Russian => format!("Мод \"{name}\" установлен!"),
            Self::Ukrainian => format!("Мод \"{name}\" встановлено!"),
        }
    }

    pub fn err_mod_copy_failed(&self, name: &str, err: &str) -> String {
        match self {
            Self::English => format!("Failed to copy {name}: {err}"),
            Self::Russian => format!("Не удалось скопировать {name}: {err}"),
            Self::Ukrainian => format!("Не вдалося скопіювати {name}: {err}"),
        }
    }

    pub fn toast_world_imported(&self, name: &str) -> String {
        match self {
            Self::English => format!("World \"{name}\" imported!"),
            Self::Russian => format!("Мир \"{name}\" импортирован!"),
            Self::Ukrainian => format!("Світ \"{name}\" імпортовано!"),
        }
    }

    pub fn err_world_import_failed(&self, err: &str) -> String {
        match self {
            Self::English => format!("Failed to import world: {err}"),
            Self::Russian => format!("Не удалось импортировать мир: {err}"),
            Self::Ukrainian => format!("Не вдалося імпортувати світ: {err}"),
        }
    }

    pub fn err_mod_resolve_failed(&self, err: &str) -> String {
        match self {
            Self::English => format!("Failed to resolve mod file: {err}"),
            Self::Russian => format!("Не удалось получить файл: {err}"),
            Self::Ukrainian => format!("Не вдалося отримати файл: {err}"),
        }
    }

    pub fn console_started(&self, pid: u32) -> String {
        match self {
            Self::English => format!("Minecraft started, PID {pid}"),
            Self::Russian => format!("Minecraft успешно запущен, PID {pid}"),
            Self::Ukrainian => format!("Minecraft успішно запущено, PID {pid}"),
        }
    }

    pub fn home_direct_play(&self) -> &'static str {
        match self {
            Self::English => "Direct Connect",
            Self::Russian => "Быстрый вход",
            Self::Ukrainian => "Швидкий вхід",
        }
    }

    pub fn bottom_account_ms(&self) -> &'static str {
        match self {
            Self::English => "Microsoft License",
            Self::Russian => "Лицензия Microsoft",
            Self::Ukrainian => "Ліцензія Microsoft",
        }
    }

    pub fn bottom_account_wetid(&self) -> &'static str {
        match self {
            Self::English => "WetID AlephTrust",
            Self::Russian => "WetID AlephTrust",
            Self::Ukrainian => "WetID AlephTrust",
        }
    }

    pub fn bottom_account_offline(&self) -> &'static str {
        match self {
            Self::English => "Offline Free",
            Self::Russian => "Офлайн",
            Self::Ukrainian => "Офлайн",
        }
    }

    pub fn login_offline_placeholder(&self) -> &'static str {
        match self {
            Self::English => "Player",
            Self::Russian => "Игрок",
            Self::Ukrainian => "Гравець",
        }
    }

    pub fn login_wetid_hint(&self) -> &'static str {
        match self {
            Self::English => "Access to AlephTrust servers, cloud skins and stats",
            Self::Russian => "Доступ к серверам AlephTrust, облачным скинам и статистике",
            Self::Ukrainian => "Доступ до серверів AlephTrust, хмарних скінів та статистики",
        }
    }

    pub fn login_ms_prompt(&self) -> &'static str {
        match self {
            Self::English => "Sign in via official Microsoft OAuth2 Device Code Flow:",
            Self::Russian => "Вход через официальный OAuth2 Device Code Flow:",
            Self::Ukrainian => "Вхід через офіційний OAuth2 Device Code Flow:",
        }
    }

    pub fn login_ms_hint(&self) -> &'static str {
        match self {
            Self::English => "Required for playing on official licensed servers (Hypixel, etc.)",
            Self::Russian => {
                "Требуется для игры на официальных лицензионных серверах (Hypixel и др.)"
            }
            Self::Ukrainian => "Потрібно для гри на офіційних ліцензійних серверах (Hypixel тощо)",
        }
    }

    pub fn err_nickname_invalid(&self) -> &'static str {
        match self {
            Self::English => "Invalid nickname (3-16 latin letters, digits, _)",
            Self::Russian => "Некорректный ник (3-16 латинских букв, цифр, _)",
            Self::Ukrainian => "Некоректний нік (3-16 латинських літер, цифр, _)",
        }
    }

    pub fn skins_account_label(&self) -> &'static str {
        match self {
            Self::English => "Account",
            Self::Russian => "Аккаунт",
            Self::Ukrainian => "Акаунт",
        }
    }

    pub fn skins_custom_name(&self) -> &'static str {
        match self {
            Self::English => "Custom Skin",
            Self::Russian => "Свой скин",
            Self::Ukrainian => "Власний скін",
        }
    }

    pub fn inst_copy_name(&self, name: &str) -> String {
        match self {
            Self::English => format!("{name} (Copy)"),
            Self::Russian => format!("{name} (Копия)"),
            Self::Ukrainian => format!("{name} (Копія)"),
        }
    }

    pub fn inst_cmd_hint(&self) -> &'static str {
        match self {
            Self::English => "cmd /C ... / sh -c ...",
            Self::Russian => "cmd /C ... / sh -c ...",
            Self::Ukrainian => "cmd /C ... / sh -c ...",
        }
    }

    pub fn err_backup_failed(&self, err: &str) -> String {
        match self {
            Self::English => format!("Backup failed: {err}"),
            Self::Russian => format!("Ошибка резервной копии: {err}"),
            Self::Ukrainian => format!("Помилка резервної копії: {err}"),
        }
    }

    pub fn err_delete_failed(&self, err: &str) -> String {
        match self {
            Self::English => format!("Delete failed: {err}"),
            Self::Russian => format!("Ошибка удаления: {err}"),
            Self::Ukrainian => format!("Помилка видалення: {err}"),
        }
    }

    pub fn err_restore_failed(&self, err: &str) -> String {
        match self {
            Self::English => format!("Restore failed: {err}"),
            Self::Russian => format!("Ошибка восстановления: {err}"),
            Self::Ukrainian => format!("Помилка відновлення: {err}"),
        }
    }

    pub fn err_save_failed(&self, err: &str) -> String {
        match self {
            Self::English => format!("Save failed: {err}"),
            Self::Russian => format!("Ошибка сохранения: {err}"),
            Self::Ukrainian => format!("Помилка збереження: {err}"),
        }
    }

    pub fn err_archive_failed(&self, err: &str) -> String {
        match self {
            Self::English => format!("Archive failed: {err}"),
            Self::Russian => format!("Ошибка архива: {err}"),
            Self::Ukrainian => format!("Помилка архіву: {err}"),
        }
    }

    pub fn err_transfer_export_failed(&self, err: &str) -> String {
        match self {
            Self::English => format!("Export failed: {err}"),
            Self::Russian => format!("Ошибка экспорта: {err}"),
            Self::Ukrainian => format!("Помилка експорту: {err}"),
        }
    }

    pub fn err_transfer_import_failed(&self, err: &str) -> String {
        match self {
            Self::English => format!("Import failed: {err}"),
            Self::Russian => format!("Ошибка импорта: {err}"),
            Self::Ukrainian => format!("Помилка імпорту: {err}"),
        }
    }

    pub fn err_shortcut_failed(&self, err: &str) -> String {
        match self {
            Self::English => format!("Shortcut failed: {err}"),
            Self::Russian => format!("Ошибка ярлыка: {err}"),
            Self::Ukrainian => format!("Помилка ярлика: {err}"),
        }
    }

    pub fn err_server_eula_required(&self) -> &'static str {
        match self {
            Self::English => "Accept the Mojang EULA to start the server",
            Self::Russian => "Примите EULA Mojang для запуска сервера",
            Self::Ukrainian => "Прийміть EULA Mojang для запуску сервера",
        }
    }

    pub fn err_pack_no_files(&self) -> &'static str {
        match self {
            Self::English => "Pack has no downloadable files",
            Self::Russian => "В сборке нет файлов для загрузки",
            Self::Ukrainian => "У збірці немає файлів для завантаження",
        }
    }

    pub fn err_mrpack_bad(&self, err: &str) -> String {
        match self {
            Self::English => format!("Bad .mrpack: {err}"),
            Self::Russian => format!("Битый .mrpack: {err}"),
            Self::Ukrainian => format!("Битий .mrpack: {err}"),
        }
    }

    pub fn err_cursepack_bad(&self, err: &str) -> String {
        match self {
            Self::English => format!("Bad CurseForge pack: {err}"),
            Self::Russian => format!("Битый CurseForge-пак: {err}"),
            Self::Ukrainian => format!("Битий CurseForge-пак: {err}"),
        }
    }

    pub fn err_optifine_no_build(&self, mc: &str) -> String {
        match self {
            Self::English => format!("No OptiFine builds published for MC {mc}"),
            Self::Russian => format!("Нет сборок OptiFine для MC {mc}"),
            Self::Ukrainian => format!("Немає збірок OptiFine для MC {mc}"),
        }
    }

    pub fn account_tag_ms(&self) -> &'static str {
        match self {
            Self::English => "Microsoft",
            Self::Russian => "Microsoft",
            Self::Ukrainian => "Microsoft",
        }
    }

    pub fn account_tag_wetid(&self) -> &'static str {
        match self {
            Self::English => "WetID",
            Self::Russian => "WetID",
            Self::Ukrainian => "WetID",
        }
    }

    pub fn account_tag_offline(&self) -> &'static str {
        match self {
            Self::English => "Offline",
            Self::Russian => "Офлайн",
            Self::Ukrainian => "Офлайн",
        }
    }

    pub fn home_hero_kicker(&self) -> &'static str {
        match self {
            Self::English => "Minecraft launcher",
            Self::Russian => "Лаунчер Minecraft",
            Self::Ukrainian => "Лаунчер Minecraft",
        }
    }

    pub fn settings_speed_unit(&self) -> &'static str {
        match self {
            Self::English => "KB/s",
            Self::Russian => "КБ/с",
            Self::Ukrainian => "КБ/с",
        }
    }

    pub fn hotkey_tabs_suffix(&self) -> &'static str {
        match self {
            Self::English => "+ 1…6",
            Self::Russian => "+ 1…6",
            Self::Ukrainian => "+ 1…6",
        }
    }

    pub fn profile_uuid_label(&self, uuid: &str) -> String {
        match self {
            Self::English => format!("UUID: {uuid}"),
            Self::Russian => format!("UUID: {uuid}"),
            Self::Ukrainian => format!("UUID: {uuid}"),
        }
    }
}

/// Raw OS locale tag: `"ru-RU"` on Windows, `"uk_UA.UTF-8"` on POSIX.
/// Empty string when detection fails — [`Language::from_locale_tag`] maps it
/// to the English fallback.
#[cfg(target_os = "windows")]
fn system_locale_tag() -> String {
    windows_locale_tag().unwrap_or_default()
}

#[cfg(target_os = "windows")]
fn windows_locale_tag() -> Option<String> {
    // LOCALE_NAME_MAX_LENGTH (85 wchar units, including the NUL terminator).
    let mut buf = [0u16; 85];
    // SAFETY: `buf` is a valid writable UTF-16 buffer of the documented
    // capacity; the API writes at most `buf.len()` units including NUL.
    let len = unsafe {
        windows_sys::Win32::Globalization::GetUserDefaultLocaleName(
            buf.as_mut_ptr(),
            buf.len() as i32,
        )
    };
    if len <= 0 {
        return None;
    }
    let end = buf.iter().position(|&c| c == 0).unwrap_or(buf.len());
    Some(String::from_utf16_lossy(&buf[..end]))
}

#[cfg(not(target_os = "windows"))]
fn system_locale_tag() -> String {
    for key in ["LC_ALL", "LC_MESSAGES", "LANG"] {
        if let Ok(val) = std::env::var(key) {
            let val = val.trim().to_string();
            if !val.is_empty() && val != "C" && val != "POSIX" {
                return val;
            }
        }
    }
    String::new()
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
    fn test_disk_size_formatting() {
        assert_eq!(Language::English.disk_size(0), "0 KB");
        assert_eq!(Language::English.disk_size(512 * 1024), "512 KB");
        assert_eq!(Language::English.disk_size(1536 * 1024), "1.5 MB");
        assert_eq!(
            Language::English.disk_size(2 * 1024 * 1024 * 1024),
            "2.0 GB"
        );
        assert_eq!(Language::Russian.disk_size(1536 * 1024), "1,5 МБ");
        assert_eq!(
            Language::Russian.disk_size(2 * 1024 * 1024 * 1024),
            "2,0 ГБ"
        );
        assert_eq!(Language::Ukrainian.disk_size(100 * 1024), "100 КБ");
    }

    #[test]
    fn test_locale_tag_mapping() {
        // BCP-47 (Windows) and POSIX forms, any letter case.
        assert_eq!(Language::from_locale_tag("ru-RU"), Language::Russian);
        assert_eq!(Language::from_locale_tag("ru_RU.UTF-8"), Language::Russian);
        assert_eq!(Language::from_locale_tag("RU"), Language::Russian);
        assert_eq!(Language::from_locale_tag("be-BY"), Language::Russian);
        assert_eq!(Language::from_locale_tag("uk-UA"), Language::Ukrainian);
        assert_eq!(
            Language::from_locale_tag("uk_UA.UTF-8"),
            Language::Ukrainian
        );
        assert_eq!(Language::from_locale_tag("en-US"), Language::English);
        assert_eq!(Language::from_locale_tag("en_US.UTF-8"), Language::English);
        assert_eq!(Language::from_locale_tag("EN-us"), Language::English);
        // Unknown / empty tags fall back to English, never panic.
        assert_eq!(Language::from_locale_tag("de-DE"), Language::English);
        assert_eq!(Language::from_locale_tag("ja_JP.UTF-8"), Language::English);
        assert_eq!(Language::from_locale_tag("C"), Language::English);
        assert_eq!(Language::from_locale_tag(""), Language::English);
        // Detection itself must never panic, whatever the OS reports.
        let _ = Language::detect_system();
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
            assert!(!lang.mods_tab_installed_plain().is_empty());
            assert!(!lang.mods_tab_search_plain().is_empty());
            assert!(!lang.mods_open_folder_plain().is_empty());
            assert!(!lang.mods_empty_title().is_empty());
            assert!(!lang.mods_empty_hint().is_empty());
            assert!(!lang.mods_browse_action().is_empty());
            assert!(!lang.skins_title().is_empty());
            assert!(!lang.skins_selector_title().is_empty());
            assert!(!lang.skins_saved_title().is_empty());
            assert!(!lang.skins_add().is_empty());
            assert!(!lang.skins_active_badge().is_empty());
            assert!(!lang.skins_defaults().is_empty());
            assert!(!lang.skins_no_saved().is_empty());
            assert!(!lang.skins_demo_notice().is_empty());
            assert!(!lang.skins_load_skin().is_empty());
            assert!(!lang.skins_model_classic().is_empty());
            assert!(!lang.skins_model_slim().is_empty());
            assert!(!lang.login_modal_title().is_empty());
            assert!(!lang.console_title().is_empty());
            assert!(!lang.diag_header_label().is_empty());
            assert!(!lang.diag_oom_title().is_empty());
            assert!(!lang.diag_java_title().is_empty());
            assert!(!lang.diag_fabric_dep_title().is_empty());
            assert!(!lang.diag_mixin_title().is_empty());
            assert!(!lang.diag_gpu_title().is_empty());
            assert!(!lang.inst_btn_clone().is_empty());
            assert!(!lang.inst_library_title().is_empty());
            assert!(!lang.inst_library_search().is_empty());
            assert!(!lang.inst_btn_new_group().is_empty());
            assert!(!lang.inst_btn_new_instance().is_empty());
            assert!(!lang.inst_group_none().is_empty());
            assert!(!lang.inst_filter_by().is_empty());
            assert!(!lang.inst_sort_last_played().is_empty());
            assert!(!lang.inst_sort_hours_played().is_empty());
            assert!(!lang.inst_sort_date_created().is_empty());
            assert!(!lang.inst_word_loader().is_empty());
            assert!(!lang.inst_word_game_version().is_empty());
            assert!(!lang.inst_group_custom().is_empty());
            assert!(!lang.inst_group_by().is_empty());
            assert!(!lang.inst_group_ungrouped().is_empty());
            assert!(!lang.inst_group_create().is_empty());
            assert!(!lang.inst_group_name().is_empty());
            assert!(!lang.inst_group_name_hint().is_empty());
            assert!(!lang.inst_group_search().is_empty());
            assert!(!lang.inst_group_empty().is_empty());
            assert!(!lang.inst_group_add().is_empty());
            assert!(!lang.inst_group_added().is_empty());
            assert!(!lang.inst_group_cancel().is_empty());
            assert!(!lang.inst_filter_clear().is_empty());
            assert!(!lang.inst_ctx_play().is_empty());
            assert!(!lang.inst_ctx_edit().is_empty());
            assert!(!lang.inst_ctx_duplicate().is_empty());
            assert!(!lang.inst_ctx_manage().is_empty());
            assert!(!lang.inst_ctx_shortcut().is_empty());
            assert!(!lang.inst_ctx_folder().is_empty());
            assert!(!lang.inst_ctx_delete().is_empty());
            assert!(!lang.modrinth_title_back().is_empty());
            assert!(!lang.modrinth_title_forward().is_empty());
            assert!(!lang.modrinth_toggle_sidebar().is_empty());
            assert!(!lang.modrinth_no_running().is_empty());
            assert!(!lang.modrinth_playing_as().is_empty());
            assert!(!lang.modrinth_no_account().is_empty());
            assert!(!lang.modrinth_sign_in().is_empty());
            assert!(!lang.modrinth_friends().is_empty());
            assert!(!lang.modrinth_no_friends().is_empty());
            assert!(!lang.modrinth_news().is_empty());
            assert!(!lang.modrinth_news_updates().is_empty());
            assert!(!lang.modrinth_news_community().is_empty());
            assert!(!lang.modrinth_news_today().is_empty());
            assert!(!lang.shell_folders().is_empty());
            assert!(!lang.shell_view_instance_folder().is_empty());
            assert!(!lang.shell_view_central_mods().is_empty());
            assert!(!lang.shell_help().is_empty());
            assert!(!lang.shell_help_report_bug().is_empty());
            assert!(!lang.shell_help_github().is_empty());
            assert!(!lang.shell_update().is_empty());
            assert!(!lang.shell_support().is_empty());
            assert!(!lang.shell_meow().is_empty());
            assert!(!lang.shell_accounts().is_empty());
            assert!(!lang.shell_manage_accounts().is_empty());
            assert!(!lang.shell_status_no_instance().is_empty());
            assert!(!lang.shell_total_playtime().is_empty());
            assert!(!lang.shell_news_none().is_empty());
            assert!(!lang.shell_more_news().is_empty());
            assert!(!lang.shell_change_group().is_empty());
            assert!(!lang.shell_launch_offline().is_empty());
            assert!(!lang.shell_edit_instance().is_empty());
            assert!(!lang.shell_edit_notes().is_empty());
            assert!(!lang.shell_view_worlds().is_empty());
            assert!(!lang.shell_manage_screenshots().is_empty());
            assert!(!lang.shell_folder_minecraft().is_empty());
            assert!(!lang.shell_folder_config().is_empty());
            assert!(!lang.shell_folder_instance().is_empty());
            assert!(!lang.shell_create_shortcut().is_empty());
            assert!(!lang.shell_export_instance().is_empty());
            assert!(!lang.shell_delete().is_empty());
            assert!(!lang.shell_copy_instance().is_empty());
            assert!(!lang.shell_profile_label().is_empty());
            assert!(!lang.shell_version_label().is_empty());
            assert!(!lang.shell_new_profile().is_empty());
            assert!(!lang.shell_no_profile().is_empty());
            assert!(!lang.shell_use_latest().is_empty());
            assert!(!lang.shell_rail_play().is_empty());
            assert!(!lang.shell_rail_builds().is_empty());
            assert!(!lang.shell_rail_catalog().is_empty());
            assert!(!lang.shell_tile_builds().is_empty());
            assert!(!lang.shell_tile_mods().is_empty());
            assert!(!lang.shell_tile_skins().is_empty());
            assert!(!lang.shell_tile_settings().is_empty());
            assert!(!lang.shell_mode_today().is_empty());
            assert!(!lang.shell_mode_empty().is_empty());
            assert!(!lang.shell_pick_build().is_empty());
            assert!(!lang.shell_ready_to_launch().is_empty());
            assert!(!lang.shell_promo_label().is_empty());
            assert!(!lang.shell_promo_action().is_empty());
            assert!(!lang.mods_cat_mods().is_empty());
            assert!(!lang.mods_cat_resourcepacks().is_empty());
            assert!(!lang.mods_cat_shaders().is_empty());
            assert!(!lang.home_direct_play().is_empty());
            assert!(!lang.settings_mode_title().is_empty());
            assert!(!lang.settings_mode_simple().is_empty());
            assert!(!lang.settings_mode_simple_desc().is_empty());
            assert!(!lang.settings_mode_pro().is_empty());
            assert!(!lang.settings_mode_pro_desc().is_empty());
            for (name, desc) in [
                (
                    lang.settings_mode_multimc(),
                    lang.settings_mode_multimc_desc(),
                ),
                (lang.settings_mode_ultra(), lang.settings_mode_ultra_desc()),
                (
                    lang.settings_mode_legacy(),
                    lang.settings_mode_legacy_desc(),
                ),
                (
                    lang.settings_mode_millida(),
                    lang.settings_mode_millida_desc(),
                ),
                (
                    lang.settings_mode_modrinth(),
                    lang.settings_mode_modrinth_desc(),
                ),
                (lang.status_ready(), lang.shell_nickname()),
                (lang.shell_version(), lang.shell_quick_play()),
                (lang.shell_lobby_title(), lang.shell_play_tile()),
                (lang.shell_versions_tile(), lang.shell_add_instance()),
                (lang.shell_launch_instance(), lang.shell_quick_play()),
            ] {
                assert!(!name.is_empty());
                assert!(!desc.is_empty());
            }
            assert!(!lang.wizard_title().is_empty());
            assert!(!lang.wizard_step_language().is_empty());
            assert!(!lang.wizard_step_mode().is_empty());
            assert!(!lang.wizard_step_account().is_empty());
            assert!(!lang.wizard_next().is_empty());
            assert!(!lang.wizard_back().is_empty());
            assert!(!lang.wizard_ms_title().is_empty());
            assert!(!lang.wizard_ms_desc().is_empty());
            assert!(!lang.wizard_offline_title().is_empty());
            assert!(!lang.wizard_offline_hint().is_empty());
            assert!(!lang.wizard_offline_invalid().is_empty());
            assert!(!lang.wizard_wetid_title().is_empty());
            assert!(!lang.wizard_wetid_soon().is_empty());
            assert!(!lang.wizard_hw_title().is_empty());
            assert!(!lang.wizard_hw_body().is_empty());
            assert!(!lang.wizard_hw_specs("8 GB", 8, "100 GB").is_empty());
            assert!(!lang.wizard_hw_ok().is_empty());
            assert!(!lang.wizard_hw_sodium().is_empty());
            assert!(!lang.wizard_hw_no_instance().is_empty());
            assert!(!lang.wizard_hw_bad_loader().is_empty());
            assert!(!lang.status_resolving_loader("Forge", "1.20.1").is_empty());
            assert!(!lang.err_loader_failed("Forge", "x").is_empty());
            assert!(!lang.disk_size(500).is_empty());
            assert!(!lang.err_low_disk("1 GB", "1 MB").is_empty());
            assert!(!lang.status_download_cancelled().is_empty());
            assert!(!lang.status_download_paused().is_empty());
            assert!(!lang.overlay_title().is_empty());
            assert!(!lang.overlay_file("x.jar").is_empty());
            assert!(!lang.overlay_btn_cancel().is_empty());
            assert!(!lang.overlay_btn_pause().is_empty());
            assert!(!lang.overlay_btn_resume().is_empty());
            assert!(!lang.settings_speed_title().is_empty());
            assert!(!lang.settings_speed_unlimited().is_empty());
            assert!(!lang.tmpl_title().is_empty());
            assert!(!lang.tmpl_name("empty").is_empty());
            assert!(!lang.tmpl_desc("optimized").is_empty());
            assert!(!lang.tmpl_custom_note().is_empty());
            assert!(!lang.settings_game_args_title().is_empty());
            assert!(!lang.settings_java_found_title().is_empty());
            assert!(!lang.settings_java_scanning().is_empty());
            assert!(!lang.settings_java_none_found().is_empty());
            assert!(!lang.settings_btn_use_java().is_empty());
            assert!(!lang.settings_runtimes_title().is_empty());
            assert!(!lang.sess_title().is_empty());
            assert!(!lang.inst_sort_title().is_empty());
            assert!(!lang.inst_sort_name().is_empty());
            assert!(!lang.inst_sort_recent().is_empty());
            assert!(!lang.inst_sort_created().is_empty());
            assert!(!lang.inst_sort_played().is_empty());
            assert!(!lang.inst_tag_all().is_empty());
            assert!(!lang.inst_icon_title().is_empty());
            assert!(!lang.inst_icon_custom().is_empty());
            assert!(!lang.inst_icon_none().is_empty());
            assert!(!lang.inst_icon_remove().is_empty());
            assert!(!lang.inst_tags_label().is_empty());
            assert!(!lang.inst_tags_hint().is_empty());
            assert!(!lang.inst_pin_label().is_empty());
            assert!(!lang.inst_account_label().is_empty());
            assert!(!lang.inst_account_default().is_empty());
            assert!(!lang.discover_title().is_empty());
            assert!(!lang.discover_refresh().is_empty());
            assert!(!lang.discover_install().is_empty());
            assert!(!lang.discover_no_instance().is_empty());
            assert!(!lang.tour_home_title().is_empty());
            assert!(!lang.tour_home_body().is_empty());
            assert!(!lang.tour_instances_title().is_empty());
            assert!(!lang.tour_instances_body().is_empty());
            assert!(!lang.tour_dismiss().is_empty());
            assert!(!lang.mcreuse_title().is_empty());
            assert!(!lang.mcreuse_body("/x").is_empty());
            assert!(!lang.mcreuse_use().is_empty());
            assert!(!lang.mcreuse_skip().is_empty());
            assert!(!lang.mcreuse_copied(10, 20).is_empty());
            assert!(!lang.inst_notes_label().is_empty());
            assert!(!lang.inst_notes_hint().is_empty());
            assert!(!lang.inst_pre_label().is_empty());
            assert!(!lang.inst_post_label().is_empty());
            assert!(!lang.settings_root_title().is_empty());
            assert!(!lang.settings_root_pick().is_empty());
            assert!(!lang.settings_root_restart().is_empty());
            assert!(!lang.editor_title().is_empty());
            assert!(!lang.editor_empty().is_empty());
            assert!(!lang.editor_saved().is_empty());
            assert!(!lang.health_title().is_empty());
            assert!(!lang.health_ok().is_empty());
            assert!(!lang.health_missing(3).is_empty());
            assert!(!lang.health_repair().is_empty());
            assert!(!lang.health_verify().is_empty());
            assert!(!lang.trash_title().is_empty());
            assert!(!lang.trash_restore().is_empty());
            assert!(!lang.trash_delete_forever().is_empty());
            assert!(!lang.trash_empty().is_empty());
            assert!(!lang.trash_empty_all().is_empty());
            assert!(!lang.fav_title().is_empty());
            assert!(!lang.fav_address_hint().is_empty());
            assert!(!lang.fav_add().is_empty());
            assert!(!lang.fav_remove().is_empty());
            assert!(!lang.fav_empty().is_empty());
            assert!(!lang.fav_name_hint().is_empty());
            assert!(!lang.fav_refresh().is_empty());
            assert!(!lang.fav_invalid().is_empty());
            assert!(!lang.worlds_title().is_empty());
            assert!(!lang.world_backup_now().is_empty());
            assert!(!lang.world_backups().is_empty());
            assert!(!lang.world_restore().is_empty());
            assert!(!lang.world_no_saves().is_empty());
            assert!(!lang.world_no_backups().is_empty());
            assert!(!lang.world_auto_label().is_empty());
            assert!(!lang.world_delete().is_empty());
            assert!(!lang.world_pick_folder().is_empty());
            assert!(!lang.screens_title().is_empty());
            assert!(!lang.screens_empty().is_empty());
            assert!(!lang.screens_open().is_empty());
            assert!(!lang.notif_title().is_empty());
            assert!(!lang.notif_updates().is_empty());
            assert!(!lang.notif_downloads().is_empty());
            assert!(!lang.notif_launcher().is_empty());
            assert!(!lang.update_title().is_empty());
            assert!(!lang.update_check().is_empty());
            assert!(!lang.update_available("1.0").is_empty());
            assert!(!lang.update_latest().is_empty());
            assert!(!lang.crash_title().is_empty());
            assert!(!lang.crash_enable().is_empty());
            assert!(!lang.crash_export().is_empty());
            assert!(!lang.transfer_title().is_empty());
            assert!(!lang.transfer_desc().is_empty());
            assert!(!lang.transfer_export().is_empty());
            assert!(!lang.transfer_import().is_empty());
            assert!(!lang.editor_filter_hint().is_empty());
            assert!(!lang.editor_find_hint().is_empty());
            assert!(!lang.editor_matches(2).is_empty());
            assert!(!lang.inst_group_label().is_empty());
            assert!(!lang.inst_group_hint().is_empty());
            assert!(!lang.inst_res_custom().is_empty());
            assert!(!lang.inst_res_mode().is_empty());
            assert!(!lang.inst_res_default().is_empty());
            assert!(!lang.inst_res_windowed().is_empty());
            assert!(!lang.inst_res_fullscreen().is_empty());
            assert!(!lang.group_title().is_empty());
            assert!(!lang.group_desc().is_empty());
            assert!(!lang.group_empty().is_empty());
            assert!(!lang.group_ram().is_empty());
            assert!(!lang.group_java_auto().is_empty());
            assert!(!lang.group_java_select().is_empty());
            assert!(!lang.group_name_hint().is_empty());
            assert!(!lang.group_add().is_empty());
            assert!(!lang.hotkey_title().is_empty());
            assert!(!lang.hotkey_launch().is_empty());
            assert!(!lang.hotkey_tabs().is_empty());
            assert!(!lang.loader_pinned("0.15.11").is_empty());
            assert!(!lang.loader_pin_reset().is_empty());
            assert!(!lang.dlman_title().is_empty());
            assert!(!lang.dlman_empty().is_empty());
            assert!(!lang.dlman_clear().is_empty());
            assert!(!lang.speed_value(1024.0).is_empty());
            assert!(!lang.eta_text(90).is_empty());
            assert!(!lang.server_offline().is_empty());
            assert!(!lang.status_loader_profile("Fabric", "0.15.11").is_empty());
            assert!(!lang.err_version_failed("1.20.1", "x").is_empty());
            assert!(!lang.err_java_failed("17", "x").is_empty());
            assert!(!lang.err_components_failed("x").is_empty());
            assert!(!lang.err_launch_failed("x").is_empty());
            assert!(!lang.err_mod_file_failed("m.jar", "x").is_empty());
            assert!(!lang.err_version_missing("1.20.1").is_empty());
            assert!(!lang.clone_title().is_empty());
            assert!(!lang.clone_saves().is_empty());
            assert!(!lang.inst_btn_shortcut().is_empty());
            assert!(!lang.archive_title().is_empty());
            assert!(!lang.archive_desc().is_empty());
            assert!(!lang.archive_create().is_empty());
            assert!(!lang.server_create().is_empty());
            assert!(!lang.server_stop().is_empty());
            assert!(!lang.server_running(25565).is_empty());
            assert!(!lang.server_eula().is_empty());
            assert!(!lang.picker_title().is_empty());
            assert!(!lang.picker_remember().is_empty());
            assert!(!lang.settings_ask_identity().is_empty());
            assert!(!lang.settings_forget_identity().is_empty());
            assert!(!lang.profile_title().is_empty());
            assert!(!lang.profile_nickname().is_empty());
            assert!(!lang.profile_cape_none().is_empty());
            assert!(!lang.modrinth_profile_status_signed_in().is_empty());
            assert!(!lang.modrinth_profile_status_offline().is_empty());
            assert!(!lang.modrinth_profile_status_signed_out().is_empty());
            assert!(!lang.modrinth_profile_details().is_empty());
            assert!(!lang.modrinth_profile_player_uuid().is_empty());
            assert!(!lang.modrinth_profile_skin_model().is_empty());
            assert!(!lang.modrinth_profile_change_skin().is_empty());
            assert!(!lang.modrinth_profile_manage_account().is_empty());
            assert!(!lang.modrinth_profile_copy_uuid().is_empty());
            assert!(!lang.modrinth_profile_copied().is_empty());
            assert!(!lang.modrinth_profile_no_uuid().is_empty());
            assert!(!lang.compat_ok("1.20", "Fabric").is_empty());
            assert!(!lang.compat_unknown().is_empty());
            assert!(!lang.compat_none("1.20").is_empty());
            assert!(!lang.wishlist_add().is_empty());
            assert!(!lang.wishlist_remove().is_empty());
            assert!(!lang.batch_select_all().is_empty());
            assert!(!lang.batch_enable().is_empty());
            assert!(!lang.batch_disable().is_empty());
            assert!(!lang.batch_delete().is_empty());
            assert!(!lang.mods_update_available().is_empty());
            assert!(!lang.mods_check_updates().is_empty());
            assert!(!lang.mods_update_all().is_empty());
            assert!(!lang.mods_rollback().is_empty());
            assert!(!lang.mods_pin().is_empty());
            assert!(!lang.mods_unpin().is_empty());
            assert!(!lang.mods_sort_relevance().is_empty());
            assert!(!lang.mods_sort_downloads().is_empty());
            assert!(!lang.mods_install_optifine().is_empty());
            assert!(!lang.mods_optifine_need_instance().is_empty());
            assert!(!lang.toast_mod_installed("x").is_empty());
            assert!(!lang.err_mod_copy_failed("x", "y").is_empty());
            assert!(!lang.toast_world_imported("x").is_empty());
            assert!(!lang.err_world_import_failed("y").is_empty());
            assert!(!lang.err_mod_resolve_failed("y").is_empty());
            assert!(!lang.console_started(1).is_empty());
            assert!(!lang.nav_profile().is_empty());
            assert!(!lang.tour_home_title().is_empty());
            assert!(!lang.tour_home_body().is_empty());
            assert!(!lang.tour_instances_title().is_empty());
            assert!(!lang.tour_instances_body().is_empty());
            assert!(!lang.tour_dismiss().is_empty());
            assert!(!lang.mcreuse_title().is_empty());
            assert!(!lang.mcreuse_body("/x").is_empty());
            assert!(!lang.mcreuse_use().is_empty());
            assert!(!lang.mcreuse_skip().is_empty());
            assert!(!lang.mcreuse_copied(10, 20).is_empty());
            assert!(!lang.inst_notes_label().is_empty());
            assert!(!lang.inst_notes_hint().is_empty());
            assert!(!lang.inst_pre_label().is_empty());
            assert!(!lang.inst_post_label().is_empty());
            assert!(!lang.settings_root_title().is_empty());
            assert!(!lang.settings_root_pick().is_empty());
            assert!(!lang.settings_root_restart().is_empty());
            assert!(!lang.editor_title().is_empty());
            assert!(!lang.editor_empty().is_empty());
            assert!(!lang.editor_saved().is_empty());
            assert!(!lang.health_title().is_empty());
            assert!(!lang.health_ok().is_empty());
            assert!(!lang.health_missing(3).is_empty());
            assert!(!lang.health_repair().is_empty());
            assert!(!lang.health_verify().is_empty());
            assert!(!lang.trash_title().is_empty());
            assert!(!lang.trash_restore().is_empty());
            assert!(!lang.trash_delete_forever().is_empty());
            assert!(!lang.trash_empty().is_empty());
            assert!(!lang.trash_empty_all().is_empty());
            assert!(!lang.fav_title().is_empty());
            assert!(!lang.fav_address_hint().is_empty());
            assert!(!lang.fav_add().is_empty());
            assert!(!lang.fav_remove().is_empty());
            assert!(!lang.fav_empty().is_empty());
            assert!(!lang.worlds_title().is_empty());
            assert!(!lang.world_backup_now().is_empty());
            assert!(!lang.world_backups().is_empty());
            assert!(!lang.world_restore().is_empty());
            assert!(!lang.world_no_saves().is_empty());
            assert!(!lang.world_no_backups().is_empty());
            assert!(!lang.world_auto_label().is_empty());
            assert!(!lang.world_delete().is_empty());
            assert!(!lang.world_pick_folder().is_empty());
            assert!(!lang.inst_warn_legacy_forge().is_empty());
            assert!(!lang.settings_after_launch_title().is_empty());
            assert!(!lang.settings_after_close().is_empty());
            assert!(!lang.settings_after_minimize().is_empty());
            assert!(!lang.settings_after_keep().is_empty());
            assert!(!lang.settings_autostart().is_empty());
            assert!(!lang.bottom_account_ms().is_empty());
            assert!(!lang.bottom_account_wetid().is_empty());
            assert!(!lang.bottom_account_offline().is_empty());
            assert!(!lang.login_offline_placeholder().is_empty());
            assert!(!lang.login_wetid_hint().is_empty());
            assert!(!lang.login_ms_prompt().is_empty());
            assert!(!lang.login_ms_hint().is_empty());
            assert!(!lang.err_nickname_invalid().is_empty());
            assert!(!lang.skins_account_label().is_empty());
            assert!(!lang.skins_custom_name().is_empty());
            assert!(!lang.inst_copy_name("x").is_empty());
            assert!(!lang.inst_cmd_hint().is_empty());
            assert!(!lang.err_backup_failed("e").is_empty());
            assert!(!lang.err_delete_failed("e").is_empty());
            assert!(!lang.err_restore_failed("e").is_empty());
            assert!(!lang.err_save_failed("e").is_empty());
            assert!(!lang.err_archive_failed("e").is_empty());
            assert!(!lang.err_transfer_export_failed("e").is_empty());
            assert!(!lang.err_transfer_import_failed("e").is_empty());
            assert!(!lang.err_shortcut_failed("e").is_empty());
            assert!(!lang.err_server_eula_required().is_empty());
            assert!(!lang.err_pack_no_files().is_empty());
            assert!(!lang.err_mrpack_bad("e").is_empty());
            assert!(!lang.err_cursepack_bad("e").is_empty());
            assert!(!lang.err_optifine_no_build("1.20").is_empty());
            assert!(!lang.account_tag_ms().is_empty());
            assert!(!lang.account_tag_wetid().is_empty());
            assert!(!lang.account_tag_offline().is_empty());
            assert!(!lang.home_hero_kicker().is_empty());
            assert!(!lang.settings_speed_unit().is_empty());
            assert!(!lang.hotkey_tabs_suffix().is_empty());
            assert!(!lang.profile_uuid_label("x").is_empty());
        }
    }
}
