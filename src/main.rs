use anyhow::{anyhow, Result};
use clap::Parser;
use indexmap::{IndexMap, IndexSet};
use scraper::{Html, Selector};
use serde::Serialize;
use std::path::{Path, PathBuf};

//--------------------------------------------------------------------------------------------------

const BASE_URL: &str = "https://public.cyber.mil/wid/dcwf";
const ELEMENTS_PATH: &str = "workforce-elements";
const USER_AGENT: &str = "dcwf/1.0.0";

//--------------------------------------------------------------------------------------------------

#[derive(Parser)]
#[command(about, version, max_term_width = 80)]
struct Cli {
    /// Output format (json, json-pretty)
    #[arg(short, default_value = "json")]
    format: String,

    /// Data directory
    #[arg(short, value_name = "PATH", default_value = "data")]
    directory: PathBuf,
}

//--------------------------------------------------------------------------------------------------

fn main() -> Result<()> {
    let cli = Cli::parse();

    // Output format
    if !["json", "json-pretty"].contains(&cli.format.as_str()) {
        return Err(anyhow!(format!("Invalid output format: {:?}", cli.format)));
    }

    // Data directory
    let dir = if cli.directory.exists() {
        // Use existing output directory
        cli.directory.clone()
    } else {
        // Create the output directory
        mkdir(cli.directory.clone())?
    };

    // User Agent
    let req_cli = reqwest::blocking::Client::builder()
        .user_agent(USER_AGENT)
        .build()?;

    // Selectors
    let span_sel = Selector::parse("span.spec-area-title").unwrap();
    let a_sel = Selector::parse("a").unwrap();
    let header_sel = Selector::parse("div.new-accordion-header").unwrap();
    let title_sel = Selector::parse("span.new-acc-title a").unwrap();
    let meta_sel = Selector::parse("span.acc-meta").unwrap();
    let desc_sel = Selector::parse("span.acc-desc p").unwrap();
    let content_sel = Selector::parse("div.new-accordion-content").unwrap();
    let col_sel = Selector::parse("div.col-md-6").unwrap();
    let tr_sel = Selector::parse("tbody tr").unwrap();
    let td_sel = Selector::parse("td").unwrap();
    let p_sel = Selector::parse("p").unwrap();

    // Elements page

    let elements_html = get(
        &format!("{BASE_URL}/{ELEMENTS_PATH}"),
        &dir.join("elements.html"),
        &req_cli,
    )?;

    let doc = Html::parse_document(&elements_html);

    let mut elements = doc
        .select(&span_sel)
        .map(|x| {
            let a = x.select(&a_sel).next().unwrap();
            let el = Element::new(&a.inner_html(), a.value().attr("href").unwrap());
            (el.id.clone(), el)
        })
        .collect::<IndexMap<String, Element>>();

    // Each element page

    let mut roles = IndexMap::new();
    let mut ksats = IndexMap::new();

    let elements_dir = mkdir(dir.join("elements"))?;

    for el in elements.values_mut() {
        // Download/open the element HTML
        let el_html = get(
            &el.url,
            &elements_dir.join(&format!("{}.html", el.id)),
            &req_cli,
        )?;

        // Parse the HTML
        let doc = Html::parse_document(&el_html);

        // Extract the roles
        let role_ids = doc
            .select(&header_sel)
            .map(|x| {
                let meta = x.select(&meta_sel).next().unwrap().inner_html();

                let id = meta.strip_prefix("Work Role ID: ").unwrap();
                let id = id[..id.find(' ').unwrap()].to_string();

                if !roles.contains_key(&id) {
                    let title = x.select(&title_sel).next().unwrap();
                    let nist_id = meta.split(' ').next_back().unwrap();
                    let nist_id = nist_id.strip_suffix(')').unwrap();
                    let desc = x.select(&desc_sel).next().unwrap().inner_html();

                    roles.insert(
                        id.clone(),
                        Role::new(
                            &title.inner_html(),
                            title.value().attr("href").unwrap(),
                            &id,
                            nist_id,
                            &desc[..desc.find("<br>").unwrap_or(desc.len())],
                            &el.id,
                        ),
                    );
                }

                id
            })
            .collect::<Vec<_>>();

        // KSATs

        for (role_i, content) in doc.select(&content_sel).enumerate() {
            // Each role

            let role_id = &role_ids[role_i];

            let mut core_ksats = IndexSet::new();
            let mut addl_ksats = IndexSet::new();

            for (col_i, col) in content.select(&col_sel).enumerate() {
                // Each KSAT column (core, additional)

                for tr in col.select(&tr_sel) {
                    // Each KSAT

                    let mut id = None;
                    let mut description = None;
                    let mut kind = None;

                    for (td_i, td) in tr.select(&td_sel).enumerate() {
                        // Each KSAT cell

                        if td_i == 0 {
                            id = Some(td.select(&a_sel).next().unwrap().inner_html());
                        } else if td_i == 1 {
                            let s = td.select(&p_sel).next().unwrap().inner_html();
                            if let Some(s) = s.strip_prefix("* ") {
                                description = Some(s.to_string());
                            } else {
                                description = Some(s);
                            }
                        } else if td_i == 2 {
                            kind = Some(td.inner_html().trim().to_string());
                        } else {
                            panic!("KSAT table should not have more than 3 columns");
                        }
                    }

                    let id = id.expect("KSAT ID");
                    let description = description.expect("KSAT Description");
                    let kind = kind.expect("KSAT Kind");

                    let ksat = Ksat {
                        id,
                        description,
                        kind,
                    };

                    if col_i == 0 {
                        // Add core KSAT to role
                        if !core_ksats.contains(&ksat.id) {
                            core_ksats.insert(ksat.id.clone());
                        }
                    } else {
                        // Add additional KSAT to role
                        if !addl_ksats.contains(&ksat.id) {
                            addl_ksats.insert(ksat.id.clone());
                        }
                    }

                    // Add KSAT
                    if !ksats.contains_key(&ksat.id) {
                        ksats.insert(ksat.id.clone(), ksat);
                    }
                }
            }

            // Add KSATs to role
            let role = roles.get_mut(role_id).unwrap();
            role.core_ksats = core_ksats;
            role.additional_ksats = addl_ksats;
        }

        // Add roles to element
        el.role_ids = role_ids;
    }

    let data = Data {
        elements,
        roles,
        ksats,
    };

    match cli.format.as_str() {
        "json" => println!("{}", serde_json::to_string(&data)?),
        "json-pretty" => println!("{}", serde_json::to_string_pretty(&data)?),
        _ => unreachable!(),
    }

    Ok(())
}

