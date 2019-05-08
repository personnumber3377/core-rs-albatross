use beserial::{Deserialize, Serialize};

use hash::{Blake2bHash, SerializeContent};
use super::signed;
use bls::bls12_381::PublicKey;
use super::MacroHeader;


pub type SignedPbftProposal = signed::SignedMessage<MacroHeader>;


#[derive(Clone, Debug, Serialize, Deserialize, SerializeContent, PartialEq, Eq)]
pub struct PbftPrepareMessage {
    pub block_hash: Blake2bHash, // 32 bytes
}

impl signed::Message for PbftPrepareMessage {
    const PREFIX: u8 = signed::PREFIX_PBFT_PREPARE;
}

pub type SignedPbftPrepareMessage = signed::SignedMessage<PbftPrepareMessage>;


#[derive(Clone, Debug, Serialize, Deserialize, SerializeContent, PartialEq, Eq)]
pub struct PbftCommitMessage {
    pub block_hash: Blake2bHash, // 32 bytes
}

impl signed::Message for PbftCommitMessage {
    const PREFIX: u8 = signed::PREFIX_PBFT_COMMIT;
}

pub type SignedPbftCommitMessage = signed::SignedMessage<PbftCommitMessage>;


#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub struct PbftProof {
    pub prepare: signed::AggregateProof<PbftPrepareMessage>,
    pub commit: signed::AggregateProof<PbftCommitMessage>,
}

impl PbftProof {
    pub fn new() -> Self {
        Self {
            prepare: signed::AggregateProof::new(),
            commit: signed::AggregateProof::new(),
        }
    }

    /// Verify if we have enough valid signatures in the prepare phase
    pub fn verify_prepare(&self, block_hash: Blake2bHash, threshold: u16) -> bool {
        let prepare = PbftPrepareMessage{ block_hash };
        self.prepare.verify(&prepare, threshold)
    }

    /// Verify that we have enough valid commit signatures that also signed the prepare
    pub fn verify(&self, block_hash: Blake2bHash, threshold: u16) -> bool {
        // XXX if we manually hash the message prefix and the block hash, we don't need to clone
        let prepare = PbftPrepareMessage { block_hash: block_hash.clone() };
        let commit = PbftCommitMessage { block_hash };

        self.prepare.verify(&prepare, threshold)
            && self.commit.verify(&commit, threshold)
            && (&self.prepare.signers & &self.commit.signers).len() as u16 > threshold
    }

    pub fn add_prepare_signature(&mut self, public_key: &PublicKey, slots: u16, prepare: &SignedPbftPrepareMessage) -> bool {
        self.prepare.add_signature(public_key, slots, prepare)
    }

    pub fn add_commit_signature(&mut self, public_key: &PublicKey, slots: u16, commit: &SignedPbftCommitMessage) -> bool {
        self.commit.add_signature(public_key, slots, commit)
    }

    pub fn clear(&mut self) {
        self.prepare.clear();
        self.commit.clear();
    }
}
