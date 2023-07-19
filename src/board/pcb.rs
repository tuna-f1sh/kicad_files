//! Parser for the .kicad_pcb [`PCB`] file format and it's sub sections
//!
//! Refers to the [KiCad Board File Format](https://dev-docs.kicad.org/en/file-formats/sexpr-pcb/).
use std::fmt;
use std::str::FromStr;
use serde::{Deserialize, Serialize};
use serde::de::Deserializer;
use serde_sexpr::untagged;

use crate::mm;
use crate::internal::{tuple, option_tuple};
use crate::common::{Paper, TitleBlock};
use crate::board::graphic::GraphicItem;

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields, rename = "version")]
pub struct Version(u32);

impl Default for Version {
    fn default() -> Self {
        Self(20211123)
    }
}

impl Version {
    pub fn new() -> Self {
        Self::default()
    }
}

#[derive(Debug, Default, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename = "general")]
pub struct General {
    #[serde(with = "tuple")]
    pub thickness: mm,
    #[serde(with = "option_tuple")]
    drawings: Option<u32>,
    #[serde(with = "option_tuple")]
    tracks: Option<u32>,
    #[serde(with = "option_tuple")]
    zones: Option<u32>,
    #[serde(with = "option_tuple")]
    modules: Option<u32>,
    #[serde(with = "option_tuple")]
    nets: Option<u32>,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum LayerType {
    User,
    Signal,
    Jumper,
    Mixed,
    Power,
}

impl fmt::Display for LayerType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            LayerType::User => write!(f, "user"),
            LayerType::Signal => write!(f, "signal"),
            LayerType::Jumper => write!(f, "jumper"),
            LayerType::Mixed => write!(f, "mixed"),
            LayerType::Power => write!(f, "power"),
        }
    }
}

impl Default for LayerType {
    fn default() -> Self {
        Self::User
    }
}

// TODO Custom serializer/deserializer for LayerList because it has no name and rename "" does not work (leaves space char)
#[derive(Debug, Default, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename = "")]
pub struct Layer {
    number: u32,
    name: String,
    layer_type: LayerType,
    #[serde(with = "serde_sexpr::Option")]
    user: Option<String>, // this changed in KiCad 6, but there's no documentation yet
}

impl FromStr for Layer {
    type Err = serde_sexpr_base::Error;

    fn from_str(s: &str) -> Result<Self, serde_sexpr_base::Error> {
        match serde_sexpr_base::from_str::<(u32, String, String, String)>(s) {
            Ok(layer_tuple) => {
                Ok(Self {
                    number: layer_tuple.0,
                    name: layer_tuple.1.replace("\"", ""),
                    layer_type: match layer_tuple.2.as_str() {
                        "user" => LayerType::User,
                        "signal" => LayerType::Signal,
                        "jumper" => LayerType::Jumper,
                        "mixed" => LayerType::Mixed,
                        "power" => LayerType::Power,
                        _ => LayerType::default(),
                    },
                    user: Some(layer_tuple.3),
                })
            }
            Err(_) => {
                let layer_tuple = serde_sexpr_base::from_str::<(u32, String, String)>(s)?;
                Ok(Self {
                    number: layer_tuple.0,
                    name: layer_tuple.1.replace("\"", ""),
                    layer_type: match layer_tuple.2.as_str() {
                        "user" => LayerType::User,
                        "signal" => LayerType::Signal,
                        "jumper" => LayerType::Jumper,
                        "mixed" => LayerType::Mixed,
                        "power" => LayerType::Power,
                        _ => LayerType::default(),
                    },
                    user: None,
                })
            }
        }
    }
}

impl ToString for Layer {
    fn to_string(&self) -> String {
        match self.user {
            Some(ref user) => format!("({} \"{}\" {} {})", self.number, self.name, self.layer_type, user),
            None => format!("({} \"{}\" {})", self.number, self.name, self.layer_type),
        }
    }
}

#[derive(Debug, Default, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename = "layers")]
pub struct LayersList {
    #[serde(default, rename = "")]
    pub layers: Vec<Layer>
}

#[derive(Debug, Default, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename = "net")]
pub struct Net {
    number: u8,
    name: String,
}

untagged! {
    #[derive(Clone, Debug, PartialEq)]
    /// Parts of the PCB file which are not always present
    pub enum PCBContent {
        LayersList(LayersList),
        // Setup(Setup),
        // Properties(Properties),
        Net(Net),
        // Footprints(Footprints),
        GraphicItem(GraphicItem)
        // Images(Images),
        // Tracks(Tracks),
        // Zones(Zones),
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename = "kicad_pcb")]
pub struct PCB {
    pub version: Version,
    #[serde(with = "tuple")]
    pub generator: String,
    pub general: General,
    pub page: Paper,
    pub title_block: TitleBlock,
    // pub layers: LayersList,
    #[serde(rename = "")]
    pub pcb_content: Vec<PCBContent>,
}

