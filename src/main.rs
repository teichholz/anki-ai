mod backup;
mod cards;
mod collection;
mod decks;
mod media;
mod notes;
mod notetypes;
mod sync;
mod tags;

use std::collections::HashMap;
use std::io::{self, BufRead, Write};
use std::path::PathBuf;

use anyhow::{anyhow, Result};
use clap::{Parser, Subcommand};

use crate::backup::{create_snapshot, list_snapshots, restore_snapshot};
use crate::cards::{find_cards, get_card_info, suspend_cards, unsuspend_cards};
use crate::collection::{get_collection_path, open_collection};
use crate::decks::{create_deck, delete_deck, list_decks};
use crate::media::add_media_file;
use crate::notes::{add_note, delete_note, get_note, search_notes, update_note};
use crate::notetypes::{get_notetype_fields, list_notetypes};
use crate::sync::{run_sync, save_hkey};
use crate::tags::{bulk_add_tags, bulk_remove_tags, list_tags, rename_tag};

// ---------------------------------------------------------------------------
// CLI structure
// ---------------------------------------------------------------------------

#[derive(Parser)]
#[command(
    name = "anki-ai",
    about = "Headless Anki manager for AI agents",
    version
)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Manage AnkiWeb authentication.
    Auth {
        #[command(subcommand)]
        cmd: AuthCmd,
    },
    /// Pull from (and push to) AnkiWeb.
    Sync {
        /// Sync media files (default: true; use --media=false to disable).
        #[arg(long = "media", default_value_t = true, action = clap::ArgAction::Set)]
        media: bool,
        /// Upload collection to AnkiWeb (default: download).
        #[arg(long)]
        upload: bool,
    },
    /// Create a timestamped snapshot of the collection file.
    Snapshot,
    /// List available snapshots.
    Snapshots,
    /// Restore the collection from a snapshot.
    Restore {
        /// Snapshot filename or full path.
        snapshot: String,
        /// Skip confirmation prompt.
        #[arg(short, long)]
        yes: bool,
    },
    /// Show collection statistics.
    Info,
    /// Deck operations.
    Decks {
        #[command(subcommand)]
        cmd: DecksCmd,
    },
    /// Note operations.
    Notes {
        #[command(subcommand)]
        cmd: NotesCmd,
    },
    /// Card operations.
    Cards {
        #[command(subcommand)]
        cmd: CardsCmd,
    },
    /// Tag operations.
    Tags {
        #[command(subcommand)]
        cmd: TagsCmd,
    },
    /// Note type introspection.
    Notetypes {
        #[command(subcommand)]
        cmd: NotetypesCmd,
    },
    /// Media file operations.
    Media {
        #[command(subcommand)]
        cmd: MediaCmd,
    },
    /// Print the path to the installed Claude skill file.
    Skill,
}

// ---------------------------------------------------------------------------
// Sub-command enums
// ---------------------------------------------------------------------------

#[derive(Subcommand)]
enum AuthCmd {
    /// Exchange AnkiWeb credentials for an auth token and store it.
    Login {
        /// AnkiWeb email address.
        #[arg(long)]
        email: Option<String>,
        /// AnkiWeb password (prefer interactive prompt to avoid shell history exposure).
        #[arg(long)]
        password: Option<String>,
    },
}

#[derive(Subcommand)]
enum DecksCmd {
    /// List decks with due card counts (new / learning / review).
    List,
    /// Create a deck. Returns the existing deck if the name is already taken.
    Create {
        /// Deck name (use '::' for nested decks).
        name: String,
    },
    /// Delete a deck and all its cards.
    Delete {
        /// Deck name to delete.
        name: String,
        /// Skip confirmation prompt.
        #[arg(short, long)]
        yes: bool,
    },
}

#[derive(Subcommand)]
enum NotesCmd {
    /// Add a note to a deck.
    Add {
        /// Target deck name.
        #[arg(long)]
        deck: String,
        /// Note type name.
        #[arg(long = "type", default_value = "Basic")]
        note_type: String,
        /// Field as Name=Value. Repeat for each field.
        #[arg(long = "field")]
        fields: Vec<String>,
    },
    /// Get a note by ID.
    Get {
        /// Note ID.
        note_id: i64,
    },
    /// Search notes and return results as JSON.
    Search {
        /// Anki search query.
        query: String,
    },
    /// Update one or more fields of an existing note.
    Update {
        /// Note ID to update.
        note_id: i64,
        /// Field as Name=Value. Repeat for each field.
        #[arg(long = "field")]
        fields: Vec<String>,
    },
    /// Delete a note by ID.
    Delete {
        /// Note ID to delete.
        note_id: i64,
        /// Skip confirmation prompt.
        #[arg(short, long)]
        yes: bool,
    },
}

