mod call;
mod genesis;

#[cfg(test)]
mod tests;

pub use call::CallMessage;
use ibc::applications::transfer::{MODULE_ID_STR, PORT_ID_STR};
use ibc::core::ics24_host::identifier::PortId;
use ibc::core::router::{self, ModuleId, Router};
use sov_ibc_transfer::context::TransferContext;
use sov_modules_api::{Error, ModuleInfo};
use sov_state::WorkingSet;

pub struct ExampleModuleConfig {}

/// A new module:
/// - Must derive `ModuleInfo`
/// - Must contain `[address]` field
/// - Can contain any number of ` #[state]` or `[module]` fields
/// - Should derive `ModuleCallJsonSchema` if the "native" feature is enabled.
///   This is optional, and is only used to generate a JSON Schema for your
///   module's call messages (which is useful to develop clients, CLI tooling
///   etc.).
#[cfg_attr(feature = "native", derive(sov_modules_api::ModuleCallJsonSchema))]
#[derive(ModuleInfo)]
pub struct IbcRouterModule<C: sov_modules_api::Context> {
    /// Address of the module.
    #[address]
    pub address: C::Address,

    /// Reference to the Transfer module.
    #[module]
    pub(crate) transfer: sov_ibc_transfer::Transfer<C>,
}

impl<C: sov_modules_api::Context> sov_modules_api::Module for IbcRouterModule<C> {
    type Context = C;

    type Config = ExampleModuleConfig;

    type CallMessage = call::CallMessage;

    fn genesis(
        &self,
        config: &Self::Config,
        working_set: &mut WorkingSet<C::Storage>,
    ) -> Result<(), Error> {
        // The initialization logic
        Ok(self.init_module(config, working_set)?)
    }

    fn call(
        &self,
        _msg: Self::CallMessage,
        _context: &Self::Context,
        _working_set: &mut WorkingSet<C::Storage>,
    ) -> Result<sov_modules_api::CallResponse, Error> {
        // Note: I don't think we need to support a `call()`?
        // We mainly expect the `sov_ibc::Ibc` module to use the router directly
        todo!()
    }
}

pub struct IbcRouter<'ws, C: sov_modules_api::Context> {
    pub transfer_ctx: TransferContext<'ws, C>,
}

impl<'t, 'ws, C> IbcRouter<'ws, C>
where
    C: sov_modules_api::Context,
{
    pub fn new(
        router_mod: &'t IbcRouterModule<C>,
        working_set: &'ws mut WorkingSet<C::Storage>,
    ) -> IbcRouter<'ws, C> {
        IbcRouter {
            transfer_ctx: router_mod.transfer.clone().into_context(working_set),
        }
    }
}

impl<'ws, C> Router for IbcRouter<'ws, C>
where
    C: sov_modules_api::Context,
{
    fn get_route(&self, module_id: &ModuleId) -> Option<&dyn router::Module> {
        if *module_id == ModuleId::new(MODULE_ID_STR.to_string()) {
            Some(&self.transfer_ctx)
        } else {
            None
        }
    }

    fn get_route_mut(&mut self, module_id: &ModuleId) -> Option<&mut dyn router::Module> {
        if *module_id == ModuleId::new(MODULE_ID_STR.to_string()) {
            Some(&mut self.transfer_ctx)
        } else {
            None
        }
    }

    fn lookup_module(&self, port_id: &PortId) -> Option<ModuleId> {
        if port_id.as_str() == PORT_ID_STR {
            Some(ModuleId::new(MODULE_ID_STR.to_string()))
        } else {
            None
        }
    }
}