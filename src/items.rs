use anyhow::{Context as _, Result};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ItemId(u16);

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ItemPackageName(String);

impl ItemPackageName {
    pub fn name(&self) -> &str {
        &self.0
    }
    pub fn sf_name(&self) -> String {
        format!("{}_SF", self.name())
    }
    pub fn filename(&self) -> String {
        format!("{}.upk", self.sf_name())
    }
    pub fn backup_filename(&self) -> String {
        format!("{}.upk.bak", self.sf_name())
    }
}

// there are more but this is just items
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
pub enum ItemSlot {
    Antenna,
    Body,
    #[default]
    Boost,
    Decal,
    Explosion,
    PaintFinish,
    Topper,
    Trail,
    Wheel,
}

impl ItemSlot {
    fn from_slot_str(slot: &str) -> Option<Self> {
        Some(match slot {
            "Antenna" => Self::Antenna,
            "Body" => Self::Body,
            "Decal" => Self::Decal,
            "Rocket Boost" => Self::Boost,
            "Goal Explosion" => Self::Explosion,
            "Paint Finish" => Self::PaintFinish,
            "Topper" => Self::Topper,
            "Trail" => Self::Trail,
            "Wheels" => Self::Wheel,
            _ => return None,
        })
    }

    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Antenna => "Antenna",
            Self::Body => "Body",
            Self::Boost => "Boost",
            Self::Decal => "Decal",
            Self::Explosion => "Goal explosion",
            Self::PaintFinish => "Paint finish",
            Self::Topper => "Topper",
            Self::Trail => "Trail",
            Self::Wheel => "Wheels",
        }
    }
}

const URL: &str =
    "https://raw.githubusercontent.com/ShinyEmii/Toga-Files/refs/heads/master/products.csv";

#[derive(Debug, Clone)]
pub struct Item {
    pub id: ItemId,
    pub name: String,
    pub slot: ItemSlot,
    pub package: ItemPackageName,
    pub key: String,
}

pub async fn fetch_items() -> Result<Vec<Item>> {
    let response = reqwest::get(URL).await.context("fetching items csv")?;
    let text = response.text().await.context("getting items csv text")?;
    let parsed = parse_csv_response(text).context("parsing items csv")?;

    Ok(parsed)
}

fn trim_quotes(string: &str) -> &str {
    &string[1..string.len() - 1]
}

fn parse_csv_response(csv: String) -> Result<Vec<Item>> {
    let mut items = Vec::new();

    // skip table header
    for line in csv.lines().skip(1) {
        let mut values = line.split(',');
        let id = ItemId(values.next().context("loading next id")?.parse()?);
        values.next(); // skip name
        let name = trim_quotes(values.next().context("loading next label")?).to_owned(); // Label

        let Some(slot) =
            ItemSlot::from_slot_str(trim_quotes(values.next().context("loading next slot")?))
        else {
            continue;
        };

        values.next(); // skip quality
        values.next(); // skip unlock method
        values.next(); // skip pack
        let package = ItemPackageName(
            trim_quotes(values.next().context("loading package value")?).to_owned(),
        );

        let key = trim_quotes(values.next().context("loading key value")?);
        if key.is_empty() {
            continue;
        }
        let key = key.to_string();

        items.push(Item {
            id,
            name,
            slot,
            package,
            key,
        });
    }

    items.sort_by_key(|i| i.name.to_lowercase());
    Ok(items)
}
