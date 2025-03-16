use candid::Principal;
use ic_cdk_macros::{update, query, init};
use ic_cdk::api::caller;
use std::cell::RefCell;
use ic_stable_structures::{
    StableBTreeMap, 
    memory_manager::{MemoryId, MemoryManager, VirtualMemory},
    DefaultMemoryImpl,
};

pub type UserId = String;

const USER_DB_MEMORY_ID: MemoryId = MemoryId::new(0);

thread_local! {
    static MEMORY_MANAGER: RefCell<MemoryManager<DefaultMemoryImpl>> = RefCell::new(MemoryManager::init(DefaultMemoryImpl::default()));
    static USER_DB: RefCell<StableBTreeMap<UserId, Principal, VirtualMemory<DefaultMemoryImpl>>> = MEMORY_MANAGER
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
pub fn add_user(user_id: UserId, principal: Principal) -> Result<String, String> {
    ensure_master_principal();

    USER_DB.with(|user_db| {
        let mut db = user_db.borrow_mut();

        if db.contains_key(&user_id) {
            return Err("User already exists".to_string());
        }

        if db.values().any(|p| p == principal) {
            return Err("Principal already associated with a user.".to_string());
        }

        db.insert(user_id.clone(), principal);
        ic_cdk::println!("user_id: {}, principal: {}", user_id, principal); //TODO comment

        Ok("User added successfully".to_string())
    })
}

#[update]
pub fn remove_user_by_userid(user_id: UserId) {
    ensure_master_principal();

    USER_DB.with(|user_db| {
        let mut db = user_db.borrow_mut();

        if db.remove(&user_id).is_none() {
            ic_cdk::trap("User ID not found.");
        }
    });
}

#[query]
pub fn get_principal_by_userid(user_id: UserId) -> Option<Principal> {
    ensure_master_principal();

    USER_DB.with(|user_db| {
        user_db.borrow().get(&user_id).map(|s| s.clone())
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

    MASTER_PRINCIPAL.with(|mw| {
        *mw.borrow_mut() = Some(new_master_principal);
    });
}