impl FromStr for PCB {
    type Err = serde_sexpr::de::Error;

    fn from_str(s: &str) -> Result<Self, serde_sexpr::de::Error> {
        serde_sexpr::from_str(s)
    }
}

#[cfg(test)]
mod tests {
    use std::path::PathBuf;
    use std::fs;
    use crate::Unit;
    use crate::common::{PaperSize, Point};
    use crate::board::graphic::Circle;
    use uuid::Uuid;
    use super::*;

    sexpr_test_case! {
        name: general,
        input: r#"(general (thickness 0.89))"#,
        value: General { thickness: 0.89.mm(), ..Default::default() }
    }

    #[test]
    fn test_tuple_layer_sexpr() {
        let input = r#"(0 "F.Cu" signal)"#;
        let expected = Layer { number: 0, name: "F.Cu".to_string(), layer_type: LayerType::Signal, user: None };
        let actual = Layer::from_str(input).unwrap();
        assert_eq!(actual, expected);

        assert_eq!(actual.to_string(), input);
    }

    // sexpr_test_case! {
    //     name: pcb_layer,
    //     input: r#"(0 "F.Cu" signal)"#,
    //     value: Layer { number: 0, name: "F.Cu".to_string(), layer_type: LayerType::Signal, user: None }
    // }

    // sexpr_test_case! {
    //     name: layers_list,
    //     input: r#"(layers (0 "F.Cu" signal) (31 "B.Cu" signal) (40 "Dwgs.User" user "User.Drawings"))"#,
    //     value: LayersList { 
    //         layers: vec![
    //             Layer { number: 0, name: "F.Cu".to_string(), layer_type: LayerType::Signal, user: None },
    //             Layer { number: 31, name: "B.Cu".to_string(), layer_type: LayerType::Signal, user: None },
    //             Layer { number: 40, name: "Dwgs.User".to_string(), layer_type: LayerType::User, user: Some("User.Drawings".to_string()) }
    //         ]
    //     }
    // }

    sexpr_test_case! {
        name: net,
        input: r#"(net 1 "+3V3")"#,
        value: Net { number: 1, name: "+3V3".to_string() }
    }

    sexpr_test_case! {
        name: pcb_content,
        input: r#"(gr_circle (center 1 1) (end 2 2) (width 0.12) (tstamp "00000000-0000-0000-0000-000000000000"))"#,
        value: PCBContent::GraphicItem(
            GraphicItem::Circle(
                Circle {
                    center: Point::new(1.0.mm(), 1.0.mm()),
                    end: Point::new(2.0.mm(), 2.0.mm()),
                    layer: None,
                    width: 0.12.mm(),
                    fill: None,
                    tstamp: Uuid::nil()
                }
            )
        )
    }


    // sexpr_test_case! {
    //     name: kicad_pcb,
    //     input: r#"(kicad_pcb (version 20221018) (generator pcbnew) (general (thickness 0.89)) (paper A4) (title_block (title Minnow)) (layers (0 "F.Cu" signal)))"#,
    //     value: PCB {
    //         version: Version(20221018),
    //         generator: "pcbnew".to_string(),
    //         general: General {
    //             thickness: 0.89.mm(),
    //             ..Default::default()
    //         },
    //         page: Paper {
    //             size: PaperSize::A4,
    //             portrait: false,
    //         },
    //         title_block: TitleBlock {
    //             title: Some("Minnow".to_string()),
    //             date: None,
    //             revision: None,
    //             company: None,
    //             comments: vec![],
    //         },
    //         layers: LayersList { 
    //             layers: vec![
    //                 Layer { number: 0, name: "F.Cu".to_string(), layer_type: LayerType::Signal, user: None }
    //             ] 
    //         },
    //         pcb_content: vec![]
    //     }
    // }

    #[test]
    fn test_deserialize_kicad_pcb_file() {
        let cargo_dir: PathBuf = env!("CARGO_MANIFEST_DIR").parse().unwrap();
        let filepath = cargo_dir.join("tests").join("minnow.kicad_pcb");
        let mut contents =
            fs::read_to_string(filepath).expect("Test .kicad_pcb file missing or unreadable");
        contents = contents.trim().to_string();

        let result = contents.parse::<PCB>().unwrap();

        assert!(result.version == Version(20221018));
        assert!(result.generator == "pcbnew");
    }
}
