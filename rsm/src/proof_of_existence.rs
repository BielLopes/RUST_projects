use core::fmt::Debug;
use std::collections::BTreeMap;

use thiserror::Error;

#[derive(Debug, Error)]
pub enum ClaimError {
    #[error("Claim already exists.")]
    ClaimAlreadyExists,
    #[error("Claim does not exists.")]
    ClaimDoesNotExists,
    #[error("Claimer is not the owner of the content.")]
    ClaimerNotOwnerContent,
}

pub trait Config: crate::system::Config {
    /// O tipo que representa o conteúdo que pode ser reivindicado usando este pallet.
    /// Pode ser o conteúdo diretamente como bytes, ou melhor ainda, o hash desse conteúdo.
    /// Deixamos essa decisão para o desenvolvedor do runtime.
    type Content: Debug + Ord;
}

/// Este é o Módulo de Prova de Existência.
/// É um módulo simples que permite que contas reivindiquem a existência de alguns dados.
#[derive(Debug)]
pub struct Pallet<T: Config> {
    /// Um simples mapa de armazenamento de conteúdo para o proprietário desse conteúdo.
    /// As contas podem fazer várias reivindicações diferentes, mas cada reivindicação só pode ter um proprietário.
    claims: BTreeMap<T::Content, T::AccountId>,
}

impl<T: Config> Pallet<T> {
    /// Cria uma nova instância do Módulo de Prova de Existência.
    pub fn new() -> Self {
        Self {
            claims: BTreeMap::new(),
        }
    }

    /// Obtém o proprietário (se houver) de uma reivindicação.
    pub fn get_claim(&self, claim: &T::Content) -> Option<T::AccountId> {
        self.claims.get(claim).cloned()
    }

    /// Cria uma nova reivindicação em nome do `caller`.
    /// Esta função retornará um erro se alguém já tiver reivindicado esse conteúdo.
    pub fn create_claim(
        &mut self,
        caller: T::AccountId,
        claim: T::Content,
    ) -> Result<(), ClaimError> {
        match self.claims.get(&claim) {
            Some(_) => Err(ClaimError::ClaimAlreadyExists),
            None => {
                self.claims.insert(claim, caller);
                Ok(())
            }
        }
    }

    /// Revoga uma reivindicação existente em algum conteúdo.
    /// Esta função só deve ter sucesso se o chamador for o proprietário de uma reivindicação existente.
    /// Retornará um erro se a reivindicação não existir ou se o chamador não for o proprietário.
    pub fn revoke_claim(
        &mut self,
        caller: &T::AccountId,
        claim: &T::Content,
    ) -> Result<(), ClaimError> {
        let claim_owner = self
            .get_claim(claim)
            .ok_or(ClaimError::ClaimDoesNotExists)?;

        if &claim_owner != caller {
            return Err(ClaimError::ClaimerNotOwnerContent);
        }

        self.claims.remove(claim);
        Ok(())
    }
}

pub enum Call<'a, T: Config> {
    CreateClaim { claim: T::Content },
    RevokeClaim { claim: &'a T::Content },
}

impl<'a, T: Config> crate::support::Dispatch<'a> for Pallet<T>
where
    T::AccountId: 'a,
    T::Content: 'a,
{
    type Caller = &'a T::AccountId;
    type Call = Call<'a, T>;

    fn dispatch(
        &mut self,
        caller: Self::Caller,
        call: Self::Call,
    ) -> crate::support::DispatchResult {
        match call {
            Call::CreateClaim { claim } => self.create_claim(caller.clone(), claim)?,
            Call::RevokeClaim { claim } => self.revoke_claim(caller, claim)?,
        }
        Ok(())
    }
}

#[cfg(test)]
mod test {
    struct TestConfig;

    impl super::Config for TestConfig {
        type Content = String;
    }

    impl crate::system::Config for TestConfig {
        type AccountId = String;
        type BlockNumber = u32;
        type Nonce = u32;
    }

    fn setup() -> (String, String, super::Pallet<TestConfig>) {
        (
            String::from("Alice"),
            String::from("Asset"),
            super::Pallet::new(),
        )
    }

    #[test]
    fn create_proof_of_existence() {
        let (alice, asset, mut poe) = setup();

        assert_eq!(poe.get_claim(&asset), None);
        assert_eq!(poe.create_claim(alice.clone(), asset.clone()).unwrap(), ());
        assert_eq!(poe.get_claim(&asset), Some(alice));
    }

    #[test]
    fn unique_proof_of_existence() {
        let (alice, asset, mut poe) = setup();
        poe.create_claim(alice, asset.clone()).unwrap();

        let err = poe
            .create_claim(String::from("bob"), asset.clone())
            .unwrap_err();
        assert_eq!(matches!(err, super::ClaimError::ClaimAlreadyExists), true);
    }

    #[test]
    fn revoke_proof_of_existence() {
        let (alice, asset, mut poe) = setup();

        let err = poe.revoke_claim(&alice, &asset).unwrap_err();
        assert_eq!(matches!(err, super::ClaimError::ClaimDoesNotExists), true);

        poe.create_claim(alice.clone(), asset.clone()).unwrap();
        let err = poe.revoke_claim(&String::from("bob"), &asset).unwrap_err();
        assert_eq!(
            matches!(err, super::ClaimError::ClaimerNotOwnerContent),
            true
        );

        assert_eq!(poe.revoke_claim(&alice, &asset).unwrap(), ());
        assert_eq!(poe.get_claim(&asset), None);
    }
}
