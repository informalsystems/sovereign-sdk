use sov_modules_api::hooks::SlotHooks;
use sov_modules_api::{Context, Spec};
use sov_rollup_interface::da::BlockHeaderTrait;
use sov_state::{AccessoryWorkingSet, Storage, WorkingSet};

use super::ChainState;
use crate::{StateTransitionId, TransitionInProgress};

impl<C: Context, Da: sov_modules_api::DaSpec> SlotHooks<Da> for ChainState<C, Da> {
    type Context = C;

    fn begin_slot_hook(
        &self,
        slot_header: &Da::BlockHeader,
        validity_condition: &Da::ValidityCondition,
        working_set: &mut WorkingSet<<Self::Context as Spec>::Storage>,
    ) {
        if self.genesis_hash.get(working_set).is_none() {
            // The genesis hash is not set, hence this is the
            // first transition right after the genesis block
            self.genesis_hash.set(
                &working_set
                    .backing()
                    .get_state_root(&Default::default())
                    .expect("Should have a state root"),
                working_set,
            )
        } else {
            let transition: StateTransitionId<Da> = {
                let last_transition_in_progress = self
                    .in_progress_transition
                    .get(working_set)
                    .expect("There should always be a transition in progress");

                StateTransitionId {
                    da_block_hash: last_transition_in_progress.da_block_hash,
                    post_state_root: working_set
                        .backing()
                        .get_state_root(&Default::default())
                        .expect("Should have a state root"),
                    validity_condition: last_transition_in_progress.validity_condition,
                }
            };

            self.store_state_transition(
                self.slot_height
                    .get(working_set)
                    .expect("Block height must be set"),
                transition,
                working_set,
            );
        }

        self.increment_slot_height(working_set);
        self.time.set(&slot_header.time(), working_set);

        self.in_progress_transition.set(
            &TransitionInProgress {
                da_block_hash: slot_header.hash(),
                validity_condition: *validity_condition,
            },
            working_set,
        );
    }

    fn end_slot_hook(&self, _working_set: &mut WorkingSet<<Self::Context as Spec>::Storage>) {}

    fn finalize_slot_hook(
        &self,
        _root_hash: [u8; 32],
        _accesorry_working_set: &mut AccessoryWorkingSet<<Self::Context as Spec>::Storage>,
    ) {
    }
}
