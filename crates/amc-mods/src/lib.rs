pub mod curseforge;
pub mod local;
pub mod modrinth;
pub mod types;

pub use curseforge::CurseForgeClient;
pub use local::LocalModManager;
pub use modrinth::ModrinthClient;
pub use types::{LocalMod, ModDownloadFile, ModSearchResult, ModSource};