#[derive(Subcommand)]
enum CardsCmd {
    /// Find cards matching a search query.
    List {
        /// Anki search query.
        query: String,
    },
    /// Show scheduling info for a card.
    Info {
        /// Card ID.
        card_id: i64,
    },
    /// Suspend cards by ID.
    Suspend {
        /// Card IDs to suspend.
        #[arg(required = true)]
        card_ids: Vec<i64>,
    },
    /// Unsuspend cards by ID.
    Unsuspend {
        /// Card IDs to unsuspend.
        #[arg(required = true)]
        card_ids: Vec<i64>,
    },
}

#[derive(Subcommand)]
enum TagsCmd {
    /// List all tags in the collection.
    List,
    /// Add tags to notes matching a search query.
    Add {
        /// Tags to add.
        #[arg(required = true)]
        tags: Vec<String>,
        /// Anki search query to select notes.
        #[arg(long, short = 'q', default_value = "deck:current")]
        query: String,
    },
    /// Remove tags from notes matching a search query.
    Remove {
        /// Tags to remove.
        #[arg(required = true)]
        tags: Vec<String>,
        /// Anki search query to select notes.
        #[arg(long, short = 'q')]
        query: String,
    },
    /// Rename a tag across all notes.
    Rename {
        /// Existing tag name.
        old: String,
        /// New tag name.
        new: String,
    },
}

#[derive(Subcommand)]
enum NotetypesCmd {
    /// List all note types with their note counts.
    List,
    /// List the fields of a note type.
    Fields {
        /// Note type name.
        name: String,
    },
}

#[derive(Subcommand)]
enum MediaCmd {
    /// Copy local files into the collection media folder.
    Upload {
        /// File(s) to copy into the collection media folder.
        #[arg(required = true)]
        paths: Vec<PathBuf>,
    },
}

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

fn parse_fields(fields: &[String]) -> Result<HashMap<String, String>> {
    let mut map = HashMap::new();
    for f in fields {
        let (k, v) = f
            .split_once('=')
            .ok_or_else(|| anyhow!("Field must be 'Name=Value', got: {:?}", f))?;
        map.insert(k.to_string(), v.to_string());
    }
    Ok(map)
}

fn confirm(prompt: &str) -> Result<bool> {
    print!("{prompt} [y/N] ");
    io::stdout().flush()?;
    let stdin = io::stdin();
    let mut line = String::new();
    stdin.lock().read_line(&mut line)?;
    Ok(matches!(
        line.trim().to_ascii_lowercase().as_str(),
        "y" | "yes"
    ))
}

// ---------------------------------------------------------------------------
// Main
// ---------------------------------------------------------------------------

#[tokio::main]
async fn main() {
    if let Err(e) = run().await {
        eprintln!("Error: {e:#}");
        std::process::exit(1);
    }
}

