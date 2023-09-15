use std::cell::RefCell;
use std::rc::Rc;

use ibc::clients::ics07_tendermint::client_type as tm_client_type;
use ibc::clients::ics07_tendermint::consensus_state::ConsensusState as TmConsensusState;
use ibc::core::ics02_client::client_state::ClientStateCommon;
use ibc::core::ics02_client::ClientExecutionContext;
use ibc::core::ics03_connection::connection::{
    ConnectionEnd, Counterparty as ConnCounterparty, State as ConnectionState,
};
use ibc::core::ics03_connection::version::Version as ConnectionVersion;
use ibc::core::ics04_channel::channel::{
    ChannelEnd, Counterparty as ChanCounterparty, Order, State as ChannelState,
};
use ibc::core::ics04_channel::packet::Sequence;
use ibc::core::ics04_channel::Version as ChannelVersion;
use ibc::core::ics24_host::identifier::{ChainId, ChannelId, ClientId, ConnectionId, PortId};
use ibc::core::ics24_host::path::{
    ChannelEndPath, ClientConsensusStatePath, ClientStatePath, ConnectionPath, SeqSendPath,
};
use ibc::core::{ExecutionContext, ValidationContext};
use ibc::Height;
use sov_bank::Bank;
use sov_modules_api::Context;
use sov_state::WorkingSet;
use tendermint::{Hash, Time};

use crate::context::clients::{AnyClientState, AnyConsensusState};
use crate::context::IbcExecutionContext;
use crate::test_utils::cosmos::helpers::dummy_tm_client_state;
use crate::Ibc;

/// Defines test fixture structure to interact with the bank and ibc modules
pub struct TestApp<'a, C: Context> {
    chain_id: ChainId,
    sdk_ctx: C,
    bank: Bank<C>,
    ibc_ctx: IbcExecutionContext<'a, C>,
}

impl<'a, C: Context> TestApp<'a, C> {
    /// Initializes the test fixture
    pub fn new(
        chain_id: ChainId,
        sdk_ctx: C,
        bank: Bank<C>,
        ibc: Ibc<C>,
        working_set: &'a mut WorkingSet<C::Storage>,
    ) -> Self {
        let shared_working_set = Rc::new(RefCell::new(working_set));

        let ibc_execution_ctx = IbcExecutionContext {
            ibc,
            working_set: shared_working_set,
        };

        Self {
            chain_id,
            sdk_ctx,
            bank,
            ibc_ctx: ibc_execution_ctx,
        }
    }

    pub fn chain_id(&self) -> &ChainId {
        &self.chain_id
    }

    /// Returns access to the ibc module
    pub fn working_set(&self) -> &Rc<RefCell<&'a mut WorkingSet<C::Storage>>> {
        &self.ibc_ctx.working_set
    }

    pub fn sdk_ctx(&self) -> &C {
        &self.sdk_ctx
    }

    /// Returns access to the bank module
    pub fn bank(&self) -> &Bank<C> {
        &self.bank
    }

    pub fn ibc_ctx(&self) -> IbcExecutionContext<'a, C> {
        self.ibc_ctx.clone()
    }

    /// Returns access to the transfer module
    pub fn transfer(&self) -> &sov_ibc_transfer::Transfer<C> {
        &self.ibc_ctx.ibc.transfer
    }

    pub fn get_balance_of(&self, user_address: C::Address, token_address: C::Address) -> u64 {
        self.bank()
            .get_balance_of(
                user_address,
                token_address,
                &mut self.working_set().borrow_mut(),
            )
            .unwrap()
    }

    pub fn get_escrow_address(&self, token_address: C::Address) -> Option<C::Address> {
        // Checks that the token has been transferred
        self.transfer().escrowed_tokens.get(
            &token_address.to_string(),
            &mut self.working_set().borrow_mut(),
        )
    }

    /// Establishes a tendermint light client on the ibc module
    pub fn setup_client(&mut self) {
        let client_counter = self.ibc_ctx.client_counter().unwrap();

        let client_id = ClientId::new(tm_client_type(), client_counter).unwrap();

        let client_state_path = ClientStatePath::new(&client_id);

        let client_state = AnyClientState::Tendermint(dummy_tm_client_state(
            self.chain_id.clone(),
            Height::new(0, 10).unwrap(),
        ));

        let latest_height = client_state.latest_height();

        self.ibc_ctx
            .store_update_time(
                client_id.clone(),
                latest_height,
                self.ibc_ctx.host_timestamp().unwrap(),
            )
            .unwrap();

        self.ibc_ctx
            .store_update_height(
                client_id.clone(),
                latest_height,
                self.ibc_ctx.host_height().unwrap(),
            )
            .unwrap();

        self.ibc_ctx.increase_client_counter().unwrap();

        self.ibc_ctx
            .store_client_state(client_state_path, client_state)
            .unwrap();

        let consensus_state_path =
            ClientConsensusStatePath::new(&client_id, &Height::new(0, 10).unwrap());

        let consensus_state = AnyConsensusState::Tendermint(TmConsensusState::new(
            vec![].into(),
            Time::now(),
            Hash::None,
        ));

        self.ibc_ctx
            .store_consensus_state(consensus_state_path, consensus_state)
            .unwrap();
    }

    /// Establishes a connection on the ibc module with `Open` state
    pub fn setup_connection(&mut self) {
        let client_counter = self
            .ibc_ctx
            .client_counter()
            .unwrap()
            .checked_sub(1)
            .unwrap();

        let client_id = ClientId::new(tm_client_type(), client_counter).unwrap();

        let connection_id = ConnectionId::new(0);

        let connection_path = ConnectionPath::new(&connection_id);

        let prefix = self.ibc_ctx.commitment_prefix();

        let connection_end = ConnectionEnd::new(
            ConnectionState::Open,
            client_id.clone(),
            ConnCounterparty::new(client_id, Some(connection_id), prefix),
            vec![ConnectionVersion::default()],
            Default::default(),
        )
        .unwrap();

        self.ibc_ctx
            .store_connection(&connection_path, connection_end)
            .unwrap();
    }

    /// Establishes a channel on the ibc module with `Open` state
    pub fn setup_channel(&mut self) {
        let connection_id = ConnectionId::new(0);

        let channel_id = ChannelId::new(0);

        let port_id = PortId::transfer();

        let channel_end_path = ChannelEndPath::new(&port_id, &channel_id);

        let channel_end = ChannelEnd::new(
            ChannelState::Open,
            Order::default(),
            ChanCounterparty::new(PortId::transfer(), Some(channel_id.clone())),
            vec![connection_id],
            ChannelVersion::default(),
        )
        .unwrap();

        self.ibc_ctx
            .store_channel(&channel_end_path, channel_end)
            .unwrap();

        let seq_send_path = SeqSendPath::new(&port_id, &channel_id);

        let initial_seq = Sequence::from(0);

        self.ibc_ctx
            .store_next_sequence_send(&seq_send_path, initial_seq)
            .unwrap();
    }
}
