use candid::Principal;
use ic_cdk_macros::{update, query, init};
use ic_cdk::api::caller;
use std::cell::RefCell;
use ic_stable_structures::{
    StableCell,
    StableBTreeMap, 
    memory_manager::{MemoryId, MemoryManager, VirtualMemory},
    DefaultMemoryImpl,
};
use ic_canister_log::log;
use crate::logs::{DEBUG, INFO};

pub type UserId = String;

const MASTER_PRINCIPAL_MEMORY_ID: MemoryId = MemoryId::new(0);
const USER_DB_MEMORY_ID: MemoryId = MemoryId::new(1);

thread_local! {
    static MEMORY_MANAGER: RefCell<MemoryManager<DefaultMemoryImpl>> = RefCell::new(MemoryManager::init(DefaultMemoryImpl::default()));
    static USER_DB: RefCell<StableBTreeMap<UserId, Principal, VirtualMemory<DefaultMemoryImpl>>> = RefCell::new(
        StableBTreeMap::init(
            MEMORY_MANAGER.with(|m| m.borrow().get(USER_DB_MEMORY_ID)),
        )
    );
    static MASTER_PRINCIPAL: RefCell<StableCell<Option<Principal>, VirtualMemory<DefaultMemoryImpl>>> = RefCell::new(
        StableCell::init(
            MEMORY_MANAGER.with(|m| m.borrow().get(MASTER_PRINCIPAL_MEMORY_ID)),
            None,
        ).expect("Failed to initialize StableCell")
    );
}

#[init]
pub fn init(master_principal: Option<Principal>) {
    MASTER_PRINCIPAL.with(|mp| mp.borrow_mut().set(master_principal).expect("Failed to set master principal"));
}

fn ensure_master_principal() {
    let caller = caller();
    let is_master = MASTER_PRINCIPAL.with(|mp| *mp.borrow().get() == Some(caller));

    if !is_master {
        ic_cdk::trap("Unauthorized: Only the master wallet can perform this action.");
    }
}

#[update]
pub fn add_user(user_id: UserId, principal: Principal) -> Result<String, String> {
    ensure_master_principal();

    USER_DB.with(|user_db| {
        let mut db = user_db.borrow_mut();

        if db.contains_key(&user_id) {
            log!(DEBUG, "User already exists, user_id: {:?}, principal: {:?}", user_id, principal.to_text());
            return Err("User already exists".to_string());
        }

        if db.values().any(|p| p == principal) {
            log!(INFO, "Principal already associated with a user, user_id: {:?}, principal: {:?}", user_id, principal.to_text());
            return Err("Principal already associated with a user.".to_string());
        }

        db.insert(user_id.clone(), principal);
        log!(DEBUG, "User added successfully, user_id: {:?}, principal: {:?} ", user_id, principal.to_text());

        Ok("User added successfully".to_string())
    })
}

#[update]
pub fn remove_user_by_userid(user_id: UserId) -> Result<String, String> {
    ensure_master_principal();

    USER_DB.with(|user_db| {
        let mut db = user_db.borrow_mut();

        if db.remove(&user_id).is_none() {
            return Err("User ID not found".to_string());
        }
        Ok(())
    })?;
    log!(DEBUG, "User {:?} deleted successfully", user_id);
    Ok("User {user_id:?} deleted successfully".to_string())
}

#[query]
pub fn get_principal_by_userid(user_id: UserId) -> Option<Principal> {
    ensure_master_principal();

    USER_DB.with(|user_db| {
        user_db.borrow().get(&user_id)
    })
}

#[query]
pub fn get_userid_by_principal(principal: Principal) -> Option<UserId> {
    ensure_master_principal();

    USER_DB.with(|user_db| {
        user_db.borrow().iter().find_map(|(user_id, p)| if p == principal { Some(user_id.clone()) } else { None })
    })
}

#[query]
pub fn get_all_userids() -> Vec<UserId> {
    ensure_master_principal();

    USER_DB.with(|user_db| {
        user_db.borrow().keys().map(|key| key.clone()).collect::<Vec<UserId>>()
    })
}

#[query]
pub fn get_userids_by_principals(principals: Vec<Principal>) -> Vec<Option<UserId>> {
    ensure_master_principal();

    USER_DB.with(|user_db| {
        principals.iter().map(|principal| {
            user_db.borrow().iter().find_map(|(user_id, p)| {
                if p == *principal {
                    Some(user_id.clone())
                } else {
                    None
                }
            })
        }).collect()
    })
}

#[update]
pub fn change_master_principal(new_master_principal: Principal) {
    ensure_master_principal();

    MASTER_PRINCIPAL.with(|mp| {
        mp.borrow_mut().set(Some(new_master_principal)).expect("Failed to set new master principal");
    });
}

