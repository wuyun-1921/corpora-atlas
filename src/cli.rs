use clap::Parser;
use regex::Regex;

use crate::backends::{
    aard2::Aard2Backend, gd::GdBackend, kiwix::KiwixBackend, mediawiki::MediaWikiBackend,
    Backend, BackendOutput, QueryOptions,
};
use crate::error::{Error, Result};
use crate::html::lean;

#[derive(Parser, Debug)]
#[command(name = "corpora-atlas", version)]
pub struct Cli {
    #[arg()]
    pub query: Vec<String>,

    // GD
    #[arg(long)]
    pub gd: bool,
    #[arg(short = 'g')]
    pub gd_group: Option<String>,
    #[arg(short = 'd')]
    pub gd_dicts: Option<String>,
    #[arg(short = 'a')]
    pub gd_all: bool,
    #[arg(short = 'm')]
    pub gd_multi_file: bool,
    #[arg(short = 'n')]
    pub gd_anchors: bool,

    // Kiwix
    #[arg(long)]
    pub kiwix: bool,
    #[arg(short = 'z')]
    pub kiwix_zim: Option<String>,
    #[arg(long, default_value_t = 1)]
    pub kiwix_page: u32,

    // Aard2
    #[arg(long)]
    pub aard2: bool,
    #[arg(short = 's')]
    pub aard2_slob: Option<String>,

    // MediaWiki — site key from config or any URL with /api.php
    #[arg(long)]
    pub mw: Option<String>,
    #[arg(long)]
    pub mw_search: bool,
    #[arg(long, default_value_t = 1)]
    pub mw_page: u32,

    // Lean / formatting
    #[arg(long)]
    pub html: bool,
    #[arg(long)]
    pub lean_toc: bool,
    #[arg(long)]
    pub lean_section: Option<String>,
    #[arg(long)]
    pub lean_text: bool,

    // Daemon / server mode
    #[arg(long)]
    pub daemon: bool,
    #[arg(long)]
    pub toggle_gd_auto_clip: bool,
    #[arg(long)]
    pub toggle_gd_auto_focus: bool,
    #[arg(long)]
    pub gd_clip: bool,
    #[arg(long)]
    pub clip: Option<String>,
    #[arg(long)]
    pub serve: bool,
}

fn apply_lean(content: &str, args: &Cli) -> Option<String> {
    if !args.lean_toc && args.lean_section.is_none() && !args.lean_text {
        return Some(content.to_string());
    }
    if args.lean_toc {
        if let Some(toc) = lean::extract_toc(content) {
            let formatted = lean::format_toc(&toc, 0);
            println!("{formatted}");
        } else {
            println!("(no TOC found)");
        }
        return None;
    }
    if let Some(ref section) = args.lean_section {
        println!("{}", lean::lean_text(content, Some(section)));
        return None;
    }
    if args.lean_text {
        println!("{}", lean::lean_text(content, None));
        return None;
    }
    Some(content.to_string())
}

fn write_output(query: &str, backend: &str, output: &BackendOutput) -> Result<String> {
    let outdir = crate::config::Config::global().paths.output.join(query).join(backend);
    std::fs::create_dir_all(&outdir).map_err(|e| Error::Io(e))?;

    match output {
        BackendOutput::Multi(map) => {
            for (name, text) in map {
                let safe = Regex::new(r"[^a-zA-Z0-9\u4e00-\u9fff _-]")
                    .unwrap()
                    .replace_all(name, "");
                let safe: String = safe.chars().take(40).collect();
                let safe = safe.trim().replace(' ', "_");
                std::fs::write(outdir.join(format!("{safe}.txt")), text)
                    .map_err(|e| Error::Io(e))?;
            }
            Ok(outdir.to_string_lossy().to_string())
        }
        BackendOutput::Text(text) | BackendOutput::Html(text) => {
            if text.trim().is_empty() {
                return Ok(String::new());
            }
            let p = outdir.join("output.txt");
            std::fs::write(&p, text).map_err(|e| Error::Io(e))?;
            Ok(p.to_string_lossy().to_string())
        }
    }
}

