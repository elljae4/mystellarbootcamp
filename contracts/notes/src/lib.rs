#![no_std]
use soroban_sdk::{
    contract, contractimpl, contracttype,
    Address, Env, Vec, token,
};


#[contracttype]
pub enum DataKey {
    TokenContract,
    Circle,
}


#[contracttype]
#[derive(Clone, PartialEq)]
pub enum CircleStatus {
    /// Accepting member contributions
    Open,
    /// Pot has been claimed for the current round
    Paid,
    /// All rounds complete
    Complete,
}

// ---------------------------------------------------------------------------
// The paluwagan (rotating savings circle) state — stored as a single struct
// ---------------------------------------------------------------------------
#[contracttype]
#[derive(Clone)]
pub struct SavingsCircle {
    /// Admin who created the circle
    pub admin: Address,
    /// List of members in payout order (index = round number)
    pub members: Vec<Address>,
    /// Each member's per-round USDC contribution in stroops
    pub contribution_per_member: i128,
    /// Current round index (0-based)
    pub current_round: u32,
    /// Number of members who have contributed THIS round
    pub contributions_this_round: u32,
    /// Status of the current round
    pub status: CircleStatus,
}

// ---------------------------------------------------------------------------
// Contract
// ---------------------------------------------------------------------------
#[contract]
pub struct TipidCircle;

#[contractimpl]
impl TipidCircle {
    // -----------------------------------------------------------------------
    // init — admin creates the circle with member list and per-round amount
    // -----------------------------------------------------------------------
    pub fn init(
        env: Env,
        token_contract: Address,
        admin: Address,
        members: Vec<Address>,
        contribution_per_member: i128,
    ) {
        admin.require_auth();

        if env.storage().instance().has(&DataKey::Circle) {
            panic!("circle already initialised");
        }
        if members.len() < 2 {
            panic!("need at least 2 members");
        }
        if contribution_per_member <= 0 {
            panic!("contribution must be positive");
        }

        env.storage()
            .instance()
            .set(&DataKey::TokenContract, &token_contract);

        let circle = SavingsCircle {
            admin,
            members,
            contribution_per_member,
            current_round: 0,
            contributions_this_round: 0,
            status: CircleStatus::Open,
        };
        env.storage().instance().set(&DataKey::Circle, &circle);
    }

    // -----------------------------------------------------------------------
    // contribute — a member pays their share for the current round
    //
    // Flow: member → sends USDC to contract → contribution counter incremented
    //       → when all members have paid, the pot is automatically sent to
    //         the round's designated recipient (current_round index in members)
    // -----------------------------------------------------------------------
    pub fn contribute(env: Env, member: Address) {
        member.require_auth();

        let mut circle: SavingsCircle = env
            .storage()
            .instance()
            .get(&DataKey::Circle)
            .expect("circle not initialised");

        if circle.status == CircleStatus::Complete {
            panic!("all rounds complete");
        }
        if circle.status == CircleStatus::Paid {
            panic!("round already paid out — wait for next round to open");
        }

        // Verify caller is a member
        let is_member = circle.members.iter().any(|m| m == member);
        if !is_member {
            panic!("caller is not a circle member");
        }

        let token_id: Address = env
            .storage()
            .instance()
            .get(&DataKey::TokenContract)
            .unwrap();
        let token = token::Client::new(&env, &token_id);

        // Pull contribution from member
        token.transfer(
            &member,
            &env.current_contract_address(),
            &circle.contribution_per_member,
        );

        circle.contributions_this_round += 1;

        // If all members have contributed, pay out the pot to this round's recipient
        let member_count = circle.members.len() as u32;
        if circle.contributions_this_round == member_count {
            let recipient = circle
                .members
                .get(circle.current_round)
                .expect("invalid round index");

            let pot = circle.contribution_per_member * member_count as i128;
            token.transfer(&env.current_contract_address(), &recipient, &pot);

            // Advance to next round or complete
            if circle.current_round + 1 >= member_count {
                circle.status = CircleStatus::Complete;
            } else {
                circle.current_round += 1;
                circle.contributions_this_round = 0;
                circle.status = CircleStatus::Open;
            }
        } else {
            circle.status = CircleStatus::Open; // still collecting
        }

        env.storage().instance().set(&DataKey::Circle, &circle);
    }

    // -----------------------------------------------------------------------
    // get_circle — read current circle state
    // -----------------------------------------------------------------------
    pub fn get_circle(env: Env) -> SavingsCircle {
        env.storage()
            .instance()
            .get(&DataKey::Circle)
            .expect("circle not initialised")
    }
}