use chrono::Utc;
use serde::Deserialize;

#[derive(Debug, Deserialize)]
pub struct ALTransaction {
    #[serde(rename = "Beløb")]
    pub amount: String,
    #[serde(rename = "Exportkonto")]
    pub exporter: String,
    #[serde(rename = "Modtagerkonto")]
    pub recipient: String,
    #[serde(rename = "Afsenderkonto")]
    pub sender: String,
    #[serde(rename = "Dato", with = "csv_date_format")]
    pub date: chrono::Date<Utc>,
}

mod csv_date_format {
    use chrono::{Date, NaiveDate, Utc};
    use serde::{self, Deserialize, Deserializer};

    pub fn deserialize<'de, D>(deserializer: D) -> Result<Date<Utc>, D::Error>
    where
        D: Deserializer<'de>,
    {
        let s = String::deserialize(deserializer)?;
        Ok(Date::from_utc(
            NaiveDate::parse_from_str(&s, "%d-%m-%Y").map_err(serde::de::Error::custom)?,
            Utc,
        ))
    }
}