pub async fn run_query(args: &Cli) -> Result<i32> {
    let query = args.query.join(" ");
    if query.is_empty() {
        eprintln!("Error: no query provided");
        return Ok(1);
    }
    let as_html = args.html;

    if !args.gd && !args.kiwix && !args.aard2 && args.mw.is_none() {
        // Default: just GD with no special flags → catalog mode
        return run_gd_default(&query, as_html).await;
    }

    if args.gd {
        run_gd(args, &query, as_html).await?;
    }

    if args.kiwix {
        run_kiwix(args, &query, as_html).await?;
    }

    if args.aard2 {
        run_aard2(args, &query, as_html).await?;
    }

    if args.mw.is_some() {
        run_mediawiki(args, &query, as_html).await?;
    }

    Ok(0)
}

async fn run_gd_default(query: &str, as_html: bool) -> Result<i32> {
    let backend = GdBackend;
    let opts = QueryOptions { as_html, ..Default::default() };
    let output = backend.query(query, &opts).await?;
    print_output_as_html(&output, as_html);
    Ok(0)
}

async fn run_gd(args: &Cli, query: &str, as_html: bool) -> Result<()> {
    let mut opts = QueryOptions {
        as_html,
        group: args.gd_group.clone(),
        extract_all: args.gd_all,
        multi_file: args.gd_multi_file,
        anchors: args.gd_anchors,
        select_dicts: args.gd_dicts.as_ref().map(|d| {
            d.split(',').map(|s| s.trim().to_string()).filter(|s| !s.is_empty()).collect()
        }),
    };

    let backend = GdBackend;
    if opts.multi_file || opts.select_dicts.is_some() || opts.extract_all {
        let output = backend.query(query, &opts).await?;
        let path = write_output(query, "gd", &output)?;
        if !path.is_empty() {
            println!("{path}");
        }
    } else {
        opts.multi_file = true;
        let output = backend.query(query, &opts).await?;
        print_output_as_html(&output, as_html);
    }
    Ok(())
}

async fn run_kiwix(args: &Cli, query: &str, as_html: bool) -> Result<()> {
    let zim = match &args.kiwix_zim {
        Some(z) => crate::tokens::resolve_zim(z).to_string(),
        None => {
            eprintln!("Error: --kiwix requires -z");
            return Ok(());
        }
    };
    let backend = KiwixBackend::new(&zim);
    let opts = QueryOptions { as_html, ..Default::default() };
    let output = backend.query(query, &opts).await?;
    let text = match &output {
        BackendOutput::Text(t) | BackendOutput::Html(t) => t.clone(),
        _ => String::new(),
    };
    if !text.is_empty() {
        apply_lean(&text, args);
    }
    Ok(())
}

async fn run_aard2(args: &Cli, query: &str, as_html: bool) -> Result<()> {
    let _slob = args.aard2_slob.as_deref();
    let backend = Aard2Backend::new();
    let opts = QueryOptions { as_html, ..Default::default() };
    let output = backend.query(query, &opts).await?;
    let text = match &output {
        BackendOutput::Text(t) | BackendOutput::Html(t) => t.clone(),
        _ => String::new(),
    };
    if !text.is_empty() {
        apply_lean(&text, args);
    }
    Ok(())
}

async fn run_mediawiki(args: &Cli, query: &str, _as_html: bool) -> Result<()> {
    let site = args.mw.as_deref().unwrap_or("en.wikipedia");
    let backend = MediaWikiBackend::new(site);

    let mut opts = QueryOptions { as_html: true, ..Default::default() };

    if args.mw_search {
        opts.select_dicts = Some(vec!["search".to_string()]);
        let output = backend.query(query, &opts).await?;
        if let BackendOutput::Text(text) = &output {
            print!("{text}");
        }
        return Ok(());
    }

    let output = backend.query(query, &opts).await?;
    let html = match &output {
        BackendOutput::Html(h) => h.clone(),
        _ => String::new(),
    };
    if !html.is_empty() {
        apply_lean(&html, args);
    }
    Ok(())
}

fn print_output_as_html(output: &BackendOutput, as_html: bool) {
    match output {
        BackendOutput::Text(t) => print!("{t}"),
        BackendOutput::Html(h) => print!("{h}"),
        BackendOutput::Multi(map) => {
            if as_html {
                for (_name, text) in map {
                    print!("{text}");
                }
            } else {
                for (name, text) in map {
                    println!("# {name}");
                    println!("{text}");
                    println!();
                }
            }
        }
    }
}
