use candid::Principal;
use ic_cdk_macros::{update, query, init};
use ic_cdk::api::caller;
use std::cell::RefCell;
use ic_stable_structures::{
    StableBTreeMap, 
    memory_manager::{MemoryId, MemoryManager, VirtualMemory},
    DefaultMemoryImpl,
};

pub type CurveId = [u8; 12];
pub type UUID = String;

const USER_DB_MEMORY_ID: MemoryId = MemoryId::new(0);

thread_local! {
    static MEMORY_MANAGER: RefCell<MemoryManager<DefaultMemoryImpl>> = RefCell::new(MemoryManager::init(DefaultMemoryImpl::default()));
    static USER_DB: RefCell<StableBTreeMap<CurveId, UUID, VirtualMemory<DefaultMemoryImpl>>> = MEMORY_MANAGER
        .with(|m| 
            RefCell::new(
                StableBTreeMap::init(
                    m.borrow().get(USER_DB_MEMORY_ID)
                )
            )
        );

    static MASTER_PRINCIPAL: RefCell<Option<Principal>> = RefCell::new(None);
}

#[init]
pub fn init(master_principal: Principal) {
    MASTER_PRINCIPAL.with(|mw| *mw.borrow_mut() = Some(master_principal));
}

fn ensure_master_principal() {
    let caller = caller();
    let is_master = MASTER_PRINCIPAL.with(|mw| *mw.borrow() == Some(caller));

    if !is_master {
        ic_cdk::trap("Unauthorized: Only the master wallet can perform this action.");
    }
}

#[update]
pub fn add_user(uuid: UUID) {
    ensure_master_principal();

    USER_DB.with(|user_db| {
        let mut db = user_db.borrow_mut();

        let new_id = if db.is_empty() {
            [0u8; 12]
        } else {
            let last_id = db.keys().max().unwrap();
            let mut new_id = last_id;
            for i in (0..12).rev() {
                if new_id[i] < 255 {
                    new_id[i] += 1;
                    break;
                } else {
                    new_id[i] = 0;
                }
            }
            new_id
        };

        if db.contains_key(&new_id) || db.values().any(|v| v == uuid) {
            ic_cdk::trap("User ID or UUID already exists.");
        }

        db.insert(new_id.clone(), uuid);
    });
}

#[update]
pub fn remove_user_by_curveid(id: CurveId) {
    ensure_master_principal();

    USER_DB.with(|user_db| {
        let mut db = user_db.borrow_mut();
        if db.remove(&id).is_none() {
            ic_cdk::trap("User ID not found.");
        }
    });
}

#[update]
pub fn remove_user_by_uuid(uuid: UUID) {
    ensure_master_principal();

    USER_DB.with(|user_db| {
        let mut db = user_db.borrow_mut();

        if let Some(id) = db.iter().find_map(|(k, v)| if *v == uuid { Some(k.clone()) } else { None }) {
            db.remove(&id);
        } else {
            ic_cdk::trap("UUID not found.");
        }
    });
}

#[query]
pub fn get_curveid_by_uuid(uuid: UUID) -> Option<CurveId> {
    ensure_master_principal();
    
    USER_DB.with(|user_db| {
        user_db.borrow().iter().find_map(|(id, u)| if *u == uuid { Some(id.clone()) } else { None })
    })
}

#[query]
pub fn get_uuid_by_curveid(id: CurveId) -> Option<UUID> {
    ensure_master_principal();

    USER_DB.with(|user_db| {
        user_db.borrow().get(&id).map(|s| s.clone())
    })
}

#[query]
pub fn get_all_curveids() -> Vec<CurveId> {
    ensure_master_principal();

    USER_DB.with(|user_db| {
        user_db.borrow().keys().collect()
    })
}

