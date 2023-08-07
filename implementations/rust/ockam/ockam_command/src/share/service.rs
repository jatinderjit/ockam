use clap::Args;
use colorful::Colorful;
use miette::IntoDiagnostic;
use tokio::sync::Mutex;
use tokio::try_join;
use tracing::debug;

use ockam::Context;
use ockam_api::cloud::share::{CreateServiceInvitation, SentInvitation};
use ockam_identity::IdentityIdentifier;

use crate::node::util::delete_embedded_node;
use crate::util::api::{self, CloudOpts};
use crate::util::{node_rpc, Rpc};
use crate::{docs, fmt_ok, CommandGlobalOpts};

const PREVIEW_TAG: &str = include_str!("../static/preview_tag.txt");

#[derive(Clone, Debug, Args)]
#[command(
    before_help = docs::before_help(PREVIEW_TAG),
)]
pub struct ServiceCreateCommand {
    #[command(flatten)]
    pub cloud_opts: CloudOpts,
    pub project_id: String,
    pub recipient_email: String,

    pub project_identity: IdentityIdentifier,
    pub project_route: String,
    pub project_authority_identity: IdentityIdentifier,
    pub project_authority_route: String,
    pub shared_node_identity: IdentityIdentifier,
    pub shared_node_route: String,

    #[arg(long, short = 'x')]
    pub expires_at: Option<String>,
}

impl ServiceCreateCommand {
    pub fn run(self, options: CommandGlobalOpts) {
        node_rpc(rpc, (options, self));
    }
}

impl From<ServiceCreateCommand> for CreateServiceInvitation {
    fn from(val: ServiceCreateCommand) -> Self {
        let ServiceCreateCommand {
            expires_at,
            project_id,
            recipient_email,

            project_identity,
            project_route,
            project_authority_identity,
            project_authority_route,
            shared_node_identity,
            shared_node_route,
            ..
        } = val;
        Self {
            expires_at,
            project_id,
            recipient_email,

            project_identity,
            project_route,
            project_authority_identity,
            project_authority_route,
            shared_node_identity,
            shared_node_route,
        }
    }
}

async fn rpc(
    mut ctx: Context,
    (opts, cmd): (CommandGlobalOpts, ServiceCreateCommand),
) -> miette::Result<()> {
    run_impl(&mut ctx, opts, cmd).await
}

async fn run_impl(
    ctx: &mut Context,
    opts: CommandGlobalOpts,
    cmd: ServiceCreateCommand,
) -> miette::Result<()> {
    let is_finished: Mutex<bool> = Mutex::new(false);
    let mut rpc = Rpc::embedded(ctx, &opts).await?;

    let send_req = async {
        let req = cmd.into();
        debug!(?req);

        rpc.request(api::share::create_service_invitation(
            req,
            &CloudOpts::route(),
        ))
        .await?;
        *is_finished.lock().await = true;
        rpc.parse_response_body::<SentInvitation>()
    };

    let output_messages = vec![format!("Creating invitation...\n",)];

    let progress_output = opts
        .terminal
        .progress_output(&output_messages, &is_finished);

    let (sent, _) = try_join!(send_req, progress_output)?;

    debug!(?sent);

    delete_embedded_node(&opts, rpc.node_name()).await;

    let plain = fmt_ok!(
        "Invitation {} to {} {} created, with {} remaining uses and expiring at {}.{}",
        sent.id,
        sent.scope,
        sent.target_id,
        sent.remaining_uses,
        sent.expires_at,
        sent.recipient_email
            .as_ref()
            .map(|e| format!(" {e} will be notified via email."))
            .unwrap_or("".to_string())
    );
    let json = serde_json::to_string_pretty(&sent).into_diagnostic()?;
    opts.terminal
        .stdout()
        .plain(plain)
        .json(json)
        .write_line()?;

    Ok(())
}