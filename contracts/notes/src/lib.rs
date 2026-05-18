#![no_std] 

use soroban_sdk::{contract, contractimpl, contracttype, Address, Env, String, Vec};

// Definisi Tipe Data
#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum AchievementType {
    Certificate,
    Skill,
    Competition,
    Organization,
    Volunteer,
}

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Achievement {
    pub id: u64,
    pub achievement_type: AchievementType,
    pub title: String,
    pub issuer: String,
    pub description: String,
    pub timestamp: u64,
    pub is_verified: bool, // HR / Admin verifikasi 
}

#[contracttype]
pub enum DataKey {
    Admin,
    NextId,
    Passport(Address),
    AchievementById(u64),
}

// Deklarasi Contract
#[contract]
pub struct SkillPassportContract;

// Implementasi Logika CRUD
#[contractimpl]
impl SkillPassportContract {
    /// Inisialisasi Admin (Kampus/Platform)
    pub fn initialize(env: Env, admin: Address) {
        if env.storage().instance().has(&DataKey::Admin) {
            panic!("Already initialized");
        }
        env.storage().instance().set(&DataKey::Admin, &admin);
        env.storage().instance().set(&DataKey::NextId, &1u64);
    }

    /// CREATE: Tambah Achievement ke Passport Mahasiswa
    pub fn add_achievement(
        env: Env,
        mahasiswa: Address,
        achievement_type: AchievementType,
        title: String,
        issuer: String,
        description: String,
        timestamp: u64,
    ) -> u64 {
        mahasiswa.require_auth();

        let mut id: u64 = env.storage().instance().get(&DataKey::NextId).unwrap_or(1);

        let achievement = Achievement {
            id,
            achievement_type,
            title,
            issuer,
            description,
            timestamp,
            is_verified: false,
        };

        // Simpan ke Passport Mahasiswa
        let mut passport: Vec<Achievement> = env.storage().persistent().get(&DataKey::Passport(mahasiswa.clone())).unwrap_or(Vec::new(&env));
        passport.push_back(achievement.clone());
        env.storage().persistent().set(&DataKey::Passport(mahasiswa), &passport);

        // Simpan secara global untuk memudahkan pencarian by ID
        env.storage().persistent().set(&DataKey::AchievementById(id), &achievement);

        // Increment ID untuk data selanjutnya
        env.storage().instance().set(&DataKey::NextId, &(id + 1));

        id
    }

    /// READ: Ambil seluruh isi Passport
    pub fn get_passport(env: Env, mahasiswa: Address) -> Vec<Achievement> {
        env.storage().persistent().get(&DataKey::Passport(mahasiswa)).unwrap_or(Vec::new(&env))
    }

    /// READ: Ambil spesifik data by ID
    pub fn get_achievement_by_id(env: Env, id: u64) -> Option<Achievement> {
        env.storage().persistent().get(&DataKey::AchievementById(id))
    }

    /// UPDATE: Edit data (Hanya jika belum diverifikasi)
    pub fn update_achievement(
        env: Env,
        mahasiswa: Address,
        id: u64,
        title: String,
        issuer: String,
        description: String,
        timestamp: u64,
    ) -> bool {
        mahasiswa.require_auth();

        let mut achievement: Achievement = env.storage().persistent().get(&DataKey::AchievementById(id)).unwrap();
        
        if achievement.is_verified {
            panic!("Cannot edit verified achievement"); 
        }

        achievement.title = title;
        achievement.issuer = issuer;
        achievement.description = description;
        achievement.timestamp = timestamp;

        // Update data global
        env.storage().persistent().set(&DataKey::AchievementById(id), &achievement);

        // Update di Passport Mahasiswa
        let mut passport: Vec<Achievement> = env.storage().persistent().get(&DataKey::Passport(mahasiswa.clone())).unwrap();
        for i in 0..passport.len() {
            if passport.get(i).unwrap().id == id {
                passport.set(i, achievement.clone());
                break;
            }
        }
        env.storage().persistent().set(&DataKey::Passport(mahasiswa), &passport);

        true
    }

    /// DELETE: Hapus data (jika salah input)
    pub fn delete_achievement(env: Env, mahasiswa: Address, id: u64) -> bool {
        mahasiswa.require_auth();

        let achievement: Achievement = env.storage().persistent().get(&DataKey::AchievementById(id)).unwrap();
        if achievement.is_verified {
            panic!("Cannot delete verified achievement");
        }

        // Hapus dari data global
        env.storage().persistent().remove(&DataKey::AchievementById(id));

        // Hapus dari Passport Mahasiswa
        let mut passport: Vec<Achievement> = env.storage().persistent().get(&DataKey::Passport(mahasiswa.clone())).unwrap();
        let mut new_passport = Vec::new(&env);
        
        for item in passport.into_iter() {
            if item.id != id {
                new_passport.push_back(item);
            }
        }
        env.storage().persistent().set(&DataKey::Passport(mahasiswa), &new_passport);

        true
    }

    /// VERIFY: Fitur khusus untuk Admin / Kampus
    pub fn verify_achievement(env: Env, admin: Address, id: u64) -> bool {
        admin.require_auth();
        
        let stored_admin: Address = env.storage().instance().get(&DataKey::Admin).unwrap();
        if admin != stored_admin {
            panic!("Only admin can verify");
        }

        let mut achievement: Achievement = env.storage().persistent().get(&DataKey::AchievementById(id)).unwrap();
        achievement.is_verified = true;

        // Update Global
        env.storage().persistent().set(&DataKey::AchievementById(id), &achievement);

        true
    }
}