//--------------------------------------------------------------------------------------------------

fn get(url: &str, path: &Path, req_cli: &reqwest::blocking::Client) -> Result<String> {
    if let Ok(s) = std::fs::read_to_string(path) {
        eprintln!("Read from {path:?}");
        Ok(s)
    } else {
        eprint!("Fetching {url:?}... ");
        let s = req_cli.get(url).send()?.text()?;
        std::fs::write(path, &s)?;
        eprintln!("saved to {path:?}");
        Ok(s)
    }
}

//--------------------------------------------------------------------------------------------------

fn mkdir(dir: PathBuf) -> Result<PathBuf> {
    if dir.exists() && !dir.is_dir() {
        std::fs::remove_dir_all(&dir)?;
    }
    if !dir.exists() {
        std::fs::create_dir(&dir)?;
    }
    Ok(dir)
}

//--------------------------------------------------------------------------------------------------

#[derive(Debug, Serialize)]
struct Data {
    elements: IndexMap<String, Element>,
    roles: IndexMap<String, Role>,
    ksats: IndexMap<String, Ksat>,
}

//--------------------------------------------------------------------------------------------------

#[derive(Clone, Debug, Serialize)]
struct Element {
    name: String,
    url: String,
    id: String,
    role_ids: Vec<String>,
}

impl Element {
    fn new(name: &str, url: &str) -> Element {
        Element {
            name: name.to_string(),
            url: url.to_string(),
            id: url.split('/').nth(4).unwrap().to_string(),
            role_ids: vec![],
        }
    }
}

//--------------------------------------------------------------------------------------------------

#[derive(Clone, Debug, Serialize)]
struct Role {
    name: String,
    url: String,
    id: String,
    nist_id: Option<String>,
    name_id: String,
    description: String,
    element: String,
    core_ksats: IndexSet<String>,
    additional_ksats: IndexSet<String>,
}

impl Role {
    fn new(name: &str, url: &str, id: &str, nist_id: &str, desc: &str, element: &str) -> Role {
        Role {
            name: name.to_string(),
            url: url.to_string(),
            id: id.to_string(),
            nist_id: if nist_id == "N/A" {
                None
            } else {
                Some(nist_id.to_string())
            },
            name_id: url.split('/').nth(4).unwrap().to_string(),
            description: desc.to_string(),
            element: element.to_string(),
            core_ksats: IndexSet::new(),
            additional_ksats: IndexSet::new(),
        }
    }
}

//--------------------------------------------------------------------------------------------------

#[derive(Clone, Debug, Serialize)]
struct Ksat {
    id: String,
    description: String,
    kind: String,
}