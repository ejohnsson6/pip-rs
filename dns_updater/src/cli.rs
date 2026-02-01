use clap::Parser;

#[derive(Parser, Debug)]
pub struct Args {
    #[clap(short, long = "remote", env = "REMOTE", action = clap::ArgAction::Append)]
    pub remotes: Vec<String>,
    #[clap(short, long = "zone_id", env = "ZONE_ID", action = clap::ArgAction::Append)]
    pub zone_ids: Vec<String>,
    #[clap(short, long, env = "CLOUDFLARE_AUTH_KEY")]
    pub cloudflare_auth_key: String,
    #[clap(short, long, env = "MOCK")]
    pub mock: bool,
}
