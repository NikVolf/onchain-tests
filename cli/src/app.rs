use crate::deploy::Deploy;
use gcli::{anyhow, async_trait, clap::Parser, color_eyre, App, Command};

/// My customized sub commands.
#[derive(Debug, Parser)]
pub enum DeployCommand {
    /// GCli preset commands.
    #[clap(flatten)]
    GCliCommands(Command),
    /// My customized ping command.
    Deploy(Deploy),
}

/// My customized gcli.
#[derive(Debug, Parser)]
pub struct DeployApp {
    #[clap(subcommand)]
    command: DeployCommand,
}

#[async_trait]
impl App for DeployApp {
    async fn exec(&self) -> anyhow::Result<()> {
        match &self.command {
            DeployCommand::GCliCommands(command) => command.exec(self).await,
            DeployCommand::Deploy(deploy) => {
                let signer = self.signer().await?;
                deploy.exec(signer).await.map_err(Into::into)
            }
        }
    }
}