async fn run() -> Result<()> {
    let cli = Cli::parse();

    match cli.command {
        // -------------------------------------------------------------------
        // auth login
        // -------------------------------------------------------------------
        Commands::Auth {
            cmd: AuthCmd::Login { email, password },
        } => {
            let email = match email.or_else(|| std::env::var("ANKI_EMAIL").ok()) {
                Some(e) => e,
                None => {
                    print!("AnkiWeb email: ");
                    io::stdout().flush()?;
                    let mut buf = String::new();
                    io::stdin().lock().read_line(&mut buf)?;
                    buf.trim().to_string()
                }
            };

            let password = match password.or_else(|| std::env::var("ANKI_PASSWORD").ok()) {
                Some(p) => p,
                None => rpassword::prompt_password("Password: ")?,
            };

            let client = reqwest::Client::new();
            let auth = anki::sync::login::sync_login(email, password, None, client).await?;
            save_hkey(&auth.hkey)?;
            println!("Logged in successfully.");
        }

        // -------------------------------------------------------------------
        // sync
        // -------------------------------------------------------------------
        Commands::Sync { media, upload } => {
            let mut col = open_collection(None)?;
            run_sync(&mut col, media, upload).await?;
        }

        // -------------------------------------------------------------------
        // snapshot
        // -------------------------------------------------------------------
        Commands::Snapshot => {
            let col_path = get_collection_path()?;
            let path = create_snapshot(&col_path)?;
            println!(
                "{}",
                serde_json::to_string_pretty(&serde_json::json!({
                    "snapshot": path.to_string_lossy()
                }))?
            );
        }

        // -------------------------------------------------------------------
        // snapshots
        // -------------------------------------------------------------------
        Commands::Snapshots => {
            let col_path = get_collection_path()?;
            let snaps = list_snapshots(&col_path)?;
            println!("{}", serde_json::to_string_pretty(&snaps)?);
        }

        // -------------------------------------------------------------------
        // restore
        // -------------------------------------------------------------------
        Commands::Restore { snapshot, yes } => {
            if !yes {
                let ok = confirm(&format!(
                    "Restore from '{snapshot}'? The current collection will be overwritten."
                ))?;
                if !ok {
                    println!("Aborted.");
                    return Ok(());
                }
            }
            let col_path = get_collection_path()?;
            let path = restore_snapshot(&col_path, &snapshot)?;
            println!("Restored from '{}'.", path.display());
        }

        // -------------------------------------------------------------------
        // info
        // -------------------------------------------------------------------
        Commands::Info => {
            let col_path = get_collection_path()?;
            let mut col = open_collection(Some(&col_path))?;
            let note_count = col.search_notes_unordered("")?.len();
            let card_count = col.search_cards("", anki::search::SortMode::NoOrder)?.len();
            let data = serde_json::json!({
                "path": col_path.to_string_lossy(),
                "notes": note_count,
                "cards": card_count,
            });
            println!("{}", serde_json::to_string_pretty(&data)?);
        }

        // -------------------------------------------------------------------
        // decks
        // -------------------------------------------------------------------
        Commands::Decks { cmd } => match cmd {
            DecksCmd::List => {
                let mut col = open_collection(None)?;
                let decks = list_decks(&mut col)?;
                println!("{}", serde_json::to_string_pretty(&decks)?);
            }
            DecksCmd::Create { name } => {
                let mut col = open_collection(None)?;
                let deck = create_deck(&mut col, &name)?;
                println!("{}", serde_json::to_string_pretty(&deck)?);
            }
            DecksCmd::Delete { name, yes } => {
                if !yes {
                    let ok = confirm(&format!("Delete deck '{name}' and all its cards?"))?;
                    if !ok {
                        println!("Aborted.");
                        return Ok(());
                    }
                }
                let mut col = open_collection(None)?;
                delete_deck(&mut col, &name)?;
                println!("Deleted deck '{name}'.");
            }
        },

        // -------------------------------------------------------------------
        // notes
        // -------------------------------------------------------------------
        Commands::Notes { cmd } => match cmd {
            NotesCmd::Add {
                deck,
                note_type,
                fields,
            } => {
                if fields.is_empty() {
                    eprintln!("Provide at least one --field Name=Value.");
                    std::process::exit(1);
                }
                let parsed = parse_fields(&fields)?;
                let mut col = open_collection(None)?;
                let note_id = add_note(&mut col, &deck, &note_type, &parsed)?;
                println!(
                    "{}",
                    serde_json::to_string_pretty(&serde_json::json!({ "id": note_id }))?
                );
            }
            NotesCmd::Get { note_id } => {
                let mut col = open_collection(None)?;
                let note = get_note(&mut col, note_id)?;
                println!("{}", serde_json::to_string_pretty(&note)?);
            }
            NotesCmd::Search { query } => {
                let mut col = open_collection(None)?;
                let results = search_notes(&mut col, &query)?;
                println!("{}", serde_json::to_string_pretty(&results)?);
            }
            NotesCmd::Update { note_id, fields } => {
                if fields.is_empty() {
                    eprintln!("Provide at least one --field Name=Value.");
                    std::process::exit(1);
                }
                let parsed = parse_fields(&fields)?;
                let mut col = open_collection(None)?;
                let result = update_note(&mut col, note_id, &parsed)?;
                println!("{}", serde_json::to_string_pretty(&result)?);
            }
            NotesCmd::Delete { note_id, yes } => {
                if !yes {
                    let ok = confirm(&format!("Delete note {note_id}?"))?;
                    if !ok {
                        println!("Aborted.");
                        return Ok(());
                    }
                }
                let mut col = open_collection(None)?;
                delete_note(&mut col, note_id)?;
                println!("Deleted note {note_id}.");
            }
        },

        // -------------------------------------------------------------------
        // cards
        // -------------------------------------------------------------------
        Commands::Cards { cmd } => match cmd {
            CardsCmd::List { query } => {
                let mut col = open_collection(None)?;
                let cards = find_cards(&mut col, &query)?;
                println!("{}", serde_json::to_string_pretty(&cards)?);
            }
            CardsCmd::Info { card_id } => {
                let mut col = open_collection(None)?;
                let info = get_card_info(&mut col, card_id)?;
                println!("{}", serde_json::to_string_pretty(&info)?);
            }
            CardsCmd::Suspend { card_ids } => {
                let mut col = open_collection(None)?;
                let count = suspend_cards(&mut col, &card_ids)?;
                println!(
                    "{}",
                    serde_json::to_string_pretty(&serde_json::json!({ "suspended": count }))?
                );
            }
            CardsCmd::Unsuspend { card_ids } => {
                let len = card_ids.len();
                let mut col = open_collection(None)?;
                unsuspend_cards(&mut col, &card_ids)?;
                println!(
                    "{}",
                    serde_json::to_string_pretty(&serde_json::json!({ "unsuspended": len }))?
                );
            }
        },

        // -------------------------------------------------------------------
        // tags
        // -------------------------------------------------------------------
        Commands::Tags { cmd } => match cmd {
            TagsCmd::List => {
                let mut col = open_collection(None)?;
                let tags = list_tags(&mut col)?;
                println!("{}", serde_json::to_string_pretty(&tags)?);
            }
            TagsCmd::Add { tags, query } => {
                let mut col = open_collection(None)?;
                let count = bulk_add_tags(&mut col, &query, &tags)?;
                println!(
                    "{}",
                    serde_json::to_string_pretty(&serde_json::json!({ "updated": count }))?
                );
            }
            TagsCmd::Remove { tags, query } => {
                let mut col = open_collection(None)?;
                let count = bulk_remove_tags(&mut col, &query, &tags)?;
                println!(
                    "{}",
                    serde_json::to_string_pretty(&serde_json::json!({ "updated": count }))?
                );
            }
            TagsCmd::Rename { old, new } => {
                let mut col = open_collection(None)?;
                let count = rename_tag(&mut col, &old, &new)?;
                println!(
                    "{}",
                    serde_json::to_string_pretty(&serde_json::json!({ "updated": count }))?
                );
            }
        },

        // -------------------------------------------------------------------
        // notetypes
        // -------------------------------------------------------------------
        Commands::Notetypes { cmd } => match cmd {
            NotetypesCmd::List => {
                let mut col = open_collection(None)?;
                let nts = list_notetypes(&mut col)?;
                println!("{}", serde_json::to_string_pretty(&nts)?);
            }
            NotetypesCmd::Fields { name } => {
                let mut col = open_collection(None)?;
                let field_names = get_notetype_fields(&mut col, &name)?;
                println!("{}", serde_json::to_string_pretty(&field_names)?);
            }
        },

        // -------------------------------------------------------------------
        // media
        // -------------------------------------------------------------------
        Commands::Media { cmd } => match cmd {
            MediaCmd::Upload { paths } => {
                let mut col = open_collection(None)?;
                let results: Result<Vec<_>> = paths
                    .iter()
                    .map(|p| {
                        let filename = add_media_file(&mut col, p)?;
                        Ok(serde_json::json!({ "filename": filename }))
                    })
                    .collect();
                println!("{}", serde_json::to_string_pretty(&results?)?);
            }
        },

        // -------------------------------------------------------------------
        // skill
        // -------------------------------------------------------------------
        Commands::Skill => {
            // Embed the skill file at compile time so it is always available
            // regardless of installation directory.
            const SKILL: &str = include_str!("skill/anki.md");
            println!("{SKILL}");
        }
    }

    Ok(())
}
