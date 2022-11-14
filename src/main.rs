use std::{env, fs, io};

use flate2::read::GzDecoder;
use quick_xml::{events::Event, Reader};

fn main() {
    if let Err(err) = try_main() {
        eprintln!("Error occurred: {:#?}", err);
        std::process::exit(1);
    }
}

fn try_main() -> eyre::Result<()> {
    let path = env::args().nth(1).expect("file to read");
    let out_path = env::args().nth(2).expect("file to write");
    println!("reading path: {:?}", path);
    let fp = fs::File::open(path)?;
    let fp_reader = GzDecoder::new(fp);
    let buf_reader = io::BufReader::new(fp_reader);
    let mut reader = Reader::from_reader(buf_reader);

    let out = fs::File::create(out_path)?;
    let mut lemman = Vec::new();
    let mut buf = Vec::new();

    let mut in_adds = false;
    let mut div_adds = 0;
    let mut in_superlemma = false;
    let mut div_superlemma = 0;
    let mut in_span_vt = false;
    let mut span_vt = 0;
    let mut valens = String::new();
    let mut curr = None;
    let mut curr_lemma: Option<Lemma> = None;
    let mut curr_lexem: Option<Lexem> = None;
    let mut curr_cykel: Option<Cykel> = None;
    loop {
        match reader.read_event_into(&mut buf)? {
            Event::Eof => break,
            Event::End(e) => match e.name().as_ref() {
                b"div" => {
                    if in_adds {
                        div_adds -= 1;
                        if div_adds == 0 {
                            in_adds = false;
                            println!("leaving adds ...");
                        }
                    }
                    if in_superlemma {
                        div_superlemma -= 1;
                        if div_superlemma == 0 {
                            in_superlemma = false;
                            let superlemma = curr.clone().unwrap();
                            lemman.push(superlemma);
                            curr = None;
                            println!("leaving superlemma ...");
                        }
                    }
                }
                b"span" => {
                    if in_span_vt {
                        valens.push_str("</span>");
                        span_vt -= 1;
                        if span_vt == 0 {
                            in_span_vt = false;
                            println!("valens = {}", valens);
                        }
                    }
                }
                _tag => {}
            },
            Event::Start(e) => match e.name().as_ref() {
                b"div" => {
                    if in_adds {
                        div_adds += 1;
                    }
                    if in_superlemma {
                        div_superlemma += 1;
                    }
                    match e.try_get_attribute("class")? {
                        Some(attr) => {
                            // println!("got class={:?}", attr);
                            match attr.unescape_value()?.as_ref() {
                                "adds" => {
                                    div_adds = 1;
                                    in_adds = true;
                                    println!("entering adds ...");
                                }
                                "superlemma" => {
                                    div_superlemma = 1;
                                    in_superlemma = true;
                                    println!("entering superlemma ...");
                                }
                                "lexem" => {
                                    if let Some(id) = e.try_get_attribute("id")? {
                                        let x_nr = id.unescape_value()?;
                                        println!("x_nr = {}", x_nr);
                                    }
                                }
                                _class => {} //println!("got Start={:?}, skipping ...", e),
                            }
                        }
                        None => {}
                    }
                }
                b"span" => {
                    if in_span_vt {
                        span_vt += 1;
                    }
                    match e.try_get_attribute("class")? {
                        Some(attr) => {
                            // println!("got class={:?}", attr);
                            match attr.unescape_value()?.as_ref() {
                                "caps" => valens.push_str(r#"<span class="caps">"#),
                                "cbetydelse" => {
                                    if let Some(id) = e.try_get_attribute("id")? {
                                        let kc_nr = id.unescape_value()?;
                                        println!("kc_nr = {} [cykel]", kc_nr);
                                    }
                                }
                                "kbetydelse" => {
                                    if let Some(id) = e.try_get_attribute("id")? {
                                        let kc_nr = id.unescape_value()?;
                                        println!("kc_nr = {} [kernel]", kc_nr);
                                    }
                                }
                                "lopnr" => {
                                    if let Event::Text(lopnr) = reader.read_event_into(&mut buf)? {
                                        let s_nr = lopnr.unescape()?;
                                        println!("s_nr = {}", s_nr);
                                        curr = Some(Superlemma::new(s_nr.parse()?));
                                    }
                                }
                                "lemvarhuvud" => {
                                    if let Some(id) = e.try_get_attribute("id")? {
                                        let l_nr = id.unescape_value()?;
                                        println!("l_nr = {}", l_nr);
                                    }
                                }
                                "vt" => {
                                    in_span_vt = true;
                                    span_vt = 1;
                                    valens.clear();
                                    valens.push_str(r#"<span class="vt">"#);
                                }
                                // "adds" => div_adds = 1,
                                // "superlemma" => div_superlemma = 1,
                                _class => {} // println!("got Start={:?}, skipping ...", e),
                            }
                        }
                        None => {}
                    }
                }
                _tag => {} // println!("got Start={:?}, skipping ...", e),
            },
            Event::Text(e) => {
                if in_span_vt {
                    valens.push_str(e.unescape()?.as_ref());
                }
            }
            _e => {} //println!("got event={:?}, skipping ...", e),
        }
    }
    serde_json::to_writer(&lemman, out)?;
    Ok(())
}

#[derive(Debug, Clone, serde::Serialize)]
pub struct Superlemma {
    pub s_nr: u32,
    pub lemman: Vec<Lemma>,
}

impl Superlemma {
    pub fn new(s_nr: u32) -> Self {
        Self {
            s_nr,
            lemman: Vec::new(),
        }
    }
}
#[derive(Debug, Clone, serde::Serialize)]
pub struct Lemma {
    pub l_nr: u32,
    pub lexem: Vec<Lexem>,
}
#[derive(Debug, Clone, serde::Serialize)]
pub struct Lexem {
    pub x_nr: u32,
    pub kc_nr: u32,
    pub cykler: Vec<Cykel>,
    pub valenser: Vec<String>,
}
#[derive(Debug, Clone, serde::Serialize)]
pub struct Cykel {
    pub kc_nr: u32,
    pub valenser: Vec<String>,
}
