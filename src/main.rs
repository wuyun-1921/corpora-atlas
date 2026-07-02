use corpora_atlas::cli::Cli;
use corpora_atlas::daemon::ipc;
use corpora_atlas::{config, daemon};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let args = <Cli as clap::Parser>::parse();

    if let Err(e) = config::Config::init() {
        if !args.daemon && !args.toggle_gd_auto_clip && !args.gd_clip && !args.toggle_gd_auto_focus {
            eprintln!("Error: {e}");
            std::process::exit(1);
        }
    }

    if args.toggle_gd_auto_clip {
        let resp = ipc::send_daemon("toggle", None).await;
        match resp {
            Ok(v) => {
                let monitoring = v["monitoring"].as_bool().unwrap_or(false);
                println!("clipboard monitoring: {}", if monitoring { "ON" } else { "OFF" });
                return Ok(());
            }
            Err(_) => {
                return daemon::Daemon::new().run().await
                    .map_err(|e| anyhow::anyhow!("{e}"));
            }
        }
    }

    if args.toggle_gd_auto_focus {
        let resp = ipc::send_daemon("toggle_focus", None).await;
        match resp {
            Ok(v) => {
                let focus = v["focus_gd"].as_bool().unwrap_or(false);
                println!("GD auto-focus: {}", if focus { "ON" } else { "OFF" });
                return Ok(());
            }
            Err(_) => {
                eprintln!("Error: daemon not running");
                std::process::exit(1);
            }
        }
    }

    if args.daemon {
        let d = daemon::Daemon::new();
        return d.run().await.map_err(|e| anyhow::anyhow!("{e}"));
    }

    if args.gd_clip {
        let socket_path = config::Config::global().paths.socket.clone();
        if !socket_path.exists() {
            tokio::spawn(async {
                let d = daemon::Daemon::new();
                if let Err(e) = d.run().await {
                    eprintln!("daemon error: {e}");
                }
            });
            tokio::time::sleep(std::time::Duration::from_millis(500)).await;
            for i in 0..10 {
                if socket_path.exists() {
                    break;
                }
                if i == 9 {
                    eprintln!("Error: daemon failed to start after 1.5s — check daemon error output above");
                    std::process::exit(1);
                }
                tokio::time::sleep(std::time::Duration::from_millis(100)).await;
            }
        }
        let extra = args.clip.as_ref().map(|c| ("clip", c.as_str()));
        let resp = ipc::send_daemon("cycle", extra).await;
        match resp {
            Ok(v) => {
                if let Some(status) = v.get("status").and_then(|s| s.as_str()) {
                    println!("{status}");
                }
            }
            Err(e) => {
                eprintln!("{e}");
            }
        }
        return Ok(());
    }

    if args.serve {
        eprintln!("Error: server.py not found — the web UI is in a separate branch.");
        std::process::exit(1);
    }

    let exit_code = corpora_atlas::cli::run_query(&args).await?;
    std::process::exit(exit_code);
}
