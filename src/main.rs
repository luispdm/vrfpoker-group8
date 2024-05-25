use rand::rngs::OsRng;
use schnorrkel::vrf::VRFProof;
use schnorrkel::{context::SigningContext, vrf::VRFPreOut, Keypair};
use std::collections::hash_map::DefaultHasher;
use std::hash::{Hash, Hasher};

struct Player {
    keypair: Keypair,
    number: u64,
    number_hash: String,
    vrf_output: Option<VRFPreOut>,
    vrf_proof: Option<VRFProof>,
}

impl Player {
    fn new() -> Self {
        let keypair = Keypair::generate_with(&mut OsRng);
        let number = rand::random::<u64>();
        let mut hasher = DefaultHasher::new();
        number.hash(&mut hasher);
        let number_hash = hex::encode(hasher.finish().to_be_bytes());
        Self {
            keypair,
            number,
            number_hash,
            vrf_output: None,
            vrf_proof: None,
        }
    }

    fn commit(&mut self, input: &[u8]) {
        let context = SigningContext::new(b"VRF Poker");
        let context = context.bytes(input);
        let (vrf_input, vrf_proof, _) = self.keypair.vrf_sign(context);
        self.vrf_proof = Some(vrf_proof);
        self.vrf_output = Some(vrf_input.to_preout());
    }

    fn reveal_vrf_output(&self) -> Option<VRFPreOut> {
        self.vrf_output
    }

    fn vrf_verify(&self, input: &[u8]) -> bool {
        let context = SigningContext::new(b"VRF Poker");
        let context = context.bytes(input);
        if let Some(preout) = &self.vrf_output {
            self.keypair
                .public
                .vrf_verify(context, preout, self.vrf_proof.as_ref().unwrap())
                .is_ok()
        } else {
            false
        }
    }
}

struct Croupier;

impl Croupier {
    fn verify_hash(player: &Player) -> bool {
        let mut hasher = DefaultHasher::new();
        player.number.hash(&mut hasher);
        hex::encode(hasher.finish().to_be_bytes()) == player.number_hash
    }

    fn collect_hashes(players: &[Player]) -> Vec<String> {
        players
            .iter()
            .filter(|p| Self::verify_hash(p))
            .map(|p| p.number_hash.clone())
            .collect()
    }

    fn distribute_input(players: &mut [Player], input: &String) {
        let input_bytes = input.as_bytes();
        for player in players.iter_mut() {
            player.commit(input_bytes);
        }
    }
}

fn main() {
    let mut players = vec![Player::new(), Player::new()];
    for player in &players {
        println!(
            "Player's random input: {}, hash: {}",
            player.number, player.number_hash
        );
    }

    let final_input = Croupier::collect_hashes(&players).concat();

    Croupier::distribute_input(&mut players, &final_input);

    let mut highest_vrf_output: Option<u64> = None;
    let mut winner: Option<usize> = None;

    for (i, player) in players.iter().enumerate() {
        if player.vrf_verify(final_input.as_bytes()) {
            if let Some(vrf_output) = player.reveal_vrf_output() {
                let card_value = (vrf_output.to_bytes()[0] as u64) % 52;
                println!("Player {}'s card value: {}", i, card_value);
                if highest_vrf_output.is_none() || card_value > highest_vrf_output.unwrap() {
                    highest_vrf_output = Some(card_value);
                    winner = Some(i);
                }
            }
        } else {
            println!("Player {} is disqualified.", i);
        }
    }

    if let Some(winner) = winner {
        println!("Player {} wins!", winner);
    } else {
        println!("No winner, something went wrong!");
    }
}
