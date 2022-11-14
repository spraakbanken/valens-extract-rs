use std::{env, fs, io};

use flate2::read::GzDecoder;
use json_arrays::Writer;
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
    let mut writer = Writer::from_path(out_path)?;

    // let out = fs::File::create(out_path)?;
    // let mut lemman = Vec::new();
    let mut buf = Vec::new();

    // let mut in_adds = false;
    // let mut div_adds = 0;
    let mut in_superlemma = false;
    let mut superlemma_count = 0;
    let mut in_span_vt = false;
    let mut span_vt = 0;
    let mut in_lemma = false;
    let mut in_lexem = false;
    let mut in_cykel = false;
    let mut in_span_kernel = false;
    let mut lemma_count = 0;
    let mut lexem_count = 0;
    let mut span_kernel = 0;
    let mut cykel_count = 0;
    let mut valens = String::new();
    let mut curr: Option<Superlemma> = None;
    let mut curr_lemma: Option<Lemma> = None;
    let mut curr_lexem: Option<Lexem> = None;
    let mut curr_cykel: Option<Cykel> = None;
    loop {
        match reader.read_event_into(&mut buf)? {
            Event::Eof => break,
            Event::End(e) => match e.name().as_ref() {
                b"div" => {
                    // if in_adds {
                    //     div_adds -= 1;
                    //     if div_adds == 0 {
                    //         in_adds = false;
                    //         println!("leaving adds ...");
                    //     }
                    // }
                    if in_superlemma {
                        superlemma_count -= 1;
                        if superlemma_count == 0 {
                            in_superlemma = false;
                            let mut superlemma = curr.take().unwrap();
                            if curr_lemma.is_some() {
                                superlemma.lemman.push(curr_lemma.take().unwrap());
                            }
                            writer.serialize(superlemma)?;
                            // lemman.push(superlemma);
                            // curr = None;
                            println!("leaving superlemma ...");
                        }
                    }
                    if in_lexem {
                        lexem_count -= 1;
                        if lexem_count == 0 {
                            in_lexem = false;
                            println!("leaving lexem ...");
                            curr_lemma
                                .as_mut()
                                .unwrap()
                                .lexem
                                .push(curr_lexem.take().unwrap());
                        }
                    }
                    if in_cykel {
                        cykel_count -= 1;
                        if cykel_count == 0 {
                            in_cykel = false;
                            println!("leaving cykel ...");
                            curr_lexem
                                .as_mut()
                                .unwrap()
                                .cykler
                                .push(curr_cykel.take().unwrap());
                        }
                    }
                    if in_lemma {
                        lemma_count -= 1;
                        if lemma_count == 0 {
                            in_lemma = false;
                            println!("leaving lemma ...");
                        }
                    }
                }
                b"span" => {
                    if in_span_vt {
                        valens.push_str("]");
                        span_vt -= 1;
                        if span_vt == 0 {
                            in_span_vt = false;
                            println!("valens = {}", valens);
                            if in_cykel {
                                curr_cykel.as_mut().unwrap().valenser.push(valens.clone());
                            } else if in_lexem {
                                curr_lexem.as_mut().unwrap().valenser.push(valens.clone());
                            }
                        }
                    }

                    if in_span_kernel {
                        span_kernel -= 1;
                        if span_kernel == 0 {
                            in_span_kernel = false;
                            println!("leaving kernel ...");
                        }
                    }
                }
                _tag => {}
            },
            Event::Start(e) => match e.name().as_ref() {
                b"div" => {
                    // if in_adds {
                    //     div_adds += 1;
                    // }
                    if in_superlemma {
                        superlemma_count += 1;
                    }
                    if in_lexem {
                        lexem_count += 1;
                    }
                    if in_cykel {
                        cykel_count += 1;
                    }
                    if in_lemma {
                        lemma_count += 1;
                    }
                    match e.try_get_attribute("class")? {
                        Some(attr) => {
                            // println!("got class={:?}", attr);
                            match attr.unescape_value()?.as_ref() {
                                // "adds" => {
                                //     div_adds = 1;
                                //     in_adds = true;
                                //     println!("entering adds ...");
                                // }
                                "cykel" => {
                                    println!("entering cykel ...");
                                    in_cykel = true;
                                    cykel_count = 1;
                                    curr_cykel = Some(Cykel::new());
                                }
                                "lemvar" => {
                                    println!("entering lemma ...");
                                    in_lemma = true;
                                    lemma_count = 1;
                                    curr_lemma = Some(Lemma::new());
                                }
                                "superlemma" => {
                                    superlemma_count = 1;
                                    in_superlemma = true;
                                    println!("entering superlemma ...");
                                    curr = Some(Superlemma::new());
                                }
                                "lexem" => {
                                    println!("entering lexem ...");
                                    in_lexem = true;
                                    lexem_count = 1;
                                    curr_lexem = Some(Lexem::new());
                                    if let Some(id) = e.try_get_attribute("id")? {
                                        let x_nr: u32 =
                                            id.unescape_value()?.as_ref()[3..].parse()?;
                                        println!("x_nr = {}", x_nr);
                                        curr_lexem.as_mut().unwrap().x_nr = x_nr;
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

                    if in_span_kernel {
                        span_kernel += 1;
                    }

                    match e.try_get_attribute("class")? {
                        Some(attr) => {
                            // println!("got class={:?}", attr);
                            match attr.unescape_value()?.as_ref() {
                                "caps" => valens.push_str("[caps "),
                                "cbetydelse" => {
                                    if let Some(id) = e.try_get_attribute("id")? {
                                        let kc_nr = id.unescape_value()?.as_ref()[4..].parse()?;
                                        println!("kc_nr = {} [cykel]", kc_nr);
                                        curr_cykel.as_mut().unwrap().kc_nr = kc_nr;
                                    }
                                }
                                "kbetydelse" => {
                                    if let Some(id) = e.try_get_attribute("id")? {
                                        let kc_nr = id.unescape_value()?.as_ref()[4..].parse()?;
                                        println!("kc_nr = {} [kernel]", kc_nr);
                                        curr_lexem.as_mut().unwrap().kc_nr = kc_nr;
                                    }
                                }
                                "kernel" => {
                                    println!("entering kernel ...");
                                    in_span_kernel = true;
                                    span_kernel = 1;
                                }
                                "lopnr" => {
                                    if let Event::Text(lopnr) = reader.read_event_into(&mut buf)? {
                                        let s_nr = lopnr.unescape()?.parse()?;
                                        println!("s_nr = {}", s_nr);
                                        curr.as_mut().unwrap().s_nr = s_nr;
                                    }
                                }
                                "lemvarhuvud" => {
                                    if let Some(id) = e.try_get_attribute("id")? {
                                        let l_nr = id.unescape_value()?.as_ref()[3..].parse()?;
                                        println!("l_nr = {}", l_nr);
                                        // if curr_lemma.is_some() {
                                        //     curr.as_mut()
                                        //         .unwrap()
                                        //         .lemman
                                        //         .push(curr_lemma.take().unwrap());
                                        // }
                                        curr_lemma.as_mut().unwrap().l_nr = l_nr;
                                    }
                                }
                                "vt" => {
                                    in_span_vt = true;
                                    span_vt = 1;
                                    valens.clear();
                                    valens.push_str("[vt ");
                                }
                                // "adds" => div_adds = 1,
                                // "superlemma" => superlemma_count = 1,
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
    // serde_json::to_writer(&out, &lemman)?;
    Ok(())
}

#[derive(Debug, Clone, serde::Serialize)]
pub struct Superlemma {
    pub s_nr: u32,
    pub lemman: Vec<Lemma>,
}

impl Superlemma {
    pub fn new() -> Self {
        Self {
            s_nr: 0,
            lemman: Vec::new(),
        }
    }
}
#[derive(Debug, Clone, serde::Serialize)]
pub struct Lemma {
    pub l_nr: u32,
    pub lexem: Vec<Lexem>,
}
impl Lemma {
    pub fn new() -> Self {
        Self {
            l_nr: 0,
            lexem: Vec::new(),
        }
    }
}
#[derive(Debug, Clone, serde::Serialize)]
pub struct Lexem {
    pub x_nr: u32,
    pub kc_nr: u32,
    pub cykler: Vec<Cykel>,
    pub valenser: Vec<String>,
}
impl Lexem {
    pub fn new() -> Self {
        Self {
            x_nr: 0,
            kc_nr: 0,
            cykler: Vec::new(),
            valenser: Vec::new(),
        }
    }
}
#[derive(Debug, Clone, serde::Serialize)]
pub struct Cykel {
    pub kc_nr: u32,
    pub valenser: Vec<String>,
}

impl Cykel {
    pub fn new() -> Self {
        Self {
            kc_nr: 0,
            valenser: Vec::new(),
        }
    }
}
