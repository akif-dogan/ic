use crate::common::doubles::{LedgerExpect, MockLedger};
use candid::Principal;
use ic_base_types::PrincipalId;
use ic_sns_swap::pb::v1::{
    settle_community_fund_participation_result,
    sns_neuron_recipe::{Investor::Direct, NeuronAttributes},
    CanisterCallError, DirectInvestment, RestoreDappControllersResponse,
    SetDappControllersCallResult, SetDappControllersResponse,
    SettleCommunityFundParticipationResult, SnsNeuronRecipe, Swap, TransferableAmount,
};
use std::sync::{Arc, Mutex};

pub mod doubles;

/// Intermediate structure that helps calculate an investor's SNS NeuronId
pub enum TestInvestor {
    /// The CommunityFund Investor with the memo used to calculate it's SNS NeuronId
    CommunityFund(u64),
    /// The Individual Investor with the PrincipalId used to calculate its SNS NeuronId
    Direct(PrincipalId),
}

/// Given a vector of NeuronRecipes, return all related NeuronRecipes for
/// the given buyer_principal
pub fn select_direct_investment_neurons<'a>(
    ns: &'a Vec<SnsNeuronRecipe>,
    buyer_principal: &str,
) -> Vec<&'a SnsNeuronRecipe> {
    let mut neurons = vec![];
    for n in ns {
        match &n.investor {
            Some(Direct(DirectInvestment {
                buyer_principal: buyer,
            })) => {
                if buyer == buyer_principal {
                    neurons.push(n);
                }
            }
            _ => continue,
        }
    }
    if neurons.is_empty() {
        panic!("Cannot find principal {}", buyer_principal);
    }

    neurons
}

pub fn verify_participant_balances(
    swap: &Swap,
    buyer_principal: &PrincipalId,
    icp_balance_e8s: u64,
    sns_balance_e8s: u64,
) {
    let buyer = swap.buyers.get(&buyer_principal.to_string()).unwrap();
    assert_eq!(icp_balance_e8s, buyer.amount_icp_e8s());
    let total_neuron_recipe_sns_e8s_for_principal: u64 =
        select_direct_investment_neurons(&swap.neuron_recipes, &buyer_principal.to_string())
            .iter()
            .map(|neuron_recipe| neuron_recipe.amount_e8s())
            .sum();
    assert_eq!(total_neuron_recipe_sns_e8s_for_principal, sns_balance_e8s);
}

pub fn i2principal_id_string(i: u64) -> String {
    Principal::from(PrincipalId::new_user_test_id(i)).to_text()
}

pub fn create_single_neuron_recipe(amount_e8s: u64, buyer_principal: String) -> SnsNeuronRecipe {
    SnsNeuronRecipe {
        sns: Some(TransferableAmount {
            amount_e8s,
            transfer_start_timestamp_seconds: 0,
            transfer_success_timestamp_seconds: 0,
        }),
        neuron_attributes: Some(NeuronAttributes {
            memo: 0,
            dissolve_delay_seconds: 0,
        }),
        investor: Some(Direct(DirectInvestment { buyer_principal })),
    }
}

pub fn mock_stub(mut expect: Vec<LedgerExpect>) -> MockLedger {
    expect.reverse();
    let e = Arc::new(Mutex::new(expect));
    MockLedger { expect: e }
}

pub fn extract_canister_call_error(
    restore_dapp_controller_response: &RestoreDappControllersResponse,
) -> &CanisterCallError {
    use ic_sns_swap::pb::v1::restore_dapp_controllers_response::Possibility;

    match restore_dapp_controller_response.possibility.as_ref() {
        Some(Possibility::Ok(_)) | None => panic!(
            "Extracting CanisterCallError failed. Possibility was {:?}",
            restore_dapp_controller_response.possibility,
        ),
        Some(Possibility::Err(canister_call_error)) => canister_call_error,
    }
}

pub fn extract_set_dapp_controller_response(
    restore_dapp_controller_response: &RestoreDappControllersResponse,
) -> &SetDappControllersResponse {
    use ic_sns_swap::pb::v1::restore_dapp_controllers_response::Possibility;

    match restore_dapp_controller_response.possibility.as_ref() {
        Some(Possibility::Err(_)) | None => panic!(
            "Extracting SetDappControllersResponse failed. Possibility was {:?}",
            restore_dapp_controller_response.possibility,
        ),
        Some(Possibility::Ok(response)) => response,
    }
}

/// Helper method for constructing a successful response in tests
pub fn successful_settle_community_fund_participation_result(
) -> SettleCommunityFundParticipationResult {
    use ic_sns_swap::pb::v1::settle_community_fund_participation_result::Possibility;

    SettleCommunityFundParticipationResult {
        possibility: Some(Possibility::Ok(
            settle_community_fund_participation_result::Response {
                governance_error: None,
            },
        )),
    }
}

/// Helper method for constructing a successful response in tests
pub fn successful_set_dapp_controllers_call_result() -> SetDappControllersCallResult {
    use ic_sns_swap::pb::v1::set_dapp_controllers_call_result::Possibility;

    SetDappControllersCallResult {
        possibility: Some(Possibility::Ok(SetDappControllersResponse {
            failed_updates: vec![],
        })),
    }
}
