use async_trait::async_trait;
use jd_core::{AppState, base};

use crate::{
    ZkPersonaUserDmc,
    domain::{
        AuthUser, UserRepository, 
        ZkPersonaUser, ZkPersonaUserForCreate, ZkPersonaUserForUpdate, ZkPersonaUserFilter
    },
    error::{Error, Result},
};

pub struct ZkPersonaUserRepositoryImpl {
  state: AppState,
}

impl ZkPersonaUserRepositoryImpl {
  pub fn new(state: AppState) -> Self {
    Self { state }
  }
}

#[async_trait]
impl UserRepository for ZkPersonaUserRepositoryImpl {
  async fn create_user(&self, user: &AuthUser) -> Result<()> {
    let create_req = ZkPersonaUserForCreate::from(user);
    
    base::rest::create::<ZkPersonaUserDmc, _, ZkPersonaUser>(&self.state.mm(), create_req)
      .await
      .map_err(|e| Error::database_error(&e.to_string()))?;

    Ok(())
  }

  async fn get_user(&self, address: &str) -> Result<Option<AuthUser>> {
    let filter = ZkPersonaUserFilter {
      wallet_address: Some(address.into()),
      id: None,
    };

    match base::rest::first::<ZkPersonaUserDmc, _, ZkPersonaUser>(&self.state.mm(), Some(filter), None)
      .await
    {
      Ok(Some(zk_user)) => Ok(Some(AuthUser::from(zk_user))),
      Ok(None) => Ok(None),
      Err(e) => Err(Error::database_error(&e.to_string())),
    }
  }

  async fn update_user(&self, user: &AuthUser) -> Result<()> {
    let filter = ZkPersonaUserFilter {
      wallet_address: Some(user.address.clone().into()),
      id: None,
    };

    let update_req = ZkPersonaUserForUpdate {
      public_key: Some(user.public_key.clone()),
      last_login: Some(user.last_login),
      login_count: Some(user.login_count),
    };

    let updated_count = base::rest::update_by_filter::<ZkPersonaUserDmc, _, _>(
      &self.state.mm(),
      filter,
      update_req,
    )
    .await
    .map_err(|e| Error::database_error(&e.to_string()))?;

    if updated_count == 0 {
      return Err(Error::database_error("User not found for update"));
    }

    Ok(())
  }
}