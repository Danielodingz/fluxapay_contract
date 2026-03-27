use soroban_sdk::{contract, contracterror, contractimpl, contracttype, Address, Env, String};

#[contract]
pub struct MerchantRegistry;

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Merchant {
    pub merchant_id: Address,
    pub business_name: String,
    pub settlement_currency: String,
    pub verified: bool,
    pub active: bool,
    pub created_at: u64,
}

#[contracttype]
pub enum MerchantDataKey {
    Merchant(Address),
    Admin,
}

#[contracterror]
#[derive(Copy, Clone, Debug, Eq, PartialEq, PartialOrd, Ord)]
#[repr(u32)]
pub enum MerchantError {
    MerchantAlreadyExists = 1,
    MerchantNotFound = 2,
    Unauthorized = 3,
    NotVerified = 4,
    AdminAlreadySet = 5,
}

#[contractimpl]
impl MerchantRegistry {
    /// Initialize the contract with an admin address
    pub fn merchant_initialize(env: Env, admin: Address) -> Result<(), MerchantError> {
        if env.storage().persistent().has(&MerchantDataKey::Admin) {
            return Err(MerchantError::AdminAlreadySet);
        }
        env.storage().persistent().set(&MerchantDataKey::Admin, &admin);
        Ok(())
    }

    /// Register a new merchant
    pub fn register_merchant(
        env: Env,
        merchant_id: Address,
        business_name: String,
        settlement_currency: String,
    ) -> Result<(), MerchantError> {
        merchant_id.require_auth();

        if env
            .storage()
            .persistent()
            .has(&MerchantDataKey::Merchant(merchant_id.clone()))
        {
            return Err(MerchantError::MerchantAlreadyExists);
        }

        let merchant = Merchant {
            merchant_id: merchant_id.clone(),
            business_name,
            settlement_currency,
            verified: false,
            active: true,
            created_at: env.ledger().timestamp(),
        };

        env.storage()
            .persistent()
            .set(&MerchantDataKey::Merchant(merchant_id), &merchant);

        Ok(())
    }

    /// Update merchant settings
    pub fn update_merchant(
        env: Env,
        merchant_id: Address,
        business_name: Option<String>,
        settlement_currency: Option<String>,
        active: Option<bool>,
    ) -> Result<(), MerchantError> {
        merchant_id.require_auth();

        let mut merchant = Self::get_merchant_internal(&env, &merchant_id)?;

        if let Some(name) = business_name {
            merchant.business_name = name;
        }
        if let Some(currency) = settlement_currency {
            merchant.settlement_currency = currency;
        }
        if let Some(is_active) = active {
            merchant.active = is_active;
        }

        env.storage()
            .persistent()
            .set(&MerchantDataKey::Merchant(merchant_id), &merchant);

        Ok(())
    }

    /// Get merchant info
    pub fn get_merchant(env: Env, merchant_id: Address) -> Result<Merchant, MerchantError> {
        Self::get_merchant_internal(&env, &merchant_id)
    }

    /// Verify merchant (admin only)
    pub fn verify_merchant(env: Env, admin: Address, merchant_id: Address) -> Result<(), MerchantError> {
        admin.require_auth();

        let stored_admin: Address = env
            .storage()
            .persistent()
            .get(&MerchantDataKey::Admin)
            .ok_or(MerchantError::Unauthorized)?;

        if admin != stored_admin {
            return Err(MerchantError::Unauthorized);
        }

        let mut merchant = Self::get_merchant_internal(&env, &merchant_id)?;
        merchant.verified = true;

        env.storage()
            .persistent()
            .set(&MerchantDataKey::Merchant(merchant_id), &merchant);

        Ok(())
    }

    // Helper functions
    fn get_merchant_internal(env: &Env, merchant_id: &Address) -> Result<Merchant, MerchantError> {
        env.storage()
            .persistent()
            .get(&MerchantDataKey::Merchant(merchant_id.clone()))
            .ok_or(MerchantError::MerchantNotFound)
    }
}
