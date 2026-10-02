pub mod home;
pub mod instance_manage;
pub mod instances;
pub mod mods;
pub mod profile;
pub mod settings;
pub mod skins;

pub use home::{HomeContext, HomePage};
pub use instance_manage::InstanceManage;
pub use instances::{InstanceAction, InstancesContext, InstancesPage};
pub use mods::{ModsPage, UpdateOffer};
pub use profile::{ProfileAction, ProfilePage};
pub use settings::SettingsPage;
pub use skins::SkinsPage;
