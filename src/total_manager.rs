use crate::link_manager;
use crate::mods_manager;

struct Manager {
    scanner: mods_manager::mods_scanner::ModScanner,
    database_manager: mods_manager::mods_info_storage::ModManagerDb,
    link_manager: link_manager::LinkManager,
}
